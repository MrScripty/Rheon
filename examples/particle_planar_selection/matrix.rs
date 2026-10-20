//! Native constant-density viscous explicit-update contraction screen.
//! This is a floating-point matrix calculation, not an interval proof.
pub fn multiply(a: &[Vec<f64>], x: &[f64]) -> Vec<f64> {
    a.iter()
        .map(|row| row.iter().zip(x).map(|(a, x)| a * x).sum())
        .collect()
}
fn positive_definite(mut a: Vec<Vec<f64>>) -> bool {
    let n = a.len();
    for i in 0..n {
        for j in 0..=i {
            let s = a[i][j] - (0..j).map(|k| a[i][k] * a[j][k]).sum::<f64>();
            if i == j {
                if !s.is_finite() || s <= 0.0 {
                    return false;
                }
                a[i][j] = s.sqrt();
            } else {
                a[i][j] = s / a[j][j];
            }
        }
    }
    true
}
pub fn contraction_limit(a: &[Vec<f64>]) -> Option<f64> {
    let n = a.len();
    let square: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| (0..n).map(|k| a[k][i] * a[k][j]).sum())
                .collect()
        })
        .collect();
    let admissible = |dt: f64| {
        positive_definite(
            (0..n)
                .map(|i| {
                    (0..n)
                        .map(|j| -a[i][j] - a[j][i] - dt * square[i][j])
                        .collect()
                })
                .collect(),
        )
    };
    if !admissible(0.0) {
        return None;
    }
    let mut hi = 1.0;
    while admissible(hi) {
        hi *= 2.0;
        if !hi.is_finite() {
            return None;
        }
    }
    let mut lo = 0.0;
    for _ in 0..50 {
        let mid = 0.5 * (lo + hi);
        if admissible(mid) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some(lo)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_bound_comes_from_actual_operator() {
        let a = vec![vec![-2.0, 1.0], vec![1.0, -2.0]];
        assert!((contraction_limit(&a).unwrap() - 2.0 / 3.0).abs() < 1e-13);
        assert!(contraction_limit(&[vec![0.0, 1.0], vec![-1.0, 0.0]]).is_none());
        assert!(contraction_limit(&[vec![1.0, 0.0], vec![0.0, -1.0]]).is_none());
        let x = [0.3, -0.7];
        let y = multiply(&a, &x);
        let dt = 0.9 * contraction_limit(&a).unwrap();
        let old: f64 = x.iter().map(|v| v * v).sum();
        let new: f64 = x.iter().zip(y).map(|(x, y)| (x + dt * y).powi(2)).sum();
        assert!(new < old);
    }
}
