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

pub fn calculate_poseidon_commitment(chunk: &crate::chunk::Chunk) -> String {
    use starknet_crypto::{Felt, poseidon_hash_many};

    let mut data = Vec::new();

    // Parse seed hash as Felt
    let seed_fe = Felt::from_hex(&chunk.seed_hash).unwrap_or(Felt::ZERO);
    data.push(seed_fe);
    data.push(Felt::from(chunk.chunk_coord.x as u64));
    data.push(Felt::from(chunk.chunk_coord.y as u64));
    data.push(Felt::from(chunk.chunk_size.width as u64));
    data.push(Felt::from(chunk.chunk_size.height as u64));
    // Note: negative coordinates are cast to u64 for Poseidon; the verifier must match this behavior.

    for cell in &chunk.cells {
        data.push(Felt::from(cell.height as u64));
        data.push(Felt::from(cell.temperature as u64));
        data.push(Felt::from(cell.moisture as u64));
        data.push(Felt::from(cell.biome_id as u64));
        data.push(Felt::from(cell.resource_mask as u64));
    }

    let hash = poseidon_hash_many(&data);
    format!("0x{:x}", hash)
}
