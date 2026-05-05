pub mod fixed;
pub mod noise;
pub mod module;
pub mod commitment;

#[cfg(test)]
mod tests {
    use super::module::generate_terrain_v1;
    use super::commitment::calculate_commitment;

    #[test]
    fn test_generate_and_commit() {
        let seed = 123;
        let chunk_x = 0;
        let chunk_y = 0;
        let width = 16;
        let height = 16;

        let cells = generate_terrain_v1(seed, chunk_x, chunk_y, width, height);
        let commitment = calculate_commitment(seed, chunk_x, chunk_y, width, height, cells.span());

        assert(commitment != 0, 'Commitment should not be zero');
    }
}
