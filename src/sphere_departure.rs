//! Exact signs of bounded sums of products of represented finite binary64 data.
//! This kernel is audited independently; the Lean development proves the real
//! separating-plane premise, not refinement of this bit implementation.
use crate::{
    RigidPoseSnapshot, SphereContactError, SphereContactHit, SphericalRigidMotion, SurfaceStamp,
    TriangleSurface,
};
use std::cmp::Ordering;

const LIMBS: usize = 66;
pub(crate) struct ProductSign {
    positive: [u64; LIMBS],
    negative: [u64; LIMBS],
    terms: usize,
}
// Arrays longer than 32 need an explicit Default on the supported compiler.
impl Default for ProductSign {
    fn default() -> Self {
        Self {
            positive: [0; LIMBS],
            negative: [0; LIMBS],
            terms: 0,
        }
    }
}
impl ProductSign {
    pub(crate) fn add(&mut self, a: f64, b: f64, subtract: bool) -> Result<(), SphereContactError> {
        if self.terms >= 16 || !a.is_finite() || !b.is_finite() {
            return Err(SphereContactError::ArithmeticFailure);
        }
        self.terms += 1;
        let split = |x: f64| {
            let bits = x.to_bits();
            let e = ((bits >> 52) & 2047) as i32;
            let fraction = bits & ((1_u64 << 52) - 1);
            (
                bits >> 63 != 0,
                if e == 0 {
                    fraction
                } else {
                    fraction | (1_u64 << 52)
                },
                if e == 0 { -1074 } else { e - 1075 },
            )
        };
        let (sa, ma, ea) = split(a);
        let (sb, mb, eb) = split(b);
        let product = u128::from(ma) * u128::from(mb);
        let shift =
            usize::try_from(ea + eb + 2148).map_err(|_| SphereContactError::ArithmeticFailure)?;
        let target = if sa ^ sb ^ subtract {
            &mut self.negative
        } else {
            &mut self.positive
        };
        for (word, offset) in [
            (product as u64, shift),
            ((product >> 64) as u64, shift + 64),
        ] {
            let index = offset / 64;
            let bit = offset % 64;
            Self::word(target, index, word << bit)?;
            if bit != 0 {
                Self::word(target, index + 1, word >> (64 - bit))?;
            }
        }
        Ok(())
    }
    fn word(
        target: &mut [u64; LIMBS],
        mut index: usize,
        mut word: u64,
    ) -> Result<(), SphereContactError> {
        while word != 0 {
            let slot = target
                .get_mut(index)
                .ok_or(SphereContactError::ArithmeticFailure)?;
            let (sum, carry) = slot.overflowing_add(word);
            *slot = sum;
            word = u64::from(carry);
            index += 1;
        }
        Ok(())
    }
    pub(crate) fn sign(&self) -> Ordering {
        self.positive.iter().rev().cmp(self.negative.iter().rev())
    }
}
/// Exact dot product (a-p)·(b-q), expanded before any floating subtraction.
pub(crate) fn difference_dot(
    a: [f64; 3],
    p: [f64; 3],
    b: [f64; 3],
    q: [f64; 3],
) -> Result<ProductSign, SphereContactError> {
    let mut sign = ProductSign::default();
    for i in 0..3 {
        sign.add(a[i], b[i], false)?;
        sign.add(a[i], q[i], true)?;
        sign.add(p[i], b[i], true)?;
        sign.add(p[i], q[i], false)?;
    }
    Ok(sign)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SphereDepartureKind {
    Separating,
    Tangent,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SphereDepartureReport {
    pub triangle: usize,
    pub point: [f64; 3],
    pub from: RigidPoseSnapshot,
    pub static_surface: SurfaceStamp,
    pub radius_m: f64,
    pub kind: SphereDepartureKind,
}
pub(crate) struct QualifiedDeparture {
    pub(crate) report: SphereDepartureReport,
}
impl QualifiedDeparture {
    pub(crate) fn new(
        owner: &SphericalRigidMotion,
        surface: &TriangleSurface,
        radius: f64,
        hit: SphereContactHit,
    ) -> Result<Self, SphereContactError> {
        let from = owner.snapshot();
        if hit.static_surface != surface.stamp() || !radius.is_finite() || radius <= 0. {
            return Err(SphereContactError::InvalidDeparture);
        }
        let c = from.body.center_of_mass;
        let p = hit.point;
        let mut clearance = difference_dot(c, p, c, p)?;
        clearance.add(radius, radius, true)?;
        if clearance.sign() == Ordering::Less {
            return Err(SphereContactError::InvalidDeparture);
        }
        let indices = surface
            .triangles()
            .get(hit.triangle)
            .ok_or(SphereContactError::InvalidDeparture)?;
        for &i in indices {
            if difference_dot(c, p, surface.vertices()[i], p)?.sign() == Ordering::Greater {
                return Err(SphereContactError::InvalidDeparture);
            }
        }
        let velocity = difference_dot(c, p, from.body.velocity_m_s, [0.; 3])?.sign();
        if velocity == Ordering::Less {
            return Err(SphereContactError::InvalidDeparture);
        }
        Ok(Self {
            report: SphereDepartureReport {
                triangle: hit.triangle,
                point: p,
                from,
                static_surface: surface.stamp(),
                radius_m: radius,
                kind: if velocity == Ordering::Equal {
                    SphereDepartureKind::Tangent
                } else {
                    SphereDepartureKind::Separating
                },
            },
        })
    }
    pub(crate) fn matches(
        &self,
        owner: &SphericalRigidMotion,
        surface: &TriangleSurface,
        radius: f64,
    ) -> bool {
        self.report.from == owner.snapshot()
            && self.report.static_surface == surface.stamp()
            && self.report.radius_m == radius
    }
    pub(crate) fn endpoint(&self, center: [f64; 3]) -> Result<(), SphereContactError> {
        let c = self.report.from.body.center_of_mass;
        if difference_dot(c, self.report.point, center, c)?.sign() == Ordering::Less {
            Err(SphereContactError::DepartureEndpoint)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_support_clearance_and_actual_endpoint_refuse_inward_data() {
        let c = [0., 0., 0.25];
        let p = [0.; 3];
        let mut gap = difference_dot(c, p, c, p).unwrap();
        gap.add(0.25, 0.25, true).unwrap();
        assert_eq!(gap.sign(), Ordering::Equal);
        let inside = [0., 0., 0.25_f64.next_down()];
        let mut gap = difference_dot(inside, p, inside, p).unwrap();
        gap.add(0.25, 0.25, true).unwrap();
        assert_eq!(gap.sign(), Ordering::Less);
        assert_eq!(
            difference_dot(c, p, [1., 2., 0.], p).unwrap().sign(),
            Ordering::Equal
        );
        assert_eq!(
            difference_dot(c, p, [0., 0., f64::from_bits(1)], p)
                .unwrap()
                .sign(),
            Ordering::Greater
        );
        assert_eq!(
            difference_dot(c, p, inside, c).unwrap().sign(),
            Ordering::Less
        );
        assert_eq!(
            difference_dot(c, p, [0., 0., 0.25_f64.next_up()], c)
                .unwrap()
                .sign(),
            Ordering::Greater
        );
    }
}
