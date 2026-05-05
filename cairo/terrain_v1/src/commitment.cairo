use core::poseidon::poseidon_hash_span;
use zenoform_terrain_v1::module::Cell;

pub fn calculate_commitment(
    seed: i128,
    chunk_x: i32,
    chunk_y: i32,
    width: u32,
    height: u32,
    cells: Span<Cell>
) -> felt252 {
    let mut data = array![
        seed.into(),
        chunk_x.into(),
        chunk_y.into(),
        width.into(),
        height.into()
    ];

    let mut i = 0;
    while i < cells.len() {
        let cell = *cells.at(i);
        data.append(cell.height.into());
        data.append(cell.temperature.into());
        data.append(cell.moisture.into());
        data.append(cell.biome_id.into());
        data.append(cell.resource_mask.into());
        i += 1;
    };

    poseidon_hash_span(data.span())
}
