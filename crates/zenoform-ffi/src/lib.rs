use zenoform_core::{chunk::Chunk, proof::ProofPackage};
use zenoform_verifier::verify_chunk;

/// Verify a chunk against a proof package.
/// Returns 1 on success, 0 on failure.
/// Both inputs must be valid null-terminated UTF-8 strings.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn zenoform_verify_chunk(
    chunk_json: *const libc::c_char,
    proof_json: *const libc::c_char,
) -> libc::c_int {
    if chunk_json.is_null() || proof_json.is_null() {
        return 0;
    }

    let c_chunk = unsafe { std::ffi::CStr::from_ptr(chunk_json) };
    let c_proof = unsafe { std::ffi::CStr::from_ptr(proof_json) };

    let chunk_str = match c_chunk.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let proof_str = match c_proof.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let chunk: Chunk = match serde_json::from_str(chunk_str) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let proof: ProofPackage = match serde_json::from_str(proof_str) {
        Ok(p) => p,
        Err(_) => return 0,
    };

    if verify_chunk(&chunk, &proof).is_ok() { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use zenoform_core::{
        Cell, Chunk, ChunkCoord, ChunkSize,
        commitment::calculate_poseidon_commitment,
        proof::{ProofPackage, PublicInputs},
    };

    fn make_test_chunk() -> Chunk {
        let mut chunk = Chunk::new_v1(
            "ffi-world".to_string(),
            "0xdeadbeef".to_string(),
            ChunkCoord::new(0, 0, 0),
            ChunkSize::new(1, 1),
            "terrain.fixed_noise.v1".to_string(),
            "0xmodule".to_string(),
        );
        chunk.cells = vec![Cell {
            local_x: 0,
            local_y: 0,
            height: 100,
            temperature: 50,
            moisture: 75,
            biome_id: 0,
            resource_mask: 0,
        }];
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
    fn test_ffi_valid_chunk_and_proof_returns_1() {
        let chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);

        let chunk_cstring = CString::new(serde_json::to_string(&chunk).unwrap()).unwrap();
        let proof_cstring = CString::new(serde_json::to_string(&proof).unwrap()).unwrap();

        let result = zenoform_verify_chunk(chunk_cstring.as_ptr(), proof_cstring.as_ptr());
        assert_eq!(result, 1);
    }

    #[test]
    fn test_ffi_null_pointers_return_0() {
        assert_eq!(zenoform_verify_chunk(std::ptr::null(), std::ptr::null()), 0);
    }

    #[test]
    fn test_ffi_null_chunk_returns_0() {
        let proof = make_test_proof(&make_test_chunk());
        let proof_cstring = CString::new(serde_json::to_string(&proof).unwrap()).unwrap();
        assert_eq!(zenoform_verify_chunk(std::ptr::null(), proof_cstring.as_ptr()), 0);
    }

    #[test]
    fn test_ffi_null_proof_returns_0() {
        let chunk = make_test_chunk();
        let chunk_cstring = CString::new(serde_json::to_string(&chunk).unwrap()).unwrap();
        assert_eq!(zenoform_verify_chunk(chunk_cstring.as_ptr(), std::ptr::null()), 0);
    }

    #[test]
    fn test_ffi_invalid_json_returns_0() {
        let bad_json = CString::new("not json").unwrap();
        let chunk = make_test_chunk();
        let chunk_cstring = CString::new(serde_json::to_string(&chunk).unwrap()).unwrap();

        assert_eq!(zenoform_verify_chunk(bad_json.as_ptr(), bad_json.as_ptr()), 0);
        assert_eq!(zenoform_verify_chunk(chunk_cstring.as_ptr(), bad_json.as_ptr()), 0);
    }

    #[test]
    fn test_ffi_tampered_chunk_returns_0() {
        let mut chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);

        chunk.cells[0].height = 9999;

        let chunk_cstring = CString::new(serde_json::to_string(&chunk).unwrap()).unwrap();
        let proof_cstring = CString::new(serde_json::to_string(&proof).unwrap()).unwrap();

        let result = zenoform_verify_chunk(chunk_cstring.as_ptr(), proof_cstring.as_ptr());
        assert_eq!(result, 0);
    }

    #[test]
    fn test_ffi_mismatched_proof_returns_0() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.public_inputs.output_commitment = "0xbad".to_string();

        let chunk_cstring = CString::new(serde_json::to_string(&chunk).unwrap()).unwrap();
        let proof_cstring = CString::new(serde_json::to_string(&proof).unwrap()).unwrap();

        let result = zenoform_verify_chunk(chunk_cstring.as_ptr(), proof_cstring.as_ptr());
        assert_eq!(result, 0);
    }
}
