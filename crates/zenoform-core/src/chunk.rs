use serde::{Deserialize, Serialize};
use crate::coord::{ChunkCoord, ChunkSize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub local_x: u32,
    pub local_y: u32,
    pub height: u16,
    pub temperature: u16,
    pub moisture: u16,
    pub biome_id: u8,
    pub resource_mask: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub schema_version: String,
    pub protocol_version: String,
    pub world_id: String,
    pub seed_hash: String,
    pub chunk_coord: ChunkCoord,
    pub chunk_size: ChunkSize,
    pub module_id: String,
    pub module_hash: String,
    pub cells: Vec<Cell>,
    pub commitment: String,
}

impl Chunk {
    pub fn new_v1(
        world_id: String,
        seed_hash: String,
        coord: ChunkCoord,
        size: ChunkSize,
        module_id: String,
        module_hash: String,
    ) -> Self {
        Self {
            schema_version: "zenoform.chunk.v1".to_string(),
            protocol_version: "zenoform-terrain-v1".to_string(),
            world_id,
            seed_hash,
            chunk_coord: coord,
            chunk_size: size,
            module_id,
            module_hash,
            cells: Vec::with_capacity(size.total_cells()),
            commitment: String::new(),
        }
    }
}
