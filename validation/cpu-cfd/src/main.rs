//! Independent elementary measurements through public Rheon CPU APIs.
use rheon::{
    Axis, GridGeometry, PressureImplementation, PressureOperator, PressureSettings,
    PressureWorkspace, ViscosityWorkspace,
};
use std::{
    error::Error,
    f64::consts::PI,
    io::{self, Write},
};

type Fields = [Vec<f32>; 3];
const RHO: f64 = 2.0;
const MU: f64 = 0.1;
const TIME: f64 = 0.1;

fn fields(g: &GridGeometry) -> Fields {
    std::array::from_fn(|d| vec![0.0; g.face_len([Axis::X, Axis::Y, Axis::Z][d])])
}
fn refs(v: &Fields) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn muts(v: &mut Fields) -> [&mut [f32]; 3] {
    let [x, y, z] = v;
    [x, y, z]
}

// This traversal intentionally does not use GridGeometry's index routines or
// PressureOperator::divergence: it observes independent physical face fluxes.
fn diagnostics(g: &GridGeometry, v: &Fields) -> (f64, f64, f64, f64) {
    let n = g.counts();
    let h = g.spacing();
    let at = |d: usize, p: [usize; 3]| {
        let mut m = n;
        m[d] += 1;
        f64::from(v[d][p[0] + m[0] * (p[1] + m[1] * p[2])])
    };
    let mut div_max = 0.0_f64;
    let mut flux = 0.0;
    let mut wall_max = 0.0_f64;
    for k in 0..n[2] {
        for j in 0..n[1] {
            for i in 0..n[0] {
                let p = [i, j, k];
                let mut div = 0.0;
                for d in 0..3 {
                    let mut q = p;
                    q[d] += 1;
                    div += (at(d, q) - at(d, p)) / h[d];
                    let area = h[(d + 1) % 3] * h[(d + 2) % 3];
                    if p[d] == 0 {
                        flux -= at(d, p) * area;
                        wall_max = wall_max.max(at(d, p).abs());
                    }
                    if q[d] == n[d] {
                        flux += at(d, q) * area;
                        wall_max = wall_max.max(at(d, q).abs());
                    }
                }
                div_max = div_max.max(div.abs());
            }
        }
    }
    let energy = 0.5
        * RHO
        * g.cell_volume()
        * v.iter()
            .flatten()
            .map(|&x| f64::from(x).powi(2))
            .sum::<f64>();
    (div_max, flux, wall_max, energy)
}

fn pressure(
    n: usize,
    method: PressureImplementation,
    out: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let h = 1.0 / n as f64;
    let g = GridGeometry::new([n as u64; 3], [h; 3], [0.0; 3])?;
    let op = PressureOperator::new(&g, RHO)?;
    let exact: Vec<f64> = (0..n * n * n)
        .map(|s| {
            let p = [s % n, s / n % n, s / (n * n)];
            p.into_iter()
                .map(|i| (PI * (i as f64 + 0.5) * h).cos())
                .product()
        })
        .collect();
    // Continuum -laplacian(p)/rho integrated over a cell to midpoint order.
    // Do not use apply_full(exact): that would hide truncation errors.
    let rhs: Vec<f64> = exact
        .iter()
        .map(|p| 3.0 * PI * PI * h.powi(3) * p / RHO)
        .collect();
    let settings = PressureSettings {
        relative_residual: 1e-11,
        absolute_residual: 1e-13,
        divergence_limit: 1e-9,
        max_iterations: 5000,
    };
    let mut ws = PressureWorkspace::with_implementation(&g, 64 << 20, method)?;
    let report = ws.solve_rhs(&op, &rhs, 0.02, settings, || false)?;
    let p = ws.pressure();
    let error = p
        .iter()
        .zip(&exact)
        .map(|(a, b)| (a - (b - exact[0])).abs())
        .fold(0.0_f64, f64::max);
    let mut residual = 0.0_f64;
    for s in 0..p.len() {
        let q = [s % n, s / n % n, s / (n * n)];
        let mut ap = 0.0;
        for d in 0..3 {
            let stride = [1, n, n * n][d];
            if q[d] > 0 {
                ap += h / RHO * (p[s] - p[s - stride]);
            }
            if q[d] + 1 < n {
                ap += h / RHO * (p[s] - p[s + stride]);
            }
        }
        residual = residual.max((rhs[s] - ap).abs());
    }
    // Separate projection observation: an independently sampled gradient is
    // projected using the real solve_velocity path, then stored f32 is measured.
    let mut velocity = fields(&g);
    for d in 0..3 {
        let mut m = [n; 3];
        m[d] += 1;
        for k in 0..m[2] {
            for j in 0..m[1] {
                for i in 0..m[0] {
                    let q = [i, j, k];
                    if q[d] == 0 || q[d] == n {
                        continue;
                    }
                    let value = (0..3)
                        .map(|a| {
                            let x = PI * (q[a] as f64 + if a == d { 0.0 } else { 0.5 }) * h;
                            if a == d { -PI * x.sin() } else { x.cos() }
                        })
                        .product::<f64>()
                        * 0.02
                        / RHO;
                    velocity[d][i + m[0] * (j + m[1] * k)] = value as f32;
                }
            }
        }
    }
    ws.solve_velocity(&op, refs(&velocity), 0.02, settings, || false)?;
    op.correct_candidate_in_place(ws.pressure(), 0.02, muts(&mut velocity))?;
    let (div, flux, wall, energy) = diagnostics(&g, &velocity);
    writeln!(
        out,
        "pressure_mesh,{},{},0,0,{:.17e},0,{:.17e},{:.17e},{:.17e},0,{:.17e},{:.17e},{}",
        method.id(),
        n,
        error,
        residual,
        div,
        flux,
        wall,
        energy,
        report.iterations
    )?;
    Ok(())
}

fn viscous(
    n: usize,
    steps: usize,
    study: &str,
    out: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let h = 1.0 / n as f64;
    let g = GridGeometry::new([n as u64, n as u64, 1], [h, h, 0.75], [0.0; 3])?;
    let mut initial = fields(&g);
    for j in 0..n {
        for i in 1..n {
            let x = (PI * i as f64 * h).sin() * (PI * (j as f64 + 0.5) * h).cos();
            initial[0][i + (n + 1) * j] = x as f32;
            initial[1][j + n * i] = -x as f32;
        }
    }
    let mut u = initial.clone();
    let mut next = fields(&g);
    let mut ws = ViscosityWorkspace::new(g.clone(), 64 << 20)?;
    let dt = TIME / steps as f64;
    let mut energy_growth = 0.0_f64;
    let mut div = 0.0_f64;
    let mut flux = 0.0_f64;
    let mut wall = 0.0_f64;
    for _ in 0..steps {
        let before = diagnostics(&g, &u).3;
        ws.update(refs(&u), RHO, MU, dt, muts(&mut next), |_| false)?;
        let d = diagnostics(&g, &next);
        div = div.max(d.0);
        flux = flux.max(d.1.abs());
        wall = wall.max(d.2);
        energy_growth = energy_growth.max(d.3 - before);
        std::mem::swap(&mut u, &mut next);
    }
    let continuum = (-(MU / RHO) * 2.0 * PI * PI * TIME).exp();
    let discrete_lambda = 8.0 * (PI * h / 2.0).sin().powi(2) / (h * h);
    let semidiscrete = (-(MU / RHO) * discrete_lambda * TIME).exp();
    let error_to = |factor: f64| {
        u.iter()
            .flatten()
            .zip(initial.iter().flatten())
            .map(|(&a, &b)| (f64::from(a) - factor * f64::from(b)).abs())
            .fold(0.0_f64, f64::max)
    };
    writeln!(
        out,
        "{study},symmetric-strain-explicit-v1,{n},{steps},{dt:.17e},{:.17e},{:.17e},0,{div:.17e},{flux:.17e},{energy_growth:.17e},{wall:.17e},{:.17e},0",
        error_to(continuum),
        error_to(semidiscrete),
        diagnostics(&g, &u).3
    )?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut out = io::BufWriter::new(io::stdout().lock());
    writeln!(
        out,
        "case,method,n,steps,dt_s,continuum_linf,time_linf,residual_max,divergence_max_per_s,net_flux_m3_per_s,energy_growth_j,wall_normal_max_m_per_s,energy_j,iterations"
    )?;
    for method in PressureImplementation::ALL {
        for n in [6, 12, 24] {
            pressure(n, method, &mut out)?;
        }
    }
    for n in [8, 16, 32] {
        viscous(n, n * n, "viscosity_mesh", &mut out)?;
    }
    for steps in [16, 32, 64] {
        viscous(12, steps, "viscosity_time", &mut out)?;
    }
    out.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_diagnostics_distinguish_local_divergence_boundary_flux_and_energy() {
        let g = GridGeometry::new([2, 1, 1], [0.5, 1.0, 1.0], [0.0; 3]).unwrap();
        let mut u = fields(&g);
        u[0][1] = 2.0;
        assert_eq!(diagnostics(&g, &u), (4.0, 0.0, 0.0, 2.0));
        u[0][0] = 1.0;
        assert_eq!(diagnostics(&g, &u), (4.0, -1.0, 1.0, 2.5));
    }
}
