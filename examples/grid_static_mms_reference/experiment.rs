use super::super::{
    GATE, MU, RHO,
    grid_cut_coupled_reference::solver::{Factor, action, dot},
    grid_cut_variational_reference::{
        assembly::Operators, export, geometry::Polygon, observe::Report, trace::Traces,
    },
};
use super::{cases::Case, field, loads::Loads};
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
fn scaled(value: f64, scale: f64) -> Result<f64, String> {
    if !value.is_finite() || !scale.is_finite() || scale <= 0.0 {
        return Err("invalid CV3 fixed normalization".into());
    }
    Ok(value.abs() / scale)
}
pub fn run(root: &Path, case: &Case) -> Result<(), String> {
    let operators = root.join("operators");
    std::fs::create_dir(&operators).map_err(|e| e.to_string())?;
    export::inputs(&operators, &case.vertices, case.n, case.origin, 1.0)
        .map_err(|e| e.to_string())?;
    let mut input = writer(root, "input.csv", "key,value,bits")?;
    for (k, v) in [
        ("rho", RHO),
        ("mu", MU),
        ("dt", field::DT),
        ("Lref", field::LENGTH),
        ("Uref", field::SPEED),
        ("Href", field::DEPTH),
        ("tau_ref", field::STRESS),
        ("extent", 1.0),
        ("origin_x", case.origin[0]),
        ("origin_y", case.origin[1]),
    ] {
        writeln!(input, "{k},{v:.17e},{}", v.to_bits()).map_err(|e| e.to_string())?;
    }
    for (k, v) in [
        ("case", case.name.as_str()),
        ("geometry", case.geometry),
        ("phase", case.phase),
        ("source_csv_sha256", case.source_sha256),
        ("model", "CV3-A-static-polygon-MMS-v1"),
        ("field_load", "global-polynomial-continuum-v1"),
        ("basis", "CV1-full-consistent-Q1-P0"),
        ("solver", "CV2-scaled-Cholesky-QR-refinement3-cap544"),
        ("traction_rule", "split-three-point-Gauss-v1"),
        ("pressure_gauge", "none"),
        ("u_star", "zero"),
    ] {
        writeln!(input, "{k},{v},").map_err(|e| e.to_string())?;
    }
    writeln!(input, "n,{},", case.n).map_err(|e| e.to_string())?;
    for (i, p) in case.vertices.iter().enumerate() {
        for (d, value) in p.iter().enumerate() {
            writeln!(input, "vertex_{i}_{d},{:.17e},{}", value, value.to_bits())
                .map_err(|e| e.to_string())?;
        }
    }
    input.flush().map_err(|e| e.to_string())?;
    let op = Operators::build(
        case.n,
        case.origin,
        1.0,
        Polygon::new(case.vertices.clone())?,
    )?;
    let tr = Traces::build(&op)?;
    let report = Report::new(&op, &tr)?;
    export::all(&operators, &op, &tr, &report).map_err(|e| e.to_string())?;
    if report.gates.values().any(|v| !v.is_finite() || *v > GATE) {
        return Err("inherited operator consistency gate failed".into());
    }
    let loads = Loads::new(&op, &tr)?;
    let mut w = writer(root, "loads.csv", "index,body,boundary,rhs")?;
    let rhs: Vec<_> = loads
        .body
        .iter()
        .zip(&loads.boundary)
        .map(|(b, t)| b + t)
        .collect();
    for (i, value) in rhs.iter().enumerate() {
        writeln!(
            w,
            "{i},{:.17e},{:.17e},{:.17e}",
            loads.body[i], loads.boundary[i], value
        )
        .map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    let mut w = writer(root, "h1.csv", "row,column,value")?;
    for (i, row) in loads.h1.iter().enumerate() {
        for (j, v) in row.iter().enumerate() {
            writeln!(w, "{i},{j},{v:.17e}").map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())?;
    let mut w = writer(
        root,
        "traction-points.csv",
        "q,record,x,y,nx_weight,ny_weight,gx_weight,gy_weight",
    )?;
    let mut b = writer(root, "traction-basis.csv", "q,index,phi")?;
    for (id, p) in loads.points.iter().enumerate() {
        writeln!(
            w,
            "{id},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            p.record,
            p.point[0],
            p.point[1],
            p.normal_weight[0],
            p.normal_weight[1],
            p.traction_weight[0],
            p.traction_weight[1]
        )
        .map_err(|e| e.to_string())?;
        for (i, phi) in &p.basis {
            writeln!(b, "{id},{i},{phi:.17e}").map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())?;
    b.flush().map_err(|e| e.to_string())?;
    let a = op
        .mass
        .iter()
        .zip(&op.stiffness)
        .map(|(m, k)| m.iter().zip(k).map(|(m, k)| m / field::DT + k).collect())
        .collect();
    let factor = Factor::new(a, op.divergence.clone())?;
    export_factors(root, &factor)?;
    let mut w = writer(
        root,
        "states.csv",
        "method,iteration,kind,index,value,original_residual",
    )?;
    let (u, p) = factor.refined(&rhs, |iteration, (u, p)| {
        let (ru, g) = factor.residual(&rhs, u, p);
        for (kind, values, residuals) in [("u", u, &ru), ("p", p, &g)] {
            for (i, (v, r)) in values.iter().zip(residuals).enumerate() {
                // Actual lower KKT b-Cz is +Bu, opposite correction API g.
                let r = if kind == "p" { -r } else { *r };
                writeln!(w, "selected,{iteration},{kind},{i},{v:.17e},{r:.17e}")
                    .map_err(|e| e.to_string())?;
            }
        }
        w.flush().map_err(|e| e.to_string())
    })?;
    let au = action(&factor.a, &u);
    let bu = action(&factor.b, &u);
    let mut residual = Vec::new();
    let mut gates = BTreeMap::new();
    let mut rowgate = 0.0_f64;
    for i in 0..u.len() {
        let col: Vec<_> = factor.b.iter().map(|r| r[i]).collect();
        let r = dot(&[au[i], -dot(&col, &p), -rhs[i]], &[1.0; 3]);
        residual.push(r);
        let denom = factor.a[i].iter().map(|v| v.abs()).sum::<f64>() * field::SPEED
            + col.iter().map(|v| v.abs()).sum::<f64>() * field::STRESS
            + rhs[i].abs();
        rowgate = rowgate.max(scaled(r, denom)?);
    }
    gates.insert("original_momentum_rows", rowgate);
    let mut con = 0.0_f64;
    let mut flux = 0.0_f64;
    let grid = action(&tr.grid, &u);
    let boundary_flux = action(&tr.physical, &u);
    for c in 0..p.len() {
        con = con.max(scaled(
            bu[c],
            factor.b[c].iter().map(|v| v.abs()).sum::<f64>() * field::SPEED,
        )?);
        flux = flux.max(scaled(
            grid[c] + boundary_flux[c] - bu[c],
            op.layout.h * field::SPEED,
        )?);
        flux = flux.max(scaled(
            grid[c] + boundary_flux[c],
            op.layout.h * field::SPEED,
        )?);
    }
    gates.insert("original_continuity_rows", con);
    gates.insert("physical_closed_cell_flux", flux);
    let power_body = dot(&u, &loads.body);
    let power_boundary = dot(&u, &loads.boundary);
    let mut direct_body = 0.0;
    for q in &op.quadrature {
        let mut uh = [0.0; 2];
        for a in &q.values {
            uh[a.axis] += u[op.map[a.global]] * a.phi;
        }
        direct_body += q.weight * field::DEPTH * dot(&uh, &field::body(q.point));
    }
    let mut direct_boundary = 0.0;
    for q in &loads.points {
        let mut uh = [0.0; 2];
        for &(i, phi) in &q.basis {
            uh[op.active[i].axis] += u[i] * phi;
        }
        direct_boundary += dot(&uh, &q.traction_weight);
    }
    let volume = op.volume.iter().sum::<f64>() * field::DEPTH;
    let energy_scale = RHO * volume * field::SPEED * field::SPEED;
    let mu = action(&op.mass, &u);
    let ku = action(&op.stiffness, &u);
    let energy = 0.5 * dot(&u, &mu);
    let dissipation = dot(&u, &ku);
    let raw = 2.0 * energy + field::DT * (dissipation - power_body - power_boundary);
    let pressure_work = dot(&p, &bu);
    let residual_work = dot(&u, &residual);
    gates.insert("forced_raw_work", scaled(raw, energy_scale)?);
    gates.insert(
        "forced_residual_corrected_work",
        scaled(
            raw - field::DT * (pressure_work + residual_work),
            energy_scale,
        )?,
    );
    gates.insert(
        "absolute_residual_work",
        scaled(
            field::DT
                * (p.iter().zip(&bu).map(|(p, b)| (p * b).abs()).sum::<f64>()
                    + u.iter()
                        .zip(&residual)
                        .map(|(u, r)| (u * r).abs())
                        .sum::<f64>()),
            energy_scale,
        )?,
    );
    gates.insert(
        "physical_body_power",
        scaled(field::DT * (direct_body - power_body), energy_scale)?,
    );
    gates.insert(
        "physical_boundary_power",
        scaled(field::DT * (direct_boundary - power_boundary), energy_scale)?,
    );
    gates.insert(
        "negative_viscous_dissipation",
        scaled(
            (-dissipation).max(0.0),
            MU * volume * field::SPEED * field::SPEED / (field::LENGTH * field::LENGTH),
        )?,
    );
    let mut observations = BTreeMap::from(
        [
            ("energy", energy),
            ("dissipation", dissipation),
            ("body_power", power_body),
            ("boundary_power", power_boundary),
            ("direct_body_power", direct_body),
            ("direct_boundary_power", direct_boundary),
            ("pressure_work", pressure_work),
            ("residual_work", residual_work),
            ("raw_work", raw),
        ]
        .map(|(k, v)| (k.to_string(), v)),
    );
    for d in 0..3 {
        let virtual_field: Vec<_> = op
            .active
            .iter()
            .map(|node| match d {
                0 => {
                    if node.axis == 0 {
                        1.0
                    } else {
                        0.0
                    }
                }
                1 => {
                    if node.axis == 1 {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => {
                    if node.axis == 0 {
                        -node.point[1]
                    } else {
                        node.point[0]
                    }
                }
            })
            .collect();
        let mom: Vec<_> = op.moment.iter().map(|v| v[d]).collect();
        let body = dot(&virtual_field, &loads.body);
        let boundary = dot(&virtual_field, &loads.boundary);
        let inertia = dot(&mom, &u) / field::DT;
        let scale =
            RHO * volume * field::SPEED / field::DT * if d == 2 { field::LENGTH } else { 1.0 };
        let key = match d {
            0 => "x",
            1 => "y",
            _ => "torque",
        };
        gates.insert(
            match d {
                0 => "momentum_x",
                1 => "momentum_y",
                _ => "angular_momentum",
            },
            scaled(inertia - body - boundary, scale)?,
        );
        gates.insert(
            match d {
                0 => "body_resultant_x",
                1 => "body_resultant_y",
                _ => "body_torque",
            },
            scaled(body - loads.body_resultant[d], scale)?,
        );
        gates.insert(
            match d {
                0 => "boundary_resultant_x",
                1 => "boundary_resultant_y",
                _ => "boundary_torque",
            },
            scaled(boundary - loads.boundary_resultant[d], scale)?,
        );
        for (prefix, v) in [
            ("inertia", inertia),
            ("body_nodal", body),
            ("boundary_nodal", boundary),
            ("body_direct", loads.body_resultant[d]),
            ("boundary_direct", loads.boundary_resultant[d]),
        ] {
            observations.insert(format!("{prefix}_{key}"), v);
        }
    }
    let mut w = writer(root, "metrics.csv", "kind,name,value")?;
    for (name, value) in &gates {
        writeln!(w, "gate,{name},{value:.17e}").map_err(|e| e.to_string())?;
    }
    for (name, value) in &observations {
        writeln!(w, "observation,{name},{value:.17e}").map_err(|e| e.to_string())?;
    }
    w.flush().map_err(|e| e.to_string())?;
    if gates
        .values()
        .chain(observations.values())
        .any(|v| !v.is_finite())
    {
        return Err("nonfinite CV3 work/momentum observation".into());
    }
    if gates.values().any(|v| *v > GATE) {
        return Err("fixed CV3 original-equation/work/boundary gate failed".into());
    }
    std::fs::write(
        root.join("native-status.txt"),
        "native algebraic gates passed; independent physical/numerical qualification pending\n",
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn export_factors(root: &Path, f: &Factor) -> Result<(), String> {
    let mut w = writer(root, "factors.csv", "block,row,column,value")?;
    for (name, matrix) in [
        ("A", &f.a),
        ("scaled_A", &f.scaled_a),
        ("L", &f.l),
        ("Y", &f.y),
        ("Z", &f.z),
        ("Q", &f.q),
        ("R", &f.r),
    ] {
        for (i, row) in matrix.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                writeln!(w, "{name},{i},{j},{v:.17e}").map_err(|e| e.to_string())?;
            }
        }
    }
    for (name, values) in [("D", &f.d), ("E", &f.e)] {
        for (i, v) in values.iter().enumerate() {
            writeln!(w, "{name},{i},0,{v:.17e}").map_err(|e| e.to_string())?;
        }
    }
    w.flush().map_err(|e| e.to_string())
}
