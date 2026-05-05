use zenoform_core::chunk::Chunk;
use zenoform_core::proof::ProofPackage;
use zenoform_core::commitment::calculate_poseidon_commitment;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerifierError {
    #[error("Invalid proof")]
    InvalidProof,
    #[error("Commitment mismatch: expected {expected}, found {found}")]
    CommitmentMismatch { expected: String, found: String },
    #[error("Public input mismatch: {field}")]
    PublicInputMismatch { field: String },
}

pub fn verify_chunk(chunk: &Chunk, proof: &ProofPackage) -> Result<(), VerifierError> {
    // 1. Verify commitment matches public inputs
    if chunk.commitment != proof.public_inputs.output_commitment {
        return Err(VerifierError::CommitmentMismatch {
            expected: proof.public_inputs.output_commitment.clone(),
            found: chunk.commitment.clone(),
        });
    }

    // 2. Verify chunk data produces the commitment
    // In a real scenario, the proof proves this.
    // For local verification without a prover, we can recalculate it.
    let recalculated = calculate_poseidon_commitment(chunk);
    if recalculated != chunk.commitment {
         return Err(VerifierError::CommitmentMismatch {
            expected: chunk.commitment.clone(),
            found: recalculated,
        });
    }

    // 3. Verify public inputs match requested chunk metadata
    if proof.public_inputs.world_id != chunk.world_id {
        return Err(VerifierError::PublicInputMismatch { field: "world_id".to_string() });
    }
    if proof.public_inputs.chunk_coord != chunk.chunk_coord {
        return Err(VerifierError::PublicInputMismatch { field: "chunk_coord".to_string() });
    }

    // 4. Verify the actual STARK proof (Mocked for Windows)
    verify_stark_proof(proof)?;

    Ok(())
}

fn verify_stark_proof(_proof: &ProofPackage) -> Result<(), VerifierError> {
    // TODO: Integrate with S-two verifier
    // For now, we assume all proofs are "valid" if they exist
    Ok(())
}
