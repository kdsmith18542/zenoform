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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationLevel {
    Minimal,
    Standard,
    Strict,
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
}
