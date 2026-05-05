use crate::fixed::Fixed;

/// Simple deterministic hash for noise.
pub fn hash_2d(seed: i32, x: i32, y: i32) -> i32 {
    let mut h = seed ^ (x.wrapping_mul(374761393)) ^ (y.wrapping_mul(668265263));
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

/// Simple linear interpolation for fixed-point.
pub fn lerp(a: Fixed, b: Fixed, t: Fixed) -> Fixed {
    let diff = b.sub(a);
    a.add(diff.mul(t))
}

/// Simple value noise implementation using fixed-point math.
/// Returns values in the range [0, Fixed::ONE) (i.e., [0, 1)).
pub fn value_noise_2d(seed: i32, x: Fixed, y: Fixed) -> Fixed {
    let xi = x.to_i32();
    let yi = y.to_i32();

    // Fractional part
    let xf = Fixed::from_bits(x.to_bits() & (Fixed::ONE - 1));
    let yf = Fixed::from_bits(y.to_bits() & (Fixed::ONE - 1));

    let v00 = Fixed::from_bits(hash_2d(seed, xi, yi) & 0xFFFF);
    let v10 = Fixed::from_bits(hash_2d(seed, xi + 1, yi) & 0xFFFF);
    let v01 = Fixed::from_bits(hash_2d(seed, xi, yi + 1) & 0xFFFF);
    let v11 = Fixed::from_bits(hash_2d(seed, xi + 1, yi + 1) & 0xFFFF);

    let i1 = lerp(v00, v10, xf);
    let i2 = lerp(v01, v11, xf);

    lerp(i1, i2, yf)
}

/// Fractal noise with multiple octaves.
/// Returns values roughly in [0, Fixed::ONE).
pub fn fractal_noise_2d(seed: i32, x: Fixed, y: Fixed, octaves: u32) -> Fixed {
    let mut amplitude = Fixed::from_bits(Fixed::ONE); // 1.0 in fixed-point
    let mut frequency = Fixed::from_bits(Fixed::ONE >> 3); // Start with 0.125 frequency
    let mut result = Fixed::from_i32(0);
    let mut max_value = Fixed::from_i32(0);

    for i in 0..octaves {
        let sx = seed.wrapping_add(i as i32);
        let sample_x = x.mul(frequency);
        let sample_y = y.mul(frequency);

        result = result.add(value_noise_2d(sx, sample_x, sample_y).mul(amplitude));
        max_value = max_value.add(amplitude);

        amplitude = Fixed::from_bits(amplitude.to_bits() >> 1); // amplitude *= 0.5
        frequency = Fixed::from_bits(frequency.to_bits() << 1); // frequency *= 2
    }

    // Normalize
    result.div(max_value)
}
