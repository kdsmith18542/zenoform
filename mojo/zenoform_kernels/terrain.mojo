from algorithm import vectorize
from memory import UnsafePointer
from sys import simdwidthof
from .noise import value_noise_2d

alias type = DType.int32
alias nelts = simdwidthof[type]()

struct ChunkData:
    var height: UnsafePointer[Int16]
    var width: Int
    var height_dim: Int

    fn __init__(inout self, width: Int, height_dim: Int):
        self.width = width
        self.height_dim = height_dim
        self.height = UnsafePointer[Int16].alloc(width * height_dim)

    fn __del__(owned self):
        self.height.free()

fn generate_terrain_v1(seed: Int32, chunk_x: Int, chunk_y: Int, width: Int, height_dim: Int) -> ChunkData:
    var data = ChunkData(width, height_dim)
    alias ONE = 65536

    for y in range(height_dim):
        @parameter
        fn inner[nelts: Int](x: Int):
            let world_x = (chunk_x * width + x) * ONE
            let world_y = (chunk_y * height_dim + y) * ONE
            
            let h = value_noise_2d(seed, SIMD[type, nelts](world_x), SIMD[type, nelts](world_y))
            
            for i in range(nelts):
                 data.height[y * width + x + i] = h[i].cast[DType.int16]()

        vectorize[inner, nelts](width)

    return data
