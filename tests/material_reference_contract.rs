//! Independent small all-pairs references for the public material API.
//!
//! The production linked-cell/directed-neighbor loop is not reused. Each
//! unordered pair contributes a single equal/opposite force here. A separate
//! compression-energy finite difference checks the pressure force without
//! using any force kernel. These are finite numerical fixtures, not continuum
//! validation, general stability, or an IEEE refinement of the Lean equations.
use rheon::{
    DomainFill, FluidMaterial, FluidParticle, FluidPhase, FluidPreset, GridGeometry,
    ParticleFluidConfig, ParticleFluidSimulation, RotatingCylinderEmitter,
};
use std::f64::consts::PI;

fn close(actual: f64, expected: f64, absolute: f64, relative: f64) {
    assert!(actual.is_finite() && expected.is_finite());
    assert!(
        (actual - expected).abs() <= absolute + relative * expected.abs(),
        "actual={actual:.17e}; independent={expected:.17e}"
    );
}

fn material(preset: FluidPreset) -> FluidMaterial {
    FluidMaterial {
        viscosity: 0.0,
        surface_tension: 0.0,
        wall_drag: 0.0,
        ..preset.material()
    }
}

fn config(m: FluidMaterial) -> ParticleFluidConfig {
    ParticleFluidConfig {
        material: m,
        ambient_material: material(FluidPreset::Mercury),
        gravity: [0.0; 3],
        fill: DomainFill::EmittedFluid,
        samples_per_axis: 4,
        max_particles: 128,
        wall_heat_exchange: 0.0,
        ..ParticleFluidConfig::default()
    }
}

fn simulation(c: ParticleFluidConfig) -> ParticleFluidSimulation {
    // Shifted origin and unequal sides exercise physical coordinates rather
    // than a unit-cube indexing coincidence. There are 4*3*4=48 initial parcels.
    ParticleFluidSimulation::new(
        GridGeometry::new([12, 9, 12], [1.0 / 12.0; 3], [-0.2, 0.1, -0.3]).unwrap(),
        c,
    )
    .unwrap()
}

fn thermodynamics(
    particles: &[FluidParticle],
    materials: [FluidMaterial; 2],
    h: f64,
    sound: f64,
) -> Vec<(f64, f64, f64)> {
    // Dimensionless form W = 315/(64*pi*h^3) * (1-r^2/h^2)^3,
    // algebraically equivalent to poly6 but evaluated independently.
    particles
        .iter()
        .map(|p| {
            let m = materials[p.material as usize];
            let fraction = particles
                .iter()
                .map(|q| {
                    let s = (0..3)
                        .map(|d| ((p.position[d] - q.position[d]) / h).powi(2))
                        .sum::<f64>();
                    q.mass / materials[q.material as usize].density * 315.0
                        / (64.0 * PI * h.powi(3))
                        * (1.0 - s).max(0.0).powi(3)
                })
                .sum::<f64>();
            let density = m.density * fraction.max(0.05);
            let pressure = match m.phase {
                FluidPhase::Liquid => (sound.powi(2) * (density - m.density)).max(0.0),
                FluidPhase::Gas => {
                    density * (8.314_462_618_153_24 / m.molar_mass) * p.temperature_kelvin
                }
            };
            (density, pressure, fraction)
        })
        .collect()
}

fn all_pair_accelerations(
    particles: &[FluidParticle],
    materials: [FluidMaterial; 2],
    h: f64,
    sound: f64,
) -> Vec<[f64; 3]> {
    let values = thermodynamics(particles, materials, h, sound);
    let mut force = vec![[0.0; 3]; particles.len()];
    for i in 0..particles.len() {
        for j in i + 1..particles.len() {
            let (p, q) = (particles[i], particles[j]);
            let (mi, mj) = (
                materials[p.material as usize],
                materials[q.material as usize],
            );
            let delta: [f64; 3] = std::array::from_fn(|d| p.position[d] - q.position[d]);
            let r = delta.iter().map(|v| v * v).sum::<f64>().sqrt();
            if r >= h || r < 1e-10 * h {
                continue;
            }
            let (di, pi, _) = values[i];
            let (dj, pj, _) = values[j];
            let s = r / h;
            // The vector pressure derivative is positive in the direction
            // away from the other particle. Reference-volume factors remain.
            let pressure = p.mass
                * q.mass
                * (pi * mi.density / (mj.density * di.powi(2))
                    + pj * mj.density / (mi.density * dj.powi(2)))
                * 945.0
                / (32.0 * PI * h.powi(5))
                * (1.0 - s * s).powi(2);
            let viscosity = p.mass * q.mass * (mi.viscosity + mj.viscosity) * 45.0
                / (2.0 * PI * di * dj * h.powi(5))
                * (1.0 - s);
            let cohesion = if p.material == q.material && mi.phase == FluidPhase::Liquid {
                -(mi.surface_tension + mj.surface_tension) * p.mass * q.mass * 45.0
                    / (2.0 * PI * di * dj * h.powi(5) * r)
                    * (1.0 - s).powi(2)
            } else {
                0.0
            };
            for (d, displacement) in delta.into_iter().enumerate() {
                let pair = (pressure + cohesion) * displacement
                    + viscosity * (q.velocity[d] - p.velocity[d]);
                force[i][d] += pair;
                force[j][d] -= pair;
            }
        }
    }
    force.iter_mut().zip(particles).for_each(|(f, p)| {
        f.iter_mut().for_each(|v| *v /= p.mass);
    });
    force
}

fn assert_one_step_matches_dense_reference(sim: &mut ParticleFluidSimulation, dt: f64) {
    let before = sim.particles().to_vec();
    let c = sim.config();
    let expected = all_pair_accelerations(
        &before,
        [c.material, c.ambient_material],
        2.0 * sim.particle_spacing(),
        c.liquid_sound_speed,
    );
    assert!(expected.iter().flatten().any(|a| a.abs() > 1e-3));
    let report = sim.step(dt, None, || false).unwrap();
    assert_eq!(report.dt, dt, "reference fixture must not adapt its step");
    for ((old, new), acceleration) in before.iter().zip(sim.particles()).zip(&expected) {
        for (d, a) in acceleration.iter().enumerate() {
            let velocity = old.velocity[d] + dt * a;
            close(new.velocity[d], velocity, 2e-12, 2e-11);
            close(
                new.position[d],
                old.position[d] + dt * velocity,
                2e-14,
                2e-13,
            );
        }
        assert_eq!(old.mass.to_bits(), new.mass.to_bits());
        assert_eq!(old.material, new.material);
    }
    for d in 0..3 {
        let momentum_change = sim
            .particles()
            .iter()
            .zip(&before)
            .map(|(new, old)| old.mass * (new.velocity[d] - old.velocity[d]))
            .sum::<f64>();
        close(momentum_change, 0.0, 2e-10, 0.0);
    }
}

#[test]
fn all_particles_match_unordered_pair_reference_for_liquid_gas_and_viscosity() {
    for preset in [FluidPreset::Water, FluidPreset::Argon] {
        let mut m = material(preset);
        // Cohesion participates for water and is absent for argon.
        m.surface_tension = if preset == FluidPreset::Water {
            0.0728
        } else {
            0.0
        };
        m.viscosity = 0.2;
        let mut s = simulation(config(m));
        assert_eq!(s.particles().len(), 48);
        assert_one_step_matches_dense_reference(&mut s, 1e-6);
        // The second call has nonuniform velocities, so the viscosity check
        // cannot pass merely because every initial relative velocity is zero.
        let c = s.config();
        let full = all_pair_accelerations(
            s.particles(),
            [c.material, c.ambient_material],
            2.0 * s.particle_spacing(),
            c.liquid_sound_speed,
        );
        let mut inviscid = c.material;
        inviscid.viscosity = 0.0;
        let other = all_pair_accelerations(
            s.particles(),
            [inviscid, c.ambient_material],
            2.0 * s.particle_spacing(),
            c.liquid_sound_speed,
        );
        let mut work = 0.0;
        let mut largest = 0.0_f64;
        for ((p, a), b) in s.particles().iter().zip(full).zip(other) {
            for d in 0..3 {
                largest = largest.max((a[d] - b[d]).abs());
                work += p.mass * p.velocity[d] * (a[d] - b[d]);
            }
        }
        assert!(
            largest > 1e-8 && work < 0.0,
            "active viscosity: {largest}, {work}"
        );
        assert_one_step_matches_dense_reference(&mut s, 1e-6);
    }
}

#[test]
fn admitted_thin_gas_domain_exercises_density_floor_without_an_energy_gradient_claim() {
    let c = config(material(FluidPreset::Argon));
    let grid = GridGeometry::new([10, 2, 2], [0.1, 0.051, 0.051], [-0.2, 0.1, -0.3]).unwrap();
    let mut s = ParticleFluidSimulation::new(grid, c).unwrap();
    assert_eq!(s.particles().len(), 4);
    let values = thermodynamics(
        s.particles(),
        [c.material, c.ambient_material],
        2.0 * s.particle_spacing(),
        c.liquid_sound_speed,
    );
    assert!(values.iter().any(|(_, _, fraction)| *fraction < 0.05));
    assert!(values.iter().any(|(_, _, fraction)| *fraction > 0.05));
    assert_one_step_matches_dense_reference(&mut s, 1e-7);
}

fn compression_energy(particles: &[FluidParticle], c: ParticleFluidConfig, h: f64) -> f64 {
    let materials = [c.material, c.ambient_material];
    for (i, p) in particles.iter().enumerate() {
        for q in &particles[..i] {
            let distance_squared = (0..3)
                .map(|d| (p.position[d] - q.position[d]).powi(2))
                .sum::<f64>();
            assert!(distance_squared > (1e-10 * h).powi(2));
        }
    }
    thermodynamics(particles, materials, h, c.liquid_sound_speed)
        .iter()
        .zip(particles)
        .map(|((_, _, fraction), p)| {
            // Restrict the witness to smooth, unfloored density and remain
            // away from the liquid-law kink. No derivative through max is assumed.
            assert!(*fraction > 0.05 + 1e-3);
            let m = materials[p.material as usize];
            if m.phase == FluidPhase::Liquid {
                assert!((*fraction - 1.0).abs() > 1e-4);
            }
            let u = match m.phase {
                FluidPhase::Liquid if *fraction > 1.0 => {
                    c.liquid_sound_speed.powi(2) * (fraction.ln() + 1.0 / fraction - 1.0)
                }
                FluidPhase::Liquid => 0.0,
                FluidPhase::Gas => {
                    (8.314_462_618_153_24 / m.molar_mass) * p.temperature_kelvin * fraction.ln()
                }
            };
            p.mass * u
        })
        .sum()
}

#[test]
fn pressure_matches_energy_gradient_in_all_coordinates_including_unlike_materials() {
    for mixed in [false, true] {
        let mut c = config(material(FluidPreset::Water));
        if mixed {
            c.fill = DomainFill::OtherFluid;
        }
        let mut s = simulation(c);
        if mixed {
            let emitter = RotatingCylinderEmitter {
                center: [0.3, 0.475, 0.2],
                half_length: 0.1,
                radius: 0.2,
                nozzle_depth: 0.08,
                rotations_per_second: 0.0,
                pressure_heads: [1e6, 0.0],
                ..RotatingCylinderEmitter::default()
            };
            for _ in 0..32 {
                s.step(0.002, Some(emitter), || false).unwrap();
                if s.particles().iter().any(|p| p.material == 0) {
                    break;
                }
            }
            assert!(s.particles().iter().any(|p| p.material == 0));
            assert!(s.particles().iter().any(|p| p.material == 1));
        }
        let before = s.particles().to_vec();
        let h = 2.0 * s.particle_spacing();
        let dt = 1e-7;
        let report = s.step(dt, None, || false).unwrap();
        assert_eq!(report.dt, dt);
        let eps = 1e-6;
        let mut active = 0;
        for i in 0..before.len() {
            for d in 0..3 {
                let mut plus = before.clone();
                let mut minus = before.clone();
                plus[i].position[d] += eps;
                minus[i].position[d] -= eps;
                let reference = -(compression_energy(&plus, c, h)
                    - compression_energy(&minus, c, h))
                    / (2.0 * eps * before[i].mass);
                let measured = (s.particles()[i].velocity[d] - before[i].velocity[d]) / dt;
                close(measured, reference, 2e-5, 3e-6);
                active += usize::from(reference.abs() > 0.01);
            }
        }
        assert!(
            active >= 24,
            "energy fixture must contain nonzero gradients"
        );
    }
}

#[test]
fn first_emitted_parcel_matches_head_profile_midpoint_rotation_and_volume_credit() {
    let mut c = config(material(FluidPreset::Water));
    c.fill = DomainFill::Vacuum;
    c.samples_per_axis = 8;
    c.liquid_sound_speed = 1.0;
    let mut s = simulation(c);
    let e = RotatingCylinderEmitter {
        center: [0.3, 0.475, 0.2],
        half_length: 0.1,
        radius: 0.2,
        nozzle_depth: 0.08,
        pressure_heads: [1000.0, 0.0],
        rotations_per_second: 0.5,
        ..RotatingCylinderEmitter::default()
    };
    let dt = 0.01;
    let parcel_volume = (1.0_f64 / 8.0).powi(3);
    let peak_speed = (2000.0 / c.material.density).sqrt();
    let flux = PI * 0.2_f64.powi(2) * peak_speed * 0.43;
    let first_step = (parcel_volume / (flux * dt)).ceil() as usize;
    assert!((2..=8).contains(&first_step));
    for step in 1..=first_step {
        let before_time = s.time();
        let report = s.step(dt, Some(e), || false).unwrap();
        assert_eq!(report.dt, dt);
        if step < first_step {
            assert_eq!(report.particles, 0);
            assert_eq!(report.emitted_mass, 0.0);
            continue;
        }
        assert_eq!(report.particles, 1);
        let p = s.particles()[0];
        close(p.mass, c.material.density * parcel_volume, 0.0, 1e-15);
        close(report.emitted_mass, p.mass, 0.0, 0.0);
        let u: f64 = 0.5 * 0.754_877_666_246_692_7;
        let profile = (1.0 - u).powi(2) * 0.75;
        let speed = (2.0 * e.pressure_heads[0] * profile / c.material.density).sqrt();
        let angle = 2.0 * PI * e.rotations_per_second * (before_time + dt / 2.0);
        let axis = [angle.cos(), 0.0, angle.sin()];
        let across = e.radius * u.sqrt();
        let axial = e.half_length + e.nozzle_depth / 2.0;
        let initial = [
            e.center[0] + axis[0] * axial - axis[2] * across,
            e.center[1],
            e.center[2] + axis[2] * axial + axis[0] * across,
        ];
        for d in 0..3 {
            close(p.velocity[d], axis[d] * speed, 1e-14, 1e-13);
            close(
                p.position[d],
                initial[d] + dt * axis[d] * speed,
                1e-14,
                1e-13,
            );
        }
        close(
            0.5 * c.material.density * p.velocity.iter().map(|v| v * v).sum::<f64>(),
            e.pressure_heads[0] * profile,
            1e-12,
            1e-13,
        );
    }
}
