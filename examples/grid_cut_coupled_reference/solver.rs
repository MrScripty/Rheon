//! Invertible scaling/QR factors; original full blocks remain the solve target.
use super::super::grid_cut_variational_reference::assembly::{Matrix, zero};
pub type State = (Vec<f64>, Vec<f64>);
fn finite(values: impl Iterator<Item = f64>) -> Result<(), String> {
    if values.into_iter().any(|v| !v.is_finite()) {
        Err("nonfinite solver arithmetic".into())
    } else {
        Ok(())
    }
}
/// Compensated products and summation; no physics or pivot floor.
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    let mut sum = 0.0_f64;
    let mut correction = 0.0;
    for (&x, &y) in a.iter().zip(b) {
        let p = x * y;
        let t = sum + p;
        correction += if sum.abs() >= p.abs() {
            (sum - t) + p
        } else {
            (p - t) + sum
        };
        correction += x.mul_add(y, -p);
        sum = t;
    }
    sum + correction
}
pub fn action(a: &Matrix, x: &[f64]) -> Vec<f64> {
    a.iter().map(|r| dot(r, x)).collect()
}
fn validate(a: &Matrix) -> Result<usize, String> {
    let n = a.len();
    if n == 0 || n > 256 || a.iter().any(|r| r.len() != n) {
        return Err("bounded square matrix required".into());
    }
    finite(a.iter().flatten().copied())?;
    Ok(n)
}
pub fn cholesky(a: &Matrix) -> Result<Matrix, String> {
    let n = validate(a)?;
    let mut l = zero(n, n);
    for i in 0..n {
        for j in 0..=i {
            let v = a[i][j] - dot(&l[i][..j], &l[j][..j]);
            if i == j {
                if !v.is_finite() || v <= 0.0 {
                    return Err(format!("nonpositive Cholesky pivot {i}; no floor"));
                }
                l[i][j] = v.sqrt();
            } else {
                l[i][j] = v / l[j][j];
            }
        }
    }
    finite(l.iter().flatten().copied())?;
    Ok(l)
}
/// Upper R from a thin, unpivoted Householder QR of full-column-rank Z.
pub fn qr(z: &Matrix) -> Result<Matrix, String> {
    let n = z.len();
    let m = z.first().map_or(0, Vec::len);
    if m == 0 || m > n || n > 256 || z.iter().any(|r| r.len() != m) {
        return Err("bounded tall QR matrix required".into());
    }
    finite(z.iter().flatten().copied())?;
    let mut a = z.clone();
    for k in 0..m {
        let norm = (k..n).fold(0.0_f64, |v, i| v.hypot(a[i][k]));
        if norm == 0.0 || !norm.is_finite() {
            return Err(format!("zero QR pivot {k}; no rank deletion"));
        }
        let alpha = -norm.copysign(a[k][k]);
        let first = a[k][k] - alpha;
        if first == 0.0 || !first.is_finite() {
            return Err("invalid Householder reflector".into());
        }
        let mut v: Vec<_> = (k..n).map(|i| a[i][k] / first).collect();
        v[0] = 1.0;
        let beta = 2.0 / dot(&v, &v);
        let mut j = k + 1;
        while j < m {
            let col: Vec<_> = (k..n).map(|i| a[i][j]).collect();
            let t = beta * dot(&v, &col);
            for (i, &vi) in (k..n).zip(&v) {
                a[i][j] -= t * vi;
            }
            j += 1;
        }
        a[k][k] = alpha;
        for row in a.iter_mut().skip(k + 1) {
            row[k] = 0.0;
        }
    }
    let r: Matrix = a.into_iter().take(m).collect();
    finite(r.iter().flatten().copied())?;
    if (0..m).any(|i| r[i][i] == 0.0) {
        return Err("zero QR diagonal".into());
    }
    Ok(r)
}
/// Solve a triangular factor or its transpose; zero/nonfinite pivots refuse.
fn triangular(t: &Matrix, b: &[f64], lower: bool, transpose: bool) -> Result<Vec<f64>, String> {
    let n = t.len();
    if b.len() != n {
        return Err("triangular RHS shape".into());
    }
    let forward = lower != transpose;
    let mut x = vec![0.0; n];
    for step in 0..n {
        let i = if forward { step } else { n - 1 - step };
        let ids: Vec<_> = if forward {
            (0..i).collect()
        } else {
            (i + 1..n).collect()
        };
        let row: Vec<_> = ids
            .iter()
            .map(|&j| if transpose { t[j][i] } else { t[i][j] })
            .collect();
        let known: Vec<_> = ids.iter().map(|&j| x[j]).collect();
        let pivot = t[i][i];
        if !pivot.is_finite() || pivot == 0.0 {
            return Err("zero/nonfinite triangular pivot".into());
        }
        x[i] = (b[i] - dot(&row, &known)) / pivot;
    }
    finite(x.iter().copied())?;
    Ok(x)
}
pub struct Factor {
    pub a: Matrix,
    pub b: Matrix,
    pub scaled_a: Matrix,
    pub d: Vec<f64>,
    pub e: Vec<f64>,
    pub l: Matrix,
    pub y: Matrix,
    pub z: Matrix,
    pub r: Matrix,
    pub q: Matrix,
}
impl Factor {
    pub fn new(a: Matrix, b: Matrix) -> Result<Self, String> {
        let n = validate(&a)?;
        let m = b.len();
        if m == 0 || m > n || b.iter().any(|r| r.len() != n) {
            return Err("bounded full-row-rank divergence shape required".into());
        }
        finite(b.iter().flatten().copied())?;
        let d = (0..n)
            .map(|i| {
                let v = a[i][i];
                if !v.is_finite() || v <= 0.0 {
                    return Err("nonpositive velocity scaling diagonal".into());
                }
                let d = 1.0 / v.sqrt();
                if !d.is_finite() || d <= 0.0 {
                    return Err("unrepresentable velocity scale".into());
                }
                Ok(d)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let scaled_a: Matrix = (0..n)
            .map(|i| (0..n).map(|j| d[i] * a[i][j] * d[j]).collect())
            .collect();
        let l = cholesky(&scaled_a)?;
        let mut y = zero(n, m);
        for c in 0..m {
            let rhs: Vec<_> = (0..n).map(|i| d[i] * b[c][i]).collect();
            let col = triangular(&l, &rhs, true, false)?;
            for i in 0..n {
                y[i][c] = col[i];
            }
        }
        let e = (0..m)
            .map(|c| {
                let norm = (0..n).fold(0.0_f64, |v, i| v.hypot(y[i][c]));
                let e = 1.0 / norm;
                if !e.is_finite() || e <= 0.0 {
                    return Err("nonpositive/unrepresentable pressure scale".into());
                }
                Ok(e)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let z: Matrix = y
            .iter()
            .map(|r| r.iter().zip(&e).map(|(y, e)| y * e).collect())
            .collect();
        let r = qr(&z)?;
        let q = z
            .iter()
            .map(|row| triangular(&r, row, false, true))
            .collect::<Result<Matrix, String>>()?;
        Ok(Self {
            a,
            b,
            scaled_a,
            d,
            e,
            l,
            y,
            z,
            r,
            q,
        })
    }
    /// g is the desired B*u, not the lower KKT right-hand side -g.
    pub fn solve(&self, f: &[f64], g: &[f64]) -> Result<(Vec<f64>, Vec<f64>), String> {
        let n = self.d.len();
        let m = self.e.len();
        if f.len() != n || g.len() != m {
            return Err("coupled RHS shape".into());
        }
        finite(f.iter().chain(g).copied())?;
        let df: Vec<_> = f.iter().zip(&self.d).map(|(f, d)| f * d).collect();
        let v = triangular(&self.l, &df, true, false)?;
        let rhs: Vec<_> = (0..m)
            .map(|c| {
                let col: Vec<_> = self.y.iter().map(|r| r[c]).collect();
                self.e[c] * (g[c] - dot(&col, &v))
            })
            .collect();
        let q = triangular(&self.r, &rhs, false, true)?;
        let q = triangular(&self.r, &q, false, false)?;
        let p: Vec<_> = q.iter().zip(&self.e).map(|(q, e)| q * e).collect();
        let yp = action(&self.y, &p);
        let uv: Vec<_> = v.iter().zip(&yp).map(|(v, y)| v + y).collect();
        let u = triangular(&self.l, &uv, true, true)?
            .iter()
            .zip(&self.d)
            .map(|(u, d)| u * d)
            .collect::<Vec<_>>();
        finite(u.iter().chain(&p).copied())?;
        Ok((u, p))
    }
    pub fn residual(&self, f: &[f64], u: &[f64], p: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let bu = action(&self.b, u);
        let ru = (0..u.len())
            .map(|i| {
                let mut row: Vec<_> = self.a[i].iter().map(|a| -a).collect();
                row.extend(self.b.iter().map(|r| r[i]));
                row.push(f[i]);
                let mut value = u.to_vec();
                value.extend_from_slice(p);
                value.push(1.0);
                dot(&row, &value)
            })
            .collect();
        (ru, bu.iter().map(|b| -b).collect())
    }
    pub fn refined(
        &self,
        f: &[f64],
        mut observe: impl FnMut(usize, &State) -> Result<(), String>,
    ) -> Result<State, String> {
        let mut state = self.solve(f, &vec![0.0; self.e.len()])?;
        observe(0, &state)?;
        for i in 1..=3 {
            let (u, p) = &state;
            let (ru, g) = self.residual(f, u, p);
            let (du, dp) = self.solve(&ru, &g)?;
            let next = (
                u.iter().zip(du).map(|(u, d)| u + d).collect::<Vec<_>>(),
                p.iter().zip(dp).map(|(p, d)| p + d).collect::<Vec<_>>(),
            );
            finite(next.0.iter().chain(&next.1).copied())?;
            observe(i, &next)?;
            state = next;
        }
        Ok(state)
    }
}
/// Diagnostic baseline: established aligned-reference partial-pivot elimination.
pub fn baseline(a: &Matrix, b: &[f64]) -> Result<Vec<f64>, String> {
    let n = validate(a)?;
    if b.len() != n {
        return Err("baseline RHS shape".into());
    }
    finite(b.iter().copied())?;
    let mut rows = a.clone();
    for (r, v) in rows.iter_mut().zip(b) {
        r.push(*v);
    }
    for k in 0..n {
        let p = (k..n)
            .max_by(|&i, &j| rows[i][k].abs().total_cmp(&rows[j][k].abs()))
            .unwrap();
        if rows[p][k] == 0.0 {
            return Err("singular baseline pivot".into());
        }
        rows.swap(k, p);
        let pivot = rows[k][k];
        for x in &mut rows[k][k..] {
            *x /= pivot;
        }
        let basis = rows[k].clone();
        for (i, row) in rows.iter_mut().enumerate() {
            if i != k {
                let t = row[k];
                for (x, v) in row[k..].iter_mut().zip(&basis[k..]) {
                    *x -= t * v;
                }
            }
        }
        finite(rows.iter().flatten().copied())?;
    }
    Ok(rows.iter().map(|r| r[n]).collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn factors_refuse_invalid_data_and_allow_negative_qr_diagonal() {
        assert!(
            cholesky(&vec![vec![0.0]])
                .unwrap_err()
                .contains("nonpositive")
        );
        assert!(
            cholesky(&vec![vec![f64::NAN]])
                .unwrap_err()
                .contains("nonfinite")
        );
        let r = qr(&vec![vec![1.0, 0.0], vec![0.0, 2.0], vec![0.0, 0.0]]).unwrap();
        assert!(r[0][0] < 0.0 && r[1][1] < 0.0);
        assert!((r[0][0] * r[0][0] - 1.0).abs() < 1e-14);
        assert!(
            qr(&vec![vec![1.0, 0.0], vec![0.0, 0.0]])
                .unwrap_err()
                .contains("zero")
        );
    }
    #[test]
    fn nonzero_constraint_has_original_sign_and_bad_shape_refuses() {
        let f = Factor::new(vec![vec![2.0, 0.0], vec![0.0, 3.0]], vec![vec![1.0, 1.0]]).unwrap();
        let (u, p) = f.solve(&[1.0, 2.0], &[0.75]).unwrap();
        assert!((u[0] + u[1] - 0.75).abs() < 1e-14);
        assert!((2.0 * u[0] - p[0] - 1.0).abs() < 1e-14);
        assert!((3.0 * u[1] - p[0] - 2.0).abs() < 1e-14);
        assert!(f.solve(&[1.0], &[0.0]).unwrap_err().contains("shape"));
        assert_eq!(dot(&[1e16, 1.0, -1e16], &[1.0; 3]), 1.0);
    }
}

#[cfg(test)]
mod qr_contract {
    use super::*;
    #[test]
    fn tall_qr_preserves_gram_and_orthonormal_coordinates() {
        let z = vec![vec![1.0, 2.0], vec![-2.0, 1.0], vec![0.5, 3.0]];
        let r = qr(&z).unwrap();
        let q = z
            .iter()
            .map(|row| triangular(&r, row, false, true).unwrap())
            .collect::<Vec<_>>();
        for i in 0..2 {
            for j in 0..2 {
                let zi: Vec<_> = z.iter().map(|x| x[i]).collect();
                let zj: Vec<_> = z.iter().map(|x| x[j]).collect();
                let ri: Vec<_> = r.iter().map(|x| x[i]).collect();
                let rj: Vec<_> = r.iter().map(|x| x[j]).collect();
                assert!((dot(&zi, &zj) - dot(&ri, &rj)).abs() < 1e-13);
                let qi: Vec<_> = q.iter().map(|x| x[i]).collect();
                let qj: Vec<_> = q.iter().map(|x| x[j]).collect();
                assert!((dot(&qi, &qj) - if i == j { 1.0 } else { 0.0 }).abs() < 1e-14);
            }
        }
    }
}
#[cfg(test)]
mod history_contract {
    use super::*;
    #[test]
    fn correction_observer_keeps_successful_states_on_later_refusal() {
        let f = Factor::new(vec![vec![2.0, 0.0], vec![0.0, 2.0]], vec![vec![1.0, 1.0]]).unwrap();
        let mut seen = Vec::new();
        let result = f.refined(&[1.0, -1.0], |i, state| {
            seen.push((i, state.clone()));
            if i == 1 {
                Err("retained observer refusal".into())
            } else {
                Ok(())
            }
        });
        assert!(result.unwrap_err().contains("observer refusal"));
        assert_eq!(seen.iter().map(|(i, _)| *i).collect::<Vec<_>>(), vec![0, 1]);
        assert!((seen[0].1.0[0] - 0.5).abs() < 1e-14);
    }
}
