use clap::{Parser, Subcommand};
use std::fs;
use std::time::Instant;
use zenoform_core::{
    Chunk, ChunkCoord, ChunkSize,
    commitment::calculate_poseidon_commitment,
    module::generate_terrain_v1,
    proof::{ProofPackage, PublicInputs},
};
use zenoform_verifier::verify_chunk;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a chunk from seed and coordinate
    Generate {
        #[arg(long)]
        seed: i32,
        #[arg(long)]
        world: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        chunk: String, // format "x,y,z"
        #[arg(long, default_value = "16x16")]
        size: String, // format "WxH"
        #[arg(long)]
        out: String,
    },
    /// Produce a proof for a generated chunk (Mocked on Windows)
    Prove {
        #[arg(long)]
        chunk: String,
        #[arg(long)]
        out: String,
        /// Generate Cairo prover input JSON for use with scarb prove
        #[arg(long)]
        generate_prover_inputs: bool,
    },
    /// Verify a chunk against its proof
    Verify {
        #[arg(long)]
        chunk: String,
        #[arg(long)]
        proof: String,
    },
    /// Tamper with a chunk to test verification failure
    Tamper {
        #[arg(long)]
        chunk: String,
        #[arg(long)]
        index: usize,
        #[arg(long)]
        height: u16,
        #[arg(long)]
        out: String,
    },
    /// Benchmark generation for various chunk sizes
    Bench {
        #[arg(long)]
        seed: i32,
        #[arg(long)]
        module: String,
        #[arg(long, default_value = "16x16,32x32,64x64")]
        sizes: String,
        #[arg(long, default_value = "10")]
        runs: u32,
    },
    /// Compile a Zenoform DSL (.zf) file to various targets
    Compile {
        #[arg(long)]
        file: String,
        #[arg(long, default_value = "rust,cairo,mojo")]
        targets: String,
        #[arg(long)]
        out_dir: String,
    },
}

fn parse_chunk_coord(s: &str) -> Result<ChunkCoord, String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(format!("Expected 'x,y,z', got '{}'", s));
    }
    let x = parts[0].parse::<i32>().map_err(|e| format!("Invalid x: {}", e))?;
    let y = parts[1].parse::<i32>().map_err(|e| format!("Invalid y: {}", e))?;
    let z = parts[2].parse::<i32>().map_err(|e| format!("Invalid z: {}", e))?;
    Ok(ChunkCoord::new(x, y, z))
}

fn parse_chunk_size(s: &str) -> Result<ChunkSize, String> {
    let parts: Vec<&str> = s.split('x').collect();
    if parts.len() != 2 {
        return Err(format!("Expected 'WxH', got '{}'", s));
    }
    let w = parts[0].parse::<u32>().map_err(|e| format!("Invalid width: {}", e))?;
    let h = parts[1].parse::<u32>().map_err(|e| format!("Invalid height: {}", e))?;
    Ok(ChunkSize::new(w, h))
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { seed, world, module, chunk, size, out } => {
            let coord = parse_chunk_coord(chunk).unwrap_or_else(|e| {
                eprintln!("Error parsing chunk coordinate: {}", e);
                std::process::exit(1);
            });
            let size_struct = parse_chunk_size(size).unwrap_or_else(|e| {
                eprintln!("Error parsing chunk size: {}", e);
                std::process::exit(1);
            });

            if module != "terrain.fixed_noise.v1" {
                eprintln!("Warning: Unknown module '{}', using terrain.fixed_noise.v1", module);
            }

            let chunk_data = generate_terrain_v1(world.clone(), *seed, coord, size_struct);

            let json = serde_json::to_string_pretty(&chunk_data).unwrap();
            fs::write(out, json).expect("Unable to write file");
            println!("Chunk generated with Poseidon commitment and saved to {}", out);
        }
        Commands::Prove { chunk, out, generate_prover_inputs } => {
            let chunk_json = fs::read_to_string(chunk).expect("Unable to read chunk file");
            let chunk_data: Chunk = serde_json::from_str(&chunk_json).expect("Invalid chunk JSON");

            let public_inputs = PublicInputs {
                world_id: chunk_data.world_id.clone(),
                seed_hash: chunk_data.seed_hash.clone(),
                chunk_coord: chunk_data.chunk_coord,
                chunk_size: chunk_data.chunk_size,
                module_hash: chunk_data.module_hash.clone(),
                output_commitment: chunk_data.commitment.clone(),
            };

            let proof_package = ProofPackage::new_v1(
                "stwo-cairo-mock".to_string(),
                "0.1.0".to_string(),
                "zenoform-terrain-v1".to_string(),
                public_inputs,
                serde_json::json!({"status": "mocked", "os": "windows"}),
            );

            let json = serde_json::to_string_pretty(&proof_package).unwrap();
            fs::write(out, json).expect("Unable to write proof file");
            println!("Mock proof generated and saved to {}", out);

            if *generate_prover_inputs {
                let prover_input = serde_json::json!({
                    "input": {
                        "seed": chunk_data.seed_hash.strip_prefix("0x").unwrap_or(&chunk_data.seed_hash),
                        "chunk_x": chunk_data.chunk_coord.x.to_string(),
                        "chunk_y": chunk_data.chunk_coord.y.to_string(),
                        "width": chunk_data.chunk_size.width.to_string(),
                        "height": chunk_data.chunk_size.height.to_string(),
                    },
                    "expected_commitment": chunk_data.commitment,
                    "module": "terrain.fixed_noise.v1",
                    "cells": chunk_data.cells.iter().map(|cell| {
                        serde_json::json!({
                            "x": cell.local_x,
                            "y": cell.local_y,
                            "height": cell.height,
                            "temperature": cell.temperature,
                            "moisture": cell.moisture,
                            "biome_id": cell.biome_id,
                            "resource_mask": cell.resource_mask,
                        })
                    }).collect::<Vec<_>>(),
                });

                let prover_input_path = format!("{}_prover_input.json", out.trim_end_matches(".json"));
                fs::write(&prover_input_path, serde_json::to_string_pretty(&prover_input).unwrap())
                    .expect("Unable to write prover input file");
                println!("Cairo prover input generated at {}", prover_input_path);
                println!();
                println!("To generate a real STARK proof with Cairo:");
                println!("  1. cd cairo/terrain_v1");
                println!("  2. scarb build");
                println!("  3. scarb cairo-run --available-gas=9999999999");
                println!("     OR use: scarb prove (when snforge is available)");
            }
        }
        Commands::Verify { chunk, proof } => {
            let chunk_json = fs::read_to_string(chunk).expect("Unable to read chunk file");
            let chunk_data: Chunk = serde_json::from_str(&chunk_json).expect("Invalid chunk JSON");

            let proof_json = fs::read_to_string(proof).expect("Unable to read proof file");
            let proof_package: ProofPackage = serde_json::from_str(&proof_json).expect("Invalid proof JSON");

            match verify_chunk(&chunk_data, &proof_package) {
                Ok(_) => println!("VERIFICATION SUCCESS: Chunk follows the protocol rules."),
                Err(e) => {
                    eprintln!("VERIFICATION FAILURE: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Tamper { chunk, index, height, out } => {
            let chunk_json = fs::read_to_string(chunk).expect("Unable to read chunk file");
            let mut chunk_data: Chunk = serde_json::from_str(&chunk_json).expect("Invalid chunk JSON");

            if *index < chunk_data.cells.len() {
                println!("Tampering with cell {} (height {} -> {})", index, chunk_data.cells[*index].height, height);
                chunk_data.cells[*index].height = *height;
            } else {
                eprintln!("Index out of bounds");
                std::process::exit(1);
            }

            let json = serde_json::to_string_pretty(&chunk_data).unwrap();
            fs::write(out, json).expect("Unable to write file");
            println!("Tampered chunk saved to {}", out);
        }
        Commands::Bench { seed, module, sizes, runs } => {
            if module != "terrain.fixed_noise.v1" {
                eprintln!("Warning: Unknown module '{}', using terrain.fixed_noise.v1", module);
            }

            let size_list: Vec<&str> = sizes.split(',').collect();

            println!("{:<12} {:<8} {:<18} {:<18}", "Size", "Cells", "Avg Gen (ms)", "Commitment (ms)");
            println!("{}", "-".repeat(60));

            for size_str in size_list {
                let size = parse_chunk_size(size_str).unwrap_or_else(|e| {
                    eprintln!("Error parsing size '{}': {}", size_str, e);
                    std::process::exit(1);
                });

                let mut total_gen = std::time::Duration::ZERO;
                let mut total_commit = std::time::Duration::ZERO;

                for _ in 0..*runs {
                    let gen_start = Instant::now();
                    let mut chunk = generate_terrain_v1("bench".to_string(), *seed, ChunkCoord::new(0, 0, 0), size);
                    total_gen += gen_start.elapsed();

                    let commit_start = Instant::now();
                    chunk.commitment = calculate_poseidon_commitment(&chunk);
                    total_commit += commit_start.elapsed();
                }

                let avg_gen = total_gen.as_secs_f64() * 1000.0 / *runs as f64;
                let avg_commit = total_commit.as_secs_f64() * 1000.0 / *runs as f64;

                println!("{:<12} {:<8} {:<18.4} {:<18.4}", size_str, size.total_cells(), avg_gen, avg_commit);
            }
        }
        Commands::Compile { file, targets, out_dir } => {
            let source = fs::read_to_string(file).expect("Unable to read DSL file");
            let target_list: Vec<&str> = targets.split(',').collect();

            if fs::metadata(out_dir).is_err() {
                fs::create_dir_all(out_dir).expect("Unable to create output directory");
            }

            for target in target_list {
                match target {
                    "rust" => {
                        let code = zenoform_dsl::compile_to_rust(&source);
                        fs::write(format!("{}/module.rs", out_dir), code).unwrap();
                    }
                    "cairo" => {
                        let code = zenoform_dsl::compile_to_cairo(&source);
                        fs::write(format!("{}/module.cairo", out_dir), code).unwrap();
                    }
                    "mojo" => {
                        let code = zenoform_dsl::compile_to_mojo(&source);
                        fs::write(format!("{}/module.mojo", out_dir), code).unwrap();
                    }
                    _ => eprintln!("Unknown target: {}", target),
                }
            }
            println!("DSL compiled to targets: {}", targets);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn cli_bin() -> Command {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        path.pop();
        let exe = if cfg!(windows) { "zenoform-cli.exe" } else { "zenoform-cli" };
        path.push(exe);
        Command::new(path)
    }

    #[test]
    fn test_cli_generate_creates_valid_chunk() {
        let tmp = std::env::temp_dir().join("zenoform_test_chunk.json");
        let output = cli_bin()
            .args([
                "generate",
                "--seed",
                "42",
                "--world",
                "test",
                "--module",
                "terrain.fixed_noise.v1",
                "--chunk",
                "0,0,0",
                "--size",
                "4x4",
                "--out",
                tmp.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to run generate");

        assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
        assert!(tmp.exists());

        let content = fs::read_to_string(&tmp).unwrap();
        let chunk: Chunk = serde_json::from_str(&content).unwrap();
        assert_eq!(chunk.chunk_size.width, 4);
        assert_eq!(chunk.chunk_size.height, 4);
        assert_eq!(chunk.cells.len(), 16);
        assert!(!chunk.commitment.is_empty());

        fs::remove_file(&tmp).ok();
    }

    #[test]
    fn test_cli_generate_and_verify_roundtrip() {
        let tmp_dir = std::env::temp_dir().join("zenoform_roundtrip");
        fs::create_dir_all(&tmp_dir).unwrap();
        let chunk_file = tmp_dir.join("chunk.json");
        let proof_file = tmp_dir.join("proof.json");

        let gen_out = cli_bin()
            .args([
                "generate",
                "--seed",
                "99",
                "--world",
                "roundtrip",
                "--module",
                "terrain.fixed_noise.v1",
                "--chunk",
                "1,2,3",
                "--size",
                "8x8",
                "--out",
                chunk_file.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(gen_out.status.success());

        let prove_out = cli_bin()
            .args(["prove", "--chunk", chunk_file.to_str().unwrap(), "--out", proof_file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(prove_out.status.success());

        let verify_out = cli_bin()
            .args(["verify", "--chunk", chunk_file.to_str().unwrap(), "--proof", proof_file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(verify_out.status.success(), "verify stderr: {}", String::from_utf8_lossy(&verify_out.stderr));
        let stdout = String::from_utf8_lossy(&verify_out.stdout);
        assert!(stdout.contains("VERIFICATION SUCCESS"));

        fs::remove_dir_all(&tmp_dir).ok();
    }

    #[test]
    fn test_cli_tampered_chunk_fails_verification() {
        let tmp_dir = std::env::temp_dir().join("zenoform_tamper");
        fs::create_dir_all(&tmp_dir).unwrap();
        let chunk_file = tmp_dir.join("chunk.json");
        let proof_file = tmp_dir.join("proof.json");
        let tampered_file = tmp_dir.join("tampered.json");

        cli_bin()
            .args([
                "generate",
                "--seed",
                "1",
                "--world",
                "tamper",
                "--module",
                "terrain.fixed_noise.v1",
                "--chunk",
                "0,0,0",
                "--size",
                "4x4",
                "--out",
                chunk_file.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        cli_bin()
            .args(["prove", "--chunk", chunk_file.to_str().unwrap(), "--out", proof_file.to_str().unwrap()])
            .output()
            .unwrap();

        let tamper_out = cli_bin()
            .args([
                "tamper",
                "--chunk",
                chunk_file.to_str().unwrap(),
                "--index",
                "0",
                "--height",
                "9999",
                "--out",
                tampered_file.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(tamper_out.status.success());

        let verify_out = cli_bin()
            .args(["verify", "--chunk", tampered_file.to_str().unwrap(), "--proof", proof_file.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(!verify_out.status.success());
        let stderr = String::from_utf8_lossy(&verify_out.stderr);
        assert!(stderr.contains("VERIFICATION FAILURE"));

        fs::remove_dir_all(&tmp_dir).ok();
    }

    #[test]
    fn test_cli_compile_dsl() {
        let tmp_dir = std::env::temp_dir().join("zenoform_compile");
        fs::create_dir_all(&tmp_dir).unwrap();

        let dsl_source = r#"module terrain.test {
    input:
        seed: Field
    cell:
        height = noise2d(seed, world_x, world_y)
    output:
        height: u16
}"#;
        let dsl_file = tmp_dir.join("test.zf");
        fs::write(&dsl_file, dsl_source).unwrap();

        let compile_out = cli_bin()
            .args([
                "compile",
                "--file",
                dsl_file.to_str().unwrap(),
                "--targets",
                "rust,cairo",
                "--out-dir",
                tmp_dir.to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(compile_out.status.success(), "compile stderr: {}", String::from_utf8_lossy(&compile_out.stderr));

        assert!(tmp_dir.join("module.rs").exists());
        assert!(tmp_dir.join("module.cairo").exists());

        fs::remove_dir_all(&tmp_dir).ok();
    }

    #[test]
    fn test_cli_bench_runs_without_error() {
        let bench_out = cli_bin()
            .args(["bench", "--seed", "1", "--module", "terrain.fixed_noise.v1", "--sizes", "4x4", "--runs", "2"])
            .output()
            .unwrap();
        assert!(bench_out.status.success(), "bench stderr: {}", String::from_utf8_lossy(&bench_out.stderr));
        let stdout = String::from_utf8_lossy(&bench_out.stdout);
        assert!(stdout.contains("4x4"));
        assert!(stdout.contains("Cells"));
    }

    #[test]
    fn test_parse_chunk_coord_valid() {
        let coord = parse_chunk_coord("1,2,3").unwrap();
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 2);
        assert_eq!(coord.z, 3);
    }

    #[test]
    fn test_parse_chunk_coord_invalid() {
        assert!(parse_chunk_coord("1,2").is_err());
        assert!(parse_chunk_coord("a,b,c").is_err());
        assert!(parse_chunk_coord("").is_err());
    }

    #[test]
    fn test_parse_chunk_size_valid() {
        let size = parse_chunk_size("32x32").unwrap();
        assert_eq!(size.width, 32);
        assert_eq!(size.height, 32);
    }

    #[test]
    fn test_parse_chunk_size_invalid() {
        assert!(parse_chunk_size("32").is_err());
        assert!(parse_chunk_size("abc").is_err());
    }
}
