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
    use starknet_crypto::{poseidon_hash_many, FieldElement};

    let mut data = Vec::new();
    
    // Parse seed hash as FieldElement
    let seed_fe = FieldElement::from_hex_be(&chunk.seed_hash).unwrap_or(FieldElement::ZERO);
    data.push(seed_fe);
    data.push(FieldElement::from(chunk.chunk_coord.x as u64)); // Note: handle negative coord properly if needed
    data.push(FieldElement::from(chunk.chunk_coord.y as u64));
    data.push(FieldElement::from(chunk.chunk_size.width as u64));
    data.push(FieldElement::from(chunk.chunk_size.height as u64));

    for cell in &chunk.cells {
        data.push(FieldElement::from(cell.height as u64));
    }

    let hash = poseidon_hash_many(&data);
    format!("0x{:x}", hash)
}
