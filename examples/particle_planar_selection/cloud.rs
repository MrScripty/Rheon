//! Actual 3D periodic ordered/perturbed native operator consistency screens.
use super::{eos::WATER, kernel::Kernel, noise};
use std::{f64::consts::PI, io::Write};
#[derive(Clone, Copy, Debug)]
pub enum Stencil {
    Wendland,
    Preview,
}
impl Stencil {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wendland => "wendland-c2-morris-3d-screen-v1",
            Self::Preview => "preview-poly6-spiky-3d-support-screen-v1",
        }
    }
    fn weights(self, k: Kernel, r: f64) -> (f64, f64, f64) {
        match self {
            Self::Wendland => (k.volume(r), k.phi(r), 2.0 * k.phi(r)),
            Self::Preview => {
                let h = k.radius;
                let poly = 315.0 / (64.0 * PI * h.powi(9));
                (
                    poly * (h * h - r * r).powi(3),
                    6.0 * poly * (h * h - r * r).powi(2),
                    45.0 / (PI * h.powi(6)) * (h - r),
                )
            }
        }
    }
}
pub fn run(
    n: usize,
    ratio: f64,
    perturbed: bool,
    stencil: Stencil,
    summary: &mut impl Write,
    raw: &mut impl Write,
) -> std::io::Result<()> {
    let length = 0.001;
    let dx = length / n as f64;
    let volume = dx.powi(3);
    let mass = WATER.rho0 * volume;
    let k = Kernel {
        radius: 2.0 * ratio * dx,
    };
    let count = n * n * n;
    let mut positions = Vec::with_capacity(count);
    let mut velocities = Vec::with_capacity(count);
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let id = positions.len();
                let displacement = std::array::from_fn::<_, 3, _>(|d| noise(id as u64, d as u64));
                let scale = displacement
                    .iter()
                    .map(|v| v * v)
                    .sum::<f64>()
                    .sqrt()
                    .max(1.0);
                let p: [f64; 3] = std::array::from_fn(|d| {
                    let coordinate = [x, y, z][d];
                    (coordinate as f64 + 0.5) * dx
                        + if perturbed {
                            0.1 * dx * displacement[d] / scale
                        } else {
                            0.0
                        }
                });
                positions.push(p);
                velocities.push(0.001 * (2.0 * PI * p[1] / length).sin());
            }
        }
    }
    let self_weight = stencil.weights(k, 0.0).0 * volume;
    let mut partition = vec![self_weight; count];
    let mut gradient = vec![0.0; count];
    let mut laplacian = vec![0.0; count];
    let mut acceleration = vec![0.0; count];
    let mut neighbors = vec![0_usize; count];
    let reach = (k.radius / dx + if perturbed { 0.2 } else { 0.0 }).ceil() as isize;
    assert!(2 * reach + 1 < n as isize); // This bounded source never duplicates periodic cells.
    for i in 0..count {
        let base = [i % n, (i / n) % n, i / (n * n)];
        for oz in -reach..=reach {
            for oy in -reach..=reach {
                for ox in -reach..=reach {
                    let index = |d: usize, offset: isize| {
                        (base[d] as isize + offset).rem_euclid(n as isize) as usize
                    };
                    let j = (index(2, oz) * n + index(1, oy)) * n + index(0, ox);
                    if j <= i {
                        continue;
                    }
                    let delta = std::array::from_fn::<_, 3, _>(|d| {
                        let v = positions[i][d] - positions[j][d];
                        v - length * (v / length).round()
                    });
                    let r2: f64 = delta.iter().map(|v| v * v).sum();
                    if r2 >= k.radius.powi(2) {
                        continue;
                    }
                    let (w, g, lap) = stencil.weights(k, r2.sqrt());
                    let a = WATER.mu / WATER.rho0 * volume * lap * (velocities[j] - velocities[i]);
                    for id in [i, j] {
                        partition[id] += volume * w;
                        gradient[id] += volume * g * delta[1].powi(2);
                        laplacian[id] += 0.5 * volume * lap * delta[1].powi(2);
                        neighbors[id] += 1;
                    }
                    acceleration[i] += a;
                    acceleration[j] -= a;
                }
            }
        }
    }
    let rms = |x: &[f64]| (x.iter().map(|v| (v - 1.0).powi(2)).sum::<f64>() / count as f64).sqrt();
    let power: f64 = velocities
        .iter()
        .zip(&acceleration)
        .map(|(v, a)| mass * v * a)
        .sum();
    let momentum_rate: f64 = acceleration.iter().map(|a| mass * a).sum();
    let rayleigh = -velocities
        .iter()
        .zip(&acceleration)
        .map(|(v, a)| v * a)
        .sum::<f64>()
        / velocities.iter().map(|v| v * v).sum::<f64>();
    let exact = WATER.mu / WATER.rho0 * (2.0 * PI / length).powi(2);
    writeln!(
        summary,
        "{},{},{},{},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
        stencil.name(),
        n,
        ratio,
        perturbed,
        neighbors.iter().min().unwrap(),
        neighbors.iter().max().unwrap(),
        neighbors.iter().sum::<usize>() as f64 / count as f64,
        rms(&partition),
        rms(&gradient),
        rms(&laplacian),
        rayleigh,
        (rayleigh / exact - 1.0).abs(),
        momentum_rate,
        power
    )?;
    writeln!(
        raw,
        "index,x_m,y_m,z_m,velocity_x_m_s,acceleration_x_m_s2,partition,gradient_y_moment,laplacian_y_moment,neighbors"
    )?;
    for i in 0..count {
        writeln!(
            raw,
            "{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
            i,
            positions[i][0],
            positions[i][1],
            positions[i][2],
            velocities[i],
            acceleration[i],
            partition[i],
            gradient[i],
            laplacian[i],
            neighbors[i]
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn periodic_three_dimensional_native_operator_has_dissipative_pair_power() {
        for stencil in [Stencil::Wendland, Stencil::Preview] {
            let mut summary = Vec::new();
            run(16, 1.5, true, stencil, &mut summary, &mut std::io::sink()).unwrap();
            let row = String::from_utf8(summary).unwrap();
            let values: Vec<_> = row.trim().split(',').collect();
            let momentum: f64 = values[12].parse().unwrap();
            let power: f64 = values[13].parse().unwrap();
            assert!(momentum.abs() < 1e-20);
            assert!(power < 0.0);
        }
    }

    #[test]
    fn linked_offsets_agree_with_independent_periodic_all_pairs() {
        let n = 10;
        let length = 0.001;
        let dx = length / n as f64;
        let h = 1.5 * dx;
        let radius = 2.0 * h;
        let mut raw = Vec::new();
        run(
            n,
            1.5,
            true,
            Stencil::Wendland,
            &mut std::io::sink(),
            &mut raw,
        )
        .unwrap();
        let text = String::from_utf8(raw).unwrap();
        let rows: Vec<Vec<f64>> = text
            .lines()
            .skip(1)
            .map(|line| line.split(',').map(|x| x.parse().unwrap()).collect())
            .collect();
        for (i, row) in rows.iter().enumerate() {
            let base = [i % n, (i / n) % n, i / (n * n)];
            let displacement: f64 = (0..3)
                .map(|d| ((row[d + 1] - (base[d] as f64 + 0.5) * dx) / dx).powi(2))
                .sum();
            assert!(displacement.sqrt() <= 0.1 + 1e-14);
            let mut partition = 0.0;
            let mut acceleration = 0.0;
            let mut neighbors = 0;
            // Deliberately no linked-cell offsets, shared kernel helper or
            // shared minimum-image expression in this enumeration oracle.
            for (j, other) in rows.iter().enumerate() {
                let r2: f64 = (0..3)
                    .map(|d| {
                        let direct = (row[d + 1] - other[d + 1]).abs();
                        direct.min(length - direct).powi(2)
                    })
                    .sum();
                if r2 >= radius * radius {
                    continue;
                }
                if i != j {
                    neighbors += 1;
                }
                let q = r2.sqrt() / (2.0 * h);
                let weight = 21.0 / (16.0 * PI * h.powi(3)) * (1.0 - q).powi(4) * (1.0 + 4.0 * q);
                partition += dx.powi(3) * weight;
                let minus_derivative_over_r = 105.0 / (16.0 * PI * h.powi(5)) * (1.0 - q).powi(3);
                acceleration += 2.0 * WATER.mu / WATER.rho0
                    * dx.powi(3)
                    * minus_derivative_over_r
                    * (other[4] - row[4]);
            }
            assert!((partition - row[6]).abs() < 2e-13);
            assert!((acceleration - row[5]).abs() < 2e-13);
            assert_eq!(neighbors, row[9] as usize);
        }
    }
}
