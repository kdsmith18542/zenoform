use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkSize {
    pub width: u32,
    pub height: u32,
}

impl ChunkSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn total_cells(&self) -> usize {
        (self.width * self.height) as usize
    }
}
