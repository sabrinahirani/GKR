use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

// the "babybear" prime: 2^31 - 2^27 + 1
pub const MODULUS: u64 = 2_013_265_921;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fp(u64);

impl Fp {

    pub fn new(value: u64) -> Self {
        Self(value % MODULUS)
    }
    pub fn zero() -> Self {
        Fp(0)
    }
    pub fn one() -> Self {
        Fp(1)
    }
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }

    pub fn pow(&self, mut exp: u64) -> Self {
        let mut base = *self;
        let mut result = Fp::one();
        // square-and-multiply algorithm (O (log exp)):
        while exp > 0 {
            // using binary repr.
            if exp % 2 == 1 {
                result = result * base;
            }
            base = base * base;
            exp /= 2;
        }
        result
    }
    pub fn inverse(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // a^(p-1) = 1 for any a != 0 by FLT so: a^(p-2) = a^(-1)
            Some(self.pow(MODULUS-2))
        }
    }

}

impl From<u64> for Fp {
    fn from(value: u64) -> Self {
        Fp::new(value)
    }
}
impl Neg for Fp {
    type Output = Fp;
    fn neg(self) -> Fp {
        if self.is_zero() { self } else { Fp::new(MODULUS - self.0) }
    }
}
impl Add for Fp {
    type Output = Self;
    fn add(self, rhs: Fp) -> Fp {
        Fp::new(self.0 + rhs.0)
    }
}
impl Sub for Fp {
    type Output = Self;
    fn sub(self, rhs: Fp) -> Fp {
        Fp::new(self.0 + MODULUS - rhs.0)
    }
}
impl Mul for Fp {
    type Output = Self;
    fn mul(self, rhs: Fp) -> Fp {
        Fp::new(self.0 * rhs.0)
    }
}
impl fmt::Display for Fp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_addition() {
        assert_eq!(Fp::new(7) + Fp::new(6), Fp::new(13));
    }

    #[test]
    fn basic_subtraction() {
        assert_eq!(Fp::new(7) - Fp::new(6), Fp::new(1));
    }

    #[test]
    fn basic_multiplication() {
        assert_eq!(Fp::new(7) * Fp::new(6), Fp::new(42));
    }
 
    #[test]
    fn add_wraps_around_modulus() {
        let a = Fp::new(MODULUS - 1);
        let b = Fp::new(2);
        assert_eq!(a + b, Fp::new(1));
    }
 
    #[test]
    fn sub_wraps_around_modulus() {
        let a = Fp::new(1);
        let b = Fp::new(2);
        assert_eq!(a - b, Fp::new(MODULUS - 1));
    }
 
    #[test]
    fn neg_is_additive_inverse() {
        let a = Fp::new(12345);
        assert_eq!(a + (-a), Fp::zero());
    }
 
    #[test]
    fn inverse_takes_to_one() {
        let a = Fp::new(999);
        let inv = a.inverse().unwrap();
        assert_eq!(a * inv, Fp::one());
    }
 
    #[test]
    fn zero_has_no_inverse() {
        assert!(Fp::zero().inverse().is_none());
    }
 
    #[test]
    fn multiplicative_identity_is_one() {
        let a = Fp::new(54321);
        assert_eq!(a * Fp::one(), a);
    }
 
    #[test]
    fn pow_matches_repeated_multiplication() {
        let a = Fp::new(3);
        let mut expected = Fp::one();
        for _ in 0..10 {
            expected = expected * a;
        }
        assert_eq!(a.pow(10), expected);
    }
}