use crate::mesh::Mesh;
// a chunk is a size of 32x32x32 Blocks

pub struct Chunk {
    position: [u8; 2],  // position is relative towards to its parent sector; in chunks
    size: u8,
    mesh_ind: Vec<(u32, u32)>,
}

impl Chunk {
    pub fn new(position: [u8; 2]) -> Self {
        Self {
            position: position,
            size: u8,
            mesh_ind: Vec::new(),
        }
    }

    pub fn generate() {

    }

    pub fn render() {

    }

    pub fn update() {

    }
}