use crate::block::Block;
use crate::mesh::mesh::MeshType;

use std::collections::HashMap;
use crate::texture::TextureAtlas;
use std::rc::Rc;

enum Biome {
    FlatPlains
}

pub struct Terrain {
    blocks: HashMap<&'static str, Block>,
}

impl Terrain {
    pub fn new(txtr: Rc<TextureAtlas>) -> Self {
        let mut blockspace = HashMap::new();

        blockspace.insert("air", Block::new(MeshType::Cube, txtr.clone(), "air", &[[3,0], [3,0], [3,0], [3, 0], [3,0], [3,0]], 0));
        blockspace.insert("dirt", Block::new(MeshType::Cube, txtr.clone(), "dirt", &[[2,0], [2,0], [2,0], [2, 0], [2,0], [2,0]], 0));
        blockspace.insert("grass", Block::new(MeshType::Cube, txtr.clone(), "grass", &[[0,0], [2,0], [1,0], [1,0], [1,0], [1,0]], 0));

        Self {
            blocks: blockspace,
        }
    }

    pub fn generate(&mut self, position: &[u32; 3], size: usize) -> Vec<Block> { // generates in mesh
        println!("Input size constant pre-check: {:?} Blocks", size);
        println!("Terrain size allocated: {:?} Blocks", size*size*size);

        let mut block_data: Vec<Block> = Vec::with_capacity(size*size*size);

        for x in position[0]..position[0]+size as u32 {
            for y in position[1]..position[1]+size as u32 {
                for z in position[2]..position[2]+size as u32  {
                    if y < size as u32-1 {
                        block_data.push(self.blocks["dirt"].clone());
                    } else {
                        block_data.push(self.blocks["grass"].clone());
                    }
                }
            }
        }

        block_data
    }
}
