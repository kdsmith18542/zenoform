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
