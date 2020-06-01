//! This module implements the `BigInt` operations.

use num_traits::cast::ToPrimitive;
use num_traits::pow::Pow;

use super::BigInt;

impl BigInt {
    #[inline]
    pub fn pow(self, other: &Self) -> Self {
        Self(
            self.0.pow(
                other
                    .0
                    .to_biguint()
                    .expect("RangeError: \"BigInt negative exponent\""),
            ),
        )
    }

    #[inline]
    pub fn mod_floor(self, other: &Self) -> Self {
        use num_integer::Integer;
        Self(self.0.mod_floor(&other.0))
    }
}

macro_rules! impl_bigint_operator {
    ($op:ident, $op_method:ident, $assign_op:ident, $assign_op_method:ident) => {
        impl std::ops::$op for BigInt {
            type Output = Self;

            fn $op_method(mut self, other: Self) -> Self {
                std::ops::$assign_op::$assign_op_method(&mut self.0, other.0);
                self
            }
        }
    };
}

impl_bigint_operator!(Add, add, AddAssign, add_assign);
impl_bigint_operator!(Sub, sub, SubAssign, sub_assign);
impl_bigint_operator!(Mul, mul, MulAssign, mul_assign);
impl_bigint_operator!(Div, div, DivAssign, div_assign);
impl_bigint_operator!(Rem, rem, RemAssign, rem_assign);
impl_bigint_operator!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_bigint_operator!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_bigint_operator!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl std::ops::Shr for BigInt {
    type Output = Self;

    fn shr(mut self, other: Self) -> Self::Output {
        use std::ops::ShlAssign;
        use std::ops::ShrAssign;

        if let Some(n) = other.0.to_i32() {
            if n > 0 {
                self.0.shr_assign(n as usize)
            } else {
                self.0.shl_assign(n.abs() as usize)
            }

            return self;
        }

        panic!("RangeError: Maximum BigInt size exceeded");
    }
}

impl std::ops::Shl for BigInt {
    type Output = Self;

    fn shl(mut self, other: Self) -> Self::Output {
        use std::ops::ShlAssign;
        use std::ops::ShrAssign;

        if let Some(n) = other.0.to_i32() {
            if n > 0 {
                self.0.shl_assign(n as usize)
            } else {
                self.0.shr_assign(n.abs() as usize)
            }

            return self;
        }

        panic!("RangeError: Maximum BigInt size exceeded");
    }
}

impl std::ops::Neg for BigInt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl std::fmt::Debug for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
