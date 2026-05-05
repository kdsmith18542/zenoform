use wasm_bindgen::prelude::*;
use zenoform_core::{chunk::Chunk, proof::ProofPackage};
use zenoform_verifier::verify_chunk;

#[wasm_bindgen]
pub fn wasm_verify_chunk(chunk_json: &str, proof_json: &str) -> bool {
    let chunk: Chunk = match serde_json::from_str(chunk_json) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let proof: ProofPackage = match serde_json::from_str(proof_json) {
        Ok(p) => p,
        Err(_) => return false,
    };

    verify_chunk(&chunk, &proof).is_ok()
}

#[wasm_bindgen]
pub fn wasm_verify_chunk_detailed(chunk_json: &str, proof_json: &str) -> String {
    let chunk: Chunk = match serde_json::from_str(chunk_json) {
        Ok(c) => c,
        Err(e) => return format!("ERROR: Invalid chunk JSON: {}", e),
    };

    let proof: ProofPackage = match serde_json::from_str(proof_json) {
        Ok(p) => p,
        Err(e) => return format!("ERROR: Invalid proof JSON: {}", e),
    };

    match verify_chunk(&chunk, &proof) {
        Ok(_) => "VALID".to_string(),
        Err(e) => format!("INVALID: {}", e),
    }
}

#[wasm_bindgen]
pub fn wasm_recalculate_commitment(chunk_json: &str) -> String {
    use zenoform_core::commitment::calculate_poseidon_commitment;

    let chunk: Chunk = match serde_json::from_str(chunk_json) {
        Ok(c) => c,
        Err(e) => return format!("ERROR: {}", e),
    };

    calculate_poseidon_commitment(&chunk)
}

#[wasm_bindgen]
pub fn wasm_generate_chunk_json(
    world: &str,
    seed: i32,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    width: u32,
    height: u32,
) -> String {
    use zenoform_core::{
        coord::{ChunkCoord, ChunkSize},
        module::generate_terrain_v1,
    };

    let coord = ChunkCoord::new(chunk_x, chunk_y, chunk_z);
    let size = ChunkSize::new(width, height);
    let chunk = generate_terrain_v1(world.to_string(), seed, coord, size);

    serde_json::to_string(&chunk).unwrap_or_default()
}

#[wasm_bindgen]
pub fn wasm_get_commitment(chunk_json: &str) -> String {
    let chunk: Chunk = match serde_json::from_str(chunk_json) {
        Ok(c) => c,
        Err(_) => return "".to_string(),
    };

    chunk.commitment
}

#[cfg(test)]
mod tests {
    use super::*;
    use zenoform_core::{
        Cell, Chunk, ChunkCoord, ChunkSize,
        commitment::calculate_poseidon_commitment,
        proof::{ProofPackage, PublicInputs},
    };

    fn make_test_chunk() -> Chunk {
        let mut chunk = Chunk::new_v1(
            "wasm-world".to_string(),
            "0xabcdef".to_string(),
            ChunkCoord::new(1, 2, 0),
            ChunkSize::new(2, 2),
            "terrain.fixed_noise.v1".to_string(),
            "0xmodulehash".to_string(),
        );
        chunk.cells = vec![
            Cell { local_x: 0, local_y: 0, height: 50, temperature: 30, moisture: 60, biome_id: 0, resource_mask: 0 },
            Cell { local_x: 1, local_y: 0, height: 51, temperature: 31, moisture: 61, biome_id: 1, resource_mask: 1 },
            Cell { local_x: 0, local_y: 1, height: 52, temperature: 32, moisture: 62, biome_id: 2, resource_mask: 2 },
            Cell { local_x: 1, local_y: 1, height: 53, temperature: 33, moisture: 63, biome_id: 3, resource_mask: 3 },
        ];
        chunk.commitment = calculate_poseidon_commitment(&chunk);
        chunk
    }

    fn make_test_proof(chunk: &Chunk) -> ProofPackage {
        ProofPackage::new_v1(
            "mock".to_string(),
            "0.1.0".to_string(),
            "zenoform-terrain-v1".to_string(),
            PublicInputs {
                world_id: chunk.world_id.clone(),
                seed_hash: chunk.seed_hash.clone(),
                chunk_coord: chunk.chunk_coord,
                chunk_size: chunk.chunk_size,
                module_hash: chunk.module_hash.clone(),
                output_commitment: chunk.commitment.clone(),
            },
            serde_json::json!({}),
        )
    }

    #[test]
    fn test_wasm_verify_valid_returns_true() {
        let chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);
        let chunk_json = serde_json::to_string(&chunk).unwrap();
        let proof_json = serde_json::to_string(&proof).unwrap();
        assert!(wasm_verify_chunk(&chunk_json, &proof_json));
    }

    #[test]
    fn test_wasm_verify_bad_json_returns_false() {
        assert!(!wasm_verify_chunk("bad json", "bad json"));
    }

    #[test]
    fn test_wasm_verify_detailed_returns_valid() {
        let chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);
        let chunk_json = serde_json::to_string(&chunk).unwrap();
        let proof_json = serde_json::to_string(&proof).unwrap();
        assert_eq!(wasm_verify_chunk_detailed(&chunk_json, &proof_json), "VALID");
    }

    #[test]
    fn test_wasm_verify_detailed_returns_invalid() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.public_inputs.output_commitment = "0xbad".to_string();
        let chunk_json = serde_json::to_string(&chunk).unwrap();
        let proof_json = serde_json::to_string(&proof).unwrap();
        let result = wasm_verify_chunk_detailed(&chunk_json, &proof_json);
        assert!(result.starts_with("INVALID"));
    }

    #[test]
    fn test_wasm_recalculate_commitment() {
        let chunk = make_test_chunk();
        let chunk_json = serde_json::to_string(&chunk).unwrap();
        let commitment = wasm_recalculate_commitment(&chunk_json);
        assert!(!commitment.is_empty());
        assert!(commitment.starts_with("0x"));
    }

    #[test]
    fn test_wasm_generate_chunk_json() {
        let json = wasm_generate_chunk_json("test", 42, 0, 0, 0, 4, 4);
        let chunk: Chunk = serde_json::from_str(&json).unwrap();
        assert_eq!(chunk.chunk_size.total_cells(), 16);
        assert_eq!(chunk.world_id, "test");
    }

    #[test]
    fn test_wasm_get_commitment() {
        let chunk = make_test_chunk();
        let chunk_json = serde_json::to_string(&chunk).unwrap();
        let c = wasm_get_commitment(&chunk_json);
        assert_eq!(c, chunk.commitment);
    }
}
