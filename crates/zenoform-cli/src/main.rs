use clap::{Parser, Subcommand};
use zenoform_core::{ChunkCoord, ChunkSize, module::generate_terrain_v1};
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
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Generate { seed, world, module: _, chunk, size, out } => {
            let coords: Vec<i32> = chunk.split(',').map(|s| s.parse().unwrap()).collect();
            let coord = ChunkCoord::new(coords[0], coords[1], coords[2]);

            let sizes: Vec<u32> = size.split('x').map(|s| s.parse().unwrap()).collect();
            let size_struct = ChunkSize::new(sizes[0], sizes[1]);

            let chunk_data = generate_terrain_v1(world.clone(), *seed, coord, size_struct);
            
            let json = serde_json::to_string_pretty(&chunk_data).unwrap();
            fs::write(out, json).expect("Unable to write file");
            println!("Chunk generated and saved to {}", out);
        }
    }
}
