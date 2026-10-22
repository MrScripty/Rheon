//! Exact represented-source clipping and positive integration pieces.
use super::{
    exact::rational,
    geometry::{Point, Polygon, Segment},
};
use num_rational::BigRational as R;
use num_traits::{ToPrimitive, Zero};
pub type QPoint = [R; 2];
#[derive(Clone)]
pub struct Source {
    pub points: Vec<QPoint>,
}
#[derive(Clone, Debug)]
pub struct Triangle {
    pub points: [Point; 3],
    pub exact_cross: R,
    pub cross: f64,
}
fn cross(a: &QPoint, b: &QPoint, c: &QPoint) -> R {
    (&b[0] - &a[0]) * (&c[1] - &a[1]) - (&b[1] - &a[1]) * (&c[0] - &a[0])
}
pub fn native(p: &QPoint) -> Result<Point, String> {
    let x = p[0].to_f64().ok_or("unrepresentable coordinate")?;
    let y = p[1].to_f64().ok_or("unrepresentable coordinate")?;
    if !x.is_finite() || !y.is_finite() {
        return Err("nonfinite represented coordinate".into());
    }
    Ok([x, y])
}
pub fn area(v: &[QPoint]) -> R {
    if v.len() < 3 {
        return R::zero();
    }
    v.iter()
        .zip(v.iter().cycle().skip(1))
        .take(v.len())
        .map(|(a, b)| &a[0] * &b[1] - &a[1] * &b[0])
        .sum::<R>()
        / rational(2.0).unwrap()
}
impl Source {
    pub fn new(p: &Polygon) -> Result<Self, String> {
        Ok(Self {
            points: p
                .vertices()
                .iter()
                .map(|p| Ok([rational(p[0])?, rational(p[1])?]))
                .collect::<Result<_, String>>()?,
        })
    }
    pub fn clip(&self, lo: Point, hi: Point) -> Result<Vec<QPoint>, String> {
        if (0..2).any(|d| !lo[d].is_finite() || !hi[d].is_finite() || lo[d] >= hi[d]) {
            return Err("finite ordered clip box required".into());
        }
        let mut v = self.points.clone();
        for d in 0..2 {
            for (bound, positive) in [(lo[d], true), (hi[d], false)] {
                let plane = rational(bound)?;
                let mut out = Vec::new();
                for (a, b) in v.iter().zip(v.iter().cycle().skip(1)).take(v.len()) {
                    let ia = if positive {
                        a[d] >= plane
                    } else {
                        a[d] <= plane
                    };
                    let ib = if positive {
                        b[d] >= plane
                    } else {
                        b[d] <= plane
                    };
                    if ia != ib {
                        let t = (&plane - &a[d]) / (&b[d] - &a[d]);
                        let mut p = std::array::from_fn(|k| &a[k] + &t * (&b[k] - &a[k]));
                        p[d] = plane.clone();
                        if out.last() != Some(&p) {
                            out.push(p);
                        }
                    }
                    if ib && out.last() != Some(b) {
                        out.push(b.clone());
                    }
                }
                if out.len() > 1 && out.first() == out.last() {
                    out.pop();
                }
                v = out;
            }
        }
        let a = area(&v);
        if a < R::zero() {
            return Err("negative exact clipped area".into());
        }
        if a.is_zero() {
            v.clear();
        }
        Ok(v)
    }
}
pub fn triangles(v: &[QPoint]) -> Result<Vec<Triangle>, String> {
    let mut out = Vec::new();
    if v.len() < 3 {
        return Ok(out);
    }
    for k in 1..v.len() - 1 {
        let exact_cross = cross(&v[0], &v[k], &v[k + 1]);
        if exact_cross < R::zero() {
            return Err("negative exact fan triangle".into());
        }
        if exact_cross.is_zero() {
            continue;
        }
        let points = [native(&v[0])?, native(&v[k])?, native(&v[k + 1])?];
        let [a, b, c] = points;
        let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
        if !cross.is_finite()
            || cross <= 0.0
            || exact_cross
                .to_f64()
                .is_none_or(|x| !x.is_finite() || x <= 0.0)
        {
            return Err("positive exact triangle collapsed or unrepresentable; no deletion".into());
        }
        out.push(Triangle {
            points,
            exact_cross,
            cross,
        });
    }
    Ok(out)
}
pub fn clipped_segment(
    a: &QPoint,
    b: &QPoint,
    lo: Point,
    hi: Point,
) -> Result<Option<[QPoint; 2]>, String> {
    let mut lower = R::zero();
    let mut upper = rational(1.0)?;
    for d in 0..2 {
        let delta = &b[d] - &a[d];
        let l = rational(lo[d])?;
        let h = rational(hi[d])?;
        if delta.is_zero() {
            if a[d] < l || a[d] > h {
                return Ok(None);
            }
        } else {
            let t0 = (l - &a[d]) / &delta;
            let t1 = (h - &a[d]) / delta;
            lower = lower.max(t0.clone().min(t1.clone()));
            upper = upper.min(t0.max(t1));
        }
    }
    if upper <= lower {
        return Ok(None);
    }
    Ok(Some([
        std::array::from_fn(|d| &a[d] + &lower * (&b[d] - &a[d])),
        std::array::from_fn(|d| &a[d] + &upper * (&b[d] - &a[d])),
    ]))
}
pub fn split_segment(s: &[QPoint; 2], planes: &[Vec<R>; 2]) -> Result<Vec<Segment>, String> {
    let mut cuts = vec![R::zero(), rational(1.0)?];
    for d in 0..2 {
        let delta = &s[1][d] - &s[0][d];
        if delta.is_zero() {
            continue;
        }
        for p in &planes[d] {
            let t = (p - &s[0][d]) / &delta;
            if t > R::zero() && t < rational(1.0)? {
                cuts.push(t);
            }
        }
    }
    cuts.sort();
    cuts.dedup();
    let mut result = Vec::new();
    for c in cuts.windows(2) {
        let a = native(&std::array::from_fn(|d| {
            &s[0][d] + &c[0] * (&s[1][d] - &s[0][d])
        }))?;
        let b = native(&std::array::from_fn(|d| {
            &s[0][d] + &c[1] * (&s[1][d] - &s[0][d])
        }))?;
        if a == b || (0..2).any(|d| !(b[d] - a[d]).is_finite()) {
            return Err("positive exact boundary segment collapsed; no deletion".into());
        }
        result.push(Segment { a, b });
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_tangent_and_positive_clips() {
        let p = Polygon::new(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]).unwrap();
        let s = Source::new(&p).unwrap();
        assert!(s.clip([1.0, 0.0], [2.0, 1.0]).unwrap().is_empty());
        assert!(area(&s.clip([1.0_f64.next_down(), 0.0], [2.0, 1.0]).unwrap()) > R::zero());
    }
    #[test]
    fn positive_triangle_collapse_is_refused() {
        let one = rational(1.0).unwrap();
        let tiny = rational(2.0_f64.powi(-200)).unwrap();
        let v = vec![
            [one.clone(), R::zero()],
            [&one + tiny, R::zero()],
            [one, rational(1.0).unwrap()],
        ];
        assert!(triangles(&v).unwrap_err().contains("collapsed"));
    }
}
