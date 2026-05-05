use godot::prelude::*;
use zenoform_core::{Chunk, ChunkCoord, ChunkSize, module::generate_terrain_v1};
use zenoform_verifier::verify_chunk;

struct ZenoformExtension;

#[gdextension]
impl ExtensionLibrary for ZenoformExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct ZenoformNode {
    base: Base<Node>,
}

#[godot_api]
impl INode for ZenoformNode {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl ZenoformNode {
    #[func]
    fn generate_chunk_json(&self, seed: i32, x: i32, y: i32, z: i32, width: u32, height: u32) -> GString {
        let coord = ChunkCoord::new(x, y, z);
        let size = ChunkSize::new(width, height);
        let chunk = generate_terrain_v1("godot-demo".to_string(), seed, coord, size);
        
        serde_json::to_string(&chunk).unwrap().into()
    }

    #[func]
    fn verify_chunk_json(&self, chunk_json: GString, proof_json: GString) -> bool {
        let chunk: Chunk = match serde_json::from_str(&chunk_json.to_string()) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let proof: zenoform_core::proof::ProofPackage = match serde_json::from_str(&proof_json.to_string()) {
            Ok(p) => p,
            Err(_) => return false,
        };

        verify_chunk(&chunk, &proof).is_ok()
    }
}

// --- C ABI ---

#[no_mangle]
pub extern "C" fn zenoform_verify_chunk(
    chunk_json: *const libc::c_char,
    proof_json: *const libc::c_char,
) -> bool {
    if chunk_json.is_null() || proof_json.is_null() {
        return false;
    }

    let c_chunk = unsafe { std::ffi::CStr::from_ptr(chunk_json) };
    let c_proof = unsafe { std::ffi::CStr::from_ptr(proof_json) };

    let chunk_str = match c_chunk.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let proof_str = match c_proof.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let chunk: Chunk = match serde_json::from_str(chunk_str) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let proof: zenoform_core::proof::ProofPackage = match serde_json::from_str(proof_str) {
        Ok(p) => p,
        Err(_) => return false,
    };

    verify_chunk(&chunk, &proof).is_ok()
}
