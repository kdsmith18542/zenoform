use crate::chunk::{Chunk, Cell};
use crate::coord::{ChunkCoord, ChunkSize};
use crate::fixed::Fixed;
use crate::noise::value_noise_2d;

pub fn generate_terrain_v1(
    world_id: String,
    seed: i32,
    coord: ChunkCoord,
    size: ChunkSize,
) -> Chunk {
    let mut chunk = Chunk::new_v1(
        world_id,
        format!("{:x}", seed), // Simple seed hash placeholder
        coord,
        size,
        "terrain.fixed_noise.v1".to_string(),
        "0x-module-hash-placeholder".to_string(),
    );

    for y in 0..size.height {
        for x in 0..size.width {
            let world_x = Fixed::from_i32(coord.x * size.width as i32 + x as i32);
            let world_y = Fixed::from_i32(coord.y * size.height as i32 + y as i32);

            let height_fixed = value_noise_2d(seed, world_x, world_y);
            
            chunk.cells.push(Cell {
                local_x: x,
                local_y: y,
                height: height_fixed.to_bits() as u16,
                temperature: 0,
                moisture: 0,
                biome_id: 0,
                resource_mask: 0,
            });
        }
    }

    // Calculate commitment
    chunk.commitment = crate::commitment::calculate_chunk_commitment(&chunk);
    chunk
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::{ChunkCoord, ChunkSize};

    #[test]
    fn test_generate_chunk() {
        let world_id = "test-world".to_string();
        let seed = 42;
        let coord = ChunkCoord::new(0, 0, 0);
        let size = ChunkSize::new(16, 16);
        
        let chunk = generate_terrain_v1(world_id, seed, coord, size);
        
        assert_eq!(chunk.cells.len(), 256);
        assert!(!chunk.commitment.is_empty());
        println!("Commitment: {}", chunk.commitment);
    }
}
