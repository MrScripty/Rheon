use super::super::{
    GATE, MU, RHO,
    grid_cut_variational_reference::{
        assembly::{Matrix, Operators, zero},
        basis::{affine, work_vectors},
        trace::Traces,
    },
};
use super::solver::{self, Factor, action, dot};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};
fn writer(root: &Path, name: &str, header: &str) -> Result<BufWriter<File>, String> {
    let mut w = BufWriter::new(File::create(root.join(name)).map_err(|e| e.to_string())?);
    writeln!(w, "{header}").map_err(|e| e.to_string())?;
    Ok(w)
}
fn finite(v: f64) -> Result<f64, String> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err("nonfinite physical observation".into())
    }
}
fn scaled(v: f64, s: f64) -> Result<f64, String> {
    if !s.is_finite() || s <= 0.0 {
        return Err("nonpositive fixed normalization".into());
    }
    finite(v.abs() / s)
}
fn norm(a: &Matrix) -> Result<f64, String> {
    let mut maximum = 0.0_f64;
    for row in a {
        if row.iter().any(|v| !v.is_finite()) {
            return Err("nonfinite matrix norm entry".into());
        }
        let sum = row.iter().map(|x| x.abs()).sum::<f64>();
        if !sum.is_finite() {
            return Err("nonfinite matrix norm row sum".into());
        }
        maximum = maximum.max(sum);
    }
    Ok(maximum)
}
pub fn assemble(op: &Operators, dt: f64) -> Result<Factor, String> {
    if !dt.is_finite() || dt <= 0.0 {
        return Err("positive finite dt required".into());
    }
    let a = op
        .mass
        .iter()
        .zip(&op.stiffness)
        .map(|(m, k)| m.iter().zip(k).map(|(m, k)| m / dt + k).collect())
        .collect();
    Factor::new(a, op.divergence.clone())
}
pub fn factor_export(root: &Path, f: &Factor) -> Result<(), String> {
    let mut w = writer(root, "factors.csv", "block,row,column,value")?;
    for (name, m) in [
        ("A", &f.a),
        ("scaled_A", &f.scaled_a),
        ("L", &f.l),
        ("Y", &f.y),
        ("Z", &f.z),
        ("Q", &f.q),
        ("R", &f.r),
    ] {
        for (i, row) in m.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                writeln!(w, "{name},{i},{j},{v:.17e}").map_err(|e| e.to_string())?;
            }
        }
    }
    for (name, v) in [("D", &f.d), ("E", &f.e)] {
        for (i, x) in v.iter().enumerate() {
            writeln!(w, "{name},{i},0,{x:.17e}").map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())?;
    let n = f.d.len();
    let m = f.e.len();
    let mut le = zero(n, n);
    let mut sym = zero(n, n);
    for i in 0..n {
        for j in 0..n {
            le[i][j] = dot(&f.l[i], &f.l[j]);
            sym[i][j] = f.scaled_a[i.max(j)][i.min(j)];
            le[i][j] -= sym[i][j];
        }
    }
    let mut qe = zero(n, m);
    for (i, row) in qe.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            let col: Vec<_> = f.r.iter().map(|r| r[j]).collect();
            *v = dot(&f.q[i], &col) - f.z[i][j];
        }
    }
    let mut orth = zero(m, m);
    for (i, row) in orth.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            let qi: Vec<_> = f.q.iter().map(|r| r[i]).collect();
            let qj: Vec<_> = f.q.iter().map(|r| r[j]).collect();
            *v = dot(&qi, &qj) - if i == j { 1.0 } else { 0.0 };
        }
    }
    let gates = [
        ("cholesky_relative_inf", scaled(norm(&le)?, norm(&sym)?)?),
        ("qr_relative_inf", scaled(norm(&qe)?, norm(&f.z)?)?),
        ("q_orthogonality_inf", norm(&orth)?),
    ];
    let mut w = writer(root, "factor-metrics.csv", "name,value")?;
    for (k, v) in gates {
        writeln!(w, "{k},{v:.17e}").map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    if gates.iter().any(|(_, v)| *v > GATE) {
        return Err("fixed factor identity gate failed".into());
    }
    Ok(())
}
pub fn run(
    root: &Path,
    op: &Operators,
    tr: &Traces,
    factor: &Factor,
    dt: f64,
    scenario: &str,
) -> Result<f64, String> {
    let n = op.active.len();
    let m = op.cells.len();
    let pref = RHO / dt + MU;
    let star: Vec<_> = op
        .active
        .iter()
        .map(|p| match scenario {
            "unforced" => work_vectors(p)[1],
            "rigid" => {
                if p.axis == 0 {
                    0.3 - p.point[1]
                } else {
                    -0.2 + p.point[0]
                }
            }
            "affine" => affine(p),
            _ => unreachable!(),
        })
        .collect();
    let sigma = if scenario == "affine" {
        [MU * 0.5 - 2.0, MU * (0.75 - 0.2), -MU * 0.5 - 2.0]
    } else {
        [0.0; 3]
    };
    let load: Vec<_> = tr.gradient.iter().map(|g| dot(g, &sigma)).collect();
    let mass_star = action(&op.mass, &star);
    let rhs: Vec<_> = mass_star
        .iter()
        .zip(&load)
        .map(|(v, l)| v / dt + l)
        .collect();
    let mut meta = writer(root, "input.csv", "key,value")?;
    for (k, v) in [
        ("dt", dt),
        ("rho", RHO),
        ("mu", MU),
        ("Uref", 1.0),
        ("Lref", 1.0),
        ("Href", 1.0),
        ("Pref", pref),
        ("grad_xx", 0.25),
        ("grad_xy", 0.75),
        ("grad_yx", -0.2),
        ("grad_yy", -0.25),
        ("offset_x", 0.1),
        ("offset_y", -0.3),
        ("rigid_offset_x", 0.3),
        ("rigid_offset_y", -0.2),
        ("pressure", 2.0),
        ("sigma_xx", sigma[0]),
        ("sigma_xy", sigma[1]),
        ("sigma_yy", sigma[2]),
    ] {
        writeln!(meta, "{k},{v:.17e}").map_err(|e| e.to_string())?;
    }
    writeln!(
        meta,
        "scenario,{scenario}\nmethod,scaled-Cholesky-QR-refinement3\npressure_gauge,none"
    )
    .map_err(|e| e.to_string())?;
    meta.flush().map_err(|e| e.to_string())?;
    let mut inputs = writer(root, "vectors.csv", "index,star,load,rhs")?;
    for i in 0..n {
        writeln!(
            inputs,
            "{i},{:.17e},{:.17e},{:.17e}",
            star[i], load[i], rhs[i]
        )
        .map_err(|e| e.to_string())?;
    }
    inputs.flush().map_err(|e| e.to_string())?;
    let mut states = writer(
        root,
        "states.csv",
        "method,iteration,kind,index,value,original_residual",
    )?;
    let mut kkt = zero(n + m, n + m);
    for (i, row) in kkt.iter_mut().take(n).enumerate() {
        row[..n].copy_from_slice(&factor.a[i]);
        for c in 0..m {
            row[n + c] = -factor.b[c][i];
        }
    }
    for (c, row) in kkt.iter_mut().skip(n).enumerate() {
        for (i, value) in row.iter_mut().take(n).enumerate() {
            *value = -factor.b[c][i];
        }
    }
    let mut combined = rhs.clone();
    combined.resize(n + m, 0.0);
    match solver::baseline(&kkt, &combined) {
        Ok(x) => write_state(&mut states, "baseline", 0, &x[..n], &x[n..], factor, &rhs)?,
        Err(e) => {
            std::fs::write(root.join("baseline-refusal.txt"), e).map_err(|e| e.to_string())?
        }
    }
    let (u, p) = factor.refined(&rhs, |i, (u, p)| {
        write_state(&mut states, "selected", i, u, p, factor, &rhs)
    })?;
    states.flush().map_err(|e| e.to_string())?;
    let (u, p) = (&u, &p);
    let delta: Vec<_> = u.iter().zip(&star).map(|(u, v)| u - v).collect();
    let md = action(&op.mass, &delta);
    let ku = action(&op.stiffness, u);
    let bu = action(&op.divergence, u);
    let mut physical_r = vec![0.0; n];
    let mut gates = BTreeMap::new();
    let mut obs = BTreeMap::new();
    let mut mom = 0.0_f64;
    for i in 0..n {
        let bc: Vec<_> = op.divergence.iter().map(|r| r[i]).collect();
        physical_r[i] = dot(&[md[i] / dt, ku[i], -dot(&bc, p), -load[i]], &[1.0; 4]);
        let denom = op.mass[i].iter().map(|x| x.abs()).sum::<f64>() / dt
            + op.stiffness[i].iter().map(|x| x.abs()).sum::<f64>()
            + bc.iter().map(|x| x.abs()).sum::<f64>() * pref
            + load[i].abs();
        mom = mom.max(scaled(physical_r[i], denom)?);
    }
    gates.insert("physical_momentum_rows", mom);
    let mut continuity = 0.0_f64;
    let mut actual_flux = 0.0_f64;
    let grid = action(&tr.grid, u);
    let physical = action(&tr.physical, u);
    for c in 0..m {
        continuity = continuity.max(scaled(
            bu[c],
            op.divergence[c].iter().map(|v| v.abs()).sum(),
        )?);
        actual_flux = actual_flux.max(scaled(grid[c] + physical[c] - bu[c], op.layout.h)?);
        actual_flux = actual_flux.max(scaled(grid[c] + physical[c], op.layout.h)?);
    }
    gates.insert("continuity_rows", continuity);
    gates.insert("physical_closed_cell_flux", actual_flux);
    let e = 0.5 * dot(u, &action(&op.mass, u));
    let old = 0.5 * dot(&star, &mass_star);
    let echar = 0.5 * op.mass.iter().flatten().sum::<f64>();
    let dloss = 0.5 * dot(&delta, &md);
    let diss = dot(u, &ku);
    let boundary = dot(u, &load);
    let pw = dot(p, &bu);
    let rw = dot(u, &physical_r);
    let raw = e - old + dloss + dt * diss - dt * boundary;
    let corrected = raw - dt * pw - dt * rw;
    let mut mtstar = vec![0.0; n];
    for (i, v) in mtstar.iter_mut().enumerate() {
        let col: Vec<_> = op.mass.iter().map(|r| r[i]).collect();
        *v = dot(&col, &star);
    }
    let skew = 0.5
        * dot(
            u,
            &mass_star
                .iter()
                .zip(mtstar)
                .map(|(a, b)| a - b)
                .collect::<Vec<_>>(),
        );
    gates.insert("raw_work_balance", scaled(raw, echar)?);
    gates.insert("residual_corrected_balance", scaled(corrected, echar)?);
    gates.insert("skew_corrected_balance", scaled(corrected - skew, echar)?);
    gates.insert(
        "absolute_residual_work",
        scaled(
            dt * (p.iter().zip(&bu).map(|(p, b)| (p * b).abs()).sum::<f64>()
                + u.iter()
                    .zip(&physical_r)
                    .map(|(u, r)| (u * r).abs())
                    .sum::<f64>()),
            echar,
        )?,
    );
    if scenario != "affine" {
        gates.insert(
            "unforced_energy_increase",
            scaled((e - old).max(0.0), echar)?,
        );
    }
    gates.insert("negative_dissipation", scaled((-diss).max(0.0), MU)?);
    let mut traced_work = 0.0;
    for q in &tr.points {
        if tr.records[q.record].kind != "global" {
            continue;
        }
        let [nx, ny] = q.normal_weight;
        let traction = [sigma[0] * nx + sigma[1] * ny, sigma[1] * nx + sigma[2] * ny];
        traced_work += q
            .values
            .iter()
            .map(|&(i, phi)| u[i] * phi * traction[op.active[i].axis])
            .sum::<f64>();
    }
    gates.insert(
        "physical_boundary_work",
        scaled(traced_work - boundary, 1.0)?,
    );
    let mut global = 0.0_f64;
    for d in 0..3 {
        let moment: Vec<_> = op.moment.iter().map(|m| m[d]).collect();
        let w: Vec<_> = op
            .active
            .iter()
            .map(|p| match d {
                0 => {
                    if p.axis == 0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                1 => {
                    if p.axis == 1 {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => {
                    if p.axis == 0 {
                        -p.point[1]
                    } else {
                        p.point[0]
                    }
                }
            })
            .collect();
        global = global.max(scaled(dot(&moment, &delta) - dt * dot(&w, &load), 1.0)?);
    }
    gates.insert("physical_linear_angular_momentum", global);
    if scenario == "rigid" || scenario == "affine" {
        let ue = u
            .iter()
            .zip(&star)
            .map(|(u, s)| (u - s).abs())
            .fold(0.0_f64, f64::max);
        let expected = if scenario == "affine" { 2.0 } else { 0.0 };
        let pe = p
            .iter()
            .map(|p| (p - expected).abs() / pref)
            .fold(0.0_f64, f64::max);
        gates.insert("analytic_velocity_coefficients", ue);
        gates.insert("analytic_absolute_pressure", pe);
    }
    for (k, v) in [
        ("energy", e),
        ("old_energy", old),
        ("Echar", echar),
        ("delta_energy", dloss),
        ("dissipation", diss),
        ("boundary_work", boundary),
        ("trace_boundary_work", traced_work),
        ("pressure_work", pw),
        ("momentum_residual_work", rw),
        ("mass_skew", skew),
        ("raw_work", raw),
        ("residual_corrected_work", corrected),
    ] {
        obs.insert(k, finite(v)?);
    }
    let worst = gates.values().copied().fold(0.0_f64, f64::max);
    let mut metrics = writer(root, "metrics.csv", "name,value,gate")?;
    for (k, v) in gates {
        writeln!(metrics, "{k},{v:.17e},true").map_err(|e| e.to_string())?;
    }
    for (k, v) in obs {
        writeln!(metrics, "{k},{v:.17e},false").map_err(|e| e.to_string())?;
    }
    metrics.flush().map_err(|e| e.to_string())?;
    if worst > GATE {
        return Err(format!(
            "fixed native solve gate {worst:.17e} exceeds {GATE}"
        ));
    }
    Ok(worst)
}
fn write_state(
    w: &mut BufWriter<File>,
    method: &str,
    iteration: usize,
    u: &[f64],
    p: &[f64],
    f: &Factor,
    rhs: &[f64],
) -> Result<(), String> {
    let (ru, g) = f.residual(rhs, u, p);
    // Lower KKT residual is +B*u; the correction solve takes g=-B*u.
    let lower: Vec<_> = g.iter().map(|v| -v).collect();
    for (kind, v, residual) in [("velocity", u, &ru), ("pressure", p, &lower)] {
        for (i, x) in v.iter().enumerate() {
            writeln!(
                w,
                "{method},{iteration},{kind},{i},{x:.17e},{:.17e}",
                residual[i]
            )
            .map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_dt_refuses_before_operator_arithmetic() {
        let src = super::super::super::fixtures().remove(0);
        let op = Operators::build(
            src.n,
            src.origin,
            src.extent,
            super::super::super::Polygon::new(src.vertices).unwrap(),
        )
        .unwrap();
        assert!(assemble(&op, 0.0).err().unwrap().contains("dt"));
        assert!(assemble(&op, f64::NAN).err().unwrap().contains("dt"));
    }
}
#[cfg(test)]
mod finite_contract {
    use super::*;
    #[test]
    fn nonfinite_matrix_norm_cannot_be_hidden_by_maximum() {
        assert!(
            norm(&vec![vec![f64::NAN], vec![1.0]])
                .unwrap_err()
                .contains("entry")
        );
        assert!(
            norm(&vec![vec![f64::MAX, f64::MAX]])
                .unwrap_err()
                .contains("row sum")
        );
    }
}
