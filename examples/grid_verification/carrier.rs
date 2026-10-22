//! Source-declared f32 exactly discrete-solenoidal RT0 face carrier.
use rheon::{Axis, GridGeometry};
const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
pub fn fields(g: &GridGeometry, value: [f32; 3]) -> [Vec<f32>; 3] {
    std::array::from_fn(|d| vec![value[d]; g.face_len(AXES[d])])
}
pub fn vortex(g: &GridGeometry) -> [Vec<f32>; 3] {
    let [n, ny, _] = g.counts();
    assert_eq!(n, ny);
    assert!(n.is_power_of_two());
    let mut psi = vec![0_i64; (n + 1) * (n + 1)];
    for j in 1..n {
        for i in 1..n {
            psi[i + (n + 1) * j] = ((std::f64::consts::PI * i as f64 / n as f64).sin().powi(2)
                * (std::f64::consts::PI * j as f64 / n as f64).sin().powi(2)
                / std::f64::consts::PI
                * 1048576.0)
                .round() as i64;
        }
    }
    let mut u = fields(g, [0.0; 3]);
    for k in 0..g.counts()[2] {
        for j in 0..ny {
            for i in 0..=n {
                let q = (psi[i + (n + 1) * (j + 1)] - psi[i + (n + 1) * j]) * n as i64;
                assert!(q.abs() < (1 << 24));
                u[0][g.face_index(Axis::X, [i, j, k]).unwrap()] = q as f32 * 2.0_f32.powi(-20);
            }
        }
    }
    for k in 0..g.counts()[2] {
        for j in 0..=ny {
            for i in 0..n {
                let q = -(psi[i + 1 + (n + 1) * j] - psi[i + (n + 1) * j]) * n as i64;
                assert!(q.abs() < (1 << 24));
                u[1][g.face_index(Axis::Y, [i, j, k]).unwrap()] = q as f32 * 2.0_f32.powi(-20);
            }
        }
    }
    u
}
pub fn reverse(u: &[Vec<f32>; 3]) -> [Vec<f32>; 3] {
    u.each_ref().map(|f| f.iter().map(|u| -u).collect())
}
pub fn rates(g: &GridGeometry, u: &[Vec<f32>; 3]) -> (f64, f64) {
    let [nx, ny, nz] = g.counts();
    let h = g.spacing();
    let mut outward: f64 = 0.0;
    let mut divergence: f64 = 0.0;
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let p = [i, j, k];
                let mut c = 0.0;
                let mut div = 0.0;
                for d in 0..3 {
                    let mut hi = p;
                    hi[d] += 1;
                    let a = f64::from(u[d][g.face_index(AXES[d], p).unwrap()]);
                    let b = f64::from(u[d][g.face_index(AXES[d], hi).unwrap()]);
                    c += (b.max(0.0) - a.min(0.0)) / h[d];
                    div += (b - a) / h[d];
                }
                outward = outward.max(c);
                divergence = divergence.max(div.abs());
            }
        }
    }
    (outward, divergence)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quantized_curl_and_reversal_have_exact_zero_discrete_divergence() {
        let g = GridGeometry::new([8; 3], [0.125; 3], [0.0; 3]).unwrap();
        let u = vortex(&g);
        let (a, b) = rates(&g, &u);
        let (c, d) = rates(&g, &reverse(&u));
        assert_eq!(b, 0.0);
        assert_eq!(d, 0.0);
        assert_eq!(a, c);
    }
}
#[cfg(test)]
mod native_invariance {
    use super::*;
    use rheon::{LiquidFlowInterval, LiquidVolumeSettings, LiquidVolumeState, VolumeStamp};
    #[test]
    fn discrete_curl_preserves_full_and_dry_at_dyadic_step() {
        let g = GridGeometry::new([8; 3], [0.125; 3], [0.0; 3]).unwrap();
        let u = vortex(&g);
        for fill in [0.0, 1.0] {
            let mut state = LiquidVolumeState::new(
                g.clone(),
                998.2,
                VolumeStamp { id: 1, version: 0 },
                vec![fill; g.cell_len()],
                1 << 20,
            )
            .unwrap();
            let flow = LiquidFlowInterval::new(
                &g,
                VolumeStamp { id: 2, version: 0 },
                u.each_ref().map(|f| f.as_slice()),
                0.0,
                2.0_f64.powi(-10),
            )
            .unwrap();
            let r = state
                .advance_periodic(
                    flow,
                    VolumeStamp { id: 3, version: 0 },
                    None,
                    LiquidVolumeSettings {
                        max_outward_courant: 1.0,
                        actual_divergence_limit: 0.0,
                    },
                    |_| false,
                )
                .unwrap();
            assert!(state.state().fraction.iter().all(|v| *v == fill));
            assert_eq!(r.actual_divergence_max, 0.0);
        }
    }
}
