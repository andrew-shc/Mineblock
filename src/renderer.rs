use vulkano;

#[derive(Default, Copy, Clone)]
pub struct CubeVtx {
    pub position: [f32; 3],
    pub txtr_crd: [f32; 2],
}

vulkano::impl_vertex!(CubeVtx, position, txtr_crd);


/*
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