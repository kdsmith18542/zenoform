use zenoform_terrain_v1::fixed::{Fixed, FixedTrait};
use zenoform_terrain_v1::noise::fractal_noise_2d;

#[derive(Copy, Drop, Serde)]
pub struct Cell {
    pub local_x: u32,
    pub local_y: u32,
    pub height: u16,
    pub temperature: u16,
    pub moisture: u16,
    pub biome_id: u8,
    pub resource_mask: u16,
}

pub fn generate_terrain_v1(
    seed: i128,
    chunk_x: i32,
    chunk_y: i32,
    width: u32,
    height: u32,
) -> Array<Cell> {
    let mut cells = array![];

    let mut y: u32 = 0;
    while y < height {
        let mut x: u32 = 0;
        while x < width {
            let cx: i128 = chunk_x.into();
            let wx: i128 = width.into();
            let lx: i128 = x.into();
            let world_x = FixedTrait::from_i128(cx * wx + lx);

            let cy: i128 = chunk_y.into();
            let hy: i128 = height.into();
            let ly: i128 = y.into();
            let world_y = FixedTrait::from_i128(cy * hy + ly);

            let height_noise = fractal_noise_2d(seed, @world_x, @world_y, 4);
            let height_felt: felt252 = height_noise.mag.into();
            let height_raw: u256 = height_felt.try_into().unwrap();
            let height_val: u16 = ((height_raw * 65535_u256 / 65536_u256) % 65536_u256).try_into().unwrap();

            let temp_noise = fractal_noise_2d(seed + 1, @world_x, @world_y, 3);
            let temp_felt: felt252 = temp_noise.mag.into();
            let temp_raw: u256 = temp_felt.try_into().unwrap();
            let temperature: u16 = ((temp_raw * 65535_u256 / 65536_u256) % 65536_u256).try_into().unwrap();

            let moisture_noise = fractal_noise_2d(seed + 2, @world_x, @world_y, 3);
            let moisture_felt: felt252 = moisture_noise.mag.into();
            let moisture_raw: u256 = moisture_felt.try_into().unwrap();
            let moisture: u16 = ((moisture_raw * 65535_u256 / 65536_u256) % 65536_u256).try_into().unwrap();

            let biome_id = classify_biome(height_val, temperature, moisture);
            let resource_mask = place_resources(seed + 3, @world_x, @world_y, biome_id, height_val);

            cells.append(Cell {
                local_x: x,
                local_y: y,
                height: height_val,
                temperature,
                moisture,
                biome_id,
                resource_mask,
            });
            x += 1;
        };
        y += 1;
    };

    cells
}

pub fn classify_biome(height: u16, temperature: u16, moisture: u16) -> u8 {
    let WATER_LEVEL: u16 = 15000;
    let BEACH_LEVEL: u16 = 18000;
    let MOUNTAIN_LEVEL: u16 = 45000;
    let SNOW_LEVEL: u16 = 55000;
    let COLD: u16 = 20000;
    let HOT: u16 = 50000;
    let DRY: u16 = 20000;
    let WET: u16 = 50000;

    if height < WATER_LEVEL {
        return 0;
    }
    if height < BEACH_LEVEL {
        return 1;
    }
    if height > SNOW_LEVEL {
        return 2;
    }
    if height > MOUNTAIN_LEVEL {
        return 3;
    }
    if temperature < COLD {
        if moisture > WET {
            return 4;
        }
        return 5;
    }
    if temperature > HOT {
        if moisture > WET {
            return 6;
        }
        if moisture > DRY {
            return 7;
        }
        return 8;
    }
    if moisture > WET {
        return 9;
    }
    if moisture > DRY {
        return 10;
    }
    11
}

pub fn place_resources(seed: i128, world_x: @Fixed, world_y: @Fixed, biome_id: u8, height: u16) -> u16 {
    let resource_noise = fractal_noise_2d(seed, world_x, world_y, 2);
    let resource_val: u16 = (resource_noise.mag % 0x10000).try_into().unwrap();

    let mut mask: u16 = 0;

    let WOOD: u16 = 1;
    let STONE: u16 = 2;
    let ORE: u16 = 4;
    let GEM: u16 = 8;
    let SAND: u16 = 16;
    let CLAY: u16 = 32;
    let COAL: u16 = 64;
    let GOLD: u16 = 128;

    let RESOURCE_THRESHOLD: u16 = 48000;

    if resource_val < RESOURCE_THRESHOLD {
        return mask;
    }

    if biome_id == 0 {
        if resource_val > 55000 {
            mask = mask | CLAY;
        }
    } else if biome_id == 1 {
        mask = mask | SAND;
        if resource_val > 52000 {
            mask = mask | CLAY;
        }
    } else if biome_id == 2 || biome_id == 5 {
        mask = mask | STONE;
        if resource_val > 56000 {
            mask = mask | COAL;
        }
    } else if biome_id == 3 {
        mask = mask | STONE;
        if resource_val > 51000 {
            mask = mask | ORE;
        }
        if resource_val > 58000 {
            mask = mask | GEM;
        }
        if height > 52000 && resource_val > 60000 {
            mask = mask | GOLD;
        }
    } else if biome_id == 4 || biome_id == 10 {
        mask = mask | WOOD;
        if resource_val > 53000 {
            mask = mask | STONE;
        }
    } else if biome_id == 6 || biome_id == 9 {
        mask = mask | WOOD;
        if resource_val > 54000 {
            mask = mask | CLAY;
        }
    } else if biome_id == 7 {
        mask = mask | WOOD;
        if resource_val > 55000 {
            mask = mask | STONE;
        }
    } else if biome_id == 8 {
        mask = mask | SAND;
        if resource_val > 57000 {
            mask = mask | GOLD;
        }
    } else if biome_id == 11 {
        if resource_val > 50000 {
            mask = mask | WOOD;
        }
        if resource_val > 56000 {
            mask = mask | CLAY;
        }
    }

    mask
}
