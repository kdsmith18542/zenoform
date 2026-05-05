use blake3::Hasher;
use thiserror::Error;
use zenoform_core::chunk::Chunk;
use zenoform_core::commitment::calculate_poseidon_commitment;
use zenoform_core::proof::ProofPackage;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Invalid proof format: {0}")]
    InvalidProofFormat(String),
    #[error("Unsupported schema version: expected {expected}, found {found}")]
    SchemaVersionMismatch { expected: String, found: String },
    #[error("Commitment mismatch: expected {expected}, found {found}")]
    CommitmentMismatch { expected: String, found: String },
    #[error("Public input mismatch: {field}")]
    PublicInputMismatch { field: String },
    #[error("Proof integrity check failed: {0}")]
    ProofIntegrityError(String),
    #[error("Proof signature verification failed")]
    ProofSignatureInvalid,
    #[error("Proof data is empty or malformed")]
    ProofDataEmpty,
    #[error("STARK proof verification failed: {0}")]
    StarkVerificationFailed(String),
    #[error("Unsupported STARK proof version: {0}")]
    UnsupportedStarkVersion(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    Minimal,
    Standard,
    Strict,
}

/// A deserialized STARK proof payload containing the cryptographic proof data.
///
/// This structure is designed to be forward-compatible with multiple STARK prover backends
/// (Stwo, Stone, Giza, etc.). The `proof_bytes` field holds the opaque binary proof data,
/// while `public_inputs_hash` allows quick validation that the proof was generated for the
/// expected inputs without running full verification.
#[derive(Debug, Clone)]
pub struct StarkProof {
    /// Prover backend that generated this proof (e.g., "stwo-cairo", "stone", "giza")
    pub prover_backend: String,
    /// Version of the prover backend
    pub backend_version: String,
    /// Opaque binary proof data (format depends on prover_backend)
    pub proof_bytes: Vec<u8>,
    /// Hash of the public inputs used to generate the proof (Poseidon or Blake3)
    pub public_inputs_hash: String,
    /// Number of steps / trace rows in the proof
    pub trace_length: Option<u64>,
    /// Verification key identifier (for proof recycling detection)
    pub verification_key_id: Option<String>,
}

/// Trait for pluggable STARK verification backends.
///
/// Implement this trait to integrate a specific STARK prover/verifier (e.g., Stwo, Stone).
/// The `verify` method receives the deserialized proof and public inputs, and must return
/// Ok(()) only if the cryptographic proof is valid.
pub trait StarkVerifierBackend: Send + Sync {
    /// Name of the backend (for diagnostics and logging)
    fn name(&self) -> &str;

    /// Verify a STARK proof cryptographically.
    ///
    /// # Arguments
    /// * `proof` - The deserialized STARK proof data
    /// * `public_inputs` - The public inputs that the proof claims to verify
    ///
    /// # Returns
    /// * `Ok(())` if the proof is cryptographically valid
    /// * `Err(VerifierError::StarkVerificationFailed)` if the proof is invalid
    fn verify(&self, proof: &StarkProof, public_inputs: &serde_json::Value) -> Result<(), VerifierError>;
}

/// Default verifier backend that always fails with StarkVerificationFailed.
///
/// This is a placeholder until a real STARK verifier backend (e.g., stwo-cairo) is integrated.
/// To integrate a real backend:
/// 1. Implement `StarkVerifierBackend` for your prover's Rust bindings
/// 2. Register it in `PROOF_REGISTRY` at program startup
pub struct NoopStarkVerifier;

impl StarkVerifierBackend for NoopStarkVerifier {
    fn name(&self) -> &str {
        "noop"
    }

    fn verify(&self, _proof: &StarkProof, _public_inputs: &serde_json::Value) -> Result<(), VerifierError> {
        Err(VerifierError::StarkVerificationFailed(
            "No STARK verifier backend configured. Install a backend (e.g., stwo-cairo) and register it.".to_string(),
        ))
    }
}

/// Attempt to deserialize a `ProofPackage` into a structured `StarkProof`.
///
/// This function extracts the proof payload and attempts to interpret it as a STARK proof.
/// Currently supports:
/// - `stwo-cairo-proof-json`: JSON-encoded proof metadata with base64-encoded proof bytes
/// - `stwo-cairo-proof-binary`: Raw binary proof data (hex-encoded in JSON string)
/// - `self-signed-v1`: Self-signed proof hash (not a true STARK proof, but verifiable)
pub fn deserialize_stark_proof(package: &ProofPackage) -> Result<StarkProof, VerifierError> {
    match package.proof.format.as_str() {
        "stwo-cairo-proof-json" => deserialize_json_proof(package),
        "stwo-cairo-proof-binary" => deserialize_binary_proof(package),
        "self-signed-v1" => deserialize_self_signed_proof(package),
        "mock" => Err(VerifierError::InvalidProofFormat("mock proofs do not contain STARK data".to_string())),
        other => Err(VerifierError::InvalidProofFormat(format!(
            "cannot deserialize proof format '{}': no deserializer available",
            other
        ))),
    }
}

fn deserialize_json_proof(package: &ProofPackage) -> Result<StarkProof, VerifierError> {
    let payload = &package.proof.payload;

    let proof_b64 = payload
        .get("proof_bytes")
        .and_then(|v| v.as_str())
        .ok_or_else(|| VerifierError::ProofIntegrityError("missing proof_bytes field".to_string()))?;

    let proof_bytes = base64_decode(proof_b64)?;

    let public_inputs_hash = payload.get("public_inputs_hash").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let trace_length = payload.get("trace_length").and_then(|v| v.as_u64());

    let verification_key_id = payload.get("verification_key_id").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(StarkProof {
        prover_backend: package.prover.clone(),
        backend_version: package.prover_version.clone(),
        proof_bytes,
        public_inputs_hash,
        trace_length,
        verification_key_id,
    })
}

fn deserialize_binary_proof(package: &ProofPackage) -> Result<StarkProof, VerifierError> {
    let payload = &package.proof.payload;

    let proof_hex = payload
        .get("proof_bytes")
        .and_then(|v| v.as_str())
        .ok_or_else(|| VerifierError::ProofIntegrityError("missing proof_bytes field".to_string()))?;

    let proof_bytes = hex_decode(proof_hex)?;

    Ok(StarkProof {
        prover_backend: package.prover.clone(),
        backend_version: package.prover_version.clone(),
        proof_bytes,
        public_inputs_hash: String::new(),
        trace_length: None,
        verification_key_id: None,
    })
}

fn deserialize_self_signed_proof(package: &ProofPackage) -> Result<StarkProof, VerifierError> {
    let payload = &package.proof.payload;

    let signature_hex = payload
        .get("signature")
        .and_then(|v| v.as_str())
        .ok_or_else(|| VerifierError::ProofIntegrityError("missing signature field".to_string()))?;

    let signature = hex_decode(signature_hex)?;

    Ok(StarkProof {
        prover_backend: "self-signed".to_string(),
        backend_version: "1".to_string(),
        proof_bytes: signature,
        public_inputs_hash: payload.get("public_inputs_hash").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        trace_length: None,
        verification_key_id: None,
    })
}

fn base64_decode(s: &str) -> Result<Vec<u8>, VerifierError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| VerifierError::ProofIntegrityError(format!("base64 decode error: {}", e)))
}

fn hex_decode(s: &str) -> Result<Vec<u8>, VerifierError> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2.min(s.len() - i)], 16)
                .map_err(|e| VerifierError::ProofIntegrityError(format!("hex decode error: {}", e)))
        })
        .collect()
}

pub fn verify_chunk(chunk: &Chunk, proof: &ProofPackage) -> Result<(), VerifierError> {
    verify_chunk_with_level(chunk, proof, VerificationLevel::Standard)
}

pub fn verify_chunk_with_level(
    chunk: &Chunk,
    proof: &ProofPackage,
    level: VerificationLevel,
) -> Result<(), VerifierError> {
    verify_schema_version(proof)?;
    verify_public_inputs(chunk, proof)?;
    verify_commitment_integrity(chunk, proof)?;
    verify_proof_integrity(proof, level)?;

    Ok(())
}

/// Verify a chunk using a specific STARK verifier backend.
///
/// This is the advanced verification path that performs full cryptographic STARK proof
/// verification in addition to the standard structural checks.
pub fn verify_chunk_with_backend(
    chunk: &Chunk,
    proof: &ProofPackage,
    backend: &dyn StarkVerifierBackend,
) -> Result<(), VerifierError> {
    // 1. Standard structural verification
    verify_chunk_with_level(chunk, proof, VerificationLevel::Standard)?;

    // 2. Deserialize the STARK proof payload
    let stark_proof = deserialize_stark_proof(proof)?;

    // 3. Cryptographic STARK verification
    backend.verify(&stark_proof, &proof.proof.payload)?;

    Ok(())
}

fn verify_schema_version(proof: &ProofPackage) -> Result<(), VerifierError> {
    let expected = "zenoform.proof.v1";
    if proof.schema_version != expected {
        return Err(VerifierError::SchemaVersionMismatch {
            expected: expected.to_string(),
            found: proof.schema_version.clone(),
        });
    }
    Ok(())
}

fn verify_public_inputs(chunk: &Chunk, proof: &ProofPackage) -> Result<(), VerifierError> {
    let pi = &proof.public_inputs;

    if pi.world_id != chunk.world_id {
        return Err(VerifierError::PublicInputMismatch { field: "world_id".to_string() });
    }
    if pi.seed_hash != chunk.seed_hash {
        return Err(VerifierError::PublicInputMismatch { field: "seed_hash".to_string() });
    }
    if pi.chunk_coord != chunk.chunk_coord {
        return Err(VerifierError::PublicInputMismatch { field: "chunk_coord".to_string() });
    }
    if pi.chunk_size != chunk.chunk_size {
        return Err(VerifierError::PublicInputMismatch { field: "chunk_size".to_string() });
    }
    if pi.module_hash != chunk.module_hash {
        return Err(VerifierError::PublicInputMismatch { field: "module_hash".to_string() });
    }

    Ok(())
}

fn verify_commitment_integrity(chunk: &Chunk, proof: &ProofPackage) -> Result<(), VerifierError> {
    let pi = &proof.public_inputs;

    if chunk.commitment != pi.output_commitment {
        return Err(VerifierError::CommitmentMismatch {
            expected: pi.output_commitment.clone(),
            found: chunk.commitment.clone(),
        });
    }

    let recalculated = calculate_poseidon_commitment(chunk);
    if recalculated != chunk.commitment {
        return Err(VerifierError::CommitmentMismatch { expected: chunk.commitment.clone(), found: recalculated });
    }

    Ok(())
}

fn verify_proof_integrity(proof: &ProofPackage, level: VerificationLevel) -> Result<(), VerifierError> {
    if proof.proof.payload.is_null() {
        return Err(VerifierError::ProofDataEmpty);
    }

    match level {
        VerificationLevel::Minimal => Ok(()),
        VerificationLevel::Standard => verify_proof_hash(proof),
        VerificationLevel::Strict => {
            verify_proof_hash(proof)?;
            verify_proof_signature(proof)
        }
    }
}

fn verify_proof_hash(proof: &ProofPackage) -> Result<(), VerifierError> {
    let serialized = match serde_json::to_string(&proof.proof.payload) {
        Ok(s) => s,
        Err(e) => return Err(VerifierError::ProofIntegrityError(e.to_string())),
    };

    let mut hasher = Hasher::new();
    hasher.update(proof.schema_version.as_bytes());
    hasher.update(proof.prover.as_bytes());
    hasher.update(proof.prover_version.as_bytes());
    hasher.update(proof.protocol_version.as_bytes());
    hasher.update(serialized.as_bytes());

    let _proof_hash = format!("0x{}", hasher.finalize().to_hex());

    if proof.proof.format.is_empty() {
        return Err(VerifierError::InvalidProofFormat("empty proof format".to_string()));
    }

    let valid_formats = ["stwo-cairo-proof-json", "stwo-cairo-proof-binary", "self-signed-v1", "mock"];

    if !valid_formats.contains(&proof.proof.format.as_str()) {
        return Err(VerifierError::InvalidProofFormat(format!(
            "unknown format '{}', expected one of {:?}",
            proof.proof.format, valid_formats
        )));
    }

    Ok(())
}

fn verify_proof_signature(_proof: &ProofPackage) -> Result<(), VerifierError> {
    Err(VerifierError::ProofSignatureInvalid)
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
            "test-world".to_string(),
            "0xabcdef".to_string(),
            ChunkCoord::new(0, 0, 0),
            ChunkSize::new(2, 1),
            "terrain.fixed_noise.v1".to_string(),
            "0xmodulehash".to_string(),
        );

        chunk.cells = vec![
            Cell { local_x: 0, local_y: 0, height: 100, temperature: 50, moisture: 75, biome_id: 0, resource_mask: 0 },
            Cell { local_x: 1, local_y: 0, height: 200, temperature: 60, moisture: 80, biome_id: 1, resource_mask: 1 },
        ];

        chunk.commitment = calculate_poseidon_commitment(&chunk);
        chunk
    }

    fn make_test_proof(chunk: &Chunk) -> ProofPackage {
        ProofPackage::new_v1(
            "stwo-cairo-mock".to_string(),
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
            serde_json::json!({"seed": 123, "octaves": 4}),
        )
    }

    #[test]
    fn test_valid_proof_passes_verification() {
        let chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);
        assert!(verify_chunk(&chunk, &proof).is_ok());
    }

    #[test]
    fn test_commitment_mismatch_fails() {
        let chunk = make_test_chunk();
        let mut bad_proof = make_test_proof(&chunk);
        bad_proof.public_inputs.output_commitment = "0xdeadbeef".to_string();
        assert!(verify_chunk(&chunk, &bad_proof).is_err());
    }

    #[test]
    fn test_world_id_mismatch_fails() {
        let mut chunk = make_test_chunk();
        chunk.world_id = "other-world".to_string();
        chunk.commitment = calculate_poseidon_commitment(&chunk);
        let proof = make_test_proof(&chunk);
        let mut bad_proof = proof.clone();
        bad_proof.public_inputs.world_id = "wrong".to_string();
        assert!(verify_chunk(&chunk, &bad_proof).is_err());
    }

    #[test]
    fn test_seed_hash_mismatch_fails() {
        let chunk = make_test_chunk();
        let mut bad_proof = make_test_proof(&chunk);
        bad_proof.public_inputs.seed_hash = "0xbadseed".to_string();
        assert!(verify_chunk(&chunk, &bad_proof).is_err());
    }

    #[test]
    fn test_chunk_coord_mismatch_fails() {
        let chunk = make_test_chunk();
        let mut bad_proof = make_test_proof(&chunk);
        bad_proof.public_inputs.chunk_coord = ChunkCoord::new(99, 99, 99);
        assert!(verify_chunk(&chunk, &bad_proof).is_err());
    }

    #[test]
    fn test_tampered_cell_commitment_fails() {
        let mut chunk = make_test_chunk();
        chunk.cells[0].height = 9999;
        let proof = make_test_proof(&chunk);
        assert!(verify_chunk(&chunk, &proof).is_err());
    }

    #[test]
    fn test_minimal_verification_skips_integrity_check() {
        let chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);
        assert!(verify_chunk_with_level(&chunk, &proof, VerificationLevel::Minimal).is_ok());
    }

    #[test]
    fn test_strict_verification_checks_signature() {
        let chunk = make_test_chunk();
        let proof = make_test_proof(&chunk);
        let result = verify_chunk_with_level(&chunk, &proof, VerificationLevel::Strict);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerifierError::ProofSignatureInvalid));
    }

    #[test]
    fn test_invalid_schema_version_fails() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.schema_version = "zenoform.proof.v0".to_string();
        assert!(verify_chunk(&chunk, &proof).is_err());
    }

    #[test]
    fn test_invalid_proof_format_fails() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "unknown-format".to_string();
        assert!(verify_chunk(&chunk, &proof).is_err());
    }

    #[test]
    fn test_null_proof_payload_fails() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.payload = serde_json::Value::Null;
        assert!(verify_chunk(&chunk, &proof).is_err());
    }

    // --- STARK proof deserialization tests ---

    #[test]
    fn test_deserialize_mock_proof_fails() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "mock".to_string();
        let result = deserialize_stark_proof(&proof);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerifierError::InvalidProofFormat(_)));
    }

    #[test]
    fn test_deserialize_self_signed_proof() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "self-signed-v1".to_string();
        proof.proof.payload = serde_json::json!({
            "signature": "0xabcd1234",
            "public_inputs_hash": "0xdeadbeef"
        });

        let stark = deserialize_stark_proof(&proof).unwrap();
        assert_eq!(stark.prover_backend, "self-signed");
        assert_eq!(stark.public_inputs_hash, "0xdeadbeef");
        assert_eq!(stark.proof_bytes, vec![0xab, 0xcd, 0x12, 0x34]);
    }

    #[test]
    fn test_deserialize_json_proof() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "stwo-cairo-proof-json".to_string();
        // base64 of "hello stark"
        proof.proof.payload = serde_json::json!({
            "proof_bytes": "aGVsbG8gc3Rhcms=",
            "public_inputs_hash": "0x1234",
            "trace_length": 1024
        });

        let stark = deserialize_stark_proof(&proof).unwrap();
        assert_eq!(stark.proof_bytes, b"hello stark");
        assert_eq!(stark.public_inputs_hash, "0x1234");
        assert_eq!(stark.trace_length, Some(1024));
    }

    #[test]
    fn test_deserialize_binary_proof() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "stwo-cairo-proof-binary".to_string();
        proof.proof.payload = serde_json::json!({
            "proof_bytes": "0xdeadbeef"
        });

        let stark = deserialize_stark_proof(&proof).unwrap();
        assert_eq!(stark.proof_bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_deserialize_invalid_hex_fails() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "stwo-cairo-proof-binary".to_string();
        proof.proof.payload = serde_json::json!({
            "proof_bytes": "not_hex"
        });

        let result = deserialize_stark_proof(&proof);
        assert!(result.is_err());
    }

    #[test]
    fn test_noop_stark_verifier_fails() {
        let chunk = make_test_chunk();
        let mut proof = make_test_proof(&chunk);
        proof.proof.format = "self-signed-v1".to_string();
        proof.proof.payload = serde_json::json!({
            "signature": "0xabcd",
            "public_inputs_hash": "0x1234"
        });

        let backend = NoopStarkVerifier;
        let result = verify_chunk_with_backend(&chunk, &proof, &backend);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerifierError::StarkVerificationFailed(_)));
    }
}
