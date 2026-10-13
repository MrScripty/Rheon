//! Bounded, transactional SPH previews in physical SI coordinates. Liquids
//! use reduced-stiffness positive pressure, gases an ideal-gas EOS. Vacuum
//! owns no particles. Pair forces use deterministic linked-cell neighbors;
//! disjoint particle evaluations can run on the caller's Rayon pool.
use crate::mesh_collision::CollisionMesh;
use crate::observation::{CalculationStage, WorkProbe};
use crate::{DomainFill, FluidPhase, GridGeometry, ParticleFluidConfig, RotatingCylinderEmitter};
use std::{fmt, sync::Arc};
const NONE: usize = usize::MAX;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluidParticle {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass: f64,
    pub temperature_kelvin: f64,
    /// 0: emitted material; 1: initial surrounding material.
    pub material: u8,
}
impl Default for FluidParticle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            mass: 0.0,
            temperature_kelvin: 293.15,
            material: 0,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ParticleFluidError {
    InvalidSettings,
    InvalidEmitter,
    InvalidTimeStep,
    CollisionMeshOutsideDomain,
    CollisionMeshOverlapsParticles,
    EmitterInsideCollisionSolid,
    CollisionQueryFailed,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    ParticleLimit { limit: usize },
    ArithmeticFailure,
    StabilityLimit { recommended_dt: f64 },
    Cancelled,
}
impl fmt::Display for ParticleFluidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Material simulation: {self:?}")
    }
}
impl std::error::Error for ParticleFluidError {}

#[derive(Debug, Clone, Copy)]
pub struct ParticleStepReport {
    pub dt: f64,
    pub particles: usize,
    pub emitted_mass: f64,
    pub total_mass: f64,
    pub max_speed: f64,
    pub max_pressure: f64,
    pub mean_temperature_kelvin: f64,
}
#[derive(Clone, Copy, Default)]
struct Thermodynamic {
    density: f64,
    pressure: f64,
}
#[derive(Clone, Copy, Default)]
struct Force {
    acceleration: [f64; 3],
    damping_rate: f64,
}

pub struct ParticleFluidSimulation {
    grid: GridGeometry,
    config: ParticleFluidConfig,
    accepted: Vec<FluidParticle>,
    candidate: Vec<FluidParticle>,
    thermodynamic: Vec<Thermodynamic>,
    force: Vec<Force>,
    heads: Vec<usize>,
    next: Vec<usize>,
    bins: [usize; 3],
    spacing: f64,
    support: f64,
    volume: f64,
    time: f64,
    generation: u64,
    credit: [f64; 2],
    emission_sequence: usize,
    allocated_bytes: usize,
    collision_mesh: Option<Arc<CollisionMesh>>,
}
fn norm(v: [f64; 3]) -> f64 {
    v.into_iter().map(|x| x * x).sum::<f64>().sqrt()
}
fn allocate<T: Copy>(
    n: usize,
    value: T,
    remaining: &mut usize,
) -> Result<Vec<T>, ParticleFluidError> {
    let required = n
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(ParticleFluidError::AllocationFailed)?;
    if required > *remaining {
        return Err(ParticleFluidError::BufferLimit {
            required,
            limit: *remaining,
        });
    }
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| ParticleFluidError::AllocationFailed)?;
    let actual = v
        .capacity()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(ParticleFluidError::AllocationFailed)?;
    if actual > *remaining {
        return Err(ParticleFluidError::BufferLimit {
            required: actual,
            limit: *remaining,
        });
    }
    *remaining -= actual;
    v.resize(n, value);
    Ok(v)
}
impl ParticleFluidSimulation {
    pub fn new(
        grid: GridGeometry,
        config: ParticleFluidConfig,
    ) -> Result<Self, ParticleFluidError> {
        if !config.material.is_valid()
            || !config.ambient_material.is_valid()
            || !config.ambient_temperature_kelvin.is_finite()
            || config.ambient_temperature_kelvin <= 0.0
            || config.gravity.iter().any(|v| !v.is_finite())
            || !(4..=64).contains(&config.samples_per_axis)
            || !(1..=262144).contains(&config.max_particles)
            || !config.liquid_sound_speed.is_finite()
            || config.liquid_sound_speed <= 0.0
            || !config.wall_heat_exchange.is_finite()
            || config.wall_heat_exchange < 0.0
        {
            return Err(ParticleFluidError::InvalidSettings);
        }
        let extent = std::array::from_fn::<_, 3, _>(|d| grid.upper()[d] - grid.origin()[d]);
        let spacing = extent.into_iter().fold(0.0_f64, f64::max) / config.samples_per_axis as f64;
        let support = 2.0 * spacing;
        let volume = spacing.powi(3);
        if !volume.is_normal() || !support.is_normal() || !support.powi(9).is_normal() {
            return Err(ParticleFluidError::InvalidSettings);
        }
        if extent.iter().any(|l| *l <= 0.4 * spacing)
            || !config.liquid_sound_speed.powi(2).is_normal()
            || !config
                .material
                .pressure(
                    config.material.density,
                    config.material.temperature_kelvin,
                    config.liquid_sound_speed,
                )
                .is_finite()
            || !config
                .ambient_material
                .pressure(
                    config.ambient_material.density,
                    config.ambient_temperature_kelvin,
                    config.liquid_sound_speed,
                )
                .is_finite()
        {
            return Err(ParticleFluidError::InvalidSettings);
        }
        let bins = extent.map(|l| (l / support).ceil().max(1.0) as usize);
        let bin_count = bins
            .into_iter()
            .try_fold(1_usize, usize::checked_mul)
            .ok_or(ParticleFluidError::AllocationFailed)?;
        let required = config
            .max_particles
            .checked_mul(
                2 * std::mem::size_of::<FluidParticle>()
                    + std::mem::size_of::<Thermodynamic>()
                    + std::mem::size_of::<Force>()
                    + std::mem::size_of::<usize>(),
            )
            .and_then(|n| n.checked_add(bin_count * std::mem::size_of::<usize>()))
            .ok_or(ParticleFluidError::AllocationFailed)?;
        if required > config.memory_limit {
            return Err(ParticleFluidError::BufferLimit {
                required,
                limit: config.memory_limit,
            });
        }
        let mut remaining = config.memory_limit;
        let mut accepted = allocate(
            config.max_particles,
            FluidParticle::default(),
            &mut remaining,
        )?;
        accepted.clear();
        let mut candidate = allocate(
            config.max_particles,
            FluidParticle::default(),
            &mut remaining,
        )?;
        candidate.clear();
        let thermodynamic = allocate(
            config.max_particles,
            Thermodynamic::default(),
            &mut remaining,
        )?;
        let force = allocate(config.max_particles, Force::default(), &mut remaining)?;
        let next = allocate(config.max_particles, NONE, &mut remaining)?;
        let heads = allocate(bin_count, NONE, &mut remaining)?;
        if config.fill != DomainFill::Vacuum {
            let counts = extent.map(|l| (l / spacing).round().max(1.0) as usize);
            let n = counts
                .into_iter()
                .try_fold(1_usize, usize::checked_mul)
                .ok_or(ParticleFluidError::AllocationFailed)?;
            if n > config.max_particles {
                return Err(ParticleFluidError::ParticleLimit {
                    limit: config.max_particles,
                });
            }
            let id = u8::from(config.fill == DomainFill::OtherFluid);
            let material = if id == 0 {
                config.material
            } else {
                config.ambient_material
            };
            let cell_volume = extent.into_iter().product::<f64>() / n as f64;
            if !(material.density * cell_volume).is_normal() {
                return Err(ParticleFluidError::InvalidSettings);
            }
            for z in 0..counts[2] {
                for y in 0..counts[1] {
                    for x in 0..counts[0] {
                        accepted.push(FluidParticle {
                            position: std::array::from_fn(|d| {
                                grid.origin()[d]
                                    + ([x, y, z][d] as f64 + 0.5) * extent[d] / counts[d] as f64
                            }),
                            velocity: [0.0; 3],
                            mass: material.density * cell_volume,
                            temperature_kelvin: if id == 0 {
                                material.temperature_kelvin
                            } else {
                                config.ambient_temperature_kelvin
                            },
                            material: id,
                        });
                    }
                }
            }
        }
        Ok(Self {
            grid,
            config,
            accepted,
            candidate,
            thermodynamic,
            force,
            heads,
            next,
            bins,
            spacing,
            support,
            volume,
            time: 0.0,
            generation: 0,
            credit: [0.0; 2],
            emission_sequence: 0,
            allocated_bytes: config.memory_limit - remaining,
            collision_mesh: None,
        })
    }
    /// Attach a static boundary transactionally. Bounds must fit entirely in
    /// the domain. At generation zero, particles inside/on a convex solid are
    /// excluded (their mass is removed, not redistributed); callers can report
    /// the before/after particle count. Later overlapping replacement is rejected.
    /// Removing a boundary does not restore excluded initial particles; reset
    /// creates a fresh simulation and reapplies the selected boundary.
    pub fn set_collision_mesh(
        &mut self,
        mesh: Option<Arc<CollisionMesh>>,
    ) -> Result<(), ParticleFluidError> {
        if let Some(mesh) = &mesh {
            let (low, high) = mesh.bounds();
            if (0..3).any(|d| low[d] < self.grid.origin()[d] || high[d] > self.grid.upper()[d]) {
                return Err(ParticleFluidError::CollisionMeshOutsideDomain);
            }
            let required = self.allocated_bytes.saturating_add(mesh.allocated_bytes());
            if required > self.config.memory_limit {
                return Err(ParticleFluidError::BufferLimit {
                    required,
                    limit: self.config.memory_limit,
                });
            }
            if self.generation != 0
                && self
                    .accepted
                    .iter()
                    .any(|p| mesh.contains_or_touches(p.position))
            {
                return Err(ParticleFluidError::CollisionMeshOverlapsParticles);
            }
            if self.generation == 0 {
                self.accepted
                    .retain(|p| !mesh.contains_or_touches(p.position));
            }
        }
        self.collision_mesh = mesh;
        Ok(())
    }
    pub fn collision_mesh(&self) -> Option<&Arc<CollisionMesh>> {
        self.collision_mesh.as_ref()
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn config(&self) -> ParticleFluidConfig {
        self.config
    }
    pub fn particles(&self) -> &[FluidParticle] {
        &self.accepted
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
            + self
                .collision_mesh
                .as_ref()
                .map_or(0, |mesh| mesh.allocated_bytes())
    }
    pub fn particle_spacing(&self) -> f64 {
        self.spacing
    }
    /// Coordinates and velocities are physical metres and metres/second.
    /// A failed step retains accepted particles, source credits and clock.
    pub fn step(
        &mut self,
        requested_dt: f64,
        emitter: Option<RotatingCylinderEmitter>,
        mut cancel: impl FnMut() -> bool,
    ) -> Result<ParticleStepReport, ParticleFluidError> {
        let mut interval = requested_dt;
        for _ in 0..16 {
            match self.prepare_step(interval, emitter, &mut cancel) {
                Err(ParticleFluidError::StabilityLimit { recommended_dt }) => {
                    interval = recommended_dt
                }
                result => return result,
            }
        }
        Err(ParticleFluidError::StabilityLimit {
            recommended_dt: interval,
        })
    }
    fn prepare_step(
        &mut self,
        requested_dt: f64,
        emitter: Option<RotatingCylinderEmitter>,
        cancel: &mut impl FnMut() -> bool,
    ) -> Result<ParticleStepReport, ParticleFluidError> {
        if !requested_dt.is_finite() || requested_dt <= 0.0 {
            return Err(ParticleFluidError::InvalidTimeStep);
        }
        if cancel() {
            return Err(ParticleFluidError::Cancelled);
        }
        if let Some(e) = emitter {
            e.validate(&self.grid, self.config.material.density)
                .map_err(|_| ParticleFluidError::InvalidEmitter)?;
        }
        let materials = [self.config.material, self.config.ambient_material];
        let max_speed = self
            .accepted
            .iter()
            .map(|p| norm(p.velocity))
            .fold(0.0_f64, f64::max);
        let source_speed = emitter.map_or(0.0, |e| {
            (2.0 * e.pressure_heads.into_iter().fold(0.0_f64, f64::max)
                / self.config.material.density)
                .sqrt()
        });
        let mut sound = if emitter.is_some() {
            self.config.material.sound_speed(
                self.config.material.temperature_kelvin,
                self.config.liquid_sound_speed,
            )
        } else {
            0.0
        };
        let mut nu = if emitter.is_some() {
            self.config.material.viscosity / self.config.material.density
        } else {
            0.0
        };
        let mut sigma = 0.0_f64;
        for p in &self.accepted {
            let m = materials[p.material as usize];
            sound = sound.max(m.sound_speed(p.temperature_kelvin, self.config.liquid_sound_speed));
            nu = nu.max(m.viscosity / m.density);
            sigma = sigma.max(m.surface_tension / m.density);
        }
        let mut dt = requested_dt;
        if sound + max_speed + source_speed > 0.0 {
            dt = dt.min(0.2 * self.support / (sound + max_speed + source_speed));
        }
        if nu > 0.0 {
            dt = dt.min(0.04 * self.support * self.support / nu);
        }
        if sigma > 0.0 {
            dt = dt.min(0.2 * (self.support.powi(3) / sigma).sqrt());
        }
        let gravity = norm(self.config.gravity);
        if gravity > 0.0 {
            dt = dt.min(0.2 * (self.support / gravity).sqrt());
        }
        if let Some(e) = emitter
            && e.rotations_per_second != 0.0
        {
            dt = dt.min(0.125 / e.rotations_per_second.abs());
        }
        if !dt.is_normal() {
            return Err(ParticleFluidError::ArithmeticFailure);
        }
        self.candidate.clear();
        self.candidate.extend_from_slice(&self.accepted);
        let mut credit = self.credit;
        let mut sequence = self.emission_sequence;
        let mut emitted_mass = 0.0;
        if let Some(e) = emitter {
            let axis = e
                .axis_at(self.time + 0.5 * dt)
                .map_err(|_| ParticleFluidError::InvalidEmitter)?;
            for (end, credit) in credit.iter_mut().enumerate() {
                let speed = (2.0 * e.pressure_heads[end] / self.config.material.density).sqrt();
                // Mean sqrt(disk profile), approximately 0.43. The source is
                // a reservoir; unused represented volume survives to next step.
                *credit += std::f64::consts::PI * e.radius * e.radius * speed * 0.43 * dt;
                if !credit.is_finite() {
                    return Err(ParticleFluidError::ArithmeticFailure);
                }
                let emit = (*credit / self.volume).floor() as usize;
                if emit > self.config.max_particles - self.candidate.len() {
                    return Err(ParticleFluidError::ParticleLimit {
                        limit: self.config.max_particles,
                    });
                }
                for _ in 0..emit {
                    if sequence.is_multiple_of(256) && cancel() {
                        return Err(ParticleFluidError::Cancelled);
                    }
                    let u = ((sequence as f64 + 0.5) * 0.754_877_666_246_692_7).fract();
                    let angle = sequence as f64 * 2.399_963_229_728_653;
                    let r = e.radius * u.sqrt();
                    let vertical = r * angle.sin();
                    let across = r * angle.cos();
                    let sign = if end == 0 { 1.0 } else { -1.0 };
                    let axial = sign * (e.half_length + e.nozzle_depth * 0.5);
                    let profile = (1.0 - u).powi(2) * (0.75 + 0.25 * vertical / e.radius);
                    let velocity = axis.map(|a| a * sign * speed * profile.sqrt());
                    let position = [
                        e.center[0] + axis[0] * axial - axis[2] * across,
                        e.center[1] + vertical,
                        e.center[2] + axis[2] * axial + axis[0] * across,
                    ];
                    if self
                        .collision_mesh
                        .as_ref()
                        .is_some_and(|mesh| mesh.contains_or_touches(position))
                    {
                        return Err(ParticleFluidError::EmitterInsideCollisionSolid);
                    }
                    let mass = self.config.material.density * self.volume;
                    self.candidate.push(FluidParticle {
                        position,
                        velocity,
                        mass,
                        temperature_kelvin: self.config.material.temperature_kelvin,
                        material: 0,
                    });
                    emitted_mass += mass;
                    sequence = sequence.wrapping_add(1);
                }
                *credit -= emit as f64 * self.volume;
            }
        }
        self.heads.fill(NONE);
        for (i, p) in self.candidate.iter().enumerate() {
            let bin = cell(p.position, &self.grid, self.support, self.bins);
            self.next[i] = self.heads[bin];
            self.heads[bin] = i;
        }
        let particles = &self.candidate;
        let h = self.support;
        let h2 = h * h;
        let poly = 315.0 / (64.0 * std::f64::consts::PI * h.powi(9));
        let spiky = 45.0 / (std::f64::consts::PI * h.powi(6));
        let grid = &self.grid;
        let heads = &self.heads;
        let next = &self.next;
        let bins = self.bins;
        let count = particles.len();
        for lower in (0..count).step_by(16384) {
            if cancel() {
                return Err(ParticleFluidError::Cancelled);
            }
            let upper = (lower + 16384).min(count);
            crate::parallel::try_for_each_chunk_mut(
                &mut self.thermodynamic[lower..upper],
                CalculationStage::ParticleDensity,
                512,
                |start, out| {
                    let probe = WorkProbe::new(CalculationStage::ParticleDensity);
                    for (local, t) in out.iter_mut().enumerate() {
                        let i = lower + start + local;
                        let p = particles[i];
                        probe.visit(coordinate(p.position, grid));
                        let mut volume_density = 0.0;
                        neighbors(p.position, grid, h, bins, heads, next, |j| {
                            let q = particles[j];
                            let r2 = (0..3)
                                .map(|d| (p.position[d] - q.position[d]).powi(2))
                                .sum::<f64>();
                            if r2 < h2 {
                                volume_density += q.mass / materials[q.material as usize].density
                                    * poly
                                    * (h2 - r2).powi(3);
                            }
                        });
                        // Reference-volume density avoids interpreting a heavy
                        // neighboring material's mass as this particle's material.
                        let m = materials[p.material as usize];
                        t.density = m.density * volume_density.max(0.05);
                        t.pressure = m.pressure(
                            t.density,
                            p.temperature_kelvin,
                            self.config.liquid_sound_speed,
                        );
                        if !t.density.is_finite() || !t.pressure.is_finite() {
                            return Err(ParticleFluidError::ArithmeticFailure);
                        }
                    }
                    Ok(())
                },
            )?;
        }
        let thermodynamic = &self.thermodynamic;
        for lower in (0..count).step_by(16384) {
            if cancel() {
                return Err(ParticleFluidError::Cancelled);
            }
            let upper = (lower + 16384).min(count);
            crate::parallel::try_for_each_chunk_mut(
                &mut self.force[lower..upper],
                CalculationStage::ParticleForces,
                512,
                |start, out| {
                    let probe = WorkProbe::new(CalculationStage::ParticleForces);
                    for (local, f) in out.iter_mut().enumerate() {
                        let i = lower + start + local;
                        let p = particles[i];
                        let t = thermodynamic[i];
                        let m = materials[p.material as usize];
                        probe.visit(coordinate(p.position, grid));
                        *f = Force {
                            acceleration: self.config.gravity,
                            damping_rate: 0.0,
                        };
                        neighbors(p.position, grid, h, bins, heads, next, |j| {
                            if i == j {
                                return;
                            }
                            let q = particles[j];
                            let tj = thermodynamic[j];
                            let mj = materials[q.material as usize];
                            let delta =
                                std::array::from_fn::<_, 3, _>(|d| p.position[d] - q.position[d]);
                            let r = norm(delta);
                            if r >= h || r < 1e-10 * h {
                                return;
                            }
                            let gradient = -spiky * (h - r).powi(2) / r;
                            // Differentiate the reference-volume density's
                            // compression energy with the SAME poly6 kernel.
                            // Mixing spiky forces with poly6 density creates
                            // artificial energy when particles compress.
                            let pressure = -q.mass
                                * (t.pressure * m.density / (mj.density * t.density * t.density)
                                    + tj.pressure * mj.density
                                        / (m.density * tj.density * tj.density));
                            let pressure_gradient = -6.0 * poly * (h2 - r * r).powi(2);
                            let viscosity = q.mass * 0.5 * (m.viscosity + mj.viscosity)
                                / (t.density * tj.density)
                                * spiky
                                * (h - r);
                            let cohesion = if m.phase == FluidPhase::Liquid
                                && mj.phase == FluidPhase::Liquid
                                && p.material == q.material
                            {
                                0.5 * (m.surface_tension + mj.surface_tension)
                                    * (p.mass / t.density)
                                    * (q.mass / tj.density)
                                    / (p.mass * h)
                            } else {
                                0.0
                            };
                            for (d, a) in f.acceleration.iter_mut().enumerate() {
                                *a += (pressure * pressure_gradient + cohesion * gradient)
                                    * delta[d]
                                    + viscosity * (q.velocity[d] - p.velocity[d]);
                            }
                            f.damping_rate += viscosity;
                        });
                        if f.acceleration.iter().any(|v| !v.is_finite())
                            || !f.damping_rate.is_finite()
                        {
                            return Err(ParticleFluidError::ArithmeticFailure);
                        }
                    }
                    Ok(())
                },
            )?;
        }
        // Refuse unstable updates. The candidate/source can be replayed at a
        // shorter requested interval; there is no hidden force/speed clipping.
        let max_acceleration = self.force[..count]
            .iter()
            .map(|f| norm(f.acceleration))
            .fold(0.0_f64, f64::max);
        let max_damping = self.force[..count]
            .iter()
            .map(|f| f.damping_rate)
            .fold(0.0_f64, f64::max);
        if max_acceleration * dt * dt > 0.1 * h || max_damping * dt > 0.2 {
            let reduced = dt
                .min(if max_acceleration > 0.0 {
                    (0.1 * h / max_acceleration).sqrt() * 0.9
                } else {
                    dt
                })
                .min(if max_damping > 0.0 {
                    0.18 / max_damping
                } else {
                    dt
                });
            // Restart from accepted fields so source volume follows the actual
            // interval. Bounded step reduction converges or returns failure.
            if reduced >= dt * 0.99 || reduced < requested_dt * 1e-8 {
                return Err(ParticleFluidError::ArithmeticFailure);
            }
            return Err(ParticleFluidError::StabilityLimit {
                recommended_dt: reduced,
            });
        }
        let radius = self.spacing * 0.2;
        let origin = grid.origin();
        let upper = grid.upper();
        for lower in (0..count).step_by(16384) {
            if cancel() {
                return Err(ParticleFluidError::Cancelled);
            }
            let end = (lower + 16384).min(count);
            let forces = &self.force;
            crate::parallel::try_for_each_chunk_mut(
                &mut self.candidate[lower..end],
                CalculationStage::FieldUpdate,
                512,
                |start, out| {
                    let probe = WorkProbe::new(CalculationStage::FieldUpdate);
                    for (local, p) in out.iter_mut().enumerate() {
                        let i = lower + start + local;
                        let material = materials[p.material as usize];
                        probe.visit(coordinate(p.position, grid));
                        let previous_position = p.position;
                        for d in 0..3 {
                            p.velocity[d] += dt * forces[i].acceleration[d];
                            p.position[d] += dt * p.velocity[d];
                        }
                        for d in 0..3 {
                            let low = origin[d] + radius;
                            let high = upper[d] - radius;
                            if p.position[d] < low || p.position[d] > high {
                                p.position[d] = p.position[d].clamp(low, high);
                                if (p.position[d] <= low && p.velocity[d] < 0.0)
                                    || (p.position[d] >= high && p.velocity[d] > 0.0)
                                {
                                    p.velocity[d] = 0.0;
                                }
                            }
                            if p.position[d] - low < radius || high - p.position[d] < radius {
                                let drag = (-material.wall_drag * dt).exp();
                                for axis in 0..3 {
                                    if axis != d {
                                        p.velocity[axis] *= drag;
                                    }
                                }
                                p.temperature_kelvin = self.config.ambient_temperature_kelvin
                                    + (p.temperature_kelvin
                                        - self.config.ambient_temperature_kelvin)
                                        * (-self.config.wall_heat_exchange * dt).exp();
                            }
                        }
                        if let Some(mesh) = &self.collision_mesh {
                            let (position, velocity) = mesh
                                .resolve_segment(previous_position, p.position, p.velocity)
                                .map_err(|_| ParticleFluidError::CollisionQueryFailed)?;
                            p.position = position;
                            p.velocity = velocity;
                            // A convex interior is never a valid accepted result.
                            // Contact retreat normally keeps this branch inactive.
                            if mesh.contains_or_touches(p.position) {
                                return Err(ParticleFluidError::CollisionQueryFailed);
                            }
                        }
                        if p.position.iter().chain(&p.velocity).any(|v| !v.is_finite())
                            || !p.temperature_kelvin.is_finite()
                        {
                            return Err(ParticleFluidError::ArithmeticFailure);
                        }
                    }
                    Ok(())
                },
            )?;
        }
        if cancel() {
            return Err(ParticleFluidError::Cancelled);
        }
        let time = self.time + dt;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ParticleFluidError::ArithmeticFailure)?;
        if !time.is_finite() || time <= self.time {
            return Err(ParticleFluidError::ArithmeticFailure);
        }
        let total_mass = self.candidate.iter().map(|p| p.mass).sum::<f64>();
        let mean_temperature = if count == 0 {
            self.config.ambient_temperature_kelvin
        } else {
            self.candidate
                .iter()
                .map(|p| p.mass * p.temperature_kelvin)
                .sum::<f64>()
                / total_mass
        };
        let max_speed = self
            .candidate
            .iter()
            .map(|p| norm(p.velocity))
            .fold(0.0_f64, f64::max);
        let max_pressure = self.thermodynamic[..count]
            .iter()
            .map(|t| t.pressure)
            .fold(0.0_f64, f64::max);
        if !total_mass.is_finite() || !mean_temperature.is_finite() || !max_speed.is_finite() {
            return Err(ParticleFluidError::ArithmeticFailure);
        }
        std::mem::swap(&mut self.accepted, &mut self.candidate);
        self.time = time;
        self.generation = generation;
        self.credit = credit;
        self.emission_sequence = sequence;
        Ok(ParticleStepReport {
            dt,
            particles: count,
            emitted_mass,
            total_mass,
            max_speed,
            max_pressure,
            mean_temperature_kelvin: mean_temperature,
        })
    }
}
fn coordinate(p: [f64; 3], grid: &GridGeometry) -> [usize; 3] {
    std::array::from_fn(|d| {
        (((p[d] - grid.origin()[d]) / grid.spacing()[d]).floor() as usize).min(grid.counts()[d] - 1)
    })
}
fn bin_coordinate(p: [f64; 3], grid: &GridGeometry, h: f64, bins: [usize; 3]) -> [usize; 3] {
    std::array::from_fn(|d| (((p[d] - grid.origin()[d]) / h).floor() as usize).min(bins[d] - 1))
}
fn cell(p: [f64; 3], grid: &GridGeometry, h: f64, bins: [usize; 3]) -> usize {
    let [x, y, z] = bin_coordinate(p, grid, h, bins);
    x + bins[0] * (y + bins[1] * z)
}
fn neighbors(
    p: [f64; 3],
    grid: &GridGeometry,
    h: f64,
    bins: [usize; 3],
    heads: &[usize],
    next: &[usize],
    mut visit: impl FnMut(usize),
) {
    let c = bin_coordinate(p, grid, h, bins);
    for z in c[2].saturating_sub(1)..=(c[2] + 1).min(bins[2] - 1) {
        for y in c[1].saturating_sub(1)..=(c[1] + 1).min(bins[1] - 1) {
            for x in c[0].saturating_sub(1)..=(c[0] + 1).min(bins[0] - 1) {
                let mut j = heads[x + bins[0] * (y + bins[1] * z)];
                while j != NONE {
                    visit(j);
                    j = next[j];
                }
            }
        }
    }
}

#[cfg(test)]
mod physics_tests {
    use super::*;
    use crate::FluidPreset;
    fn fixture(side: f64, material: crate::FluidMaterial) -> ParticleFluidSimulation {
        let grid = GridGeometry::new([16; 3], [side / 16.0; 3], [0.0; 3]).unwrap();
        ParticleFluidSimulation::new(
            grid,
            ParticleFluidConfig {
                material,
                gravity: [0.0; 3],
                samples_per_axis: 8,
                max_particles: 64,
                ..ParticleFluidConfig::default()
            },
        )
        .unwrap()
    }
    fn parcel(
        position: [f64; 3],
        velocity: [f64; 3],
        mass: f64,
        temperature: f64,
    ) -> FluidParticle {
        FluidParticle {
            position,
            velocity,
            mass,
            temperature_kelvin: temperature,
            material: 0,
        }
    }
    fn collision_plane() -> Arc<CollisionMesh> {
        Arc::new(
            CollisionMesh::new(
                vec![
                    [0.5, 0.0, 0.0],
                    [0.5, 1.0, 0.0],
                    [0.5, 1.0, 1.0],
                    [0.5, 0.0, 1.0],
                ],
                vec![[0, 1, 2], [0, 2, 3]],
                crate::mesh_collision::MeshPolicy::TwoSidedSurface,
            )
            .unwrap(),
        )
    }
    fn collision_cube() -> Arc<CollisionMesh> {
        Arc::new(
            CollisionMesh::new(
                vec![
                    [0.25, 0.25, 0.25],
                    [0.75, 0.25, 0.25],
                    [0.75, 0.75, 0.25],
                    [0.25, 0.75, 0.25],
                    [0.25, 0.25, 0.75],
                    [0.75, 0.25, 0.75],
                    [0.75, 0.75, 0.75],
                    [0.25, 0.75, 0.75],
                ],
                vec![
                    [0, 2, 1],
                    [0, 3, 2],
                    [4, 5, 6],
                    [4, 6, 7],
                    [0, 1, 5],
                    [0, 5, 4],
                    [3, 7, 6],
                    [3, 6, 2],
                    [0, 4, 7],
                    [0, 7, 3],
                    [1, 2, 6],
                    [1, 6, 5],
                ],
                crate::mesh_collision::MeshPolicy::ConvexSolid,
            )
            .unwrap(),
        )
    }
    #[test]
    fn liquid_and_gas_hit_mesh_with_no_normal_penetration_and_keep_tangent_motion() {
        for preset in [FluidPreset::Water, FluidPreset::Air] {
            let mut s = fixture(1.0, preset.material());
            s.config.gravity = [10.0, 0.0, 0.0];
            s.accepted
                .push(parcel([0.49999, 0.5, 0.5], [1.0, 0.2, 0.0], 0.001, 293.15));
            s.set_collision_mesh(Some(collision_plane())).unwrap();
            let before_mass = s.accepted[0].mass;
            s.step(0.0001, None, || false).unwrap();
            assert!(s.particles()[0].position[0] < 0.5);
            assert!(s.particles()[0].velocity[0].abs() < 1e-15);
            assert_eq!(s.particles()[0].velocity[1], 0.2);
            let first_y = s.particles()[0].position[1];
            // Remove gravity to directly observe free tangential transport.
            s.config.gravity = [0.0; 3];
            for _ in 0..100 {
                s.step(0.0001, None, || false).unwrap();
            }
            assert!(s.particles()[0].position[0] < 0.5);
            assert!(s.particles()[0].position[1] > first_y + 0.001);
            assert_eq!(s.particles()[0].mass, before_mass);
        }
    }
    #[test]
    fn liquid_and_gas_convex_solid_exclude_fill_and_stop_approaching_particles() {
        for preset in [FluidPreset::Water, FluidPreset::Air] {
            let mut s = fixture(1.0, preset.material());
            s.accepted.extend([
                parcel([0.5; 3], [0.0; 3], 0.001, 293.15),
                parcel([0.24999, 0.5, 0.5], [1.0, 0.0, 0.0], 0.001, 293.15),
            ]);
            s.set_collision_mesh(Some(collision_cube())).unwrap();
            assert_eq!(s.particles().len(), 1);
            for _ in 0..100 {
                s.step(0.0001, None, || false).unwrap();
            }
            assert!(s.particles()[0].position[0] < 0.25);
            assert_eq!(s.particles()[0].velocity, [0.0; 3]);
        }
    }
    #[test]
    fn late_overlap_replacement_and_inside_emission_leave_accepted_state_unchanged() {
        let mut s = fixture(1.0, FluidPreset::Water.material());
        s.accepted.push(parcel([0.5; 3], [0.0; 3], 0.001, 293.15));
        s.step(1e-5, None, || false).unwrap();
        let before = s.accepted.clone();
        assert_eq!(
            s.set_collision_mesh(Some(collision_cube())).unwrap_err(),
            ParticleFluidError::CollisionMeshOverlapsParticles
        );
        assert_eq!(s.accepted, before);
        assert!(s.collision_mesh().is_none());
        let mut s = fixture(1.0, FluidPreset::Water.material());
        s.set_collision_mesh(Some(collision_cube())).unwrap();
        s.credit = [s.volume, 0.0];
        let before_credit = s.credit;
        let emitter = RotatingCylinderEmitter {
            rotations_per_second: 0.0,
            ..RotatingCylinderEmitter::default()
        };
        assert_eq!(
            s.step(1e-5, Some(emitter), || false).unwrap_err(),
            ParticleFluidError::EmitterInsideCollisionSolid
        );
        assert_eq!(s.credit, before_credit);
        assert_eq!(s.generation(), 0);
        assert_eq!(s.time(), 0.0);
        assert!(s.particles().is_empty());
        s.set_collision_mesh(None).unwrap();
        s.step(1e-5, Some(emitter), || false).unwrap();
        assert_eq!(s.particles().len(), 1);
    }
    #[test]
    fn gravity_accelerates_in_metres_per_second_squared_at_both_scene_scales() {
        for side in [0.01, 1.0] {
            let mut s = fixture(side, FluidPreset::Water.material());
            s.config.gravity = [0.0, -9.80665, 0.0];
            s.accepted
                .push(parcel([side * 0.5; 3], [0.0; 3], 1.0, 293.15));
            let dt = 1e-5;
            let r = s.step(dt, None, || false).unwrap();
            assert_eq!(r.dt, dt);
            assert!((s.particles()[0].velocity[1] + 9.80665 * dt).abs() < 1e-15);
            assert!(
                (s.particles()[0].position[1] - (side * 0.5 - 9.80665 * dt * dt)).abs() < 1e-15
            );
        }
    }
    #[test]
    fn gas_pair_expands_into_vacuum_with_equal_and_opposite_momentum() {
        let mut s = fixture(1.0, FluidPreset::Argon.material());
        s.accepted.extend([
            parcel([0.46, 0.5, 0.5], [0.0; 3], 0.001, 293.15),
            parcel([0.54, 0.5, 0.5], [0.0; 3], 0.001, 293.15),
        ]);
        let r = s.step(1e-6, None, || false).unwrap();
        assert!(r.max_pressure > 0.0);
        assert!(s.particles()[0].velocity[0] < 0.0 && s.particles()[1].velocity[0] > 0.0);
        let momentum = s
            .particles()
            .iter()
            .map(|p| p.mass * p.velocity[0])
            .sum::<f64>();
        assert!(momentum.abs() < 1e-14);
    }
    #[test]
    fn pressure_acceleration_matches_independent_compression_energy_gradient() {
        for mixed in [false, true] {
            let mut material = FluidPreset::Water.material();
            material.surface_tension = 0.0;
            material.viscosity = 0.0;
            let mut s = fixture(1.0, material);
            s.config.ambient_material.viscosity = 0.0;
            for i in 0..8 {
                let id = u8::from(mixed && i % 2 == 1);
                let m = if id == 0 {
                    material
                } else {
                    s.config.ambient_material
                };
                let mut p = parcel(
                    std::array::from_fn(|d| 0.48 + 0.03 * ((i >> d) & 1) as f64),
                    [0.0; 3],
                    m.density / 512.0,
                    293.15,
                );
                p.material = id;
                s.accepted.push(p);
            }
            // Independently integrate du = p/rho² d(rho), then differentiate
            // total energy numerically. No pressure-force kernel is reused.
            let energy = |particles: &[FluidParticle]| {
                let h = s.support;
                let materials = [s.config.material, s.config.ambient_material];
                particles
                    .iter()
                    .map(|p| {
                        let m = materials[p.material as usize];
                        let f = particles
                            .iter()
                            .map(|q| {
                                let r2 = (0..3)
                                    .map(|d| (p.position[d] - q.position[d]).powi(2))
                                    .sum::<f64>();
                                if r2 >= h * h {
                                    0.0
                                } else {
                                    q.mass / materials[q.material as usize].density * 315.0
                                        / (64.0 * std::f64::consts::PI * h.powi(9))
                                        * (h * h - r2).powi(3)
                                }
                            })
                            .sum::<f64>();
                        let u = match m.phase {
                            FluidPhase::Liquid => {
                                if f > 1.0 {
                                    400.0 * (f.ln() + 1.0 / f - 1.0)
                                } else {
                                    0.0
                                }
                            }
                            FluidPhase::Gas => m.gas_constant() * p.temperature_kelvin * f.ln(),
                        };
                        p.mass * u
                    })
                    .sum::<f64>()
            };
            let eps = 1e-7;
            let mut plus = s.accepted.clone();
            let mut minus = plus.clone();
            plus[0].position[0] += eps;
            minus[0].position[0] -= eps;
            let expected = -(energy(&plus) - energy(&minus)) / (2.0 * eps * s.accepted[0].mass);
            let dt = 1e-8;
            s.step(dt, None, || false).unwrap();
            let actual = s.particles()[0].velocity[0] / dt;
            assert!(
                (actual - expected).abs() < 1e-5 * expected.abs().max(1.0),
                "mixed={mixed}: acceleration {actual}, energy gradient {expected}"
            );
        }
    }
    #[test]
    fn viscosity_reduces_pair_kinetic_energy_without_changing_total_momentum() {
        let mut material = FluidPreset::Water.material();
        material.surface_tension = 0.0;
        material.viscosity = 0.1;
        let mut s = fixture(1.0, material);
        s.accepted.extend([
            parcel([0.46, 0.5, 0.5], [1.0, 0.0, 0.0], 0.1, 293.15),
            parcel([0.54, 0.5, 0.5], [-1.0, 0.0, 0.0], 0.1, 293.15),
        ]);
        let before = s
            .particles()
            .iter()
            .map(|p| 0.5 * p.mass * norm(p.velocity).powi(2))
            .sum::<f64>();
        s.step(1e-4, None, || false).unwrap();
        let after = s
            .particles()
            .iter()
            .map(|p| 0.5 * p.mass * norm(p.velocity).powi(2))
            .sum::<f64>();
        assert!(after < before && after > 0.0);
        assert!(
            s.particles()
                .iter()
                .map(|p| p.mass * p.velocity[0])
                .sum::<f64>()
                .abs()
                < 1e-14
        );
    }
    #[test]
    fn unlike_material_pressure_conserves_momentum_and_respects_particle_inertia() {
        let mut material = FluidPreset::Water.material();
        material.viscosity = 0.0;
        material.surface_tension = 0.0;
        let mut s = fixture(1.0, material);
        s.config.ambient_material = FluidPreset::Air.material();
        s.config.ambient_material.viscosity = 0.0;
        let light_mass = s.config.ambient_material.density * 0.001;
        let mut light = parcel([0.54, 0.5, 0.5], [0.0; 3], light_mass, 293.15);
        light.material = 1;
        s.accepted.extend([
            parcel([0.46, 0.5, 0.5], [0.0; 3], material.density * 0.001, 293.15),
            light,
        ]);
        s.step(1e-7, None, || false).unwrap();
        let p = s.particles();
        assert!(p[0].velocity[0].abs() < p[1].velocity[0].abs() / 500.0);
        assert!((p[0].mass * p[0].velocity[0] + p[1].mass * p[1].velocity[0]).abs() < 1e-14);
    }
    #[test]
    fn surface_cohesion_draws_liquid_parcels_together() {
        let mut material = FluidPreset::Water.material();
        material.viscosity = 0.0;
        let mut s = fixture(1.0, material);
        s.accepted.extend([
            parcel([0.46, 0.5, 0.5], [0.0; 3], 0.1, 293.15),
            parcel([0.54, 0.5, 0.5], [0.0; 3], 0.1, 293.15),
        ]);
        s.step(1e-4, None, || false).unwrap();
        assert!(s.particles()[0].velocity[0] > 0.0 && s.particles()[1].velocity[0] < 0.0);
        assert!(
            s.particles()
                .iter()
                .map(|p| p.mass * p.velocity[0])
                .sum::<f64>()
                .abs()
                < 1e-14
        );
    }
    #[test]
    fn wall_drag_and_temperature_use_contact_rates_without_bulk_vacuum_cooling() {
        let mut material = FluidPreset::Water.material();
        material.wall_drag = 10.0;
        material.temperature_kelvin = 313.15;
        let mut s = fixture(1.0, material);
        s.config.wall_heat_exchange = 2.0;
        s.accepted
            .push(parcel([0.5, 0.03, 0.5], [1.0, 0.0, 0.0], 1.0, 313.15));
        let dt = 0.001;
        s.step(dt, None, || false).unwrap();
        let p = s.particles()[0];
        assert!((p.velocity[0] - (-10.0_f64 * dt).exp()).abs() < 1e-14);
        assert!((p.temperature_kelvin - (293.15 + 20.0 * (-2.0_f64 * dt).exp())).abs() < 1e-12);
        let mut bulk = fixture(1.0, material);
        bulk.accepted.push(parcel([0.5; 3], [0.0; 3], 1.0, 313.15));
        bulk.step(dt, None, || false).unwrap();
        assert_eq!(bulk.particles()[0].temperature_kelvin, 313.15);
    }
}
