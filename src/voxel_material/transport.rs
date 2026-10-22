//! One shared volume/phase/mass-flux generation for a disposable candidate.
use super::{
    VoxelMaterialError, VoxelMaterialState, allocate, capacity, check_limit, checked, coordinate,
    divide, face_mass, faces, multiply, payload, roundoff, sum, upper,
};
use crate::{Axis, GridGeometry};

const COURANT_LIMIT: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
pub struct TransportReport {
    pub phase_volumes: [f64; 2],
    pub phase_budgets: [f64; 2],
    pub correction_max: f64,
    pub correction_sum: f64,
    pub residual_sum: f64,
    pub volume_courant: f64,
    pub mass_courant: f64,
    pub cell_continuity_max: f64,
    pub dual_continuity_max: f64,
}

pub(crate) struct TransportWorkspace {
    grid: GridGeometry,
    q: [Vec<f64>; 3],
    h: [Vec<f64>; 3],
    j: [Vec<f64>; 3],
    residual: Vec<f64>,
    mass_defect: Vec<f64>,
    allocated_bytes: usize,
}
pub(crate) fn phase_volumes(
    grid: &GridGeometry,
    f: &[f64],
) -> Result<[f64; 2], VoxelMaterialError> {
    Ok([
        multiply(sum(f.iter().copied())?, grid.cell_volume())?,
        multiply(sum(f.iter().map(|f| 1.0 - f))?, grid.cell_volume())?,
    ])
}
impl TransportWorkspace {
    pub(crate) fn new(grid: &GridGeometry, limit: usize) -> Result<Self, VoxelMaterialError> {
        check_limit(payload(2, 3, grid)?, limit)?;
        let q = faces(grid)?;
        let h = faces(grid)?;
        let j = faces(grid)?;
        let residual = allocate(grid.cell_len())?;
        let mass_defect = allocate(grid.cell_len())?;
        let allocated_bytes = capacity(
            q.iter()
                .chain(h.iter())
                .chain(j.iter())
                .chain([&residual, &mass_defect]),
        )?;
        check_limit(allocated_bytes, limit)?;
        Ok(Self {
            grid: grid.clone(),
            q,
            h,
            j,
            residual,
            mass_defect,
            allocated_bytes,
        })
    }
    pub(crate) fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    fn cell_flux(
        &self,
        field: &[Vec<f64>; 3],
        p: [usize; 3],
    ) -> Result<(f64, f64), VoxelMaterialError> {
        let mut value = 0.0;
        let mut magnitude = 0.0;
        for axis in Axis::ALL {
            let d = axis.index();
            let mut hi = p;
            hi[d] += 1;
            let a = field[d][self.grid.face_unchecked(axis, p)];
            let b = field[d][self.grid.face_unchecked(axis, hi)];
            value = checked(value + checked(b - a)?)?;
            magnitude = checked(magnitude + a.abs() + b.abs())?;
        }
        Ok((value, magnitude))
    }
    fn close_flux(
        &mut self,
        state: &VoxelMaterialState,
        divergence_limit: f64,
        mut cancel: impl FnMut() -> bool,
    ) -> Result<(f64, f64, f64), VoxelMaterialError> {
        let n = self.grid.counts();
        let h = self.grid.spacing();
        let area = [h[1] * h[2], h[0] * h[2], h[0] * h[1]];
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                if cancel() {
                    return Err(VoxelMaterialError::Cancelled);
                }
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        let index = self.grid.face_unchecked(axis, p);
                        let velocity = state.velocity[d][index];
                        checked(velocity)?;
                        if (p[d] == 0 || p[d] == n[d]) && velocity != 0.0 {
                            return Err(
                                crate::OperatorError::NonZeroWallVelocity { axis, index }.into()
                            );
                        }
                        self.q[d][index] = multiply(area[d], velocity)?;
                    }
                }
            }
        }
        for cell in 0..self.grid.cell_len() {
            let (d, scale) = self.cell_flux(&self.q, coordinate(&self.grid, cell))?;
            let original_div = divide(d.abs(), self.grid.cell_volume())?;
            let budget = roundoff(16, divide(scale, self.grid.cell_volume())?)?;
            if original_div > divergence_limit + budget {
                return Err(VoxelMaterialError::Divergence {
                    actual: original_div,
                    limit: divergence_limit + budget,
                });
            }
            self.residual[cell] = d;
        }
        let original = sum(self.residual.iter().map(|v| v.abs()))?;
        let flux_scale = sum(self.q.iter().flat_map(|v| v.iter()).map(|v| v.abs()))?;
        let mut correction_max = 0.0_f64;
        let mut correction_sum = 0.0;
        // Every parent has a smaller x-fastest index; reverse order visits all
        // descendants first. Propagate the actual stored face correction.
        for cell in (1..self.grid.cell_len()).rev() {
            if cancel() {
                return Err(VoxelMaterialError::Cancelled);
            }
            let p = coordinate(&self.grid, cell);
            let d = if p[0] > 0 {
                0
            } else if p[1] > 0 {
                1
            } else {
                2
            };
            let mut parent = p;
            parent[d] -= 1;
            let index = self.grid.face_unchecked(Axis::ALL[d], p);
            let old = self.q[d][index];
            let new = checked(old + self.residual[cell])?;
            let correction = checked(new - old)?;
            self.q[d][index] = new;
            let parent = self.grid.cell_unchecked(parent);
            self.residual[parent] = checked(self.residual[parent] + correction)?;
            correction_max = correction_max.max(correction.abs());
            correction_sum = checked(correction_sum + correction.abs())?;
        }
        let length = (n.iter().sum::<usize>() - 3) as f64;
        let arithmetic = roundoff(16 * self.grid.cell_len(), flux_scale + original)?;
        if correction_max > original + arithmetic || correction_sum > length * original + arithmetic
        {
            return Err(VoxelMaterialError::Balance {
                stage: "flux closure correction",
                actual: correction_sum,
                budget: length * original + arithmetic,
            });
        }
        for cell in 0..self.grid.cell_len() {
            self.residual[cell] = self.cell_flux(&self.q, coordinate(&self.grid, cell))?.0;
        }
        let root_sum = sum(self.residual.iter().copied())?.abs();
        if root_sum > arithmetic {
            return Err(VoxelMaterialError::Balance {
                stage: "closed boundary flux",
                actual: root_sum,
                budget: arithmetic,
            });
        }
        Ok((
            correction_max,
            correction_sum,
            sum(self.residual.iter().map(|v| v.abs()))?,
        ))
    }
    fn build_phase_mass_flux(
        &mut self,
        state: &VoxelMaterialState,
    ) -> Result<(), VoxelMaterialError> {
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        let face = self.grid.face_unchecked(axis, p);
                        let q = self.q[d][face];
                        if q == 0.0 {
                            self.h[d][face] = 0.0;
                            self.j[d][face] = 0.0;
                            continue;
                        }
                        let mut donor = p;
                        if q > 0.0 {
                            donor[d] = donor[d]
                                .checked_sub(1)
                                .ok_or(VoxelMaterialError::ArithmeticFailure)?;
                        }
                        let f = state.liquid_fraction[self.grid.cell_unchecked(donor)];
                        let phase = multiply(q, f)?;
                        let gas = multiply(state.materials.gas().density(), q)?;
                        let contrast =
                            state.materials.liquid().density() - state.materials.gas().density();
                        let mass = checked(gas + multiply(contrast, phase)?)?;
                        if mass == 0.0 || mass.is_sign_positive() != q.is_sign_positive() {
                            return Err(VoxelMaterialError::ArithmeticFailure);
                        }
                        self.h[d][face] = phase;
                        self.j[d][face] = mass;
                    }
                }
            }
        }
        Ok(())
    }
    fn advect_phase(
        &mut self,
        state: &VoxelMaterialState,
        dt: f64,
        out: &mut [f64],
    ) -> Result<([f64; 2], [f64; 2], f64, f64), VoxelMaterialError> {
        let v = self.grid.cell_volume();
        let before = phase_volumes(&self.grid, &state.liquid_fraction)?;
        let mut local_bounds = [0.0; 2];
        let mut induced = [0.0; 2];
        let mut max_courant = 0.0_f64;
        let mut continuity = 0.0_f64;
        for (cell, output) in out.iter_mut().enumerate() {
            let p = coordinate(&self.grid, cell);
            let f = state.liquid_fraction[cell];
            let mut increment = 0.0;
            let mut incoming = 0.0;
            let mut outgoing = 0.0;
            let mut local_scale = f;
            for axis in Axis::ALL {
                let d = axis.index();
                let mut hi = p;
                hi[d] += 1;
                for (face, lower) in [(p, true), (hi, false)] {
                    let q = self.q[d][self.grid.face_unchecked(axis, face)];
                    let c = divide(multiply(dt, q.abs())?, v)?;
                    let enters = if lower { q > 0.0 } else { q < 0.0 };
                    if enters {
                        incoming = checked(incoming + c)?;
                        let mut donor = p;
                        donor[d] = if lower {
                            donor[d]
                                .checked_sub(1)
                                .ok_or(VoxelMaterialError::ArithmeticFailure)?
                        } else {
                            donor[d] + 1
                        };
                        let donor = state.liquid_fraction[self.grid.cell_unchecked(donor)];
                        increment = checked(increment + multiply(c, checked(donor - f)?)?)?;
                        local_scale = upper(local_scale + c * (donor.abs() + f.abs()))?;
                    } else {
                        outgoing = checked(outgoing + c)?;
                    }
                }
            }
            let courant = incoming.max(outgoing);
            max_courant = max_courant.max(courant);
            if courant > COURANT_LIMIT {
                return Err(VoxelMaterialError::Courant {
                    actual: courant,
                    limit: COURANT_LIMIT,
                });
            }
            let value = checked(f + increment)?;
            if !(0.0..=1.0).contains(&value) {
                return Err(VoxelMaterialError::InvalidFraction { cell });
            }
            *output = value;
            let fraction_bound = roundoff(64, multiply(v, local_scale)?)?;
            for phase in 0..2 {
                // Gas is computed as 1-F, so it inherits F's absolute update
                // error even when the gas fraction itself is arbitrarily small.
                let bound = if phase == 0 {
                    fraction_bound
                } else {
                    upper(fraction_bound + roundoff(2, v * (1.0 + value.abs()))?)?
                };
                local_bounds[phase] = upper(local_bounds[phase] + bound)?;
                let old = if phase == 0 { f } else { 1.0 - f };
                induced[phase] = upper(
                    induced[phase] + multiply(multiply(dt, old)?, self.residual[cell].abs())?,
                )?;
            }
            let rho_old = state.materials.density(f);
            let rho_new = state.materials.density(value);
            let old_mass = multiply(v, rho_old)?;
            let new_mass = multiply(v, rho_new)?;
            let (mass_flux, flux_scale) = self.cell_flux(&self.j, p)?;
            let defect = checked(checked(new_mass - old_mass)? + multiply(dt, mass_flux)?)?;
            let expected = multiply(multiply(dt, rho_old)?, self.residual[cell])?;
            let q_scale = self.cell_flux(&self.q, p)?.1;
            let bound = roundoff(
                128,
                old_mass + new_mass + dt * flux_scale + dt * rho_old * q_scale,
            )?;
            if (defect - expected).abs() > bound {
                return Err(VoxelMaterialError::Balance {
                    stage: "cell mass continuity",
                    actual: (defect - expected).abs(),
                    budget: bound,
                });
            }
            self.mass_defect[cell] = defect;
            continuity = continuity.max(defect.abs());
        }
        let after = phase_volumes(&self.grid, out)?;
        let mut bounds = [0.0; 2];
        for phase in 0..2 {
            let reduction = roundoff(self.grid.cell_len() + 4, before[phase] + after[phase])?;
            let arithmetic = upper(local_bounds[phase] + reduction)?;
            if induced[phase] > arithmetic {
                return Err(VoxelMaterialError::Balance {
                    stage: "phase residual volume",
                    actual: induced[phase],
                    budget: arithmetic,
                });
            }
            bounds[phase] = upper(arithmetic + induced[phase])?;
            let change = (after[phase] - before[phase]).abs();
            if change > bounds[phase] {
                return Err(VoxelMaterialError::Balance {
                    stage: "phase volume",
                    actual: change,
                    budget: bounds[phase],
                });
            }
        }
        Ok((after, bounds, max_courant, continuity))
    }
    fn dual_flux(
        &self,
        axis: Axis,
        p: [usize; 3],
        normal: Axis,
        upper: bool,
    ) -> Result<f64, VoxelMaterialError> {
        let a = axis.index();
        let b = normal.index();
        let mut first = p;
        let mut second = p;
        if a == b {
            if upper {
                second[b] += 1;
            } else {
                first[b] -= 1;
            }
        } else {
            first[a] -= 1;
            if upper {
                first[b] += 1;
                second[b] += 1;
            }
        }
        checked(
            0.5 * self.j[b][self.grid.face_unchecked(normal, first)]
                + 0.5 * self.j[b][self.grid.face_unchecked(normal, second)],
        )
    }
    fn advect_momentum(
        &self,
        state: &VoxelMaterialState,
        f: &[f64],
        dt: f64,
        out: &mut [Vec<f64>; 3],
    ) -> Result<(f64, f64), VoxelMaterialError> {
        let mut courant = 0.0_f64;
        let mut continuity = 0.0_f64;
        for axis in Axis::ALL {
            let a = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        let index = self.grid.face_unchecked(axis, p);
                        if p[a] == 0 || p[a] == self.grid.counts()[a] {
                            out[a][index] = 0.0;
                            continue;
                        }
                        let old_mass = face_mass(
                            &self.grid,
                            state.materials,
                            &state.liquid_fraction,
                            axis,
                            p,
                        )?;
                        let new_mass = face_mass(&self.grid, state.materials, f, axis, p)?;
                        let mut momentum_flux = 0.0;
                        let mut mass_flux = 0.0;
                        let mut mass_out = 0.0;
                        let mut scale = 0.0;
                        for normal in Axis::ALL {
                            let b = normal.index();
                            for upper_side in [false, true] {
                                let flux = self.dual_flux(axis, p, normal, upper_side)?;
                                let signed = if upper_side { flux } else { -flux };
                                mass_flux = checked(mass_flux + signed)?;
                                mass_out = checked(mass_out + signed.max(0.0))?;
                                scale = checked(scale + flux.abs())?;
                                if flux == 0.0 {
                                    continue;
                                }
                                let mut donor = p;
                                if upper_side && flux < 0.0 {
                                    donor[b] += 1;
                                }
                                if !upper_side && flux > 0.0 {
                                    donor[b] = donor[b]
                                        .checked_sub(1)
                                        .ok_or(VoxelMaterialError::ArithmeticFailure)?;
                                }
                                let velocity =
                                    state.velocity[a][self.grid.face_unchecked(axis, donor)];
                                momentum_flux =
                                    checked(momentum_flux + multiply(signed, velocity)?)?;
                            }
                        }
                        let number = divide(multiply(dt, mass_out)?, old_mass)?;
                        courant = courant.max(number);
                        if number > COURANT_LIMIT {
                            return Err(VoxelMaterialError::Courant {
                                actual: number,
                                limit: COURANT_LIMIT,
                            });
                        }
                        let defect =
                            checked(checked(new_mass - old_mass)? + multiply(dt, mass_flux)?)?;
                        let mut lo = p;
                        lo[a] -= 1;
                        let expected = 0.5 * self.mass_defect[self.grid.cell_unchecked(lo)]
                            + 0.5 * self.mass_defect[self.grid.cell_unchecked(p)];
                        let bound = roundoff(64, old_mass + new_mass + dt * scale)?;
                        if (defect - expected).abs() > bound {
                            return Err(VoxelMaterialError::Balance {
                                stage: "dual mass continuity",
                                actual: (defect - expected).abs(),
                                budget: bound,
                            });
                        }
                        continuity = continuity.max(defect.abs());
                        let momentum = checked(
                            multiply(old_mass, state.velocity[a][index])?
                                - multiply(dt, momentum_flux)?,
                        )?;
                        out[a][index] = divide(momentum, new_mass)?;
                    }
                }
            }
        }
        Ok((courant, continuity))
    }
    pub(crate) fn advect(
        &mut self,
        state: &VoxelMaterialState,
        dt: f64,
        divergence_limit: f64,
        f: &mut [f64],
        velocity: &mut [Vec<f64>; 3],
        mut cancel: impl FnMut() -> bool,
    ) -> Result<TransportReport, VoxelMaterialError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(VoxelMaterialError::InvalidTimeStep);
        }
        if state.geometry() != &self.grid
            || f.len() != self.grid.cell_len()
            || Axis::ALL
                .iter()
                .any(|&a| velocity[a.index()].len() != self.grid.face_len(a))
        {
            return Err(VoxelMaterialError::LengthMismatch);
        }
        let (correction_max, correction_sum, residual_sum) =
            self.close_flux(state, divergence_limit, &mut cancel)?;
        self.build_phase_mass_flux(state)?;
        if cancel() {
            return Err(VoxelMaterialError::Cancelled);
        }
        let (phase_volumes, phase_budgets, volume_courant, cell_continuity_max) =
            self.advect_phase(state, dt, f)?;
        if cancel() {
            return Err(VoxelMaterialError::Cancelled);
        }
        let (mass_courant, dual_continuity_max) = self.advect_momentum(state, f, dt, velocity)?;
        if cancel() {
            return Err(VoxelMaterialError::Cancelled);
        }
        Ok(TransportReport {
            phase_volumes,
            phase_budgets,
            correction_max,
            correction_sum,
            residual_sum,
            volume_courant,
            mass_courant,
            cell_continuity_max,
            dual_continuity_max,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{MaterialPair, MaterialProperties};
    use super::*;
    const LIMIT: usize = 64 * 1024 * 1024;
    fn materials(ratio: f64) -> MaterialPair {
        MaterialPair::new(
            MaterialProperties::new(ratio, 0.001).unwrap(),
            MaterialProperties::new(1.0, 0.000018).unwrap(),
        )
    }
    fn state(n: usize, pair: MaterialPair) -> VoxelMaterialState {
        let g = GridGeometry::new([n as u64; 3], [1.0; 3], [0.0; 3]).unwrap();
        let f = (0..g.cell_len())
            .map(|i| [0.0, 0.25, 0.75, 1.0][i % 4])
            .collect::<Vec<_>>();
        VoxelMaterialState::new(g, pair, &f, LIMIT).unwrap()
    }
    fn circulation(s: &mut VoxelMaterialState) {
        for (a, p, u) in [
            (Axis::X, [2, 1, 1], 0.125),
            (Axis::Y, [2, 2, 1], 0.125),
            (Axis::X, [2, 2, 1], -0.125),
            (Axis::Y, [1, 2, 1], -0.125),
        ] {
            s.velocity[a.index()][s.grid.face_unchecked(a, p)] = u;
        }
    }
    #[test]
    fn dyadic_conservative_donors_and_matched_mass_momentum() {
        for pair in [
            materials(1.0),
            materials(2.0),
            materials(10.0),
            materials(100.0),
            materials(1000.0),
            MaterialPair::water_air(),
        ] {
            let mut s = state(4, pair);
            circulation(&mut s);
            let mut w = TransportWorkspace::new(&s.grid, LIMIT).unwrap();
            let mut f = vec![0.0; s.grid.cell_len()];
            let mut u = faces(&s.grid).unwrap();
            let dt = 0.25;
            let report = w.advect(&s, dt, 1e-8, &mut f, &mut u, || false).unwrap();
            for k in 0..4 {
                for j in 0..4 {
                    for i in 0..4 {
                        let p = [i, j, k];
                        let row = i + 4 * (j + 4 * k);
                        let mut flux = 0.0;
                        for d in 0..3 {
                            for hi in [false, true] {
                                let mut q = p;
                                if hi {
                                    q[d] += 1;
                                }
                                let dims = s.grid.face_counts(Axis::ALL[d]);
                                let face = q[0] + dims[0] * (q[1] + dims[1] * q[2]);
                                let speed = s.velocity[d][face];
                                if speed == 0.0 {
                                    continue;
                                }
                                let mut donor = q;
                                if speed > 0.0 {
                                    donor[d] -= 1;
                                }
                                let fc =
                                    s.liquid_fraction[donor[0] + 4 * (donor[1] + 4 * donor[2])];
                                flux += if hi { speed * fc } else { -speed * fc };
                            }
                        }
                        assert_eq!(f[row], s.liquid_fraction[row] - dt * flux);
                    }
                }
            }
            assert_eq!(report.correction_max, 0.0);
            assert_eq!(
                report.phase_volumes,
                phase_volumes(&s.grid, &s.liquid_fraction).unwrap()
            );
            assert!(report.volume_courant <= 0.5 && report.mass_courant <= 0.5);
            assert!(report.cell_continuity_max < 1e-10 && report.dual_continuity_max < 1e-10);
            assert!(f.iter().all(|f| (0.0..=1.0).contains(f)));
        }
    }
    fn closed_patch(s: &mut VoxelMaterialState) {
        let n = s.grid.counts()[0];
        let h = s.grid.spacing()[0];
        let window = |i: usize| {
            let x = i as f64 / n as f64;
            (4.0 * x).min(4.0 * (1.0 - x)).min(1.0)
        };
        let psi = |i: usize, j: usize| 0.25 * (j as f64 * h) * window(i) * window(j);
        for k in 0..n {
            for j in 0..n {
                for i in 0..=n {
                    s.velocity[0][s.grid.face_unchecked(Axis::X, [i, j, k])] =
                        (psi(i, j + 1) - psi(i, j)) / h;
                }
            }
        }
        for k in 0..n {
            for j in 0..=n {
                for i in 0..n {
                    s.velocity[1][s.grid.face_unchecked(Axis::Y, [i, j, k])] =
                        -(psi(i + 1, j) - psi(i, j)) / h;
                }
            }
        }
    }
    #[test]
    fn local_uniform_velocity_survives_material_mass_transport() {
        for ratio in [1.0, 10.0, 1000.0] {
            let mut s = state(8, materials(ratio));
            closed_patch(&mut s);
            let mut w = TransportWorkspace::new(&s.grid, LIMIT).unwrap();
            let mut f = vec![0.0; s.grid.cell_len()];
            let mut u = faces(&s.grid).unwrap();
            w.advect(&s, 0.01, 1e-8, &mut f, &mut u, || false).unwrap();
            let face = s.grid.face_unchecked(Axis::X, [4, 4, 3]);
            assert!((u[0][face] - 0.25).abs() < 1e-12);
        }
    }
    #[test]
    fn manufactured_nonzero_dual_defect_has_the_conditional_uniform_velocity_error() {
        let n = 8usize;
        let mut state = state(n, materials(1000.0));
        closed_patch(&mut state);
        let dt = 0.01;
        let mut workspace = TransportWorkspace::new(&state.grid, LIMIT).unwrap();
        workspace.close_flux(&state, 1e-8, || false).unwrap();
        workspace.build_phase_mass_flux(&state).unwrap();
        let mut fraction = vec![0.0; n * n * n];
        workspace.advect_phase(&state, dt, &mut fraction).unwrap();
        let cell = |p: [usize; 3]| p[0] + n * (p[1] + n * p[2]);
        let face = |d: usize, p: [usize; 3]| {
            let mut dims = [n; 3];
            dims[d] += 1;
            p[0] + dims[0] * (p[1] + dims[1] * p[2])
        };
        let target = [4usize, 4, 3];
        let lower = [3usize, 4, 3];
        assert_eq!(state.liquid_fraction[cell(target)], 0.0);
        let delta = 0.0001;
        assert!(fraction[cell(target)] + delta < 1.0);
        fraction[cell(target)] += delta;
        assert!(fraction.iter().all(|f| (0.0..=1.0).contains(f)));
        // Isolated algebraic kernel oracle: this deliberate candidate defect
        // is not an accepted phase update or a new physical mass source.
        let rho = |f: f64| f * 1000.0 + (1.0 - f) * 1.0;
        let volume = state.grid.spacing().into_iter().product::<f64>();
        let old_mass = state
            .liquid_fraction
            .iter()
            .map(|&f| volume * rho(f))
            .collect::<Vec<_>>();
        let new_mass = fraction
            .iter()
            .map(|&f| volume * rho(f))
            .collect::<Vec<_>>();
        let mut defects = vec![0.0; n * n * n];
        for k in 0..n {
            for j in 0..n {
                for i in 0..n {
                    let p = [i, j, k];
                    let mut outward = 0.0;
                    for d in 0..3 {
                        let mut hi = p;
                        hi[d] += 1;
                        outward += workspace.j[d][face(d, hi)] - workspace.j[d][face(d, p)];
                    }
                    let c = cell(p);
                    defects[c] = new_mass[c] - old_mass[c] + dt * outward;
                }
            }
        }
        workspace.mass_defect.copy_from_slice(&defects);
        let old_dual_mass = 0.5 * (old_mass[cell(lower)] + old_mass[cell(target)]);
        let new_dual_mass = 0.5 * (new_mass[cell(lower)] + new_mass[cell(target)]);
        let uniform = state.velocity[0][face(0, target)];
        assert_eq!(uniform, 0.25);
        // Independently assemble the six dual faces from Cartesian primary J.
        let mut outward_mass = 0.0;
        let mut outward_momentum = 0.0;
        let mut absolute_mass_flux = 0.0;
        let mut absolute_momentum_flux = 0.0;
        for normal in 0..3 {
            for upper_side in [false, true] {
                let (first, second) = if normal == 0 {
                    let mut neighbor = target;
                    if upper_side {
                        neighbor[0] += 1;
                    } else {
                        neighbor[0] -= 1;
                    }
                    (target, neighbor)
                } else {
                    let mut first = lower;
                    let mut second = target;
                    if upper_side {
                        first[normal] += 1;
                        second[normal] += 1;
                    }
                    (first, second)
                };
                let flux = 0.5
                    * (workspace.j[normal][face(normal, first)]
                        + workspace.j[normal][face(normal, second)]);
                let signed = if upper_side { flux } else { -flux };
                let mut donor = target;
                if upper_side && flux < 0.0 {
                    donor[normal] += 1;
                }
                if !upper_side && flux > 0.0 {
                    donor[normal] -= 1;
                }
                let velocity = state.velocity[0][face(0, donor)];
                if flux != 0.0 {
                    assert_eq!(velocity, uniform);
                }
                outward_mass += signed;
                outward_momentum += signed * velocity;
                absolute_mass_flux += flux.abs();
                absolute_momentum_flux += (signed * velocity).abs();
            }
        }
        let residual = new_dual_mass - old_dual_mass + dt * outward_mass;
        let half_cell_residual = 0.5 * (defects[cell(lower)] + defects[cell(target)]);
        let epsilon = 256.0 * f64::EPSILON;
        let mass_scale = old_dual_mass + new_dual_mass + dt * absolute_mass_flux;
        assert!((residual - half_cell_residual).abs() <= epsilon * mass_scale);
        assert!(
            residual.abs() > 0.001,
            "nonzero dual-defect case must be reached"
        );
        let mut output = faces(&state.grid).unwrap();
        workspace
            .advect_momentum(&state, &fraction, dt, &mut output)
            .unwrap();
        let actual_error = output[0][face(0, target)] - uniform;
        let expected_error = -uniform * residual / new_dual_mass;
        // Absolute product/evaluation scales cover old/new momentum, flux,
        // residual cancellation and the final division/subtraction, rather
        // than scaling only by the small final signed error.
        let momentum_scale = (old_dual_mass * uniform).abs()
            + (new_dual_mass * uniform).abs()
            + dt * absolute_momentum_flux
            + uniform.abs() * mass_scale
            + (uniform * residual).abs();
        let allowance =
            epsilon * (momentum_scale / new_dual_mass + uniform.abs() + expected_error.abs());
        assert!(expected_error.abs() > 1e-6 && actual_error.abs() > 1e-6);
        assert!(
            (actual_error - expected_error).abs() <= allowance,
            "actual velocity error {actual_error}, expected {expected_error}, allowance {allowance}"
        );
        let direct_momentum = (old_dual_mass * uniform - dt * outward_momentum) / new_dual_mass;
        assert!((output[0][face(0, target)] - direct_momentum).abs() <= allowance);
    }

    #[test]
    fn finite_residual_correction_keeps_pure_phase_bounded() {
        let mut s = state(4, MaterialPair::water_air());
        s.liquid_fraction.fill(1.0);
        circulation(&mut s);
        let index = s.grid.face_unchecked(Axis::X, [2, 1, 1]);
        s.velocity[0][index] += 1e-12;
        let mut w = TransportWorkspace::new(&s.grid, LIMIT).unwrap();
        let mut f = vec![0.0; s.grid.cell_len()];
        let mut u = faces(&s.grid).unwrap();
        let r = w.advect(&s, 0.01, 1e-8, &mut f, &mut u, || false).unwrap();
        assert!(r.correction_max > 0.0);
        assert!(r.residual_sum < 1e-16);
        assert!(f.iter().all(|f| *f == 1.0));
        s.velocity[0][index] += 0.01;
        assert!(matches!(
            w.advect(&s, 0.01, 1e-8, &mut f, &mut u, || false),
            Err(VoxelMaterialError::Divergence { .. })
        ));
    }
    #[test]
    fn translated_box_reference_refines_in_uniform_interior() {
        let mut errors = Vec::new();
        for n in [8usize, 16, 32] {
            let h = 1.0 / n as f64;
            let grid = GridGeometry::new([n as u64; 3], [h; 3], [0.0; 3]).unwrap();
            let original = |p: [usize; 3]| {
                let x = p.map(|i| (i as f64 + 0.5) * h);
                if x[0] >= 0.375
                    && x[0] < 0.5
                    && x[1] >= 0.375
                    && x[1] < 0.5
                    && x[2] >= 0.375
                    && x[2] < 0.625
                {
                    1.0
                } else {
                    0.0
                }
            };
            let f = (0..grid.cell_len())
                .map(|i| original(coordinate(&grid, i)))
                .collect::<Vec<_>>();
            let mut s =
                VoxelMaterialState::new(grid, MaterialPair::water_air(), &f, LIMIT).unwrap();
            closed_patch(&mut s);
            for field in &mut s.velocity {
                for u in field {
                    *u *= 16.0;
                }
            }
            let mut w = TransportWorkspace::new(&s.grid, LIMIT).unwrap();
            let mut next = vec![0.0; s.grid.cell_len()];
            let time = 0.0625;
            let steps = (time / (0.02 * h)).ceil() as usize;
            let dt = time / steps as f64;
            for _ in 0..steps {
                w.close_flux(&s, 1e-8, || false).unwrap();
                w.build_phase_mass_flux(&s).unwrap();
                w.advect_phase(&s, dt, &mut next).unwrap();
                s.liquid_fraction.copy_from_slice(&next);
            }
            let shift = 4.0 * time;
            let lower = [0.375 + shift, 0.375, 0.375];
            let upper = [0.5 + shift, 0.5, 0.625];
            let mut error = 0.0;
            for (i, &f) in s.liquid_fraction.iter().enumerate() {
                let p = coordinate(&s.grid, i);
                let mut exact = 1.0;
                for d in 0..3 {
                    let lo = p[d] as f64 * h;
                    let hi = lo + h;
                    exact *= (hi.min(upper[d]) - lo.max(lower[d])).max(0.0) / h;
                }
                error += (f - exact).abs() * h * h * h;
            }
            errors.push(error);
        }
        assert!(errors[1] < errors[0] && errors[2] < errors[1], "{errors:?}");
    }
}
