use crate::chunk::{Cell, Chunk};
use crate::commitment::calculate_poseidon_commitment;
use crate::coord::{ChunkCoord, ChunkSize};
use crate::fixed::Fixed;
use crate::noise::fractal_noise_2d;
use crate::seed_hash::{default_registry, derive_seed_hash};

/// Generate a deterministic terrain chunk with height, temperature, moisture,
/// biome classification, and resource placement.
pub fn generate_terrain_v1(world_id: String, seed: i32, coord: ChunkCoord, size: ChunkSize) -> Chunk {
    let module_id = "terrain.fixed_noise.v1".to_string();
    let seed_hex = derive_seed_hash(&world_id, seed);

    let registry = default_registry();
    let module_hash = registry.get_hash(&module_id).unwrap_or("0x-module-hash-placeholder").to_string();

    let mut chunk = Chunk::new_v1(world_id, seed_hex, coord, size, module_id, module_hash);

    for y in 0..size.height {
        for x in 0..size.width {
            let world_x = Fixed::from_i32(coord.x * size.width as i32 + x as i32);
            let world_y = Fixed::from_i32(coord.y * size.height as i32 + y as i32);

            // Height: 4-octave fractal noise, scaled to u16 range
            let height_noise = fractal_noise_2d(seed, world_x, world_y, 4);
            let height = ((height_noise.to_bits() as u32 * 65535) >> 16) as u16;

            // Temperature: 3-octave noise, scaled to u16
            let temp_noise = fractal_noise_2d(seed.wrapping_add(1), world_x, world_y, 3);
            let temperature = ((temp_noise.to_bits() as u32 * 65535) >> 16) as u16;

            // Moisture: 3-octave noise, scaled to u16
            let moisture_noise = fractal_noise_2d(seed.wrapping_add(2), world_x, world_y, 3);
            let moisture = ((moisture_noise.to_bits() as u32 * 65535) >> 16) as u16;

            // Biome classification
            let biome_id = classify_biome(height, temperature, moisture);

            // Resource placement
            let resource_mask = place_resources(seed.wrapping_add(3), world_x, world_y, biome_id, height);

            chunk.cells.push(Cell { local_x: x, local_y: y, height, temperature, moisture, biome_id, resource_mask });
        }
    }

    // Calculate Poseidon commitment for proof-friendliness
    chunk.commitment = calculate_poseidon_commitment(&chunk);
    chunk
}

/// Simple biome classifier based on height, temperature, and moisture.
/// Uses fixed-point thresholds for determinism.
fn classify_biome(height: u16, temperature: u16, moisture: u16) -> u8 {
    // Height thresholds (in u16 space, 0-65535)
    const WATER_LEVEL: u16 = 15000;
    const BEACH_LEVEL: u16 = 18000;
    const MOUNTAIN_LEVEL: u16 = 45000;
    const SNOW_LEVEL: u16 = 55000;

    // Temperature thresholds
    const COLD: u16 = 20000;
    const HOT: u16 = 50000;

    // Moisture thresholds
    const DRY: u16 = 20000;
    const WET: u16 = 50000;

    if height < WATER_LEVEL {
        return 0; // Ocean
    }
    if height < BEACH_LEVEL {
        return 1; // Beach
    }
    if height > SNOW_LEVEL {
        return 2; // Snow/Ice
    }
    if height > MOUNTAIN_LEVEL {
        return 3; // Mountain
    }
    if temperature < COLD {
        if moisture > WET {
            return 4; // Taiga
        }
        return 5; // Tundra
    }
    if temperature > HOT {
        if moisture > WET {
            return 6; // Jungle
        }
        if moisture > DRY {
            return 7; // Savanna
        }
        return 8; // Desert
    }
    // Temperate zone
    if moisture > WET {
        return 9; // Rainforest
    }
    if moisture > DRY {
        return 10; // Forest
    }
    11 // Plains
}

/// Resource placement based on biome and local noise.
/// Returns a bitmask of resources present in the cell.
fn place_resources(seed: i32, world_x: Fixed, world_y: Fixed, biome_id: u8, height: u16) -> u16 {
    let resource_noise = fractal_noise_2d(seed, world_x, world_y, 2);
    let resource_val = resource_noise.to_bits() as u16;

    let mut mask: u16 = 0;

    // Bit flags for resources
    const WOOD: u16 = 1 << 0;
    const STONE: u16 = 1 << 1;
    const ORE: u16 = 1 << 2;
    const GEM: u16 = 1 << 3;
    const SAND: u16 = 1 << 4;
    const CLAY: u16 = 1 << 5;
    const COAL: u16 = 1 << 6;
    const GOLD: u16 = 1 << 7;

    // Threshold for resource spawning
    const RESOURCE_THRESHOLD: u16 = 48000;

    if resource_val < RESOURCE_THRESHOLD {
        return mask; // No resources
    }

    match biome_id {
        0 => {
            // Ocean
            if resource_val > 55000 {
                mask |= CLAY;
            }
        }
        1 => {
            // Beach
            mask |= SAND;
            if resource_val > 52000 {
                mask |= CLAY;
            }
        }
        2 | 5 => {
            // Snow / Tundra
            mask |= STONE;
            if resource_val > 56000 {
                mask |= COAL;
            }
        }
        3 => {
            // Mountain
            mask |= STONE;
            if resource_val > 51000 {
                mask |= ORE;
            }
            if resource_val > 58000 {
                mask |= GEM;
            }
            if height > 52000 && resource_val > 60000 {
                mask |= GOLD;
            }
        }
        4 | 10 => {
            // Taiga / Forest
            mask |= WOOD;
            if resource_val > 53000 {
                mask |= STONE;
            }
        }
        6 | 9 => {
            // Jungle / Rainforest
            mask |= WOOD;
            if resource_val > 54000 {
                mask |= CLAY;
            }
        }
        7 => {
            // Savanna
            mask |= WOOD;
            if resource_val > 55000 {
                mask |= STONE;
            }
        }
        8 => {
            // Desert
            mask |= SAND;
            if resource_val > 57000 {
                mask |= GOLD;
            }
        }
        11 => {
            // Plains
            if resource_val > 50000 {
                mask |= WOOD;
            }
            if resource_val > 56000 {
                mask |= CLAY;
            }
        }
        _ => {}
    }

    mask
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

        // Verify all cells have populated fields
        for cell in &chunk.cells {
            assert!(cell.biome_id <= 11);
        }
    }

    #[test]
    fn test_determinism() {
        let coord = ChunkCoord::new(3, -2, 0);
        let size = ChunkSize::new(16, 16);

        let chunk1 = generate_terrain_v1("w".to_string(), 123, coord, size);
        let chunk2 = generate_terrain_v1("w".to_string(), 123, coord, size);

        assert_eq!(chunk1.cells.len(), chunk2.cells.len());
        assert_eq!(chunk1.commitment, chunk2.commitment);

        for (a, b) in chunk1.cells.iter().zip(chunk2.cells.iter()) {
            assert_eq!(a.height, b.height);
            assert_eq!(a.temperature, b.temperature);
            assert_eq!(a.moisture, b.moisture);
            assert_eq!(a.biome_id, b.biome_id);
            assert_eq!(a.resource_mask, b.resource_mask);
        }
    }

    #[test]
    fn test_different_seeds_produce_different_chunks() {
        let coord = ChunkCoord::new(0, 0, 0);
        let size = ChunkSize::new(16, 16);

        let chunk1 = generate_terrain_v1("w".to_string(), 1, coord, size);
        let chunk2 = generate_terrain_v1("w".to_string(), 2, coord, size);

        assert_ne!(chunk1.commitment, chunk2.commitment);
    }

    #[test]
    fn test_commitment_changes_on_tamper() {
        let coord = ChunkCoord::new(0, 0, 0);
        let size = ChunkSize::new(16, 16);

        let mut chunk = generate_terrain_v1("w".to_string(), 42, coord, size);
        let original_commitment = chunk.commitment.clone();

        chunk.cells[0].height = 9999;
        chunk.commitment = calculate_poseidon_commitment(&chunk);

        assert_ne!(chunk.commitment, original_commitment);
    }
}
