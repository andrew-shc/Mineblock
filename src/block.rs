use crate::mesh::mesh::MeshType;
use std::fmt::{Debug, Formatter};
use std::fmt;
use std::rc::Rc;
use crate::texture::TextureAtlas;
use crate::block::BlockState::Luminosity;

// stores block info
// TODO: dynamic texture will be called through the Block class

#[derive(Clone)]
pub struct Block {
    // Rc<RefCell<dyn Mesh<Vertex=V>>>
    pub mesh: MeshType,  // the parent mesh
    pub id: &'static str,  // block id
    pub texture: &'static [[u16; 2]],  // texture info
    pub texture_coord: Vec<[[f32; 2]; 4]>,  // texture coordinate info
    pub state: u8,  // block state info TODO
    pub transparent: bool, // TODO: TEMPORARY
}

impl Block {
    // instantiate a new block
    pub fn new(mesh: MeshType, txtr: Rc<TextureAtlas>, id: &'static str, texture: &'static [[u16; 2]], state: u8, transparent: bool) -> Self {
        let top = txtr.texture_coord( texture[0][0], texture[0][1]);
        let bottom = txtr.texture_coord( texture[1][0], texture[1][1]);
        let left = txtr.texture_coord( texture[2][0], texture[2][1]);
        let right = txtr.texture_coord( texture[3][0], texture[3][1]);
        let front = txtr.texture_coord( texture[4][0], texture[4][1]);
        let back = txtr.texture_coord( texture[5][0], texture[5][1]);

        Self { mesh, id, texture, state, texture_coord: vec![top, bottom, left, right, front, back], transparent }
    }

    // // creates the new block in the world
    // pub fn create(&mut self, index: u32, position: [f32; 3], faces: Vec<CubeFace>, start: [f32; 3], end: [f32; 3]) {
    //     self.mesh.borrow_mut().onload_vert(&self.texture, position, start, end);
    //     self.mesh.borrow_mut().onload_ind(index);
    // }
}

impl Debug for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("mesh", match &self.mesh {
                MeshType::Cube => &"Cube",
                MeshType::Flora => &"Flora",
            })
            .field("id", &self.id)
            .finish()
    }
}

pub enum BlockTag {
    Nibble(&'static str),
    Integer(&'static str),
    Float(&'static str),
    String(&'static str),
}

pub enum BlockState {
    Luminosity(u8)
}

fn priv_block_state() -> BlockState {
    Luminosity(4)
}

/*
Block State Descriptor

BlockState::new()
    .liquid(False)  // ommitable, since it is false; all properties has a default value (if mandatory)
    .luminosity(5)
    .nest("custom tag")
        .val("itm1", Itm_Tag)
        .val("type 1")
        .build()
    .build()


Block {
    liquid: False,
    luminosity: 8,
    custom_tag: {
        itm1: Itm_Tag,
        typ: "type 1",
    }
}

*/

