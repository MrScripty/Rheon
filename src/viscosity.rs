//! Constant Newtonian viscosity on a fully filled, sealed Cartesian MAC box.
//! Stationary impermeable walls have zero tangential traction (free slip).
//! The assembled quadratic form is the symmetric strain, not vector smoothing.
use crate::{Axis, GridGeometry};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViscosityStage {
    BeforeAssembly,
    StrainSlice,
    BeforeUpdate,
    UpdateSlice,
    BeforeAcceptance,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ViscosityError {
    InvalidCoefficient,
    InvalidDensity,
    InvalidTimeStep,
    ShapeMismatch,
    NonzeroNormalWall,
    UnsupportedDomain,
    ArithmeticFailure,
    StabilityLimit { actual: f64, limit: f64 },
    EnergyLimit,
    Cancelled { stage: ViscosityStage },
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
}
impl fmt::Display for ViscosityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Newtonian box viscosity rejected operation: {self:?}")
    }
}
impl std::error::Error for ViscosityError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViscosityReport {
    /// Dynamic viscosity, Pa s = kg/(m s).
    pub dynamic_viscosity: f64,
    /// mu/rho, m²/s.
    pub kinematic_viscosity: f64,
    /// nu * dt * sum(1/h²), required <= 1/4. No automatic substeps.
    pub stability_number: f64,
    pub dt: f64,
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    /// mu * V * [2 sum(normal strain²) + sum(engineering shear²)], W.
    pub dissipation_before: f64,
    pub update_energy: f64,
    /// Work of converting the proposed update to stored f32, J.
    pub rounding_work: f64,
    pub energy_identity_error: f64,
    pub energy_rounding_budget: f64,
}

/// Three face-sized f64 arrays, capacity charged to an explicit cap. Scratch
/// first holds K u, then the fully checked f32 candidate (represented in f64).
/// No accepted state, per-step allocation, pressure workspace or hidden snapshot.
pub struct ViscosityWorkspace {
    grid: GridGeometry,
    scratch: [Vec<f64>; 3],
    allocated_bytes: usize,
}
fn checked(x: f64) -> Result<f64, ViscosityError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(ViscosityError::ArithmeticFailure)
    }
}
fn mul(a: f64, b: f64) -> Result<f64, ViscosityError> {
    let x = checked(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(ViscosityError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn div(a: f64, b: f64) -> Result<f64, ViscosityError> {
    if b == 0.0 || !b.is_normal() {
        return Err(ViscosityError::ArithmeticFailure);
    }
    let x = checked(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(ViscosityError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(ViscosityStage) -> bool,
    stage: ViscosityStage,
) -> Result<(), ViscosityError> {
    if cancel(stage) {
        Err(ViscosityError::Cancelled { stage })
    } else {
        Ok(())
    }
}
#[derive(Default)]
struct Sum {
    value: f64,
    correction: f64,
}
impl Sum {
    fn add(&mut self, x: f64) -> Result<(), ViscosityError> {
        let next = checked(self.value + x)?;
        let error = if self.value.abs() >= x.abs() {
            checked(checked(self.value - next)? + x)?
        } else {
            checked(checked(x - next)? + self.value)?
        };
        self.correction = checked(self.correction + error)?;
        self.value = next;
        Ok(())
    }
    fn finish(self) -> Result<f64, ViscosityError> {
        checked(self.value + self.correction)
    }
}
impl ViscosityWorkspace {
    pub fn new(grid: GridGeometry, limit: usize) -> Result<Self, ViscosityError> {
        let required = Axis::ALL.into_iter().try_fold(0_usize, |total, axis| {
            total
                .checked_add(
                    grid.face_len(axis)
                        .checked_mul(8)
                        .ok_or(ViscosityError::AllocationFailed)?,
                )
                .ok_or(ViscosityError::AllocationFailed)
        })?;
        if required > limit {
            return Err(ViscosityError::BufferLimit { required, limit });
        }
        let mut remaining = limit;
        let mut allocate = |n: usize| -> Result<Vec<f64>, ViscosityError> {
            let required = n.checked_mul(8).ok_or(ViscosityError::AllocationFailed)?;
            if required > remaining {
                return Err(ViscosityError::BufferLimit {
                    required,
                    limit: remaining,
                });
            }
            let mut v = Vec::new();
            v.try_reserve_exact(n)
                .map_err(|_| ViscosityError::AllocationFailed)?;
            let actual = v
                .capacity()
                .checked_mul(8)
                .ok_or(ViscosityError::AllocationFailed)?;
            if actual > remaining {
                return Err(ViscosityError::BufferLimit {
                    required: actual,
                    limit: remaining,
                });
            }
            remaining -= actual;
            v.resize(n, 0.0);
            Ok(v)
        };
        let scratch = [
            allocate(grid.face_len(Axis::X))?,
            allocate(grid.face_len(Axis::Y))?,
            allocate(grid.face_len(Axis::Z))?,
        ];
        Ok(Self {
            grid,
            scratch,
            allocated_bytes: limit - remaining,
        })
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn grid(&self) -> &GridGeometry {
        &self.grid
    }
    pub(crate) fn matches(&self, grid: &GridGeometry) -> bool {
        self.grid == *grid
    }
    fn add(&mut self, d: usize, p: [usize; 3], value: f64) -> Result<(), ViscosityError> {
        // Outer normal faces are prescribed zero and are not degrees of freedom.
        if p[d] == 0 || p[d] == self.grid.counts()[d] {
            return Ok(());
        }
        let i = self
            .grid
            .face_index(Axis::ALL[d], p)
            .ok_or(ViscosityError::ShapeMismatch)?;
        self.scratch[d][i] = checked(self.scratch[d][i] + value)?;
        Ok(())
    }
    pub(crate) fn prepare(
        &mut self,
        velocity: [&[f32]; 3],
        density: f64,
        dynamic_viscosity: f64,
        dt: f64,
        mut cancel: impl FnMut(ViscosityStage) -> bool,
    ) -> Result<ViscosityReport, ViscosityError> {
        if !density.is_normal() || density <= 0.0 {
            return Err(ViscosityError::InvalidDensity);
        }
        if dynamic_viscosity < 0.0 || !(dynamic_viscosity == 0.0 || dynamic_viscosity.is_normal()) {
            return Err(ViscosityError::InvalidCoefficient);
        }
        if !dt.is_normal() || dt <= 0.0 {
            return Err(ViscosityError::InvalidTimeStep);
        }
        let n = self.grid.counts();
        let h = self.grid.spacing();
        let nu = div(dynamic_viscosity, density)?;
        let scale = mul(nu, dt)?;
        let inverse_square_sum = h
            .into_iter()
            .try_fold(0.0, |s, h| checked(s + div(1.0, mul(h, h)?)?))?;
        let stability = mul(scale, inverse_square_sum)?;
        if stability > 0.25 {
            return Err(ViscosityError::StabilityLimit {
                actual: stability,
                limit: 0.25,
            });
        }
        for d in 0..3 {
            if velocity[d].len() != self.scratch[d].len() {
                return Err(ViscosityError::ShapeMismatch);
            }
            let m = self.grid.face_counts(Axis::ALL[d]);
            for k in 0..m[2] {
                for j in 0..m[1] {
                    for i in 0..m[0] {
                        let p = [i, j, k];
                        let u = velocity[d][self.grid.face_index(Axis::ALL[d], p).unwrap()];
                        if !(u == 0.0 || u.is_normal()) {
                            return Err(ViscosityError::ArithmeticFailure);
                        }
                        if (p[d] == 0 || p[d] == n[d]) && u != 0.0 {
                            return Err(ViscosityError::NonzeroNormalWall);
                        }
                    }
                }
            }
        }
        checkpoint(&mut cancel, ViscosityStage::BeforeAssembly)?;
        for v in &mut self.scratch {
            v.fill(0.0);
        }
        let mut strain_square = Sum::default();
        // Normal strains at cell centers, weight 2V. K = E^T W E / V.
        for k in 0..n[2] {
            checkpoint(&mut cancel, ViscosityStage::StrainSlice)?;
            for j in 0..n[1] {
                for i in 0..n[0] {
                    for d in 0..3 {
                        let p = [i, j, k];
                        let mut q = p;
                        q[d] += 1;
                        let a =
                            f64::from(velocity[d][self.grid.face_index(Axis::ALL[d], p).unwrap()]);
                        let b =
                            f64::from(velocity[d][self.grid.face_index(Axis::ALL[d], q).unwrap()]);
                        let e = div(checked(b - a)?, h[d])?;
                        strain_square.add(mul(2.0, mul(e, e)?)?)?;
                        let stress = div(mul(2.0, e)?, h[d])?;
                        self.add(d, p, -stress)?;
                        self.add(d, q, stress)?;
                    }
                }
            }
        }
        // Engineering shear u_a,b + u_b,a at interior edges. Boundary
        // shears are zero tangential traction; no ghost unknowns or penalties.
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            let mut m = n;
            m[a] += 1;
            m[b] += 1;
            for k in 0..m[2] {
                checkpoint(&mut cancel, ViscosityStage::StrainSlice)?;
                for j in 0..m[1] {
                    for i in 0..m[0] {
                        let q = [i, j, k];
                        if q[a] == 0 || q[a] == n[a] || q[b] == 0 || q[b] == n[b] {
                            continue;
                        }
                        let mut pa = q;
                        pa[b] -= 1;
                        let mut pb = q;
                        pb[a] -= 1;
                        let derivative = |d: usize, p, q, h| {
                            div(
                                checked(
                                    f64::from(
                                        velocity[d][self.grid.face_index(Axis::ALL[d], q).unwrap()],
                                    ) - f64::from(
                                        velocity[d][self.grid.face_index(Axis::ALL[d], p).unwrap()],
                                    ),
                                )?,
                                h,
                            )
                        };
                        let e = checked(derivative(a, pa, q, h[b])? + derivative(b, pb, q, h[a])?)?;
                        strain_square.add(mul(e, e)?)?;
                        let sa = div(e, h[b])?;
                        let sb = div(e, h[a])?;
                        self.add(a, pa, -sa)?;
                        self.add(a, q, sa)?;
                        self.add(b, pb, -sb)?;
                        self.add(b, q, sb)?;
                    }
                }
            }
        }
        checkpoint(&mut cancel, ViscosityStage::BeforeUpdate)?;
        let mass = mul(density, self.grid.cell_volume())?;
        let dissipation = mul(
            mul(dynamic_viscosity, self.grid.cell_volume())?,
            strain_square.finish()?,
        )?;
        let mut before = Sum::default();
        let mut after = Sum::default();
        let mut update_energy = Sum::default();
        let mut rounding_work = Sum::default();
        let mut rounding_budget = Sum::default();
        for (d, field) in velocity.iter().enumerate() {
            let m = self.grid.face_counts(Axis::ALL[d]);
            for k in 0..m[2] {
                checkpoint(&mut cancel, ViscosityStage::UpdateSlice)?;
                for j in 0..m[1] {
                    for i in 0..m[0] {
                        let idx = self.grid.face_index(Axis::ALL[d], [i, j, k]).unwrap();
                        let old = f64::from(field[idx]);
                        let delta = -mul(scale, self.scratch[d][idx])?;
                        let proposed = checked(old + delta)?;
                        let stored = proposed as f32;
                        if !(stored == 0.0 || stored.is_normal())
                            || (stored == 0.0 && proposed != 0.0)
                        {
                            return Err(ViscosityError::ArithmeticFailure);
                        }
                        let new = f64::from(stored);
                        let rounding = checked(new - proposed)?;
                        before.add(mul(0.5, mul(mass, mul(old, old)?)?)?)?;
                        after.add(mul(0.5, mul(mass, mul(new, new)?)?)?)?;
                        update_energy.add(mul(0.5, mul(mass, mul(delta, delta)?)?)?)?;
                        let work = mul(mass, mul(rounding, checked(proposed + 0.5 * rounding)?)?)?;
                        rounding_work.add(work)?;
                        rounding_budget.add(work.abs())?;
                        self.scratch[d][idx] = new;
                    }
                }
            }
        }
        let before = before.finish()?;
        let after = after.finish()?;
        let update_energy = update_energy.finish()?;
        let rounding_work = rounding_work.finish()?;
        let rounding_budget = rounding_budget.finish()?;
        let loss = mul(dt, dissipation)?;
        let identity_error = checked(after - before + loss - update_energy - rounding_work)?;
        let floating_budget = mul(
            64.0 * f64::EPSILON,
            checked(before + after + loss + update_energy + rounding_budget)?,
        )?;
        let budget = checked(rounding_budget + floating_budget)?;
        if identity_error.abs() > floating_budget || after - before > budget {
            return Err(ViscosityError::EnergyLimit);
        }
        checkpoint(&mut cancel, ViscosityStage::BeforeAcceptance)?;
        Ok(ViscosityReport {
            dynamic_viscosity,
            kinematic_viscosity: nu,
            stability_number: stability,
            dt,
            kinetic_before: before,
            kinetic_after: after,
            dissipation_before: dissipation,
            update_energy,
            rounding_work,
            energy_identity_error: identity_error,
            energy_rounding_budget: budget,
        })
    }
    pub(crate) fn publish(&self, output: [&mut [f32]; 3]) {
        for (out, candidate) in output.into_iter().zip(&self.scratch) {
            for (v, &u) in out.iter_mut().zip(candidate) {
                *v = u as f32;
            }
        }
    }
    /// Isolated viscosity stage. Output changes only after every numerical gate
    /// and the final callback succeed. Input and output shapes are checked first.
    #[allow(clippy::too_many_arguments)] // Explicit physical inputs and transactional output.
    pub fn update(
        &mut self,
        velocity: [&[f32]; 3],
        density: f64,
        dynamic_viscosity: f64,
        dt: f64,
        output: [&mut [f32]; 3],
        cancel: impl FnMut(ViscosityStage) -> bool,
    ) -> Result<ViscosityReport, ViscosityError> {
        if output
            .iter()
            .zip(&self.scratch)
            .any(|(a, b)| a.len() != b.len())
        {
            return Err(ViscosityError::ShapeMismatch);
        }
        let report = self.prepare(velocity, density, dynamic_viscosity, dt, cancel)?;
        self.publish(output);
        Ok(report)
    }
}
