use crate::mesh::Mesh;
use crate::mesh::CubeFace;

use std::rc::Rc;
use std::cell::RefCell;
// stores block info
// TODO: dynamic texture will be called through the Block class

pub struct Block<V>{
    mesh: Rc<RefCell<dyn Mesh<Vertex=V>>>,  // the parent mesh
    id: String,  // block id
    texture: Vec<[u16; 2]>,  // texture info
    state: u8,  // block state info TODO
}

impl<V: 'static> Block<V> {
    // instantiate a new block
    pub fn new(mesh: Rc<RefCell<dyn Mesh<Vertex=V>>>, id: String, texture: Vec<[u16; 2]>, state: u8) -> Self {
        Self { mesh, id, texture, state }
    }

    // creates the new block in the world
    pub fn create(&mut self, index: u32, position: [f32; 3], faces: Vec<CubeFace>, start: [f32; 3], end: [f32; 3]) {
        self.mesh.borrow_mut().update_vert(&self.texture, position, start, end);
        self.mesh.borrow_mut().update_ind(index);
    }
}


/*
Block State Descriptor

Block {
    liquid: False,

}

*/