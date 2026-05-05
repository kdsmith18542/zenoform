use std::fs;
use zenoform_core::{
    commitment::calculate_poseidon_commitment,
    coord::{ChunkCoord, ChunkSize},
    module::generate_terrain_v1,
};

fn main() {
    let seeds = vec![1, 42, 123, 999, 1337];
    let coords = vec![
        ChunkCoord::new(0, 0, 0),
        ChunkCoord::new(1, 0, 0),
        ChunkCoord::new(0, 1, 0),
        ChunkCoord::new(-1, 0, 0),
        ChunkCoord::new(0, -1, 0),
    ];
    let size = ChunkSize::new(16, 16);

    let out_dir = "test_vectors/terrain_v1";
    fs::create_dir_all(out_dir).expect("Failed to create test_vectors directory");

    let mut manifest = vec![];

    for seed in &seeds {
        for coord in &coords {
            let chunk = generate_terrain_v1("test-world".to_string(), *seed, *coord, size);
            let commitment = calculate_poseidon_commitment(&chunk);

            let filename = format!("seed_{:04}_chunk_{}_{}_{}.json", seed, coord.x, coord.y, coord.z);
            let filepath = format!("{}/{}", out_dir, filename);

            let json = serde_json::to_string_pretty(&chunk).unwrap();
            fs::write(&filepath, json).expect("Failed to write test vector");

            manifest.push(serde_json::json!({
                "file": filename,
                "seed": seed,
                "coord": [coord.x, coord.y, coord.z],
                "size": [size.width, size.height],
                "commitment": commitment,
                "cell_count": chunk.cells.len()
            }));
        }
    }

    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
    fs::write(format!("{}/manifest.json", out_dir), manifest_json).expect("Failed to write manifest");

    println!("Generated {} test vectors in {}", manifest.len(), out_dir);
}
