# Zenoform Protocol Integration Guide

This guide explains how to integrate Zenoform's verifiable procedural generation into your game engine or simulation.

## Core Concepts

1. **Determinism**: Every generation module must produce the exact same output given the same seed and coordinate.
2. **Commitment**: Chunks are committed to a hash (Poseidon for proof-friendliness, BLAKE3 for tooling).
3. **Verification**: A chunk is only "valid" if its data matches the commitment verified by a STARK proof.

## Integration Paths

### 1. Godot 4 (Recommended)
Use the Zenoform GDExtension.
- Copy the compiled library to your project.
- Use `ZenoformNode` to generate and verify chunks.

### 2. C ABI (Unreal / Unity / C++)
The `zenoform-ffi` library exports a stable C interface:

```c
bool zenoform_verify_chunk(const char* chunk_json, const char* proof_json);
```

### 3. Rust (Native)
Add `zenoform-verifier` to your `Cargo.toml`:

```toml
[dependencies]
zenoform-verifier = { git = "https://github.com/kdsmith18542/zenoform" }
```

## Verification Workflow

1. **Client** requests a chunk at `(x, y, z)`.
2. **Prover** (or high-trust server) generates the chunk and a STARK proof.
3. **Client** receives the chunk and the `ProofPackage`.
4. **Client** calls `verify_chunk(chunk, proof)`.
5. If `true`, the chunk is rendered. If `false`, the chunk is rejected as tampered.

## Security Considerations

- Always verify the `module_hash` against an approved list.
- Ensure the `seed_hash` is globally committed for the session.
- Coordinate bindings prevent "proof replay" attacks.
