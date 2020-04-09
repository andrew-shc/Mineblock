use vulkano;
use std::fmt;

#[derive(Default, Copy, Clone)]
pub struct CubeVtx {
    pub position: [f32; 3],
    pub txtr_crd: [f32; 2],
}

impl fmt::Debug for CubeVtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CubeVtx")
            .field("position", &self.position)
            .field("txtr_crd", &self.txtr_crd)
            .finish()
    }
}

vulkano::impl_vertex!(CubeVtx, position, txtr_crd);


/*
Mesh Types v2

Cube - (BLocks) [8 vertices]
Flora - (Flowers) [X Shape]

---------------------------
Mesh Types

QUAD - (Blocks, Slabs) [8 Vertices]
FLORA - (Flowers) [X-Shape]
LIQUID - (Water, Lava) [Wavy surfaces, Translucent, Flow direction]
PARTICLE - (Fire particle, water particle) [Multiple instance of that small picture]
XQUAD - (Chests, Flower pots, Item frame, painting)
CUSTOM -

MODEL - 3D voxel model

GUI -
SKYBOX - (Skybox, Sun, Moon, Stars)

*/