use clap::{Parser, Subcommand};
use zenoform_core::{
    Chunk, ChunkCoord, ChunkSize, 
    module::generate_terrain_v1,
    proof::{ProofPackage, PublicInputs},
    commitment::calculate_poseidon_commitment,
};
use zenoform_verifier::verify_chunk;
use std::fs;

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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { seed, world, module: _, chunk, size, out } => {
            let coords: Vec<i32> = chunk.split(',').map(|s| s.parse().unwrap()).collect();
            let coord = ChunkCoord::new(coords[0], coords[1], coords[2]);

            let sizes: Vec<u32> = size.split('x').map(|s| s.parse().unwrap()).collect();
            let size_struct = ChunkSize::new(sizes[0], sizes[1]);

            let mut chunk_data = generate_terrain_v1(world.clone(), *seed, coord, size_struct);
            
            // Overwrite commitment with Poseidon for proof-friendliness
            chunk_data.commitment = calculate_poseidon_commitment(&chunk_data);

            let json = serde_json::to_string_pretty(&chunk_data).unwrap();
            fs::write(out, json).expect("Unable to write file");
            println!("Chunk generated with Poseidon commitment and saved to {}", out);
        }
        Commands::Prove { chunk, out } => {
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

            if index < chunk_data.cells.len() {
                println!("Tampering with cell {} (height {} -> {})", index, chunk_data.cells[index].height, height);
                chunk_data.cells[index].height = *height;
            } else {
                eprintln!("Index out of bounds");
                std::process::exit(1);
            }

            let json = serde_json::to_string_pretty(&chunk_data).unwrap();
            fs::write(out, json).expect("Unable to write file");
            println!("Tampered chunk saved to {}", out);
        }
        Commands::Compile { file, targets, out_dir } => {
            let source = fs::read_to_string(file).expect("Unable to read DSL file");
            let target_list: Vec<&str> = targets.split(',').collect();

            if !fs::metadata(out_dir).is_ok() {
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
