/// Simple Q16.16 fixed-point implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixed(pub i32);

impl Fixed {
    pub const FRACTIONAL_BITS: u32 = 16;
    pub const ONE: i32 = 1 << Self::FRACTIONAL_BITS;

    pub fn from_i32(val: i32) -> Self {
        Self(val << Self::FRACTIONAL_BITS)
    }

    pub fn to_i32(self) -> i32 {
        self.0 >> Self::FRACTIONAL_BITS
    }

    pub fn from_bits(bits: i32) -> Self {
        Self(bits)
    }

    pub fn to_bits(self) -> i32 {
        self.0
    }

    pub fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }

    pub fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }

    pub fn mul(self, other: Self) -> Self {
        let res = (self.0 as i64 * other.0 as i64) >> Self::FRACTIONAL_BITS;
        Self(res as i32)
    }

    pub fn div(self, other: Self) -> Self {
        let res = ((self.0 as i64) << Self::FRACTIONAL_BITS) / other.0 as i64;
        Self(res as i32)
    }
}

impl core::ops::Add for Fixed {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add(other)
    }
}

impl core::ops::Sub for Fixed {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.sub(other)
    }
}

impl core::ops::Mul for Fixed {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.mul(other)
    }
}

impl core::ops::Div for Fixed {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        self.div(other)
    }
}
