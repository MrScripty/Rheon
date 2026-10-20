use super::super::MU;
use super::{
    assembly::{Operators, action, dot},
    basis::{affine, work_vectors},
    trace::Traces,
};
use std::collections::BTreeMap;
pub struct Report {
    pub gates: BTreeMap<&'static str, f64>,
    pub observations: BTreeMap<&'static str, f64>,
    pub full_load: Vec<f64>,
    pub omit_load: Vec<f64>,
}
fn error(a: f64, b: f64, scale: f64) -> Result<f64, String> {
    if !a.is_finite() || !b.is_finite() || !scale.is_finite() || scale <= 0.0 {
        return Err("nonfinite observation or invalid fixed scale".into());
    }
    let e = (a - b).abs() / scale;
    if !e.is_finite() {
        return Err("nonfinite scaled observation".into());
    }
    Ok(e)
}
fn vector_error(a: &[f64], b: &[f64], scale: f64) -> Result<f64, String> {
    if a.len() != b.len() {
        return Err("observation shape mismatch".into());
    }
    a.iter()
        .zip(b)
        .try_fold(0.0_f64, |e, (&a, &b)| Ok(e.max(error(a, b, scale)?)))
}
fn maximum(x: &[f64]) -> f64 {
    x.iter().map(|x| x.abs()).fold(0.0, f64::max)
}
impl Report {
    pub fn new(op: &Operators, tr: &Traces) -> Result<Self, String> {
        let h = op.layout.h;
        let nv = op.active.len();
        let mut gates = BTreeMap::new();
        let mut observations = BTreeMap::new();
        let mut partition: f64 = 0.0;
        let mut reproduction: f64 = 0.0;
        for q in &op.quadrature {
            for axis in 0..2 {
                let values: Vec<_> = q.values.iter().filter(|v| v.axis == axis).collect();
                partition = partition.max(error(values.iter().map(|v| v.phi).sum(), 1.0, 1.0)?);
                for d in 0..2 {
                    reproduction = reproduction.max(error(
                        values
                            .iter()
                            .map(|v| v.phi * op.layout.nodes[v.global].point[d])
                            .sum(),
                        q.point[d],
                        1.0,
                    )?);
                }
            }
        }
        for q in &tr.points {
            for axis in 0..2 {
                let selected: Vec<_> = q
                    .values
                    .iter()
                    .filter(|(i, _)| op.active[*i].axis == axis)
                    .collect();
                partition = partition.max(error(selected.iter().map(|(_, p)| p).sum(), 1.0, 1.0)?);
                for d in 0..2 {
                    reproduction = reproduction.max(error(
                        selected
                            .iter()
                            .map(|(i, p)| p * op.active[*i].point[d])
                            .sum(),
                        q.point[d],
                        1.0,
                    )?);
                }
            }
        }
        gates.insert("partition", partition);
        gates.insert("affine_reproduction", reproduction);
        let mut gradient: f64 = 0.0;
        for (a, b) in op.gradient.iter().zip(&tr.gradient) {
            gradient = gradient.max(vector_error(a, b, 1.0)?);
        }
        gates.insert("nodal_stress_trace", gradient);
        let mut flux: f64 = 0.0;
        for c in 0..op.cells.len() {
            for i in 0..nv {
                flux = flux.max(error(
                    op.divergence[c][i],
                    tr.grid[c][i] + tr.physical[c][i],
                    h,
                )?);
            }
        }
        gates.insert("every_cell_basis_flux", flux);
        let source_m = super::geometry::moments(op.source.vertices())?;
        let mut m = [0.0; 6];
        for q in &op.quadrature {
            let [x, y] = q.point;
            for (t, v) in m.iter_mut().zip([1.0, x, y, x * x, x * y, y * y]) {
                *t += q.weight * v;
            }
        }
        gates.insert(
            "source_moments",
            vector_error(
                &m,
                &[
                    source_m.area,
                    source_m.first[0],
                    source_m.first[1],
                    source_m.second[0],
                    source_m.second[1],
                    source_m.second[2],
                ],
                h * h,
            )?,
        );
        let affine_values: Vec<_> = op.active.iter().map(affine).collect();
        let rotation: Vec<_> = op
            .active
            .iter()
            .map(|p| if p.axis == 0 { -p.point[1] } else { p.point[0] })
            .collect();
        let translations: [Vec<f64>; 2] = std::array::from_fn(|d| {
            op.active
                .iter()
                .map(|p| if p.axis == d { 1.0 } else { 0.0 })
                .collect()
        });
        let mut rigid: f64 = 0.0;
        for v in translations.iter().chain(std::iter::once(&rotation)) {
            rigid = rigid.max(vector_error(
                &action(&op.divergence, v),
                &vec![0.0; op.cells.len()],
                1.0,
            )?);
            rigid = rigid.max(vector_error(&action(&op.stiffness, v), &vec![0.0; nv], MU)?);
        }
        gates.insert("rigid_bulk_actions", rigid);
        gates.insert(
            "affine_divergence",
            vector_error(
                &action(&op.divergence, &affine_values),
                &vec![0.0; op.cells.len()],
                1.0,
            )?,
        );
        let mut moment: f64 = 0.0;
        for (d, v) in translations
            .iter()
            .chain(std::iter::once(&rotation))
            .enumerate()
        {
            let actual = action(&op.mass, v);
            let expected: Vec<_> = op.moment.iter().map(|m| m[d]).collect();
            moment = moment.max(vector_error(&actual, &expected, 1.0)?);
        }
        gates.insert("physical_mass_moments", moment);
        let tau = [MU * 0.5, MU * (0.75 - 0.2), -MU * 0.5];
        let sigma = [tau[0] - 2.0, tau[1], tau[2] - 2.0];
        let full_load: Vec<_> = tr.gradient.iter().map(|t| dot(t, &sigma)).collect();
        let omit_load: Vec<_> = tr.gradient.iter().map(|t| dot(t, &tau)).collect();
        let bt1: Vec<f64> = (0..nv)
            .map(|i| op.divergence.iter().map(|r| r[i]).sum())
            .collect();
        let ku = action(&op.stiffness, &affine_values);
        let residual: Vec<_> = ku.iter().zip(&bt1).map(|(k, b)| k - 2.0 * b).collect();
        gates.insert(
            "affine_nodal_total_stress",
            vector_error(&residual, &full_load, 1.0)?,
        );
        let pressure_diff: Vec<_> = omit_load
            .iter()
            .zip(&full_load)
            .map(|(a, b)| a - b)
            .collect();
        let pressure_expected: Vec<_> = bt1.iter().map(|b| 2.0 * b).collect();
        gates.insert(
            "omitted_pressure_orientation",
            vector_error(&pressure_diff, &pressure_expected, 1.0)?,
        );
        if maximum(&pressure_diff) == 0.0 {
            return Err("pressure-negative control has no nodal signal".into());
        }
        let denominator = dot(&bt1, &bt1);
        if !denominator.is_finite() || denominator <= 0.0 {
            return Err("missing absolute pressure trace direction; no inherited gauge".into());
        }
        let affine_minus_load: Vec<_> = ku.iter().zip(&full_load).map(|(a, b)| a - b).collect();
        let pcoefficient = dot(&bt1, &affine_minus_load) / denominator;
        gates.insert(
            "constant_pressure_load_coefficient",
            error(pcoefficient, 2.0, 1.0)?,
        );
        observations.insert("constant_pressure_load_coefficient_Pa", pcoefficient);
        let a: Vec<_> = op.active.iter().map(|p| work_vectors(p)[0]).collect();
        let b: Vec<_> = op.active.iter().map(|p| work_vectors(p)[1]).collect();
        let delta: Vec<_> = a.iter().zip(&b).map(|(a, b)| a - b).collect();
        let ma = action(&op.mass, &a);
        let mb = action(&op.mass, &b);
        let md = action(&op.mass, &delta);
        let kb = action(&op.stiffness, &b);
        gates.insert(
            "full_mass_polarization",
            error(
                0.5 * dot(&a, &ma) - 0.5 * dot(&b, &mb) + 0.5 * dot(&delta, &md),
                dot(&a, &md),
                1.0,
            )?,
        );
        let mut viscous = 0.0;
        let mut bulk_pressure = 0.0;
        for q in &op.quadrature {
            let mut sa = [0.0; 3];
            let mut sb = [0.0; 3];
            for &v in &q.values {
                let i = op.map[v.global];
                let s = super::assembly::strain(v);
                for d in 0..3 {
                    sa[d] += s[d] * a[i];
                    sb[d] += s[d] * b[i];
                }
            }
            viscous += MU * q.weight * (2.0 * sa[0] * sb[0] + 2.0 * sa[1] * sb[1] + sa[2] * sb[2]);
            let c = q.cell;
            let xc = op.layout.origin[0] + (c % op.layout.n) as f64 * h + h / 2.0;
            let yc = op.layout.origin[1] + (c / op.layout.n) as f64 * h + h / 2.0;
            let pressure = 0.75 + xc / 8.0 - yc / 4.0;
            bulk_pressure += pressure * q.weight * (sa[0] + sa[1]);
        }
        gates.insert("viscous_work", error(dot(&a, &kb), viscous, MU)?);
        let ba = action(&op.divergence, &a);
        let pressure: Vec<_> = op
            .cells
            .iter()
            .map(|&c| {
                let x = op.layout.origin[0] + (c % op.layout.n) as f64 * h + h / 2.0;
                let y = op.layout.origin[1] + (c / op.layout.n) as f64 * h + h / 2.0;
                0.75 + x / 8.0 - y / 4.0
            })
            .collect();
        let btp: Vec<f64> = (0..nv)
            .map(|i| {
                op.divergence
                    .iter()
                    .zip(&pressure)
                    .map(|(r, p)| r[i] * p)
                    .sum()
            })
            .collect();
        gates.insert(
            "pressure_work",
            error(dot(&a, &btp), dot(&pressure, &ba), 1.0)?.max(error(
                dot(&pressure, &ba),
                bulk_pressure,
                1.0,
            )?),
        );
        observations.insert("source_volume", m[0]);
        observations.insert(
            "minimum_positive_mass_diagonal",
            op.mass
                .iter()
                .enumerate()
                .map(|(i, r)| r[i])
                .fold(f64::INFINITY, f64::min),
        );
        observations.insert(
            "zero_old_dual_active_basis",
            op.old_dual.iter().filter(|x| **x == 0.0).count() as f64,
        );
        observations.insert("pressure_nodal_signal", maximum(&pressure_diff));
        observations.insert("affine_boundary_power", dot(&affine_values, &full_load));
        observations.insert("fixed_vector_viscous_cross_work", viscous);
        observations.insert("fixed_vector_pressure_work", bulk_pressure);
        if observations.values().any(|v| !v.is_finite()) || gates.values().any(|v| !v.is_finite()) {
            return Err("nonfinite report field".into());
        }
        Ok(Self {
            gates,
            observations,
            full_load,
            omit_load,
        })
    }
}
