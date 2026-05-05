# Zenoform Protocol v1.0 — Revised Technical Specification

**Project Title:** Zenoform Protocol  
**Category:** Verifiable Procedural Generation (VPG) Engine  
**Document Type:** Developer Technical Specification / Architecture Blueprint  
**Revision:** v1.0-revised  
**Date:** May 2026  
**Primary Goal:** Build a practical proof layer for deterministic procedural world generation.

---

## 1. Executive Summary

Zenoform is a **verifiable procedural generation protocol** for games, simulations, and persistent digital worlds.

The core idea is simple:

> Generate massive worlds locally, but prove that each generated chunk followed the official generation rules.

Instead of trusting a server, client, mod, or world host to honestly place terrain, resources, structures, or spawn tables, Zenoform makes world generation deterministic and cryptographically verifiable.

The first version should **not** attempt to prove an entire world or arbitrary procedural assets. The first version should prove a bounded unit:

> **Given a world seed, protocol version, module hash, and chunk coordinate, prove that a small terrain chunk was generated according to the canonical rules.**

The recommended MVP is a **Rust + Cairo/S-two + Mojo + Godot** proof-of-concept:

- Rust hosts the CLI, schemas, test vectors, hashing, FFI, and verifier integration.
- Cairo defines the canonical proof implementation.
- S-two / stwo-cairo proves Cairo executions.
- Mojo accelerates local generation and procedural preview paths.
- Godot demonstrates verified chunks visually.

---

## 2. Revised One-Sentence Pitch

**Zenoform lets games generate procedural worlds locally while proving that every accepted chunk follows the official world-generation protocol.**

---

## 3. Product Positioning

Avoid positioning Zenoform as a full “zero-knowledge game engine” in v1. That phrase is too broad and will invite unrealistic expectations.

Use this instead:

> **Zenoform is a verifiable procedural generation protocol for games and simulations.**

Alternative taglines:

- **Generate locally. Verify globally.**
- **Trustless terrain for infinite worlds.**
- **Procedural generation with receipts.**
- **A proof layer for procedural worlds.**
- **Minecraft-style seeds, but cryptographically enforceable.**

---

## 4. Problem Statement

Procedural worlds are usually trusted, not verified.

A dishonest client, private server, modded host, or world authority can manipulate:

- terrain height
- biome placement
- ore/resource locations
- rare item spawns
- structure placement
- dungeon layouts
- world events
- loot generation
- AI-generated environment claims

This is especially important for:

- multiplayer survival games
- sandbox MMOs
- on-chain games
- modded servers
- player-hosted worlds
- procedural roguelikes
- deterministic simulation games
- AI-assisted worldbuilding tools

Zenoform addresses this by making world generation:

1. **Deterministic**
2. **Versioned**
3. **Replayable**
4. **Hash-committed**
5. **Provable**
6. **Verifier-friendly**

---

## 5. Core Design Rule

> **The world is not stored. The world is derived. The derivation is proven.**

A valid chunk is derived from:

```text
world_seed
protocol_version
module_id
module_hash
chunk_coordinate
chunk_size
generation_parameters
```

The output is accepted only if:

```text
proof is valid
output commitment matches the supplied chunk data
module hash is approved
protocol version is supported
public inputs match the requested world/chunk
```

---

## 6. Major Architectural Correction

The original plan implied:

```text
Mojo generates terrain → Cairo/S-two proves what Mojo did
```

That is too optimistic and not clean enough.

The revised architecture should be:

```text
Canonical generation spec / DSL
        ↓
Cairo proof implementation
        ↓
S-two / stwo-cairo proof
        ↓
Rust verifier + protocol tooling

Optional fast path:
Canonical generation spec / DSL
        ↓
Mojo optimized implementation
        ↓
Local preview / game runtime generation
```

### Key Principle

**Cairo/S-two is the proof authority. Mojo is the accelerator.**

Mojo can generate fast local previews, batch chunks, and run high-performance kernels, but the protocol must not trust Mojo output unless it matches canonical test vectors and the proof validates the commitment.

---

## 7. Research Notes and Current Stack Reality

### 7.1 Mojo

Mojo is publicly positioned by Modular as a language that unifies high-level AI development with low-level systems programming and targets CPUs and GPUs without vendor lock-in. Modular’s current Mojo page shows GPU programming, Python interop, metaprogramming, SIMD vectorization, and hardware dispatch examples.

**Implication for Zenoform:** Mojo is a strong choice for fast procedural kernels, but it should not be the sole canonical source of truth for proof verification.

Source: https://www.modular.com/open-source/mojo

### 7.2 S-two / stwo-cairo

S-two is StarkWare’s Circle STARK-oriented prover framework. The stwo repository describes features such as Circle STARKs and SIMD-optimized high performance. The stwo-cairo repository focuses on proving Cairo program executions using S-two.

**Implication for Zenoform:** Cairo + stwo-cairo is the right proof path for the initial prototype.

Sources:

- https://github.com/starkware-libs/stwo
- https://github.com/starkware-libs/stwo-cairo
- https://starkware.co/blog/s-two-prover/

### 7.3 Scarb Proving Integration

Scarb integrates `scarb prove` and `scarb verify`, but current documentation warns that proof soundness is not yet guaranteed by S-two, that the prover is not available on Windows, and that `stwo-cairo` can be significantly slower through Scarb. The docs recommend direct `stwo-cairo` usage for production use.

**Implication for Zenoform:** Use Scarb for early developer convenience, but benchmark and prepare direct `stwo-cairo` integration for serious prototypes.

Source: https://docs.swmansion.com/scarb/docs/extensions/prove-and-verify.html

### 7.4 Important Practical Correction

Any claim like “client-side proving in seconds for complex real-time terrain” should be treated as an **R&D target**, not a guaranteed v1 feature.

The v1 target should be:

> Prove small deterministic chunks and benchmark them honestly.

---

## 8. Recommended High-Level Architecture

Zenoform should be split into five major layers.

```text
┌──────────────────────────────────────────────────────────────┐
│ Game / Simulation / Chain / Peer-to-Peer Verifier             │
└──────────────────────────────────────────────────────────────┘
                         ▲
                         │ proof + chunk commitment
                         │
┌──────────────────────────────────────────────────────────────┐
│ Zenoform Verifier Layer                                      │
│ Rust verifier, proof schema, commitment checks, FFI           │
└──────────────────────────────────────────────────────────────┘
                         ▲
                         │
┌──────────────────────────────────────────────────────────────┐
│ Proving Layer                                                │
│ Cairo generation program + stwo-cairo prover                  │
└──────────────────────────────────────────────────────────────┘
                         ▲
                         │
┌──────────────────────────────────────────────────────────────┐
│ Canonical Spec / DSL Layer                                   │
│ deterministic math, fixed-point rules, module hashes          │
└──────────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────────┐
│ Runtime Acceleration Layer                                   │
│ Mojo kernels, Rust reference generator, engine adapters        │
└──────────────────────────────────────────────────────────────┘
```

---

## 9. Technology Stack

### 9.1 Core Host Language: Rust

Use Rust for:

- CLI tools
- schemas
- test vectors
- hashing
- proof verification orchestration
- FFI boundary
- engine binding foundation
- benchmark harness
- deterministic reference implementation

Why Rust:

- stable systems language
- strong crypto ecosystem
- good CLI tooling
- strong FFI support
- mature testing/benchmarking
- compatible with the Rust-heavy S-two ecosystem

### 9.2 Proof Language: Cairo

Use Cairo for:

- canonical provable generation logic
- proof public input handling
- output commitment calculation or validation
- proof-specific terrain/resource modules

### 9.3 Prover: S-two / stwo-cairo

Use S-two / stwo-cairo for:

- proving Cairo execution
- verifying Cairo proof output
- recursive proof experiments in later versions

### 9.4 Performance Language: Mojo

Use Mojo for:

- SIMD procedural kernels
- GPU previews
- editor-time terrain generation
- batch generation
- procedural experiments
- non-canonical acceleration paths

Do **not** make Mojo the only canonical source of truth in v1.

### 9.5 First Game Engine Target: Godot 4

Use Godot first because:

- open-source
- easier extension workflow
- good fit for a public visual demo
- easier than Unreal/Unity for early iteration
- aligns with other procedural/open-world project goals

Later targets:

1. Godot 4
2. Unreal
3. Unity
4. Bevy
5. Web/WASM demo

---

## 10. Canonical Math Requirements

This is the most important implementation discipline.

### 10.1 Avoid Floating-Point Canonical Math

Do not make floating-point GPU math part of the canonical protocol in v1.

Reason:

- GPU floating-point behavior can vary by vendor, driver, compiler, and optimization level.
- STARK circuits prefer exact arithmetic.
- Proof consistency requires deterministic integer/fixed-point operations.

### 10.2 Use Fixed-Point Arithmetic

Recommended v1 format:

```text
u16 or u32 for height/moisture/temperature
i32 for intermediate fixed-point values
Q16.16 or Q24.8 for internal math
u8/u16 for biome/resource IDs
```

Example:

```text
height_raw: u16
moisture_raw: u16
temperature_raw: u16
biome_id: u8
resource_mask: u16
```

### 10.3 Use Proof-Friendly Noise

Avoid proving classic floating-point Perlin/Simplex in v1.

Recommended v1 noise options:

- hash-grid noise
- value noise
- cellular noise
- fixed-point gradient noise
- low-octave fractal noise
- fixed-point domain warp

### 10.4 Hash Function Selection

The hash function must be chosen carefully.

Options:

- Poseidon/Poseidon2 for proof-friendliness
- Pedersen if aligned with Cairo/Starknet ecosystem needs
- BLAKE3 for non-proof-side file commitments and tooling
- Keccak only when EVM compatibility is required

Recommended v1:

```text
Proof-side hash: Poseidon/Poseidon2-compatible path
Tooling/file hash: BLAKE3
Protocol commitment: explicitly versioned hash scheme
```

Do not mix hash functions without tagging them in the schema.

---

## 11. World Model

### 11.1 Stateless World Model

Zenoform’s base model is stateless:

```text
chunk = generate(seed, protocol_version, module_hash, coordinate)
```

The world does not need to store every chunk. It only needs:

- seed hash
- module registry
- protocol version
- accepted proof/commitment records where needed

### 11.2 Stateful Extension Model

Later versions may support stateful modifications:

- player-built structures
- mined resources
- destroyed terrain
- claimed land
- faction changes
- time-based world events

But v1 should not attempt this.

For future versions, stateful data should be layered on top:

```text
canonical_generated_chunk
        +
state_delta_log
        +
state_transition_proofs
```

---

## 12. Minimal Provable Unit

The smallest useful v1 unit should be a chunk.

Recommended v1 chunk sizes to test:

```text
16x16 cells
32x32 cells
64x64 cells
```

Start with:

```text
16x16 canonical proof target
32x32 benchmark target
64x64 stretch target
```

Each cell should contain:

```text
height: u16
temperature: u16
moisture: u16
biome_id: u8
resource_mask: u16
```

Optional later fields:

```text
slope: u8
water_level: u16
structure_anchor: u8
spawn_class: u8
rarity_tier: u8
```

---

## 13. MVP Definition

The MVP should be:

> A CLI and Godot demo that generates, proves, verifies, displays, and rejects tampered deterministic terrain chunks.

### MVP Features

- Generate chunk from seed and coordinate
- Commit chunk to a hash
- Produce witness/public inputs
- Prove the Cairo generation execution
- Verify proof
- Load verified chunk in Godot
- Display invalid/tampered chunks differently
- Benchmark generation/proving/verification
- Emit stable test vectors

### MVP Non-Goals

Do not include these in v1:

- full erosion simulation
- rivers
- mesh proofing
- voxel edits
- NPC AI proofing
- full dungeon layout proofing
- player state transitions
- on-chain verifier contract
- multiplayer networking
- full Unity/Unreal support
- AI-generated asset proofing

---

## 14. End-to-End Flow

### 14.1 Request

```json
{
  "world_id": "frontier-demo",
  "protocol_version": "zenoform-terrain-v1",
  "seed_hash": "0x...",
  "chunk_coord": [12, -4, 0],
  "chunk_size": [16, 16],
  "module_id": "terrain.fixed_noise.v1",
  "module_hash": "0x..."
}
```

### 14.2 Generate

The runtime generates:

```json
{
  "coord": [12, -4, 0],
  "size": [16, 16],
  "cells": [
    {
      "x": 0,
      "y": 0,
      "height": 421,
      "temperature": 612,
      "moisture": 288,
      "biome": 4,
      "resource_mask": 16
    }
  ],
  "commitment": "0x..."
}
```

### 14.3 Prove

The proof shows:

```text
Given seed_hash, module_hash, protocol_version, and chunk_coord,
the output commitment corresponds to a chunk generated by the canonical rules.
```

### 14.4 Verify

The verifier checks:

```text
proof validity
public inputs
module hash
protocol version
chunk commitment
coordinate binding
seed binding
```

### 14.5 Accept or Reject

If the proof and commitment match, the chunk is accepted.

If any cell is tampered with, the commitment fails.

---

## 15. Public Input Schema

```json
{
  "protocol_version": "zenoform-terrain-v1",
  "world_id": "frontier-demo",
  "seed_hash": "0x...",
  "chunk_coord": [12, -4, 0],
  "chunk_size": [16, 16],
  "module_id": "terrain.fixed_noise.v1",
  "module_hash": "0x...",
  "hash_scheme": "poseidon2-v1",
  "output_commitment": "0x..."
}
```

---

## 16. Chunk Data Schema

```json
{
  "schema_version": "zenoform.chunk.v1",
  "protocol_version": "zenoform-terrain-v1",
  "world_id": "frontier-demo",
  "seed_hash": "0x...",
  "chunk_coord": [12, -4, 0],
  "chunk_size": [16, 16],
  "module_id": "terrain.fixed_noise.v1",
  "module_hash": "0x...",
  "cells": [
    {
      "local_x": 0,
      "local_y": 0,
      "height": 421,
      "temperature": 612,
      "moisture": 288,
      "biome_id": 4,
      "resource_mask": 16
    }
  ],
  "commitment": "0x..."
}
```

---

## 17. Proof Package Schema

```json
{
  "schema_version": "zenoform.proof.v1",
  "prover": "stwo-cairo",
  "prover_version": "pinned-version-here",
  "protocol_version": "zenoform-terrain-v1",
  "public_inputs": {
    "world_id": "frontier-demo",
    "seed_hash": "0x...",
    "chunk_coord": [12, -4, 0],
    "chunk_size": [16, 16],
    "module_hash": "0x...",
    "output_commitment": "0x..."
  },
  "proof": {
    "format": "stwo-cairo-proof-json",
    "payload": {}
  }
}
```

---

## 18. Recommended Repository Structure

```text
zenoform/
  README.md
  SPEC.md
  LICENSE
  CHANGELOG.md

  crates/
    zenoform-core/
      src/
        lib.rs
        coord.rs
        seed.rs
        chunk.rs
        commitment.rs
        module.rs
        fixed.rs
        errors.rs
    zenoform-cli/
      src/
        main.rs
        commands/
          generate.rs
          prove.rs
          verify.rs
          tamper.rs
          bench.rs
    zenoform-verifier/
      src/
        lib.rs
        proof_package.rs
        stwo_bridge.rs
    zenoform-ffi/
      src/
        lib.rs
        c_api.rs

  cairo/
    terrain_v1/
      Scarb.toml
      src/
        lib.cairo
        fixed.cairo
        hash_noise.cairo
        biome.cairo
        resource.cairo
        commitment.cairo
      tests/

  mojo/
    zenoform_kernels/
      terrain.mojo
      noise.mojo
      biome.mojo
      resource.mojo
      benches/

  dsl/
    grammar/
      zenoform.ebnf
    compiler/
      src/
    examples/
      terrain_v1.zf

  bindings/
    godot/
      README.md
      addon/
    unity/
      README.md
    unreal/
      README.md

  examples/
    cli_basic/
    terrain_viewer/
    godot_verified_chunks/

  test_vectors/
    terrain_v1/
      seed_001_chunk_0_0.json
      seed_001_chunk_1_0.json
      expected_commitments.json

  benches/
    proving/
    generation/
    verification/

  docs/
    architecture.md
    threat_model.md
    protocol.md
    math.md
    engine_integration.md
    proving_notes.md
```

---

## 19. CLI Design

The CLI should be the first usable product.

### 19.1 Generate

```bash
zenoform generate \
  --seed "demo-seed-001" \
  --world frontier-demo \
  --module terrain.fixed_noise.v1 \
  --chunk 12,-4,0 \
  --size 16x16 \
  --out chunk.json
```

### 19.2 Prove

```bash
zenoform prove \
  --chunk chunk.json \
  --out proof.json
```

### 19.3 Verify

```bash
zenoform verify \
  --chunk chunk.json \
  --proof proof.json
```

### 19.4 Tamper Test

```bash
zenoform tamper \
  --chunk chunk.json \
  --cell 3,7 \
  --field resource_mask \
  --value 65535 \
  --out tampered_chunk.json
```

### 19.5 Benchmark

```bash
zenoform bench \
  --seed "demo-seed-001" \
  --module terrain.fixed_noise.v1 \
  --sizes 16x16,32x32,64x64 \
  --runs 10
```

---

## 20. DSL Recommendation

A Zenoform DSL is strongly recommended.

Without a DSL, the team will manually maintain equivalent logic in:

- Rust
- Cairo
- Mojo
- engine plugins
- test vectors

That creates drift.

### 20.1 DSL Goal

Define procedural modules once, then generate:

- Cairo proof code
- Mojo generation code
- Rust reference code
- JSON schemas
- test vectors
- module hashes

### 20.2 Example DSL Sketch

```text
module terrain.fixed_noise.v1

input:
  seed: Field
  chunk_x: i32
  chunk_y: i32
  chunk_z: i32

cell:
  height = noise2d(seed, world_x, world_y, octaves=4)
  moisture = noise2d(seed + 1, world_x, world_y, octaves=3)
  temperature = noise2d(seed + 2, world_x, world_y, octaves=3)
  biome_id = biome(height, moisture, temperature)
  resource_mask = resource(seed, world_x, world_y, biome_id, height)

output:
  height: u16
  moisture: u16
  temperature: u16
  biome_id: u8
  resource_mask: u16
```

### 20.3 DSL Restrictions

The DSL should enforce:

- bounded loops only
- no arbitrary recursion
- fixed-point arithmetic only
- deterministic hash functions only
- explicit integer widths
- versioned functions
- no implicit randomness
- no engine-specific functions
- no file/network access

---

## 21. Threat Model

### 21.1 Dishonest Client

**Attack:** Client submits a fake chunk with better resources.

**Defense:** Verifier checks proof and output commitment.

### 21.2 Dishonest Server

**Attack:** Server gives favored players better terrain, spawns, or ore nodes.

**Defense:** Chunk must derive from public seed, module hash, and coordinate.

### 21.3 Proof Replay

**Attack:** Valid proof from one coordinate is reused for another coordinate.

**Defense:** Public inputs include coordinate, world ID, seed hash, module hash, and protocol version.

### 21.4 Version Drift

**Attack:** Different clients generate different results.

**Defense:** Protocol versioning, module hashes, fixed-point math, and test vectors.

### 21.5 GPU Nondeterminism

**Attack:** GPU floating-point output differs across hardware.

**Defense:** Canonical path uses fixed-point arithmetic. Mojo GPU path is acceleration/preview only unless matched to canonical vectors.

### 21.6 Malicious Mods

**Attack:** A mod changes world rules but pretends to be canonical.

**Defense:** Every module has a hash. Verifiers accept only approved module hashes.

### 21.7 Seed Manipulation

**Attack:** Host changes seed after seeing world output.

**Defense:** Seed hash is committed before generation. Optional future version can use commit-reveal for multiplayer fairness.

### 21.8 Selective Disclosure

**Attack:** Host only reveals chunks with favorable outcomes.

**Defense:** For competitive games, require public seed and deterministic chunk access rules. Future versions may add challenge-response audits.

---

## 22. Versioning and Module Registry

Every module must be identified by:

```text
module_id
module_version
module_hash
source_hash
compiler_version
proof_backend_version
hash_scheme
```

Example:

```json
{
  "module_id": "terrain.fixed_noise.v1",
  "module_version": "1.0.0",
  "module_hash": "0x...",
  "source_hash": "0x...",
  "compiler_version": "zenoform-dslc-0.1.0",
  "proof_backend": "stwo-cairo",
  "proof_backend_version": "pinned-version",
  "hash_scheme": "poseidon2-v1"
}
```

---

## 23. Engine Integration Plan

### 23.1 Godot v1

Godot should receive:

- generated chunk data
- proof status
- verifier result
- visual chunk mesh or tilemap
- debug overlay

Demo behavior:

- green overlay = verified chunk
- yellow overlay = pending proof
- red overlay = invalid/tampered chunk
- blue overlay = local preview not yet proven

### 23.2 Unity Later

Use C ABI or C# wrapper.

### 23.3 Unreal Later

Use C ABI or Rust static library with C++ wrapper.

### 23.4 Web Later

Use WASM verifier for proof checking and chunk display.

---

## 24. Visual Demo Requirements

The MVP demo should make the value obvious.

Required UI:

- seed input
- coordinate input
- generate button
- prove button
- verify button
- tamper button
- proof status panel
- chunk commitment display
- timing metrics
- terrain visualization
- resource overlay
- biome overlay

Visual states:

```text
Verified: green border
Pending: yellow border
Tampered: red border
Preview-only: blue border
Unsupported module: gray border
```

---

## 25. Benchmarks

Track these from day one.

### 25.1 Generation Metrics

```text
chunk size
cells generated
Rust generation time
Mojo CPU generation time
Mojo GPU generation time
memory usage
```

### 25.2 Proving Metrics

```text
chunk size
constraint count
witness generation time
proof generation time
peak memory usage
proof size
```

### 25.3 Verification Metrics

```text
proof verification time
public input parsing time
commitment check time
memory usage
```

### 25.4 Suggested Benchmark Table

```text
| Chunk Size | Cells | Rust Gen | Mojo Gen | Proof Time | Verify Time | Proof Size |
|------------|-------|----------|----------|------------|-------------|------------|
| 16x16      | 256   | TBD      | TBD      | TBD        | TBD         | TBD        |
| 32x32      | 1024  | TBD      | TBD      | TBD        | TBD         | TBD        |
| 64x64      | 4096  | TBD      | TBD      | TBD        | TBD         | TBD        |
```

---

## 26. Roadmap

### Phase 0 — Protocol Design

**Goal:** Define the smallest provable unit.

Deliverables:

- coordinate system
- seed format
- hash scheme
- fixed-point numeric format
- chunk size
- public input schema
- chunk schema
- proof package schema
- threat model
- module registry format

Exit criteria:

```text
All schemas are documented.
Test vectors can be described before implementation.
The team agrees on fixed-point rules.
```

---

### Phase 1 — Rust Reference Generator

**Goal:** Build deterministic terrain without proofs.

Deliverables:

- `zenoform-core`
- fixed-point math module
- hash-grid noise
- biome classification
- resource placement
- chunk commitment
- CLI `generate`
- test vectors

Exit criteria:

```text
Same seed + coordinate always produces same chunk.
Commitment changes when any cell changes.
Test vectors are stable.
```

---

### Phase 2 — Cairo Canonical Generator

**Goal:** Port the deterministic terrain rules into Cairo.

Deliverables:

- Cairo terrain module
- fixed-point Cairo helpers
- Cairo hash/noise implementation
- output commitment
- Cairo tests
- Rust/Cairo output comparison

Exit criteria:

```text
Rust and Cairo produce matching commitments.
Cairo execution works through Scarb.
Small chunks can be executed.
```

---

### Phase 3 — S-two Proof Prototype

**Goal:** Prove one chunk.

Deliverables:

- proof generation path
- proof package schema
- CLI `prove`
- CLI `verify`
- benchmark report
- tamper rejection demo

Exit criteria:

```text
16x16 chunk proof succeeds.
Verification succeeds for valid chunk.
Verification fails for tampered chunk.
Proof metrics are recorded.
```

---

### Phase 4 — Mojo Acceleration

**Goal:** Add high-performance generation path.

Deliverables:

- Mojo CPU kernel
- Mojo GPU experiment
- Rust/Mojo comparison
- deterministic compatibility tests
- generation benchmark

Exit criteria:

```text
Mojo path matches canonical test vectors.
Mojo path is faster for generation workloads.
Any nondeterminism is documented.
```

---

### Phase 5 — Godot Demo

**Goal:** Make the project visually understandable.

Deliverables:

- Godot addon prototype
- terrain viewer
- proof status overlay
- tamper button
- benchmark UI
- resource/biome overlay

Exit criteria:

```text
User can generate, prove, verify, tamper, and see rejection visually.
```

---

### Phase 6 — DSL Prototype

**Goal:** Reduce implementation drift.

Deliverables:

- DSL grammar
- parser
- module hashing
- Rust codegen
- Cairo codegen
- Mojo codegen experiment
- generated test vectors

Exit criteria:

```text
One DSL module can generate matching Rust/Cairo outputs.
Module hash is stable.
Generated code passes test vectors.
```

---

### Phase 7 — External Integration

**Goal:** Prepare for real projects.

Deliverables:

- C ABI
- Godot stable addon
- verifier library docs
- example integration guide
- CI release builds
- basic WASM verifier experiment

Exit criteria:

```text
A separate game project can import and verify Zenoform chunks.
```

---

## 27. Risk Register

### Risk: Proving Cost Too High

Mitigation:

- keep chunks small
- use fixed-point simple noise
- benchmark early
- prove commitments, not full meshes
- use recursion later
- avoid expensive procedural features in v1

### Risk: Mojo/Cairo Drift

Mitigation:

- Rust reference implementation
- test vectors
- shared DSL
- module hashes
- CI cross-checks

### Risk: GPU Nondeterminism

Mitigation:

- canonical fixed-point path
- GPU preview only
- deterministic compatibility tests
- optional CPU-only canonical generator

### Risk: S-two Tooling Changes

Mitigation:

- pin versions
- isolate prover integration behind Rust adapter
- support Scarb convenience path and direct stwo-cairo path
- maintain proof package schema independent of backend

### Risk: Over-Scoping

Mitigation:

- only prove terrain v1
- no stateful worlds in v1
- no full engine support in v1
- no on-chain verifier in v1
- no erosion/rivers in v1

### Risk: Poor Developer Ergonomics

Mitigation:

- CLI first
- simple JSON schemas
- visual Godot demo
- test vectors
- generated examples
- clean docs

---

## 28. “Do Not Build Yet” List

Do not build these until the proof prototype works:

- full world editor
- full Unity support
- full Unreal support
- multiplayer server
- on-chain verifier
- NFT/asset marketplace
- AI asset generation
- proof marketplace
- recursive proof aggregation
- chunk streaming network
- player state proof system

---

## 29. “Build First” List

Build these first:

1. Fixed-point terrain math
2. Chunk schema
3. Commitment function
4. Rust CLI generator
5. Cairo equivalent generator
6. Rust/Cairo test vectors
7. S-two proof for 16x16 chunk
8. CLI verifier
9. Tamper rejection
10. Godot visual demo

---

## 30. Strategic Project Fit

Zenoform is a better long-term R&D project than a generic blockchain or feature-heavy coin project because it combines:

- procedural generation
- cryptographic verification
- game infrastructure
- deterministic simulation
- Rust systems work
- Cairo/STARK proving
- Mojo performance work
- engine integration
- agent-friendly code generation

Recommended workload split while other major projects are still active:

```text
70% main stable project
20% Zenoform R&D
10% maintenance / cleanup / portfolio
```

Once the first proof prototype works, Zenoform can become a primary focus.

---

## 31. Final Technical Verdict

Zenoform is feasible if scoped correctly.

The correct v1 is **not**:

```text
A full zero-knowledge game engine proving Texas-sized worlds in real time.
```

The correct v1 is:

```text
A verifiable deterministic terrain-chunk protocol proving that seed + coordinate + module rules produce a committed chunk.
```

The most important architecture decision is:

> **Use Cairo/S-two as the canonical proof path, Rust as the protocol host, Mojo as the accelerator, and a DSL to prevent implementation drift.**

If this project succeeds at even the MVP level, it becomes a serious foundation for:

- trustless procedural survival games
- modded MMO world verification
- on-chain/off-chain hybrid game worlds
- fair competitive map generation
- persistent world audit tools
- AI-assisted but deterministic game content pipelines

---

## 32. Immediate Next Steps for Developers

### Week 1

- Create repo structure.
- Define chunk schema.
- Define fixed-point format.
- Implement Rust seed/coordinate/chunk structs.
- Implement first hash-grid noise.
- Generate first test vectors.

### Week 2

- Implement biome/resource rules.
- Add chunk commitment.
- Add CLI `generate`.
- Add CLI `tamper`.
- Add CLI `verify-commitment`.

### Week 3

- Port terrain rules to Cairo.
- Compare Rust and Cairo commitments.
- Add Cairo tests.
- Produce first Cairo execution trace.

### Week 4

- Integrate Scarb proving path.
- Generate first proof.
- Verify first proof.
- Benchmark 16x16 chunks.
- Document proof limits.

### Week 5+

- Add direct stwo-cairo integration.
- Add Mojo generation path.
- Add Godot visualization.
- Begin DSL prototype.

---

## 33. Suggested README Opening

```markdown
# Zenoform

Zenoform is a verifiable procedural generation protocol for games and simulations.

It allows worlds to be generated locally while proving that generated chunks follow the official deterministic rules.

The first prototype verifies terrain chunks:

seed + coordinate + module hash → generated chunk → commitment → proof → verification
```

---

## 34. Appendix: Glossary

**VPG:** Verifiable Procedural Generation.  
**Chunk:** A bounded region of generated world data.  
**Commitment:** A hash representing chunk output.  
**Witness:** Data used by the prover to construct a proof.  
**Public Inputs:** Values visible to the verifier, such as seed hash, coordinate, and output commitment.  
**Module Hash:** Hash of a generation module’s canonical rules.  
**Canonical Path:** The official deterministic implementation used for proof correctness.  
**Acceleration Path:** Fast implementation used for runtime generation or preview, usually Mojo.  
**Fixed-Point:** Integer-based representation of fractional values.  
**S-two:** StarkWare’s Circle STARK-oriented prover framework.  
**stwo-cairo:** Tooling for proving Cairo program executions using S-two.  
**DSL:** Domain-specific language for defining generation rules once and compiling to multiple targets.

---

## 35. Appendix: Source Notes

The following sources informed the stack and risk recommendations:

1. Modular Mojo overview — CPU/GPU systems programming, Python interop, SIMD and hardware dispatch examples.  
   https://www.modular.com/open-source/mojo

2. StarkWare S-two prover introduction.  
   https://starkware.co/blog/s-two-prover/

3. StarkWare S-two GitHub repository — Circle STARKs, SIMD/high-performance focus, Rust workspace.  
   https://github.com/starkware-libs/stwo

4. stwo-cairo GitHub repository — proving Cairo programs with S-two; current recommendation that direct stwo-cairo usage is preferable for now.  
   https://github.com/starkware-libs/stwo-cairo

5. Scarb proving and verification documentation — `scarb prove`, `scarb verify`, warnings about soundness, Windows availability, and performance.  
   https://docs.swmansion.com/scarb/docs/extensions/prove-and-verify.html

