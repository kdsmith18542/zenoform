# Zenoform

Zenoform is a **verifiable procedural generation protocol** for games and simulations.

It allows worlds to be generated locally while proving that every generated chunk follows the official deterministic rules.

## Core Pillars

1.  **Deterministic**: Generate massive worlds locally from a seed and coordinate.
2.  **Provable**: Every chunk comes with a cryptographic receipt (STARK proof).
3.  **Verifiable**: Clients and servers can verify world integrity without re-running the full generation.

## Technology Stack

- **Rust**: The protocol host, CLI, and engine bindings.
- **Cairo**: The canonical provable generation path using StarkWare's S-two.
- **Mojo**: High-performance SIMD kernels for real-time acceleration.
- **Godot 4**: Primary visual demonstration engine.

## Project Structure

- `crates/`: Rust workspace (Core, CLI, Verifier, FFI, DSL).
- `cairo/`: Cairo/Scarb project for proving circuits.
- `mojo/`: SIMD-optimized acceleration kernels.
- `examples/`: Godot 4 demo and CLI examples.
- `docs/`: Technical specification and integration guides.

## Getting Started

### CLI Usage
```bash
# Generate a chunk
zenoform generate --seed 123 --world frontier --chunk 0,0,0 --out chunk.json

# Produce a proof
zenoform prove --chunk chunk.json --out proof.json

# Verify integrity
zenoform verify --chunk chunk.json --proof proof.json
```

## License

This project is licensed under the **Apache License 2.0**.
