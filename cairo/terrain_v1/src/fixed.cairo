#[derive(Copy, Drop, Serde, PartialEq)]
pub struct Fixed {
    pub mag: i128
}

pub const FRACTIONAL_BITS: u8 = 16;
pub const ONE: i128 = 65536; // 1 << 16

pub trait FixedTrait {
    fn from_i128(val: i128) -> Fixed;
    fn to_i128(self: Fixed) -> i128;
    fn add(self: Fixed, other: Fixed) -> Fixed;
    fn sub(self: Fixed, other: Fixed) -> Fixed;
    fn mul(self: Fixed, other: Fixed) -> Fixed;
    fn div(self: Fixed, other: @Fixed) -> Fixed;
}

pub impl FixedImpl of FixedTrait {
    fn from_i128(val: i128) -> Fixed {
        Fixed { mag: val * ONE }
    }

    fn to_i128(self: Fixed) -> i128 {
        self.mag / ONE
    }

    fn add(self: Fixed, other: Fixed) -> Fixed {
        Fixed { mag: self.mag + other.mag }
    }

    fn sub(self: Fixed, other: Fixed) -> Fixed {
        Fixed { mag: self.mag - other.mag }
    }

    fn mul(self: Fixed, other: Fixed) -> Fixed {
        Fixed { mag: (self.mag * other.mag) / ONE }
    }

    fn div(self: Fixed, other: @Fixed) -> Fixed {
        Fixed { mag: (self.mag * ONE) / (*other).mag }
    }
}
