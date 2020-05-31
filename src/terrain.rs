use crate::block::Block;
use crate::mesh::mesh::MeshType;
use crate::texture::TextureAtlas;

use std::collections::HashMap;
use std::rc::Rc;
use rand::Rng;

enum Biome {
    FlatPlains
}

pub struct Terrain {
    blocks: HashMap<&'static str, Block>,
}

impl Terrain {
    pub fn new(txtr: Rc<TextureAtlas>) -> Self {
        let mut blockspace = HashMap::new();

        blockspace.insert("air", Block::new(MeshType::Cube, txtr.clone(), "air", &[[4,0], [4,0], [4,0], [4, 0], [4,0], [4,0]], 0, true));
        blockspace.insert("dirt", Block::new(MeshType::Cube, txtr.clone(), "dirt", &[[2,0], [2,0], [2,0], [2, 0], [2,0], [2,0]], 0, false));
        blockspace.insert("grass", Block::new(MeshType::Cube, txtr.clone(), "grass", &[[0,0], [2,0], [1,0], [1,0], [1,0], [1,0]], 0, false));
        blockspace.insert("stone", Block::new(MeshType::Cube, txtr.clone(), "stone", &[[3,0], [3,0], [3,0], [3, 0], [3,0], [3,0]], 0, false));

        Self {
            blocks: blockspace,
        }
    }

    pub fn generate(&mut self, position: &[u32; 3], size: usize) -> Vec<Block> { // generates in mesh
        println!("Input size constant pre-check: {:?} Blocks", size);
        println!("Terrain size allocated: {:?} Blocks", size*size*size);

        let ground_level = 60;

        let mut block_data: Vec<Block> = Vec::with_capacity(size*size*size);

        for x in position[0]..position[0]+size as u32 {
            let num = rand::thread_rng().gen_range(0, 3);
            for y in position[1]..position[1]+size as u32 {
                for z in position[2]..position[2]+size as u32  {
                    if y >= ground_level-num as u32-1 {
                        block_data.push(self.blocks["air"].clone());
                    } else if y >= ground_level-num-2 {
                        block_data.push(self.blocks["grass"].clone());
                    } else if y >= ground_level-num-5 {
                        block_data.push(self.blocks["dirt"].clone());
                    } else {
                        block_data.push(self.blocks["stone"].clone());
                    }
                }
            }
        }

        block_data
    }
}
