# Spec Compliance Audit

**Date:** 2026-05-05
**Scope:** `docs/protocol.md` + `docs/Zenoform_Protocol_Revised_Technical_Spec.md` vs. actual implementation
**Result:** PARTIAL — Core MVP mechanics are solid, but significant spec gaps remain.

---

## Summary Table

| Area | Status | Gap Severity |
|------|--------|-------------|
| Rust workspace (6 crates) | COMPLETE | None |
| Chunk schema | MOSTLY COMPLETE | Missing `hash_scheme` |
| Proof package schema | MOSTLY COMPLETE | Missing `hash_scheme` in public inputs |
| Cairo canonical generator | MOSTLY COMPLETE | No Rust/Cairo commitment comparison test |
| CLI (generate, prove, verify, tamper, bench) | MOSTLY COMPLETE | `tamper` is simplified; `prove` is mocked |
| Godot demo overlays | PARTIAL | Missing yellow/blue/gray states |
| WASM verifier | COMPLETE | None |
| DSL parser + codegen | MOSTLY COMPLETE | Example `.zf` has hardcoded biome/resource |
| Mojo acceleration kernels | PARTIAL | Height only; no biome/resource; no comparison tests |
| Module registry | PARTIAL | Missing version, source_hash, compiler_version, backend_version, hash_scheme |
| Test vectors | MOSTLY COMPLETE | Missing `expected_commitments.json` |
| Real S-two proving | NOT IMPLEMENTED | All proofs are mocked; no stwo-cairo integration |
| Benchmark suite | PARTIAL | No proving/verification/proof-size metrics |
| Repository structure | PARTIAL | `bindings/`, `benches/`, `dsl/compiler/` are empty |
| Documentation | PARTIAL | Missing architecture, threat_model, math, engine_integration, proving_notes |

---

## Detailed Findings

### 1. Schemas — Missing `hash_scheme` Field

**Spec:** Section 15 (Public Input Schema) and Section 22 (Versioning) both require a `hash_scheme` field (e.g., `"poseidon2-v1"`) to explicitly tag which hash function was used for the commitment.

**Actual:**
- `crates/zenoform-core/src/proof.rs` — `PublicInputs` struct has: `world_id`, `seed_hash`, `chunk_coord`, `chunk_size`, `module_hash`, `output_commitment`. **No `hash_scheme`.**
- `crates/zenoform-core/src/chunk.rs` — `Chunk` struct has no `hash_scheme` field either.

**Impact:** Without explicit hash scheme tagging, a future protocol upgrade that changes hash functions could cause silent verification failures or ambiguity about which verifier path to use.

**Fix:** Add `hash_scheme: String` to both `Chunk` and `PublicInputs`, defaulting to `"poseidon2-v1"` in `new_v1()` constructors.

---

### 2. Real S-two / stwo-cairo Proving — Not Implemented

**Spec:** Section 6, 7.2, 7.3, Phase 3. The entire point of the protocol is to generate real STARK proofs using Cairo + S-two.

**Actual:**
- `zenoform-cli/src/main.rs` line 128-151: `Commands::Prove` generates a **mock proof** with `"stwo-cairo-mock"` prover and `{"status": "mocked", "os": "windows"}` payload.
- `zenoform-verifier/src/lib.rs` line 85-97: `NoopStarkVerifier` always returns `StarkVerificationFailed`.
- `verify_chunk_with_backend()` exists but there is no real backend registered.
- No `stwo_bridge.rs` file exists (spec Section 18 lists it).

**Impact:** The "proof" is a JSON envelope with empty cryptographic content. The protocol is structurally sound but not cryptographically proven yet.

**Fix:** Integrate `stwo-cairo` (Linux-only for now) behind a feature flag. The `NoopStarkVerifier` should be replaced with a real backend on supported platforms.

---

### 3. Godot Demo — Missing Visual States

**Spec:** Section 23.1 and 24 require 5 visual states:
- Green = verified
- Yellow = pending proof
- Red = invalid/tampered
- Blue = local preview not yet proven
- Gray = unsupported module

**Actual:** `examples/godot_verified_chunks/terrain_viewer.gd` lines 67-93 only implements:
- Green (`VERIFIED`) — line 88
- Red (`FAILED`) — line 92

**Missing:** Yellow, blue, and gray overlays.

**Impact:** The demo only shows binary pass/fail. It cannot demonstrate the "pending proof" or "preview" workflows described in the spec.

**Fix:** Add `update_status_visual()` cases for pending, preview, and unsupported module states.

---

### 4. Mojo Acceleration — Incomplete

**Spec:** Phase 4 wants Mojo CPU kernels for full terrain generation (height, temperature, moisture, biome, resources) with SIMD vectorization, plus Rust/Mojo comparison tests.

**Actual:**
- `mojo/zenoform_kernels/terrain.mojo` lines 22-38: Only generates `height` using SIMD `value_noise_2d`. No temperature, moisture, biome classification, or resource placement.
- No comparison tests between Rust and Mojo output.
- No GPU experiment.

**Impact:** Mojo is not a viable acceleration path yet. The DSL generates Mojo-looking code, but there is no runtime to execute it.

**Fix:** Port the full `classify_biome` and `place_resources` logic to Mojo. Add deterministic comparison tests.

---

### 5. DSL Example — Hardcoded Biome/Resource

**Spec:** Section 20.2 shows a rich DSL with:
```
biome_id = biome(height, moisture, temperature)
resource_mask = resource(seed, world_x, world_y, biome_id, height)
```

**Actual:** `dsl/examples/terrain_v1.zf` lines 12-13:
```
biome_id = 4
resource_mask = 0
```

**Impact:** The DSL example does not demonstrate the actual terrain rules. It is unclear if the PEG grammar even supports `biome()` and `resource()` as built-in functions.

**Fix:** Update `terrain_v1.zf` to use the actual biome and resource expressions. Verify the parser handles them.

---

### 6. Module Registry — Missing Metadata Fields

**Spec:** Section 22 requires every module entry to contain:
```json
{
  "module_id": "...",
  "module_version": "1.0.0",
  "module_hash": "0x...",
  "source_hash": "0x...",
  "compiler_version": "zenoform-dslc-0.1.0",
  "proof_backend": "stwo-cairo",
  "proof_backend_version": "...",
  "hash_scheme": "poseidon2-v1"
}
```

**Actual:** `crates/zenoform-core/src/seed_hash.rs` lines 43-48:
```rust
pub struct ModuleEntry {
    pub module_id: String,
    pub module_hash: String,
    pub content_description: String,
}
```

**Missing:** `module_version`, `source_hash`, `compiler_version`, `proof_backend`, `proof_backend_version`, `hash_scheme`.

**Impact:** The registry cannot distinguish between different versions of the same module, different compilers, or different proof backends.

**Fix:** Expand `ModuleEntry` and `ModuleRegistry::register()` to accept and store the additional metadata.

---

### 7. Test Vectors — Missing `expected_commitments.json`

**Spec:** Section 18 lists:
```
test_vectors/terrain_v1/
  seed_001_chunk_0_0.json
  seed_001_chunk_1_0.json
  expected_commitments.json
```

**Actual:**
- `test_vectors/terrain_v1/` has 25 `.json` chunk files and a `manifest.json`.
- **No `expected_commitments.json` file exists.**

The `manifest.json` does contain commitments per file, so the data is there, but not in the spec-prescribed format.

**Fix:** Either rename `manifest.json` to `expected_commitments.json` or generate a separate file in the expected format.

---

### 8. Benchmark Suite — Incomplete Metrics

**Spec:** Section 25 requires tracking:
- Generation: Rust gen, Mojo CPU gen, Mojo GPU gen, memory
- Proving: constraint count, witness gen time, proof gen time, peak memory, proof size
- Verification: proof verification time, public input parsing, commitment check, memory

**Actual:** `zenoform-cli/src/main.rs` lines 184-217 (`Commands::Bench`) only tracks:
- Rust generation time
- Commitment calculation time

**Missing:** All proving metrics, verification metrics, Mojo metrics, proof size, memory usage.

**Impact:** There is no data to evaluate whether the protocol is viable for real-time or competitive use.

**Fix:** Add proof generation timing, verification timing, and peak memory tracking to the benchmark harness.

---

### 9. Repository Structure — Empty Directories

**Spec:** Section 18 lists:
- `bindings/godot/addon/` — `bindings/` exists but is **empty**
- `bindings/unity/` — **missing**
- `bindings/unreal/` — **missing**
- `dsl/compiler/src/` — `dsl/` has `examples/` but no `compiler/`
- `benches/proving/` — `benches/` exists but is **empty**
- `benches/generation/` — **missing**
- `benches/verification/` — **missing**

**Impact:** These are placeholders for future work, but their absence means the repo does not match the spec layout.

**Fix:** Create `.gitkeep` files or README stubs in empty directories so the structure matches the spec.

---

### 10. CLI Tamper Command — Simplified Interface

**Spec:** Section 19.4:
```bash
zenoform tamper --chunk chunk.json --cell 3,7 --field resource_mask --value 65535 --out tampered_chunk.json
```

**Actual:** `zenoform-cli/src/main.rs` lines 168-183:
```bash
zenoform tamper --chunk chunk.json --index 0 --height 9999 --out tampered.json
```

**Difference:** The actual CLI only supports tampering a single cell's `height` by index. The spec wants arbitrary cell coordinates, arbitrary fields (height, temperature, moisture, resource_mask), and arbitrary values.

**Impact:** Less flexible tamper testing. Cannot test tampering of temperature, moisture, or resources.

**Fix:** Expand the tamper command to accept `--cell x,y`, `--field`, and `--value` arguments.

---

### 11. Documentation — Missing Spec Files

**Spec:** Section 18 lists:
- `docs/architecture.md`
- `docs/threat_model.md`
- `docs/math.md`
- `docs/engine_integration.md`
- `docs/proving_notes.md`

**Actual docs:**
- `docs/protocol.md` — integration guide (brief, 45 lines)
- `docs/Zenoform_Protocol_Revised_Technical_Spec.md` — the master spec itself

**Missing:** All 5 specialized docs.

**Impact:** New contributors must read the 1500-line master spec instead of targeted docs.

**Fix:** Extract relevant sections from the master spec into focused documents.

---

## What IS Complete (No Gaps)

| Item | Evidence |
|------|----------|
| Rust workspace with 6 crates | `Cargo.toml` workspace definition |
| Fixed-point terrain generation (Rust) | `module.rs` — height, temp, moisture, biome, resources |
| Chunk schema (struct + JSON) | `chunk.rs` — all spec fields except `hash_scheme` |
| Proof package schema | `proof.rs` — all spec fields except `hash_scheme` in public inputs |
| Poseidon commitment | `commitment.rs` + `seed_hash.rs` |
| Seed hash derivation | `derive_seed_hash()` using `poseidon_hash_many` |
| Module registry with default terrain | `default_registry()` with `terrain.fixed_noise.v1` |
| CLI generate, prove, verify, tamper, bench | `main.rs` — all commands functional |
| FFI C ABI | `zenoform_ffi` exports `zenoform_verify_chunk` |
| WASM verifier (5 functions) | `zenoform_wasm` — verify, detailed, recalc commitment, generate, get commitment |
| Godot demo (basic) | `terrain_viewer.gd` — generate, verify, mesh, green/red overlay |
| DSL parser (PEG) | `zenoform.pest` + `lib.rs` — parses modules with dots in identifiers |
| DSL codegen (Rust, Cairo, Mojo) | `compile_to_rust`, `compile_to_cairo`, `compile_to_mojo` |
| Cairo terrain module | `cairo/terrain_v1/src/module.cairo` — full terrain, biome, resources |
| Cairo fixed-point math | `fixed.cairo` |
| Cairo noise | `noise.cairo` |
| Cairo commitment | `commitment.cairo` |
| Test vectors (25 chunks + manifest) | `test_vectors/terrain_v1/` |
| CI/CD pipeline | `.github/workflows/build.yml` — Rust, Cairo, Mojo, pre-commit |
| Linting + formatting | `rustfmt.toml`, workspace clippy, pre-commit hook installed |
| 61 passing tests | 60 Rust + 1 Cairo |

---

## Critical Path for Full Spec Compliance

To bring the implementation into full compliance with the spec docs, prioritize:

1. **Add `hash_scheme` to schemas** — Small change, high protocol correctness value.
2. **Real stwo-cairo proving integration** — The core value proposition. Requires Linux dev environment or CI.
3. **Expand ModuleRegistry metadata** — Needed before any real module versioning.
4. **Complete Godot demo states** — Yellow (pending), blue (preview), gray (unsupported).
5. **Full Mojo terrain kernel** — Height + temp + moisture + biome + resources.
6. **Rust/Cairo commitment comparison test** — Validates that both paths produce identical outputs.
7. **Expand benchmark harness** — Track proving and verification metrics.
8. **Enhance CLI tamper** — Support arbitrary cell coordinates, fields, and values.
9. **Fix DSL example** — Use real `biome()` and `resource()` expressions.
10. **Write missing docs** — architecture, threat_model, math, engine_integration, proving_notes.

---

## Conclusion

The **plan.md tasks are complete** and the **MVP is functional** — it generates deterministic terrain, mocks proofs, verifies structure, and demonstrates tamper rejection across Rust, Cairo, Godot, WASM, and FFI.

However, the **spec docs describe a more ambitious system** than what is currently built. The largest gap is the lack of **real cryptographic proving** (everything is mocked). Secondary gaps are in **schema completeness** (`hash_scheme`), **Godot demo polish** (missing visual states), **Mojo runtime completeness**, and **documentation coverage**.

The project is **honestly scoped as an MVP** and matches the "Build First" list (Section 29) reasonably well. The spec's "Do Not Build Yet" list (Section 28) has been respected.
