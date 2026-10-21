use super::{RHO, Result, carrier, sphere::Sphere};
use rheon::{
    GridGeometry, LiquidFlowInterval, LiquidVolumeSettings, LiquidVolumeState, VolumeStamp,
};
use std::{
    io::{BufWriter, Write},
    path::Path,
};
fn stamp(id: u64, version: u64) -> VolumeStamp {
    VolumeStamp { id, version }
}
fn binary(path: &Path, values: &[f64]) -> Result<()> {
    let mut f = BufWriter::new(std::fs::File::create(path)?);
    for v in values {
        f.write_all(&v.to_le_bytes())?;
    }
    f.flush()?;
    Ok(())
}
fn binary32(path: &Path, values: &[f32]) -> Result<()> {
    let mut f = BufWriter::new(std::fs::File::create(path)?);
    for v in values {
        f.write_all(&v.to_le_bytes())?;
    }
    f.flush()?;
    Ok(())
}
struct Fixture {
    fraction: Vec<f64>,
    volume: f64,
}
fn fixture(root: &Path, out: &mut impl Write, name: &str, n: usize, s: Sphere) -> Result<Fixture> {
    let h = 1.0 / n as f64;
    let dv = h * h * h;
    let mut a = Vec::with_capacity(n * n * n);
    let mut b = Vec::with_capacity(n * n * n);
    let mut estimate = 0.0;
    let mut exhausted = false;
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                let lo = [i as f64 * h, j as f64 * h, k as f64 * h];
                let hi = [(i + 1) as f64 * h, (j + 1) as f64 * h, (k + 1) as f64 * h];
                let tolerance = 1e-12 * h * h;
                let p = s.periodic_cell(lo, hi, tolerance);
                let q = s.periodic_cell(lo, hi, tolerance / 16.0);
                a.push(p.value / dv);
                b.push(q.value / dv);
                estimate += p.estimate + q.estimate;
                exhausted |= p.exhausted || q.exhausted;
            }
        }
    }
    let va = a.iter().sum::<f64>() * dv;
    let vb = b.iter().sum::<f64>() * dv;
    let pair = a
        .iter()
        .zip(&b)
        .map(|(a, b)| (a - b).abs() * dv)
        .sum::<f64>()
        / s.volume();
    let empirical = estimate / s.volume() + pair;
    let min = b.iter().copied().fold(f64::INFINITY, f64::min);
    let max = b.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    writeln!(
        out,
        "{name},{n},{:.17e},{:.17e},{:.17e},{:.17e},{va:.17e},{vb:.17e},{empirical:.17e},{pair:.17e},{exhausted},{min:.17e},{max:.17e}",
        s.center[0], s.center[1], s.center[2], s.radius
    )?;
    out.flush()?;
    binary(&root.join(format!("geometry-{name}-{n}.f64")), &b)?;
    binary(&root.join(format!("geometry-pair-{name}-{n}.f64")), &a)?;
    if exhausted
        || min < 0.0
        || max > 1.0
        || empirical > 1e-4
        || (vb - s.volume()).abs() / s.volume() > 1e-4
    {
        return Err(format!(
            "sphere geometry admission failed for {name}/{n}; retain geometry summary"
        )
        .into());
    }
    Ok(Fixture {
        fraction: b,
        volume: s.volume(),
    })
}
fn moments(f: &[f64], n: usize) -> [[f64; 2]; 3] {
    let h = 1.0 / n as f64;
    let mut m = [[0.0; 2]; 3];
    let mass = f.iter().sum::<f64>();
    for k in 0..n {
        for j in 0..n {
            for i in 0..n {
                let p = [i, j, k];
                let q = f[i + n * (j + n * k)];
                for d in 0..3 {
                    let a = 2.0 * std::f64::consts::PI * (p[d] as f64 + 0.5) * h;
                    m[d][0] += q * a.cos() / mass;
                    m[d][1] += q * a.sin() / mass;
                }
            }
        }
    }
    m
}
pub fn run(root: &Path) -> Result<()> {
    let mut geometry = BufWriter::new(std::fs::File::create(root.join("geometry.csv"))?);
    writeln!(
        geometry,
        "name,n,cx,cy,cz,r,volume_coarse,volume_fine,empirical_uncertainty,paired_l1,exhausted,min,max"
    )?;
    let mut out = BufWriter::new(std::fs::File::create(root.join("transport-summary.csv"))?);
    writeln!(
        out,
        "case,n,requested_courant,status,steps,time,initial_volume,final_volume,relative_drift,relative_l1,max_balance,max_divergence,max_courant,min,max,fourier_error"
    )?;
    let mut control = BufWriter::new(std::fs::File::create(root.join("constant-controls.csv"))?);
    writeln!(control, "n,requested_courant,fill,status,max_change,error")?;
    let mut carriers = BufWriter::new(std::fs::File::create(root.join("carriers.csv"))?);
    writeln!(
        carriers,
        "case,n,outward_rate,reverse_outward_rate,divergence,reverse_divergence"
    )?;
    for n in [16, 32, 64] {
        let h = 1.0 / n as f64;
        let grid = GridGeometry::new([n as u64; 3], [h; 3], [0.0; 3])?;
        let initial = fixture(
            root,
            &mut geometry,
            "translation-initial",
            n,
            Sphere {
                center: [0.65, 0.5, 0.5],
                radius: 0.15,
            },
        )?;
        let final_ref = fixture(
            root,
            &mut geometry,
            "translation-final",
            n,
            Sphere {
                center: [0.9, 0.625, 0.5625],
                radius: 0.15,
            },
        )?;
        let vortex = fixture(
            root,
            &mut geometry,
            "vortex-initial",
            n,
            Sphere {
                center: [0.35, 0.35, 0.5],
                radius: 0.15,
            },
        )?;
        for (case, source, target, positive) in [
            (
                "translation",
                &initial,
                &final_ref,
                carrier::fields(&grid, [0.25, 0.125, 0.0625]),
            ),
            ("vortex-adapted", &vortex, &vortex, carrier::vortex(&grid)),
        ] {
            let negative = carrier::reverse(&positive);
            let (rate, div) = carrier::rates(&grid, &positive);
            let (reverse_rate, reverse_div) = carrier::rates(&grid, &negative);
            if div != 0.0 || reverse_div != 0.0 {
                return Err("source-declared discrete curl is not exactly solenoidal".into());
            }
            for (d, field) in positive.iter().enumerate() {
                binary32(&root.join(format!("carrier-{case}-{n}-{d}.f32")), field)?;
            }
            writeln!(
                carriers,
                "{case},{n},{rate:.17e},{reverse_rate:.17e},{div:.17e},{reverse_div:.17e}"
            )?;
            carriers.flush()?;
            if case != "translation" {
                for requested in [0.1, 0.25, 0.5] {
                    for fill in [0.0, 1.0] {
                        let mut state = LiquidVolumeState::new(
                            grid.clone(),
                            RHO,
                            stamp(400, 0),
                            vec![fill; grid.cell_len()],
                            1 << 28,
                        )?;
                        let flow = LiquidFlowInterval::new(
                            &grid,
                            stamp(401, 0),
                            positive.each_ref().map(|f| f.as_slice()),
                            0.0,
                            requested / rate,
                        )?;
                        let result = state.advance_periodic(
                            flow,
                            stamp(402, 0),
                            None,
                            LiquidVolumeSettings {
                                max_outward_courant: 1.0,
                                actual_divergence_limit: 0.0,
                            },
                            |_| false,
                        );
                        let change = state
                            .state()
                            .fraction
                            .iter()
                            .map(|f| (f - fill).abs())
                            .fold(0.0, f64::max);
                        match result {
                            Ok(_) => writeln!(
                                control,
                                "{n},{requested:.17e},{fill},complete,{change:.17e},"
                            )?,
                            Err(e) => writeln!(
                                control,
                                "{n},{requested:.17e},{fill},refused,{change:.17e},\"{e:?}\""
                            )?,
                        }
                        control.flush()?;
                    }
                }
            }
            for requested in [0.1, 0.25, 0.5] {
                let tag = format!("{case}-n{n}-c{:03}", (requested * 100.0) as usize);
                let mut state = LiquidVolumeState::new(
                    grid.clone(),
                    RHO,
                    stamp(100, 0),
                    source.fraction.clone(),
                    1 << 28,
                )?;
                let mut steps = 0usize;
                let mut accepted_dt = Vec::new();
                let mut status = "complete";
                let mut failure = String::new();
                let mut balance: f64 = 0.0;
                let mut divergence: f64 = 0.0;
                let mut courant: f64 = 0.0;
                let mut ledger = BufWriter::new(std::fs::File::create(
                    root.join(format!("ledger-{tag}.csv")),
                )?);
                writeln!(
                    ledger,
                    "step,time,dt,volume,balance,divergence,courant,min,max,external_in,external_out"
                )?;
                let limit = if case == "translation" { 1.0 } else { 0.5 };
                let dt = requested / rate;
                while state.state().time < limit {
                    let actual = dt.min(limit - state.state().time);
                    let flow = LiquidFlowInterval::new(
                        &grid,
                        stamp(200, steps as u64),
                        positive.each_ref().map(|a| a.as_slice()),
                        state.state().time,
                        actual,
                    )?;
                    match state.advance_periodic(
                        flow,
                        stamp(300, 0),
                        None,
                        LiquidVolumeSettings {
                            max_outward_courant: 1.0,
                            actual_divergence_limit: 0.0,
                        },
                        |_| false,
                    ) {
                        Ok(r) => {
                            steps += 1;
                            accepted_dt.push(actual);
                            balance = balance.max(r.volume_balance_error.abs());
                            divergence = divergence.max(r.actual_divergence_max);
                            courant = courant.max(r.max_outward_courant);
                            writeln!(
                                ledger,
                                "{steps},{:.17e},{actual:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                                r.time,
                                r.liquid_volume_after,
                                r.volume_balance_error,
                                r.actual_divergence_max,
                                r.max_outward_courant,
                                r.fraction_min,
                                r.fraction_max,
                                r.inward_boundary_volume,
                                r.outward_boundary_volume
                            )?;
                        }
                        Err(e) => {
                            status = "refused";
                            failure = format!(
                                "phase=forward start={:.17e} dt={actual:.17e} error={e:?}",
                                state.state().time
                            );
                            break;
                        }
                    }
                }
                if status == "complete" && case != "translation" {
                    let sequence: Vec<_> = accepted_dt.iter().rev().copied().collect();
                    let mut j = 0;
                    while state.state().time < 1.0 {
                        let remaining = 1.0 - state.state().time;
                        let actual = sequence
                            .get(j)
                            .copied()
                            .unwrap_or(remaining)
                            .min(remaining)
                            .min(dt);
                        j += 1;
                        let flow = LiquidFlowInterval::new(
                            &grid,
                            stamp(200, steps as u64),
                            negative.each_ref().map(|a| a.as_slice()),
                            state.state().time,
                            actual,
                        )?;
                        match state.advance_periodic(
                            flow,
                            stamp(300, 0),
                            None,
                            LiquidVolumeSettings {
                                max_outward_courant: 1.0,
                                actual_divergence_limit: 0.0,
                            },
                            |_| false,
                        ) {
                            Ok(r) => {
                                steps += 1;
                                balance = balance.max(r.volume_balance_error.abs());
                                divergence = divergence.max(r.actual_divergence_max);
                                courant = courant.max(r.max_outward_courant);
                                writeln!(
                                    ledger,
                                    "{steps},{:.17e},{actual:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
                                    r.time,
                                    r.liquid_volume_after,
                                    r.volume_balance_error,
                                    r.actual_divergence_max,
                                    r.max_outward_courant,
                                    r.fraction_min,
                                    r.fraction_max,
                                    r.inward_boundary_volume,
                                    r.outward_boundary_volume
                                )?;
                            }
                            Err(e) => {
                                status = "refused";
                                failure = format!(
                                    "phase=reverse start={:.17e} dt={actual:.17e} rate={reverse_rate:.17e} error={e:?}",
                                    state.state().time
                                );
                                break;
                            }
                        }
                    }
                }
                ledger.flush()?;
                if !failure.is_empty() {
                    std::fs::write(root.join(format!("refusal-{tag}.txt")), &failure)?;
                }
                let f = state.state().fraction;
                let dv = h * h * h;
                let v0 = source.fraction.iter().sum::<f64>() * dv;
                let vf = f.iter().sum::<f64>() * dv;
                let diagnostic_l1 = f
                    .iter()
                    .zip(&target.fraction)
                    .map(|(a, b)| (a - b).abs() * dv)
                    .sum::<f64>()
                    / source.volume;
                let m = moments(f, n);
                let reference = moments(&target.fraction, n);
                let diagnostic_moment_error = (0..3)
                    .flat_map(|d| (0..2).map(move |j| (m[d][j] - reference[d][j]).abs()))
                    .fold(0.0, f64::max);
                let l1 = if status == "complete" {
                    diagnostic_l1
                } else {
                    f64::NAN
                };
                let moment_error = if status == "complete" {
                    diagnostic_moment_error
                } else {
                    f64::NAN
                };
                writeln!(
                    out,
                    "{case},{n},{requested:.17e},{status},{steps},{:.17e},{v0:.17e},{vf:.17e},{:.17e},{l1:.17e},{balance:.17e},{divergence:.17e},{courant:.17e},{:.17e},{:.17e},{moment_error:.17e}",
                    state.state().time,
                    (vf - v0).abs() / v0,
                    f.iter().copied().fold(f64::INFINITY, f64::min),
                    f.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                )?;
                out.flush()?;
                binary(&root.join(format!("accepted-{tag}.f64")), f)?;
                eprintln!(
                    "Q1 {tag}: {status}, {steps} steps, t={:.9}",
                    state.state().time
                );
            }
        }
    }
    control.flush()?;
    carriers.flush()?;
    geometry.flush()?;
    out.flush()?;
    Ok(())
}
