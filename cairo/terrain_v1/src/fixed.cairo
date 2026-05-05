#[derive(Copy, Drop, Serde, PartialEq)]
struct Fixed {
    mag: i128
}

const FRACTIONAL_BITS: u8 = 16;
const ONE: i128 = 65536; // 1 << 16

trait FixedTrait {
    fn from_i128(val: i128) -> Fixed;
    fn to_i128(self: Fixed) -> i128;
    fn add(self: Fixed, other: Fixed) -> Fixed;
    fn sub(self: Fixed, other: Fixed) -> Fixed;
    fn mul(self: Fixed, other: Fixed) -> Fixed;
}

impl FixedImpl of FixedTrait {
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
}
