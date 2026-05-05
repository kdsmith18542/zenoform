use core::poseidon::poseidon_hash_span;
use zenoform_terrain_v1::fixed::{Fixed, FixedTrait, ONE};

/// Simple deterministic hash for noise using Poseidon.
pub fn hash_2d(seed: i128, x: i128, y: i128) -> i128 {
    let mut data = array![seed.into(), x.into(), y.into()];
    let hash = poseidon_hash_span(data.span());
    let hash_u256: u256 = hash.into();
    (hash_u256.low % 0x80000000000000000000000000000000_u128).try_into().unwrap()
}

/// Simple linear interpolation for fixed-point.
pub fn lerp(a: @Fixed, b: @Fixed, t: @Fixed) -> Fixed {
    let diff = b.sub(*a);
    a.add(diff.mul(*t))
}

/// Simple value noise implementation using fixed-point math.
pub fn value_noise_2d(seed: i128, x: @Fixed, y: @Fixed) -> Fixed {
    let xi = x.to_i128();
    let yi = y.to_i128();

    let xf_mag = (*x).mag % ONE;
    let yf_mag = (*y).mag % ONE;

    let xf = Fixed { mag: xf_mag };
    let yf = Fixed { mag: yf_mag };

    let v00 = Fixed { mag: hash_2d(seed, xi, yi) % ONE };
    let v10 = Fixed { mag: hash_2d(seed, xi + 1, yi) % ONE };
    let v01 = Fixed { mag: hash_2d(seed, xi, yi + 1) % ONE };
    let v11 = Fixed { mag: hash_2d(seed, xi + 1, yi + 1) % ONE };

    let i1 = lerp(@v00, @v10, @xf);
    let i2 = lerp(@v01, @v11, @xf);

    lerp(@i1, @i2, @yf)
}

/// Fractal noise with multiple octaves.
pub fn fractal_noise_2d(seed: i128, x: @Fixed, y: @Fixed, octaves: u32) -> Fixed {
    let mut amplitude = Fixed { mag: ONE }; // 1.0
    let mut frequency = Fixed { mag: ONE / 8 }; // 0.125
    let mut result = FixedTrait::from_i128(0);
    let mut max_value = FixedTrait::from_i128(0);

    let mut i: u32 = 0;
    while i < octaves {
        let sx = seed + i.into();
        let sample_x = Fixed { mag: (*x).mag * frequency.mag / ONE };
        let sample_y = Fixed { mag: (*y).mag * frequency.mag / ONE };

        result = result.add(value_noise_2d(sx, @sample_x, @sample_y).mul(amplitude));
        max_value = max_value.add(amplitude);

        amplitude = Fixed { mag: amplitude.mag / 2 }; // *= 0.5
        frequency = Fixed { mag: frequency.mag * 2 }; // *= 2
        i += 1;
    };

    result.div(@max_value)
}
