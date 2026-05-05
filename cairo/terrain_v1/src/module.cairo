use zenoform_terrain_v1::fixed::{Fixed, FixedTrait};
use zenoform_terrain_v1::noise::value_noise_2d;

#[derive(Copy, Drop, Serde)]
struct Cell {
    local_x: u32,
    local_y: u32,
    height: u16,
}

fn generate_terrain_v1(
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
            let world_x = FixedTrait::from_i128((chunk_x * width.try_into().unwrap() + x.try_into().unwrap()).into());
            let world_y = FixedTrait::from_i128((chunk_y * height.try_into().unwrap() + y.try_into().unwrap()).into());

            let h_fixed = value_noise_2d(seed, world_x, world_y);
            
            cells.append(Cell {
                local_x: x,
                local_y: y,
                height: (h_fixed.mag % 0x10000).try_into().unwrap(), // u16
            });
            x += 1;
        };
        y += 1;
    };
    
    cells
}
