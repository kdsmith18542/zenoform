use crate::coord::{ChunkCoord, ChunkSize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicInputs {
    pub world_id: String,
    pub seed_hash: String,
    pub chunk_coord: ChunkCoord,
    pub chunk_size: ChunkSize,
    pub module_hash: String,
    pub output_commitment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPayload {
    pub format: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPackage {
    pub schema_version: String,
    pub prover: String,
    pub prover_version: String,
    pub protocol_version: String,
    pub public_inputs: PublicInputs,
    pub proof: ProofPayload,
}

impl ProofPackage {
    pub fn new_v1(
        prover: String,
        prover_version: String,
        protocol_version: String,
        public_inputs: PublicInputs,
        proof_payload: serde_json::Value,
    ) -> Self {
        Self {
            schema_version: "zenoform.proof.v1".to_string(),
            prover,
            prover_version,
            protocol_version,
            public_inputs,
            proof: ProofPayload { format: "stwo-cairo-proof-json".to_string(), payload: proof_payload },
        }
    }
}
