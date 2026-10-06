//! Research-only 106-bit binary scalar. No heap, unsafe, retained state or fallback.
use super::CoupledDiscreteError;
type Result<T> = std::result::Result<T, CoupledDiscreteError>;
const LEAD: u128 = 1u128 << 105;
const MAX_SIG: u128 = ((1u128 << 53) - 1) << 53;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Scalar {
    sig: u128,
    exponent: i32,
    negative: bool,
    pad: [u8; 11],
}
impl Scalar {
    pub const ZERO: Self = Self {
        sig: 0,
        exponent: 0,
        negative: false,
        pad: [0; 11],
    };
    fn fail<T>() -> Result<T> {
        Err(CoupledDiscreteError::LinearFailure)
    }
    fn make(sig: u128, exponent: i32, negative: bool) -> Result<Self> {
        if sig == 0 {
            return Ok(Self {
                negative,
                ..Self::ZERO
            });
        }
        if !(LEAD..2 * LEAD).contains(&sig)
            || !(-1022..=1023).contains(&exponent)
            || (exponent == 1023 && sig > MAX_SIG)
        {
            return Self::fail();
        }
        Ok(Self {
            sig,
            exponent,
            negative,
            pad: [0; 11],
        })
    }
    pub fn from_f64(x: f64) -> Result<Self> {
        let bits = x.to_bits();
        let negative = bits >> 63 != 0;
        if x == 0. {
            return Self::make(0, 0, negative);
        }
        if !x.is_normal() {
            return Self::fail();
        }
        Self::make(
            (((bits & ((1u64 << 52) - 1)) | (1u64 << 52)) as u128) << 53,
            ((bits >> 52) & 2047) as i32 - 1023,
            negative,
        )
    }
    pub fn parts(self) -> (u128, i32, bool) {
        (self.sig, self.exponent, self.negative)
    }
    pub fn is_zero(self) -> bool {
        self.sig == 0
    }
    pub fn neg(self) -> Self {
        Self {
            negative: !self.negative,
            ..self
        }
    }
    pub fn abs_ge(self, other: Self) -> bool {
        if other.sig == 0 {
            return true;
        }
        if self.sig == 0 {
            return false;
        }
        self.exponent > other.exponent || (self.exponent == other.exponent && self.sig >= other.sig)
    }
    fn rounded(value: u128, shift: u32) -> u128 {
        if shift == 0 {
            return value;
        }
        let q = value >> shift;
        let remainder = value & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        q + u128::from(remainder > half || (remainder == half && q & 1 != 0))
    }
    fn jam(value: u128, shift: u32) -> u128 {
        if shift == 0 {
            value
        } else if shift >= 128 {
            u128::from(value != 0)
        } else {
            (value >> shift) | u128::from(value & ((1u128 << shift) - 1) != 0)
        }
    }
    #[inline(never)]
    pub fn add(self, other: Self) -> Result<Self> {
        if self.sig == 0 && other.sig == 0 {
            return Self::make(0, 0, self.negative && other.negative);
        }
        if self.sig == 0 {
            return Ok(other);
        }
        if other.sig == 0 {
            return Ok(self);
        }
        let (large, small) = if self.abs_ge(other) {
            (self, other)
        } else {
            (other, self)
        };
        let left = large.sig << 3;
        let right = Self::jam(small.sig << 3, (large.exponent - small.exponent) as u32);
        let magnitude = if large.negative == small.negative {
            left + right
        } else {
            left - right
        };
        if magnitude == 0 {
            return Ok(Self::ZERO);
        }
        let len = 128 - magnitude.leading_zeros();
        let mut exponent = large.exponent + len as i32 - 109;
        let mut sig = if len > 106 {
            Self::rounded(magnitude, len - 106)
        } else {
            magnitude << (106 - len)
        };
        if sig == 2 * LEAD {
            sig >>= 1;
            exponent += 1;
        }
        Self::make(sig, exponent, large.negative)
    }
    pub fn sub(self, other: Self) -> Result<Self> {
        self.add(other.neg())
    }
    #[inline(never)]
    pub fn mul(self, other: Self) -> Result<Self> {
        let negative = self.negative != other.negative;
        if self.sig == 0 || other.sig == 0 {
            return Self::make(0, 0, negative);
        }
        let mut digits = [0u32; 8];
        for i in 0..4 {
            let a = ((self.sig >> (32 * i)) & 0xffff_ffff) as u64;
            let mut carry = 0u64;
            for j in 0..4 {
                let b = ((other.sig >> (32 * j)) & 0xffff_ffff) as u64;
                let value = a * b + u64::from(digits[i + j]) + carry;
                digits[i + j] = value as u32;
                carry = value >> 32;
            }
            digits[i + 4] = carry as u32;
        }
        let top = (0..8)
            .rev()
            .find(|&i| digits[i] != 0)
            .ok_or(CoupledDiscreteError::LinearFailure)?;
        let len = top * 32 + (32 - digits[top].leading_zeros()) as usize;
        let shift = len - 106;
        let bit = |index: usize| (digits[index / 32] >> (index % 32)) & 1;
        let mut sig = 0u128;
        for index in (shift..len).rev() {
            sig = (sig << 1) | u128::from(bit(index));
        }
        let round = bit(shift - 1) != 0;
        let sticky = (0..shift - 1).any(|index| bit(index) != 0);
        if round && (sticky || sig & 1 != 0) {
            sig += 1;
        }
        let mut exponent = self.exponent + other.exponent + len as i32 - 211;
        if sig == 2 * LEAD {
            sig >>= 1;
            exponent += 1;
        }
        Self::make(sig, exponent, negative)
    }
    #[inline(never)]
    pub fn div(self, other: Self) -> Result<Self> {
        if other.sig == 0 {
            return Self::fail();
        }
        let negative = self.negative != other.negative;
        if self.sig == 0 {
            return Self::make(0, 0, negative);
        }
        let lower = self.sig < other.sig;
        let shift = if lower { 106usize } else { 105usize };
        let mut quotient = 0u128;
        let mut remainder = 0u128;
        for index in (0..106 + shift).rev() {
            let bit = if index >= shift {
                (self.sig >> (index - shift)) & 1
            } else {
                0
            };
            remainder = (remainder << 1) | bit;
            quotient <<= 1;
            if remainder >= other.sig {
                remainder -= other.sig;
                quotient |= 1;
            }
        }
        if 2 * remainder > other.sig || (2 * remainder == other.sig && quotient & 1 != 0) {
            quotient += 1;
        }
        let mut exponent = self.exponent - other.exponent - i32::from(lower);
        if quotient == 2 * LEAD {
            quotient >>= 1;
            exponent += 1;
        }
        Self::make(quotient, exponent, negative)
    }
    #[inline(never)]
    pub fn to_f64(self) -> Result<f64> {
        let sign = u64::from(self.negative) << 63;
        if self.sig == 0 {
            return Ok(f64::from_bits(sign));
        }
        let mut sig = Self::rounded(self.sig, 53);
        let mut exponent = self.exponent;
        if sig == 1u128 << 53 {
            sig >>= 1;
            exponent += 1;
        }
        if !(-1022..=1023).contains(&exponent) {
            return Self::fail();
        }
        let value = f64::from_bits(
            sign | (((exponent + 1023) as u64) << 52) | (sig as u64 & ((1u64 << 52) - 1)),
        );
        if !value.is_normal() {
            return Self::fail();
        }
        Ok(value)
    }
}

// Every reserved scalar slot is a real, fixed object. The accepted coefficient
// extraction and descriptors are fixed scratch, not a retained owner or history.
#[repr(C)]
pub struct ChartWorkspace {
    slots: [Scalar; 1934],
    accepted_z: [f64; 22],
    pivots: [u8; 15],
    evaluations: usize,
}
const MATRIX: usize = 0;
const ORIGINAL: usize = 225;
const D: usize = 1125 + 264;
const Z0: usize = 1125;
const DELTA: usize = Z0 + 22;
const RHS: usize = DELTA + 22;
const SOLUTION: usize = RHS + 22;
impl ChartWorkspace {
    pub fn new() -> Self {
        Self {
            slots: [Scalar::ZERO; 1934],
            accepted_z: [0.; 22],
            pivots: [0; 15],
            evaluations: 0,
        }
    }
    pub fn set_accepted(&mut self, z: [f64; 22]) {
        self.accepted_z = z;
    }
    #[inline(never)]
    fn factor(&mut self) -> Result<()> {
        for j in 0..15 {
            let mut pivot = j;
            for i in j..15 {
                if self.slots[MATRIX + 15 * i + j].abs_ge(self.slots[MATRIX + 15 * pivot + j]) {
                    pivot = i;
                }
            }
            self.pivots[j] = pivot as u8;
            for k in 0..15 {
                self.slots
                    .swap(MATRIX + 15 * j + k, MATRIX + 15 * pivot + k);
            }
            self.slots.swap(RHS + j, RHS + pivot);
            if self.slots[MATRIX + 15 * j + j].is_zero() {
                return Scalar::fail();
            }
            for i in j + 1..15 {
                let factor =
                    self.slots[MATRIX + 15 * i + j].div(self.slots[MATRIX + 15 * j + j])?;
                for k in j + 1..15 {
                    self.slots[MATRIX + 15 * i + k] = self.slots[MATRIX + 15 * i + k]
                        .sub(factor.mul(self.slots[MATRIX + 15 * j + k])?)?;
                }
                self.slots[RHS + i] = self.slots[RHS + i].sub(factor.mul(self.slots[RHS + j])?)?;
                self.slots[MATRIX + 15 * i + j] = Scalar::ZERO;
            }
        }
        for i in (0..15).rev() {
            let mut tail = Scalar::ZERO;
            for j in i + 1..15 {
                tail = tail.add(self.slots[MATRIX + 15 * i + j].mul(self.slots[SOLUTION + j])?)?;
            }
            self.slots[SOLUTION + i] = self.slots[RHS + i]
                .sub(tail)?
                .div(self.slots[MATRIX + 15 * i + i])?;
        }
        Ok(())
    }
    #[inline(never)]
    pub fn solve(&mut self, d: &[[f64; 22]; 24], target: &[f64; 22]) -> Result<[f64; 15]> {
        self.slots.fill(Scalar::ZERO);
        self.pivots.fill(0);
        self.evaluations += 1;
        for j in 0..22 {
            self.slots[Z0 + j] = Scalar::from_f64(self.accepted_z[j])?;
        }
        for &j in &super::KNOWN {
            self.slots[DELTA + j] = Scalar::from_f64(target[j])?.sub(self.slots[Z0 + j])?;
        }
        for i in 0..24 {
            for j in 0..22 {
                self.slots[D + 22 * i + j] = Scalar::from_f64(d[i][j])?;
            }
        }
        for i in 0..15 {
            let row = super::ROWS[i];
            let mut rhs = Scalar::ZERO;
            for j in 0..22 {
                rhs = rhs.sub(self.slots[D + 22 * row + j].mul(self.slots[Z0 + j])?)?;
            }
            for &j in &super::KNOWN {
                rhs = rhs.sub(self.slots[D + 22 * row + j].mul(self.slots[DELTA + j])?)?;
            }
            self.slots[RHS + i] = rhs;
            for j in 0..15 {
                let a = self.slots[D + 22 * row + super::UNKNOWN[j]];
                self.slots[MATRIX + 15 * i + j] = a;
                self.slots[ORIGINAL + 15 * i + j] = a;
            }
        }
        self.factor()?;
        let mut result = [0.; 15];
        for i in 0..15 {
            result[i] = self.slots[Z0 + super::UNKNOWN[i]]
                .add(self.slots[SOLUTION + i])?
                .to_f64()?;
        }
        Ok(result)
    }
    pub fn evaluations(&self) -> usize {
        self.evaluations
    }
    pub fn conformance(&mut self) -> Result<usize> {
        let one = Scalar::from_f64(1.)?;
        let half = Scalar::from_f64(2f64.powi(-106))?;
        let unit = Scalar::from_f64(2f64.powi(-105))?;
        let odd = one.add(unit)?;
        assert_eq!(one.add(half)?.parts(), one.parts());
        assert_eq!(odd.add(half)?.parts(), (LEAD + 2, 0, false));
        assert_eq!(
            odd.mul(Scalar::from_f64(1.5)?)?.parts(),
            ((3u128 << 104) + 2, 0, false)
        );
        assert_eq!(
            Scalar::from_f64(1.5)?
                .mul(Scalar::from_f64(1.5)?)?
                .to_f64()?
                .to_bits(),
            2.25f64.to_bits()
        );
        assert_eq!(
            Scalar::from_f64(3.)?.div(Scalar::from_f64(2.)?)?.parts(),
            Scalar::from_f64(1.5)?.parts()
        );
        assert_eq!(one.add(one.neg())?.to_f64()?.to_bits(), 0f64.to_bits());
        assert_eq!(
            Scalar::from_f64(-0.)?
                .add(Scalar::from_f64(-0.)?)?
                .to_f64()?
                .to_bits(),
            (-0f64).to_bits()
        );
        assert_eq!(
            Scalar::from_f64(-0.)?.to_f64()?.to_bits(),
            (-0f64).to_bits()
        );
        assert!(
            Scalar::from_f64(f64::NAN).is_err()
                && Scalar::from_f64(f64::INFINITY).is_err()
                && Scalar::from_f64(f64::from_bits(1)).is_err()
        );
        let min = Scalar::from_f64(f64::MIN_POSITIVE)?;
        assert_eq!(
            min.mul(one)?.to_f64()?.to_bits(),
            f64::MIN_POSITIVE.to_bits()
        );
        assert_eq!(min.sub(min)?.to_f64()?.to_bits(), 0f64.to_bits());
        assert!(
            min.mul(Scalar::from_f64(0.5)?).is_err() && min.div(Scalar::from_f64(2.)?).is_err()
        );
        assert!(
            Scalar::from_f64(f64::from_bits(f64::MIN_POSITIVE.to_bits() + 1))?
                .sub(min)
                .is_err()
        );
        let max = Scalar::from_f64(f64::MAX)?;
        assert_eq!(max.to_f64()?.to_bits(), f64::MAX.to_bits());
        assert!(max.mul(Scalar::from_f64(2.)?).is_err() && one.div(Scalar::ZERO).is_err());
        let endpoint_half = Scalar::from_f64(2f64.powi(-53))?;
        assert_eq!(one.add(endpoint_half)?.to_f64()?.to_bits(), 1f64.to_bits());
        assert_eq!(
            Scalar::from_f64(f64::from_bits(1f64.to_bits() + 1))?
                .add(endpoint_half)?
                .to_f64()?
                .to_bits(),
            1f64.to_bits() + 2
        );
        self.slots.fill(Scalar::ZERO);
        for i in 0..15 {
            self.slots[MATRIX + 15 * i + i] = one;
        }
        self.slots[MATRIX] = Scalar::ZERO;
        self.slots[MATRIX + 1] = Scalar::from_f64(2.)?;
        self.slots[MATRIX + 15] = one;
        self.slots[RHS] = Scalar::from_f64(4.)?;
        self.slots[RHS + 1] = Scalar::from_f64(3.)?;
        self.factor()?;
        assert_eq!(self.pivots[0], 1);
        assert_eq!(self.slots[SOLUTION].to_f64()?.to_bits(), 1f64.to_bits());
        assert_eq!(self.slots[SOLUTION + 1].to_f64()?.to_bits(), 2f64.to_bits());
        self.slots.fill(Scalar::ZERO);
        for i in 0..15 {
            self.slots[MATRIX + 15 * i + i] = one;
        }
        self.slots[MATRIX + 1] = one;
        self.slots[MATRIX + 15] = one.neg();
        self.slots[MATRIX + 16] = Scalar::from_f64(2.)?;
        self.slots[RHS] = Scalar::from_f64(3.)?;
        self.slots[RHS + 1] = Scalar::from_f64(3.)?;
        self.factor()?;
        assert_eq!(self.pivots[0], 1);
        assert_eq!(self.slots[SOLUTION].to_f64()?.to_bits(), 1f64.to_bits());
        self.slots.fill(Scalar::ZERO);
        assert!(self.factor().is_err());
        Ok(23)
    }
}
