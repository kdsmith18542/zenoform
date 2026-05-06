use criterion::{Criterion, black_box, criterion_group, criterion_main};
use zenoform_core::{ChunkCoord, ChunkSize, commitment::calculate_poseidon_commitment, module::generate_terrain_v1};

fn bench_generate_chunk_16x16(c: &mut Criterion) {
    c.bench_function("generate_chunk_16x16", |b| {
        b.iter(|| generate_terrain_v1("bench".to_string(), 42, ChunkCoord::new(0, 0, 0), ChunkSize::new(16, 16)))
    });
}

fn bench_generate_chunk_32x32(c: &mut Criterion) {
    c.bench_function("generate_chunk_32x32", |b| {
        b.iter(|| generate_terrain_v1("bench".to_string(), 42, ChunkCoord::new(0, 0, 0), ChunkSize::new(32, 32)))
    });
}

fn bench_generate_chunk_64x64(c: &mut Criterion) {
    c.bench_function("generate_chunk_64x64", |b| {
        b.iter(|| generate_terrain_v1("bench".to_string(), 42, ChunkCoord::new(0, 0, 0), ChunkSize::new(64, 64)))
    });
}

fn bench_commitment_calculation(c: &mut Criterion) {
    c.bench_function("commitment_16x16", |b| {
        let chunk = generate_terrain_v1("bench".to_string(), 42, ChunkCoord::new(0, 0, 0), ChunkSize::new(16, 16));
        b.iter(|| calculate_poseidon_commitment(black_box(&chunk)))
    });
}

criterion_group!(
    benches,
    bench_generate_chunk_16x16,
    bench_generate_chunk_32x32,
    bench_generate_chunk_64x64,
    bench_commitment_calculation
);
criterion_main!(benches);
