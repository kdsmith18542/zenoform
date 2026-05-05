use crate::chunk::Chunk;
use blake3::Hasher;

pub fn calculate_chunk_commitment(chunk: &Chunk) -> String {
    let mut hasher = Hasher::new();
    
    // Add header info
    hasher.update(chunk.protocol_version.as_bytes());
    hasher.update(chunk.world_id.as_bytes());
    hasher.update(chunk.seed_hash.as_bytes());
    hasher.update(&chunk.chunk_coord.x.to_le_bytes());
    hasher.update(&chunk.chunk_coord.y.to_le_bytes());
    hasher.update(&chunk.chunk_coord.z.to_le_bytes());
    hasher.update(&chunk.chunk_size.width.to_le_bytes());
    hasher.update(&chunk.chunk_size.height.to_le_bytes());
    hasher.update(chunk.module_hash.as_bytes());

    // Add cell data
    for cell in &chunk.cells {
        hasher.update(&cell.height.to_le_bytes());
        hasher.update(&cell.temperature.to_le_bytes());
        hasher.update(&cell.moisture.to_le_bytes());
        hasher.update(&[cell.biome_id]);
        hasher.update(&cell.resource_mask.to_le_bytes());
    }

    let hash = hasher.finalize();
    format!("0x{}", hash.to_hex())
}
