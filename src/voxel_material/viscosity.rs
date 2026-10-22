//! Positive weighted symmetric strain on the same dual masses as projection.
use super::{
    VoxelMaterialError, VoxelMaterialState, capacity, check_limit, checked, divide, face_mass,
    faces, multiply, payload, roundoff, upper,
};
use crate::{Axis, GridGeometry};

#[derive(Debug, Clone, Copy)]
pub struct ViscosityReport {
    pub spectral_bound: f64,
    pub kinetic_before: f64,
    pub kinetic_after: f64,
    pub dissipation: f64,
    pub work_error: f64,
    pub work_budget: f64,
    pub identity_error: f64,
    pub arithmetic_budget: f64,
}

#[derive(Clone, Copy)]
struct Dof {
    axis: usize,
    index: usize,
    coefficient: f64,
    mass: f64,
}
const EMPTY: Dof = Dof {
    axis: 0,
    index: 0,
    coefficient: 0.0,
    mass: 0.0,
};
pub(crate) struct MaterialViscosityWorkspace {
    grid: GridGeometry,
    ku: [Vec<f64>; 3],
    row_bound: [Vec<f64>; 3],
    dissipation: f64,
    strain_work_scale: f64,
    allocated_bytes: usize,
}
impl MaterialViscosityWorkspace {
    pub(crate) fn new(grid: &GridGeometry, limit: usize) -> Result<Self, VoxelMaterialError> {
        check_limit(payload(0, 2, grid)?, limit)?;
        let ku = faces(grid)?;
        let row_bound = faces(grid)?;
        let allocated_bytes = capacity(ku.iter().chain(row_bound.iter()))?;
        check_limit(allocated_bytes, limit)?;
        Ok(Self {
            grid: grid.clone(),
            ku,
            row_bound,
            dissipation: 0.0,
            strain_work_scale: 0.0,
            allocated_bytes,
        })
    }
    pub(crate) fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    fn dof(
        &self,
        state: &VoxelMaterialState,
        axis: usize,
        p: [usize; 3],
        coefficient: f64,
    ) -> Result<Dof, VoxelMaterialError> {
        Ok(Dof {
            axis,
            index: self.grid.face_unchecked(Axis::ALL[axis], p),
            coefficient,
            mass: face_mass(
                &self.grid,
                state.materials,
                &state.liquid_fraction,
                Axis::ALL[axis],
                p,
            )?,
        })
    }
    fn strain(
        &mut self,
        state: &VoxelMaterialState,
        dofs: &[Dof],
        weight: f64,
    ) -> Result<(), VoxelMaterialError> {
        if weight == 0.0 {
            return Ok(());
        }
        let mut strain = 0.0;
        let mut absolute_strain = 0.0;
        let mut alpha = [0.0; 4];
        let mut alpha_sum = 0.0;
        for (i, dof) in dofs.iter().enumerate() {
            if dof.mass == 0.0 {
                continue;
            }
            strain =
                checked(strain + multiply(dof.coefficient, state.velocity[dof.axis][dof.index])?)?;
            absolute_strain = upper(
                absolute_strain
                    + upper(multiply(
                        dof.coefficient.abs(),
                        state.velocity[dof.axis][dof.index].abs(),
                    )?)?,
            )?;
            let root = dof.mass.sqrt().next_down();
            alpha[i] = upper(divide(dof.coefficient.abs(), root)?)?;
            alpha_sum = upper(alpha_sum + alpha[i])?;
        }
        self.dissipation =
            checked(self.dissipation + multiply(weight, multiply(strain, strain)?)?)?;
        // T = sum |c_i u_i| bounds cancellation inside a strain and in its
        // force-work contributions. W*T^2 also covers the sensitivity of the
        // stored strain, unlike the small signed final u.Ku or W*strain^2.
        self.strain_work_scale = upper(
            self.strain_work_scale
                + upper(multiply(
                    weight,
                    upper(multiply(absolute_strain, absolute_strain)?)?,
                )?)?,
        )?;
        for (i, dof) in dofs.iter().enumerate() {
            if dof.mass == 0.0 {
                continue;
            }
            let value = multiply(multiply(weight, dof.coefficient)?, strain)?;
            self.ku[dof.axis][dof.index] = checked(self.ku[dof.axis][dof.index] + value)?;
            let bound = upper(multiply(upper(multiply(weight, alpha[i])?)?, alpha_sum)?)?;
            self.row_bound[dof.axis][dof.index] =
                upper(self.row_bound[dof.axis][dof.index] + bound)?;
        }
        Ok(())
    }
    fn assemble(
        &mut self,
        state: &VoxelMaterialState,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<f64, VoxelMaterialError> {
        if &self.grid != state.geometry() {
            return Err(VoxelMaterialError::GeometryMismatch);
        }
        for a in &mut self.ku {
            a.fill(0.0);
        }
        for a in &mut self.row_bound {
            a.fill(0.0);
        }
        self.dissipation = 0.0;
        self.strain_work_scale = 0.0;
        let n = self.grid.counts();
        let h = self.grid.spacing();
        let v = self.grid.cell_volume();
        for k in 0..n[2] {
            if cancelled() {
                return Err(VoxelMaterialError::Cancelled);
            }
            for j in 0..n[1] {
                for i in 0..n[0] {
                    let p = [i, j, k];
                    let mu = checked(
                        state
                            .materials
                            .viscosity(state.liquid_fraction[self.grid.cell_unchecked(p)]),
                    )?;
                    for d in 0..3 {
                        let mut hi = p;
                        hi[d] += 1;
                        let a = divide(1.0, h[d])?;
                        let dofs = [self.dof(state, d, p, -a)?, self.dof(state, d, hi, a)?];
                        self.strain(state, &dofs, multiply(2.0, multiply(mu, v)?)?)?;
                    }
                }
            }
        }
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            let mut dims = n;
            dims[a] += 1;
            dims[b] += 1;
            for k in 0..dims[2] {
                if cancelled() {
                    return Err(VoxelMaterialError::Cancelled);
                }
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[a] == 0 || p[a] == n[a] || p[b] == 0 || p[b] == n[b] {
                            continue;
                        }
                        let mut cells = [p; 4];
                        cells[1][a] -= 1;
                        cells[2][b] -= 1;
                        cells[3][a] -= 1;
                        cells[3][b] -= 1;
                        let mu = cells.map(|p| {
                            state
                                .materials
                                .viscosity(state.liquid_fraction[self.grid.cell_unchecked(p)])
                        });
                        let minimum = mu.iter().copied().fold(f64::INFINITY, f64::min);
                        let harmonic = if minimum == 0.0 {
                            0.0
                        } else {
                            let mut denominator = 0.0;
                            for x in mu {
                                denominator = checked(denominator + divide(minimum, x)?)?;
                            }
                            multiply(minimum, divide(4.0, denominator)?)?
                        };
                        let mut pa = p;
                        pa[b] -= 1;
                        let mut pb = p;
                        pb[a] -= 1;
                        let ca = divide(1.0, h[b])?;
                        let cb = divide(1.0, h[a])?;
                        let mut dofs = [EMPTY; 4];
                        dofs[0] = self.dof(state, a, pa, -ca)?;
                        dofs[1] = self.dof(state, a, p, ca)?;
                        dofs[2] = self.dof(state, b, pb, -cb)?;
                        dofs[3] = self.dof(state, b, p, cb)?;
                        self.strain(state, &dofs, multiply(harmonic, v)?)?;
                    }
                }
            }
        }
        Ok(self
            .row_bound
            .iter()
            .flat_map(|v| v.iter())
            .copied()
            .fold(0.0_f64, f64::max))
    }
    pub(crate) fn apply(
        &mut self,
        state: &mut VoxelMaterialState,
        dt: f64,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<ViscosityReport, VoxelMaterialError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(VoxelMaterialError::InvalidTimeStep);
        }
        let spectral_bound = self.assemble(state, &mut cancelled)?;
        let number = multiply(dt, spectral_bound)?;
        if number > 1.0 {
            return Err(VoxelMaterialError::ViscousStability { number });
        }
        let mut before = 0.0;
        let mut after = 0.0;
        let mut update_energy = 0.0;
        let mut work = 0.0;
        let mut absolute_work = 0.0;
        let n = self.grid.counts();
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = self.grid.face_counts(axis);
            for k in 0..dims[2] {
                if cancelled() {
                    return Err(VoxelMaterialError::Cancelled);
                }
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] == 0 || p[d] == n[d] {
                            continue;
                        }
                        let index = self.grid.face_unchecked(axis, p);
                        let mass = face_mass(
                            &self.grid,
                            state.materials,
                            &state.liquid_fraction,
                            axis,
                            p,
                        )?;
                        let old = state.velocity[d][index];
                        let force = self.ku[d][index];
                        let delta = -divide(multiply(dt, force)?, mass)?;
                        let new = checked(old + delta)?;
                        before =
                            checked(before + multiply(0.5, multiply(mass, multiply(old, old)?)?)?)?;
                        after =
                            checked(after + multiply(0.5, multiply(mass, multiply(new, new)?)?)?)?;
                        update_energy = checked(
                            update_energy
                                + multiply(0.5, multiply(mass, multiply(delta, delta)?)?)?,
                        )?;
                        work = checked(work + multiply(old, force)?)?;
                        absolute_work =
                            upper(absolute_work + upper(multiply(old.abs(), force.abs())?)?)?;
                        state.velocity[d][index] = new;
                    }
                }
            }
        }
        let power_work_error = (work - self.dissipation).abs();
        let power_budget = roundoff(
            128 * self.grid.cell_len(),
            upper(upper(self.strain_work_scale + absolute_work)? + self.dissipation)?,
        )?;
        if power_work_error > power_budget {
            return Err(VoxelMaterialError::Balance {
                stage: "strain work",
                actual: power_work_error,
                budget: power_budget,
            });
        }
        let dissipated = multiply(dt, self.dissipation)?;
        let identity_error = checked(checked(after - before)? + dissipated - update_energy)?.abs();
        let direct_arithmetic_budget = roundoff(
            128 * self.grid.cell_len(),
            before + after + dissipated + update_energy,
        )?;
        let arithmetic_budget =
            upper(direct_arithmetic_budget + upper(multiply(dt, power_budget)?)?)?;
        if identity_error > arithmetic_budget || after > before + arithmetic_budget {
            return Err(VoxelMaterialError::Balance {
                stage: "viscous energy",
                actual: identity_error.max(after - before),
                budget: arithmetic_budget,
            });
        }
        Ok(ViscosityReport {
            spectral_bound,
            kinetic_before: before,
            kinetic_after: after,
            dissipation: self.dissipation,
            work_error: power_work_error,
            work_budget: power_budget,
            identity_error,
            arithmetic_budget,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::{MaterialPair, MaterialProperties};
    use super::*;
    const LIMIT: usize = 64 * 1024 * 1024;
    fn fixture(mu: [f64; 2]) -> VoxelMaterialState {
        let g = GridGeometry::new([4; 3], [0.2, 0.3, 0.4], [0.0; 3]).unwrap();
        let f = (0..g.cell_len())
            .map(|i| if (i / 4) % 4 < 2 { 1.0 } else { 0.0 })
            .collect::<Vec<_>>();
        let pair = MaterialPair::new(
            MaterialProperties::new(1000.0, mu[0]).unwrap(),
            MaterialProperties::new(1.0, mu[1]).unwrap(),
        );
        let mut s = VoxelMaterialState::new(g, pair, &f, LIMIT).unwrap();
        for axis in Axis::ALL {
            let d = axis.index();
            let dims = s.grid.face_counts(axis);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[d] > 0 && p[d] < 4 {
                            s.velocity[d][s.grid.face_unchecked(axis, p)] =
                                ((i + 2 * j + 3 * k + d) % 7) as f64 * 0.125 - 0.25;
                        }
                    }
                }
            }
        }
        s
    }
    #[test]
    fn variable_stress_matches_independent_dense_rank_one_form_and_energy() {
        let mut s = fixture([0.01, 0.0001]);
        let g = s.grid.clone();
        let h = g.spacing();
        let v = g.cell_volume();
        let lengths = Axis::ALL.map(|a| g.face_len(a));
        let offsets = [0, lengths[0], lengths[0] + lengths[1]];
        let size = lengths.iter().sum();
        let mut dense = vec![vec![0.0; size]; size];
        let index = |d: usize, p: [usize; 3]| {
            let dims = g.face_counts(Axis::ALL[d]);
            offsets[d] + p[0] + dims[0] * (p[1] + dims[1] * p[2])
        };
        let dynamic = |d: usize, p: [usize; 3]| p[d] > 0 && p[d] < 4;
        let mu = |p: [usize; 3]| if p[1] < 2 { 0.01 } else { 0.0001 };
        let mut add = |sites: &[(usize, [usize; 3], f64)], weight: f64| {
            for &(a, p, ca) in sites {
                if !dynamic(a, p) {
                    continue;
                }
                for &(b, q, cb) in sites {
                    if dynamic(b, q) {
                        dense[index(a, p)][index(b, q)] += weight * ca * cb;
                    }
                }
            }
        };
        for k in 0..4 {
            for j in 0..4 {
                for i in 0..4 {
                    let p = [i, j, k];
                    for d in 0..3 {
                        let mut q = p;
                        q[d] += 1;
                        add(&[(d, p, -1.0 / h[d]), (d, q, 1.0 / h[d])], 2.0 * v * mu(p));
                    }
                }
            }
        }
        for (a, b) in [(0, 1), (0, 2), (1, 2)] {
            let mut dims = [4; 3];
            dims[a] += 1;
            dims[b] += 1;
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if p[a] == 0 || p[a] == 4 || p[b] == 0 || p[b] == 4 {
                            continue;
                        }
                        let mut ca = p;
                        ca[a] -= 1;
                        let mut cb = p;
                        cb[b] -= 1;
                        let mut cc = ca;
                        cc[b] -= 1;
                        let harmonic =
                            4.0 / (1.0 / mu(p) + 1.0 / mu(ca) + 1.0 / mu(cb) + 1.0 / mu(cc));
                        add(
                            &[
                                (a, cb, -1.0 / h[b]),
                                (a, p, 1.0 / h[b]),
                                (b, ca, -1.0 / h[a]),
                                (b, p, 1.0 / h[a]),
                            ],
                            harmonic * v,
                        );
                    }
                }
            }
        }
        let input = s.velocity.iter().flatten().copied().collect::<Vec<_>>();
        let expected = dense
            .iter()
            .map(|row| row.iter().zip(&input).map(|(a, u)| a * u).sum::<f64>())
            .collect::<Vec<_>>();
        let mut w = MaterialViscosityWorkspace::new(&g, LIMIT).unwrap();
        let bound = w.assemble(&s, || false).unwrap();
        for (a, b) in w.ku.iter().flatten().zip(&expected) {
            assert!((a - b).abs() < 1e-13);
        }
        let work = input.iter().zip(&expected).map(|(a, b)| a * b).sum::<f64>();
        assert!((work - w.dissipation).abs() < 1e-13);
        assert!(work > 0.0);
        let mut masses = vec![0.0; size];
        for d in 0..3 {
            let dims = g.face_counts(Axis::ALL[d]);
            for k in 0..dims[2] {
                for j in 0..dims[1] {
                    for i in 0..dims[0] {
                        let p = [i, j, k];
                        if !dynamic(d, p) {
                            continue;
                        }
                        let mut q = p;
                        q[d] -= 1;
                        let rho = |r: [usize; 3]| if r[1] < 2 { 1000.0 } else { 1.0 };
                        masses[index(d, p)] = v * (rho(p) + rho(q)) * 0.5;
                    }
                }
            }
        }
        let mut independent_bound = 0.0_f64;
        for i in 0..size {
            if masses[i] == 0.0 {
                continue;
            }
            let row = (0..size)
                .filter(|&j| masses[j] > 0.0)
                .map(|j| dense[i][j].abs() / (masses[i] * masses[j]).sqrt())
                .sum::<f64>();
            independent_bound = independent_bound.max(row);
        }
        assert!(bound + 1e-12 >= independent_bound);
        let dt = 0.25 / bound;
        let report = w.apply(&mut s, dt, || false).unwrap();
        assert!(report.kinetic_after < report.kinetic_before);
        assert!(report.identity_error <= report.arithmetic_budget);
        assert!(matches!(
            w.apply(&mut s, 2.0 / bound, || false),
            Err(VoxelMaterialError::ViscousStability { .. })
        ));
    }
    #[test]
    fn zero_viscosity_is_exact_identity_and_distinct_mu_changes_stress() {
        let mut zero = fixture([0.0, 0.0]);
        let before = zero.velocity.clone();
        let mut w = MaterialViscosityWorkspace::new(&zero.grid, LIMIT).unwrap();
        let r = w.apply(&mut zero, 0.1, || false).unwrap();
        assert_eq!(zero.velocity, before);
        assert_eq!(r.spectral_bound, 0.0);
        assert_eq!(r.dissipation, 0.0);
        let low = fixture([0.0001, 0.0001]);
        let high = fixture([0.01, 0.0001]);
        w.assemble(&low, || false).unwrap();
        let force_low = w.ku.clone();
        w.assemble(&high, || false).unwrap();
        assert!(
            w.ku.iter()
                .flatten()
                .zip(force_low.iter().flatten())
                .any(|(a, b)| (a - b).abs() > 1e-8)
        );
    }
    #[test]
    fn near_rigid_strain_uses_absolute_expression_scale() {
        let g = GridGeometry::new([4; 3], [0.2, 0.3, 0.4], [0.0; 3]).unwrap();
        let pair = MaterialPair::new(
            MaterialProperties::new(1.0, 0.3).unwrap(),
            MaterialProperties::new(1.0, 0.0).unwrap(),
        );
        let f = (0..g.cell_len())
            .map(|i| {
                let p = super::super::coordinate(&g, i);
                if p.iter().all(|&x| x == 1 || x == 2) {
                    1.0
                } else {
                    0.0
                }
            })
            .collect::<Vec<_>>();
        let mut s = VoxelMaterialState::new(g.clone(), pair, &f, LIMIT).unwrap();
        let common = 2.0_f64.powi(40);
        let delta = 2.0_f64.powi(-10);
        for k in 1..=2 {
            for j in 1..=2 {
                for i in 1..=3 {
                    s.velocity[0][g.face_unchecked(Axis::X, [i, j, k])] = common;
                }
            }
        }
        let first = g.face_unchecked(Axis::X, [1, 1, 1]);
        let second = g.face_unchecked(Axis::X, [2, 1, 1]);
        s.velocity[0][second] += delta;
        let mut w = MaterialViscosityWorkspace::new(&g, LIMIT).unwrap();
        // Independent two-entry rank-one reference: c=(-1,1), W=0.3.
        // The exact dyadic difference is delta, while the two work terms
        // are about 10^15 times larger than their signed sum.
        w.strain(
            &s,
            &[
                Dof {
                    axis: 0,
                    index: first,
                    coefficient: -1.0,
                    mass: 1.0,
                },
                Dof {
                    axis: 0,
                    index: second,
                    coefficient: 1.0,
                    mass: 1.0,
                },
            ],
            0.3,
        )
        .unwrap();
        let t = common + (common + delta);
        assert!(w.strain_work_scale >= 0.3 * t * t);
        assert_eq!(w.dissipation, 0.3 * delta * delta);
        let terms = [common * w.ku[0][first], (common + delta) * w.ku[0][second]];
        let work = terms[0] + terms[1];
        let error = (work - w.dissipation).abs();
        let old_signed_budget = roundoff(128 * g.cell_len(), work.abs() + w.dissipation).unwrap();
        assert!(error > old_signed_budget);
        let absolute_budget = roundoff(
            128 * g.cell_len(),
            w.strain_work_scale + terms.iter().map(|x| x.abs()).sum::<f64>() + w.dissipation,
        )
        .unwrap();
        assert!(error <= absolute_budget);
        // The actual assembled anisotropic island also traverses apply's work
        // gate and energy gate. This is a stress component fixture, not a
        // divergence-free initial velocity or a supported authored input.
        let bound = w.assemble(&s, || false).unwrap();
        let report = w.apply(&mut s, 0.25 / bound, || false).unwrap();
        assert!(report.work_error <= report.work_budget);
        assert!(report.identity_error <= report.arithmetic_budget);
    }
}
