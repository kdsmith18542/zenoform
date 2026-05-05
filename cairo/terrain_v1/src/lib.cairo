mod fixed;
mod noise;
mod module;
mod commitment;

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
        
        // This is a placeholder for the actual expected commitment
        // We will need to run this and see the output to establish the baseline
        assert(commitment != 0, 'Commitment should not be zero');
    }
}
