use core::poseidon::poseidon_hash_span;
use zenoform_terrain_v1::fixed::{Fixed, FixedTrait, ONE};

fn hash_2d(seed: i128, x: i128, y: i128) -> i128 {
    let mut data = array![seed.into(), x.into(), y.into()];
    let hash = poseidon_hash_span(data.span());
    // Convert felt252 back to i128 (taking lower bits)
    let hash_u256: u256 = hash.into();
    (hash_u256.low % 0x80000000000000000000000000000000_u128).try_into().unwrap()
}

fn lerp(a: Fixed, b: Fixed, t: Fixed) -> Fixed {
    let diff = b.sub(a);
    a.add(diff.mul(t))
}

fn value_noise_2d(seed: i128, x: Fixed, y: Fixed) -> Fixed {
    let xi = x.to_i128();
    let yi = y.to_i128();

    let xf_mag = x.mag % ONE;
    let yf_mag = y.mag % ONE;
    
    let xf = Fixed { mag: xf_mag };
    let yf = Fixed { mag: yf_mag };

    let v00 = Fixed { mag: hash_2d(seed, xi, yi) % ONE };
    let v10 = Fixed { mag: hash_2d(seed, xi + 1, yi) % ONE };
    let v01 = Fixed { mag: hash_2d(seed, xi, yi + 1) % ONE };
    let v11 = Fixed { mag: hash_2d(seed, xi + 1, yi + 1) % ONE };

    let i1 = lerp(v00, v10, xf);
    let i2 = lerp(v01, v11, xf);

    lerp(i1, i2, yf)
}
