from time import now
from ..terrain import generate_terrain_v1

fn main():
    let seed: Int32 = 123
    let width = 64
    let height = 64
    
    let start = now()
    let chunk = generate_terrain_v1(seed, 0, 0, width, height)
    let end = now()
    
    print("Mojo Generation Time (64x64):", (end - start) / 1e6, "ms")
    print("Cells generated:", width * height)
