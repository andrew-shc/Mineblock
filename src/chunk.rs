use crate::mesh::{
    mesh::Meshes
};
use crate::block::Block;

use std::rc::Rc;
use std::cell::RefCell;


pub const CHUNK_SIZE: usize = 64;
// a chunk is a size of 64x64x64 Blocks

#[derive(Copy, Clone, PartialEq)]
pub struct ChunkID(pub u32, pub u32, pub u32);  // Chunk ID on render data & auto generation

pub struct Chunk {
    pub id: ChunkID,
    pub visible: bool,
    position: [u32; 3],  // position is relative towards to its parent sector; in chunks
    block_data: Vec<Block>,
}

impl Chunk {
    pub fn from_sector() {
        // return a new chunk from sector
    }

    pub fn new(id: ChunkID, position: [u32; 3], blocks: Vec<Block>) -> Self {
        Self {
            id: id,
            position: position,
            visible: true,
            block_data: blocks,
        }
    }

    pub fn render(&self, meshes: Rc<RefCell<Meshes>>) {
        (*meshes).borrow_mut().onload_data(self.id, [self.position[0] as f32, self.position[1] as f32, self.position[2] as f32], &self.block_data);
    }

    pub fn update(&mut self) {
    }

    fn regen(&mut self) {

    }

    pub fn save(&self) {

    }

    pub fn load(&mut self) {

    }
}