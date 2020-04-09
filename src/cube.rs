use crate::mesh::Renderable;
use crate::renderer::CubeVtx;
use crate::texture::TextureAtlas;

// each integer values correspond to the location of the quad in the atlas
pub struct Cube {
    top: [u16; 2],
    bottom: [u16; 2],
    left: [u16; 2],
    right: [u16; 2],
    front: [u16; 2],
    back: [u16; 2],
}

impl Cube {
    pub fn new(top: [u16; 2], bottom: [u16; 2], left: [u16; 2], right: [u16; 2], front: [u16; 2], back: [u16; 2]) -> Cube {
        Cube {top: top, bottom: bottom, left: left, right: right, front: front, back: back}
    }

    pub fn texture_all(coord: [u16; 2]) -> Cube {
        Cube {
            top: coord,
            bottom: coord,
            left: coord,
            right: coord,
            front: coord,
            back: coord
        }
    }
}

impl Renderable for Cube {
    type Vertex = CubeVtx;

    fn vert_data(&self, atlas: &TextureAtlas, position: [f32; 3]) -> Vec<Self::Vertex> {
        let top = atlas.texture_coord(self.top[0],self.top[1]);
        let bottom = atlas.texture_coord(self.bottom[0],self.bottom[1]);
        let left = atlas.texture_coord(self.left[0],self.left[1]);
        let right = atlas.texture_coord(self.right[0],self.right[1]);
        let front = atlas.texture_coord(self.front[0],self.front[1]);
        let back = atlas.texture_coord(self.back[0],self.back[1]);

        vec![
            Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: front[3], },
            Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: front[2], },
            Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: front[1], },
            Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: front[0], },

            Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: back[2], },
            Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: back[3], },
            Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: back[0], },
            Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: back[1], },

            Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: bottom[0], },
            Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: bottom[1], },
            Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: bottom[2], },
            Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: bottom[3], },

            Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: top[3], },
            Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: top[2], },
            Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: top[1], },
            Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: top[0], },

            Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: left[2], },
            Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: left[1], },
            Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: left[0], },
            Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: left[3], },

            Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: right[3], },
            Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: right[0], },
            Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: right[1], },
            Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: right[2], },
        ]
    }

    fn ind_data(&self) -> Vec<u32> {
        vec![  0, 1, 2, 0, 2, 3,
               4, 5, 6, 4, 6, 7,
               8, 9,10, 8,10,11,
              12,13,14,12,14,15,
              16,17,18,16,18,19,
              20,21,22,20,22,23,]
    }
}
