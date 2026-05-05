from algorithm import vectorize
from memory import UnsafePointer
from sys import simdwidthof

alias type = DType.int32
alias nelts = simdwidthof[type]()

fn hash_2d(seed: Int32, x: SIMD[type, nelts], y: SIMD[type, nelts]) -> SIMD[type, nelts]:
    var h = seed ^ (x * 374761393) ^ (y * 668265263)
    h = (h ^ (h >> 13)) * 1274126177
    return h ^ (h >> 16)

fn lerp(a: SIMD[type, nelts], b: SIMD[type, nelts], t: SIMD[type, nelts]) -> SIMD[type, nelts]:
    # Fixed-point lerp: a + (b - a) * t / ONE
    alias ONE = 65536
    let diff = b - a
    return a + (diff * t) // ONE

fn value_noise_2d(seed: Int32, x: SIMD[type, nelts], y: SIMD[type, nelts]) -> SIMD[type, nelts]:
    alias ONE = 65536
    
    let xi = x // ONE
    let yi = y // ONE
    
    let xf = x % ONE
    let yf = y % ONE

    let v00 = hash_2d(seed, xi, yi) % ONE
    let v10 = hash_2d(seed, xi + 1, yi) % ONE
    let v01 = hash_2d(seed, xi, yi + 1) % ONE
    let v11 = hash_2d(seed, xi + 1, yi + 1) % ONE

    let i1 = lerp(v00, v10, xf)
    let i2 = lerp(v01, v11, xf)

    return lerp(i1, i2, yf)
