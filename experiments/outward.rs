//! Conditional outward enclosures for the isolated aligned-Stokes experiment.
//!
//! Soundness assumes Rust's default scalar binary64 round-to-nearest,
//! ties-to-even environment, without reassociation, FTZ or changed controls.
//! Linux x86_64 is the deliberately narrow supported target. Probes can reject
//! observed violations; they cannot prove external code never changes controls.
//! Normal values and zero only; uncertainty is refused, never snapped to zero.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedTarget,
    ArithmeticProbe,
    NonFinite,
    Subnormal,
    InvalidInterval,
    OverflowOrUnderflow,
    ZeroDenominator,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    pub lo: f64,
    pub hi: f64,
}

fn valid_scalar(x: f64) -> Result<(), Error> {
    if !x.is_finite() { return Err(Error::NonFinite); }
    if x != 0.0 && !x.is_normal() { return Err(Error::Subnormal); }
    Ok(())
}

pub fn ensure_supported() -> Result<(), Error> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(Error::UnsupportedTarget);
    }
    // Black boxes keep these observations at runtime. No FP-register claim.
    let one = std::hint::black_box(1.0_f64);
    let half_ulp = std::hint::black_box(f64::EPSILON / 2.0);
    let adjacent = std::hint::black_box(one.next_up());
    let minimum = std::hint::black_box(f64::MIN_POSITIVE);
    let two = std::hint::black_box(2.0_f64);
    if one + half_ulp != one || adjacent + half_ulp != adjacent.next_up()
        || minimum / two != f64::from_bits(1_u64 << 51)
    {
        return Err(Error::ArithmeticProbe);
    }
    Ok(())
}

fn widen(x: f64) -> Result<Interval, Error> {
    if !x.is_finite() || x == 0.0 { return Err(Error::OverflowOrUnderflow); }
    valid_scalar(x)?;
    Interval::new(x.next_down(), x.next_up())
}

fn sum_endpoint(a: f64, b: f64) -> Result<Interval, Error> {
    if a == 0.0 { return Interval::point(b); }
    if b == 0.0 { return Interval::point(a); }
    if a == -b { return Interval::point(0.0); }
    widen(a + b)
}

fn product_endpoint(a: f64, b: f64) -> Result<Interval, Error> {
    if a == 0.0 || b == 0.0 { return Interval::point(0.0); }
    if a == 1.0 { return Interval::point(b); }
    if b == 1.0 { return Interval::point(a); }
    if a == -1.0 { return Interval::point(-b); }
    if b == -1.0 { return Interval::point(-a); }
    widen(a * b)
}

fn quotient_endpoint(a: f64, b: f64) -> Result<Interval, Error> {
    if b == 0.0 { return Err(Error::ZeroDenominator); }
    if a == 0.0 { return Interval::point(0.0); }
    if b == 1.0 { return Interval::point(a); }
    if b == -1.0 { return Interval::point(-a); }
    if a == b { return Interval::point(1.0); }
    if a == -b { return Interval::point(-1.0); }
    widen(a / b)
}

impl Interval {
    pub fn new(lo: f64, hi: f64) -> Result<Self, Error> {
        valid_scalar(lo)?;
        valid_scalar(hi)?;
        if lo > hi { return Err(Error::InvalidInterval); }
        Ok(Self { lo, hi })
    }
    pub fn point(x: f64) -> Result<Self, Error> { Self::new(x, x) }
    fn validate(self) -> Result<(), Error> { Self::new(self.lo, self.hi).map(|_| ()) }
    pub fn add(self, other: Self) -> Result<Self, Error> {
        self.validate()?;
        other.validate()?;
        let low = sum_endpoint(self.lo, other.lo)?;
        let high = sum_endpoint(self.hi, other.hi)?;
        Self::new(low.lo, high.hi)
    }
    pub fn sub(self, other: Self) -> Result<Self, Error> {
        self.validate()?;
        other.validate()?;
        self.add(other.neg())
    }
    // Negation flips bits exactly, even for zero. Consumers validate operands.
    pub fn neg(self) -> Self { Self { lo: -self.hi, hi: -self.lo } }
    pub fn mul(self, other: Self) -> Result<Self, Error> {
        self.validate()?;
        other.validate()?;
        let products = [
            product_endpoint(self.lo, other.lo)?,
            product_endpoint(self.lo, other.hi)?,
            product_endpoint(self.hi, other.lo)?,
            product_endpoint(self.hi, other.hi)?,
        ];
        let lo = products.iter().map(|x| x.lo).fold(f64::INFINITY, f64::min);
        let hi = products.iter().map(|x| x.hi).fold(f64::NEG_INFINITY, f64::max);
        Self::new(lo, hi)
    }
    pub fn div(self, other: Self) -> Result<Self, Error> {
        self.validate()?;
        other.validate()?;
        if other.lo <= 0.0 && other.hi >= 0.0 { return Err(Error::ZeroDenominator); }
        let quotients = [
            quotient_endpoint(self.lo, other.lo)?,
            quotient_endpoint(self.lo, other.hi)?,
            quotient_endpoint(self.hi, other.lo)?,
            quotient_endpoint(self.hi, other.hi)?,
        ];
        let lo = quotients.iter().map(|x| x.lo).fold(f64::INFINITY, f64::min);
        let hi = quotients.iter().map(|x| x.hi).fold(f64::NEG_INFINITY, f64::max);
        Self::new(lo, hi)
    }
    pub fn abs(self) -> Result<Self, Error> {
        self.validate()?;
        if self.lo >= 0.0 { return Ok(self); }
        if self.hi <= 0.0 { return Ok(self.neg()); }
        Self::new(0.0, (-self.lo).max(self.hi))
    }
    pub fn abs_upper(self) -> Result<f64, Error> { self.abs().map(|x| x.hi) }
    pub fn square(self) -> Result<Self, Error> {
        let a = self.abs()?;
        let lo = product_endpoint(a.lo, a.lo)?.lo;
        let hi = product_endpoint(a.hi, a.hi)?.hi;
        Self::new(lo, hi)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(x: f64) -> Interval { Interval::point(x).unwrap() }
    #[test]
    fn exact_identities_and_rest_remain_points() {
        ensure_supported().unwrap();
        assert_eq!(p(0.0).mul(p(1e300)).unwrap(), p(0.0));
        assert_eq!(p(0.0).add(p(3.0)).unwrap(), p(3.0));
        assert_eq!(p(3.0).sub(p(3.0)).unwrap(), p(0.0));
        assert_eq!(p(-1.0).mul(p(3.0)).unwrap(), p(-3.0));
        assert_eq!(p(3.0).div(p(3.0)).unwrap(), p(1.0));
        assert_eq!(p(0.0).square().unwrap(), p(0.0));
    }
    #[test]
    fn uncertainty_is_not_snap_cancellation() {
        let x = Interval::new(2.0_f64.next_down(), 2.0_f64.next_up()).unwrap();
        let difference = x.sub(x).unwrap();
        assert!(difference.lo < 0.0 && difference.hi > 0.0);
        let correlated = Interval::new(-2.0, 3.0).unwrap().square().unwrap();
        assert_eq!(correlated.lo, 0.0);
        assert!(correlated.hi >= 9.0);
    }
    #[test]
    fn every_operation_widens_not_just_final_result() {
        let sum = p(1e16).add(p(1.0)).unwrap();
        let difference = sum.sub(p(1e16)).unwrap();
        assert!(difference.lo <= 1.0 && difference.hi >= 1.0);
        let bad_final_only = (1e16_f64 + 1.0) - 1e16;
        assert!(bad_final_only.next_up() < 1.0);
    }
    #[test]
    fn bad_arithmetic_is_explicitly_refused() {
        assert_eq!(p(1.0).div(Interval::new(-1.0,1.0).unwrap()), Err(Error::ZeroDenominator));
        assert!(p(f64::MAX).mul(p(2.0)).is_err());
        assert!(p(2.0_f64.powi(-600)).mul(p(2.0_f64.powi(-600))).is_err());
        assert_eq!(Interval::point(f64::MIN_POSITIVE/2.0), Err(Error::Subnormal));
        assert!(Interval {lo:f64::NAN,hi:1.0}.add(p(0.0)).is_err());
        assert!(p(f64::MIN_POSITIVE).mul(p(0.5)).is_err());
        assert!(Interval::new(2.0,1.0).is_err());
    }
}
