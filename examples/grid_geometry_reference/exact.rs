//! Exact represented-binary64 source predicates; numerical endpoints are secondary.
use super::geometry::{Point, Segment};
use num_rational::BigRational;
use num_traits::{ToPrimitive, Zero};

pub fn rational(x: f64) -> Result<BigRational, String> {
    BigRational::from_float(x).ok_or_else(|| "finite exact-coordinate input required".into())
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExactFace {
    Empty,
    Point(BigRational),
    Positive {
        lower: BigRational,
        upper: BigRational,
    },
}
impl ExactFace {
    pub fn status(&self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::Point(_) => "point",
            Self::Positive { .. } => "positive",
        }
    }
    pub fn bounds(&self) -> Option<(&BigRational, &BigRational)> {
        match self {
            Self::Empty => None,
            Self::Point(x) => Some((x, x)),
            Self::Positive { lower, upper } => Some((lower, upper)),
        }
    }
    pub fn segment(&self, axis: usize, position: f64) -> Result<Option<Segment>, String> {
        if axis > 1 || !position.is_finite() {
            return Err("invalid face construction axis/position".into());
        }
        let Self::Positive { lower, upper } = self else {
            return Ok(None);
        };
        let lo = lower
            .to_f64()
            .ok_or("unrepresentable positive face lower endpoint")?;
        let hi = upper
            .to_f64()
            .ok_or("unrepresentable positive face upper endpoint")?;
        if !lo.is_finite() || !hi.is_finite() || hi <= lo || !(hi - lo).is_finite() {
            return Err(
                "exact positive aperture unrepresentable as a positive finite f64 segment".into(),
            );
        }
        let mut a = [0.0; 2];
        let mut b = [0.0; 2];
        a[axis] = position;
        b[axis] = position;
        a[1 - axis] = lo;
        b[1 - axis] = hi;
        if axis == 1 {
            std::mem::swap(&mut a, &mut b);
        }
        Ok(Some(Segment { a, b }))
    }
}
#[derive(Clone)]
pub struct LineSection {
    bounds: Option<(BigRational, BigRational)>,
}
impl LineSection {
    pub fn clip(&self, lo: f64, hi: f64) -> Result<ExactFace, String> {
        if !lo.is_finite() || !hi.is_finite() || lo >= hi {
            return Err("finite ordered face span required".into());
        }
        let Some((lower, upper)) = &self.bounds else {
            return Ok(ExactFace::Empty);
        };
        let lower = lower.clone().max(rational(lo)?);
        let upper = upper.clone().min(rational(hi)?);
        Ok(match upper.cmp(&lower) {
            std::cmp::Ordering::Less => ExactFace::Empty,
            std::cmp::Ordering::Equal => ExactFace::Point(lower),
            std::cmp::Ordering::Greater => ExactFace::Positive { lower, upper },
        })
    }
}
#[derive(Clone)]
pub struct ExactPolygon {
    points: Vec<[BigRational; 2]>,
    inputs: Vec<Point>,
}
impl ExactPolygon {
    pub fn new(vertices: &[Point]) -> Result<Self, String> {
        if !(3..=1024).contains(&vertices.len()) {
            return Err("exact reference source requires 3..1024 vertices".into());
        }
        let points: Vec<_> = vertices
            .iter()
            .map(|p| Ok([rational(p[0])?, rational(p[1])?]))
            .collect::<Result<_, String>>()?;
        for (i, p) in points.iter().enumerate() {
            if points[..i].contains(p) {
                return Err("repeated exact source vertex".into());
            }
        }
        let mut twice_area = BigRational::zero();
        for (i, a) in points.iter().enumerate() {
            let b = &points[(i + 1) % points.len()];
            twice_area += &a[0] * &b[1] - &a[1] * &b[0];
            let e = [&b[0] - &a[0], &b[1] - &a[1]];
            for p in &points {
                if &e[0] * (&p[1] - &a[1]) - &e[1] * (&p[0] - &a[0]) < BigRational::zero() {
                    return Err("exact convex CCW source required".into());
                }
            }
        }
        if twice_area <= BigRational::zero() {
            return Err("exact positive source area required".into());
        }
        Ok(Self {
            points,
            inputs: vertices.to_vec(),
        })
    }
    /// f64 comparisons are exact comparisons of the finite represented inputs.
    /// Only strict straddles divide; constructed points remain rational.
    pub fn line(&self, axis: usize, position: f64) -> Result<LineSection, String> {
        if axis > 1 || !position.is_finite() {
            return Err("invalid exact source-line query".into());
        }
        let p = rational(position)?;
        let tangent = 1 - axis;
        let mut hits = Vec::new();
        for (i, a) in self.inputs.iter().enumerate() {
            let j = (i + 1) % self.inputs.len();
            let b = self.inputs[j];
            let qa = &self.points[i];
            let qb = &self.points[j];
            if a[axis] == position {
                hits.push(qa[tangent].clone());
            }
            if (a[axis] < position && b[axis] > position)
                || (a[axis] > position && b[axis] < position)
            {
                hits.push(
                    &qa[tangent]
                        + (&p - &qa[axis]) * (&qb[tangent] - &qa[tangent])
                            / (&qb[axis] - &qa[axis]),
                );
            }
        }
        Ok(LineSection {
            bounds: hits.iter().min().cloned().zip(hits.iter().max().cloned()),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn diamond() -> (Vec<Point>, ExactPolygon) {
        let p = vec![[-0.5, 0.0], [0.0, -0.5], [0.5, 0.0], [0.0, 0.5]];
        let exact = ExactPolygon::new(&p).unwrap();
        (p, exact)
    }
    #[test]
    fn binary64_conversion_is_exact_not_decimal_approximation() {
        assert_eq!(
            rational(0.5).unwrap() * rational(2.0).unwrap(),
            rational(1.0).unwrap()
        );
        assert_ne!(
            rational(0.1).unwrap() * rational(10.0).unwrap(),
            rational(1.0).unwrap()
        );
        assert!(rational(f64::NAN).is_err());
    }
    #[test]
    fn tangent_and_adjacent_binary64_positions_have_distinct_dimensions() {
        let (_, e) = diamond();
        assert!(matches!(
            e.line(0, 0.5).unwrap().clip(-1.0, 1.0).unwrap(),
            ExactFace::Point(_)
        ));
        assert_eq!(
            e.line(0, 0.5_f64.next_up())
                .unwrap()
                .clip(-1.0, 1.0)
                .unwrap(),
            ExactFace::Empty
        );
        assert!(matches!(
            e.line(0, 0.5_f64.next_down())
                .unwrap()
                .clip(-1.0, 1.0)
                .unwrap(),
            ExactFace::Positive { .. }
        ));
        for axis in [0, 1] {
            let f = e.line(axis, 0.0).unwrap().clip(-0.25, 0.25).unwrap();
            assert!(f.segment(axis, 0.0).unwrap().unwrap().normal_measure()[axis] > 0.0);
        }
    }
    #[test]
    fn positive_exact_interval_cannot_silently_collapse_to_empty() {
        let lower = rational(1.0).unwrap();
        let upper = &lower + rational(2.0_f64.powi(-200)).unwrap();
        let face = ExactFace::Positive { lower, upper };
        assert!(
            face.segment(0, 0.0)
                .unwrap_err()
                .contains("unrepresentable")
        );
    }
    #[test]
    fn source_admission_uses_exact_convexity_and_unique_vertices() {
        assert!(ExactPolygon::new(&[[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]).is_err());
        assert!(ExactPolygon::new(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]).is_err());
        assert!(
            ExactPolygon::new(&[
                [0.0, 0.0],
                [1.0, 0.0],
                [0.5, -f64::MIN_POSITIVE],
                [0.0, 1.0]
            ])
            .is_err()
        );
    }
}
