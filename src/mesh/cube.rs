use crate::renderer::CubeVtx;
use crate::texture::TextureAtlas;
use crate::chunk::{CHUNK_SIZE, ChunkID};
use crate::chunk::Chunk;
use crate::block::Block;
use crate::mesh::mesh::{
    Mesh,
    MeshType
};


use vulkano::device::Device;
use vulkano::image::{SwapchainImage};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract, viewport::Viewport};
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::descriptor::{descriptor_set::PersistentDescriptorSet, DescriptorSet};
use vulkano::sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode};
use vulkano::memory::pool::MemoryPool;
use winit::window::Window;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;

use std::rc::Rc;
use std::sync::Arc;
use std::iter;
use std::thread::sleep;
use std::time::Duration;
use std::ops::{Sub, Range};


const CUBE_FACES: u32 = 6;
const VERT_PER_FACE: u32 = 4;
const VERT_PER_CUBE: u32 = CUBE_FACES*VERT_PER_FACE;
const IND_PER_FACE: u32 = 6;
const IND_PER_CUBE: u32 = CUBE_FACES*IND_PER_FACE;


#[derive(Eq, PartialEq)]
pub enum CubeFace {
    TOP,
    BOTTOM,
    LEFT,
    RIGHT,
    FRONT,
    BACK,
}

pub enum Bound {
    UBound, // upper bound
    LBound, // lower bound
}

pub enum Axis {
    X,
    Y,
    Z
}

// Cube Mesh
// - stores all the mesh info
// - to get a block from the mesh, you must retrieve it from a mesh struct like Cube

pub mod vs { vulkano_shaders::shader!{ty: "vertex", path: "resource/shaders/cube.vert",} }
pub mod fs { vulkano_shaders::shader!{ty: "fragment", path: "resource/shaders/cube.frag",} }

pub struct Cube {
    pub texture: Rc<TextureAtlas>,  // texture image
    chunk_data: Vec<(ChunkID, Vec<<Cube as Mesh>::Vertex>, Vec<u32>)>, // (chunk id, vert data, index data)
    // pub index: Vec<u32>,
    sampler: Arc<Sampler>,  // texture sampler
    vtx_shader: vs::Shader,
    frg_shader: fs::Shader,
}

impl Cube {
    pub fn new(device: Arc<Device>, texture: Rc<TextureAtlas>) -> Cube {
        // Filter::Nearest for rendering each pixel instead of "smudging" between the adjacent pixels
        let sampler = Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest,
                                   MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                                   SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();

        Cube { texture: texture.clone(), sampler: sampler, chunk_data: Vec::new(),
            vtx_shader: vs::Shader::load(device.clone()).expect("failed to create cube vertex shaders module"),
            frg_shader: fs::Shader::load(device.clone()).expect("failed to create cube fragment shaders module")
        }
    }

    pub fn descriptors<'b, U: Send+Sync+'b, A: MemoryPool+Sync+'b>(&self, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>, sub_buf: &CpuBufferPoolSubbuffer<U, A>) -> Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>
        where <A as MemoryPool>::Alloc: Send+Sync
    {
        let layout0 = pipeline.descriptor_set_layout(0).unwrap();
        let set0 = Arc::new(PersistentDescriptorSet::start(layout0.clone())
            .add_sampled_image(self.texture.texture.clone(), self.sampler.clone()).unwrap()
            .build().unwrap()
        );

        let layout1 = pipeline.descriptor_set_layout(1).unwrap();
        let set1 = Arc::new(PersistentDescriptorSet::start(layout1.clone())
            .add_buffer(sub_buf.clone()).unwrap()
            .build().unwrap()
        );
        vec![set0, set1]
    }
}

impl Mesh for Cube {
    type Vertex = CubeVtx;

    fn pipeline(&self,
                device: Arc<Device>,
                render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                images: &Vec<Arc<SwapchainImage<Window>>>)
                -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let dimensions = images[0].dimensions();

        Arc::new(GraphicsPipeline::start()
            .vertex_input_single_buffer::<Self::Vertex>()
            .vertex_shader(self.vtx_shader.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            }))
            .fragment_shader(self.frg_shader.main_entry_point(), ())
            .cull_mode_front()  // face culling for optimization
            .alpha_to_coverage_enabled()
            .depth_stencil_simple_depth()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone()).unwrap())
    }

    fn onload_data(&mut self, chunk: ChunkID, position: [f32; 3], block_data: &Vec<Block>) {
        let start = [
            position.clone()[0] as usize,
            position.clone()[1] as usize,
            position.clone()[2] as usize
        ];
        let end = [
            position.clone()[0] as usize+CHUNK_SIZE-1,
            position.clone()[1] as usize+CHUNK_SIZE-1,
            position.clone()[2] as usize+CHUNK_SIZE-1
        ];
        println!("START {:?}; END {:?}", start, end);

        let mut vertices = Vec::with_capacity(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE*VERT_PER_CUBE as usize);
        let mut indices: Vec<u32> = Vec::with_capacity(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE*IND_PER_CUBE as usize);

        let get_loc = |x, y, z| (x%CHUNK_SIZE)*CHUNK_SIZE*CHUNK_SIZE+(y%CHUNK_SIZE)*CHUNK_SIZE+(z%CHUNK_SIZE);  // the block location on the data

        for x in start[0]..=end[0] {
            for y in start[1]..=end[1] {
                for z in start[2]..=end[2] {
                    let block: Block = block_data[get_loc(x, y, z)].clone();

                    if block.mesh == MeshType::Cube {
                        let mut faces = 0u8;

                        // if if (1st: checks chunk border) {true} else {2nd: checks for nearby transparent block}
                        if if start[0] == x {true} else {block_data[get_loc(x-1, y, z)].clone().transparent && !block.transparent} {  // left face
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[2][3], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[2][0], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[2][1], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[2][2], });
                            faces += 1;
                        }
                        if if start[1] == y {true} else {(block_data[get_loc(x, y-1, z)].clone().transparent && !block.transparent)} {  // bottom face
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[1][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[1][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[1][2], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[1][3], });
                            faces += 1;
                        }
                        if if start[2] == z {true} else {(block_data[get_loc(x, y, z-1)].clone().transparent && !block.transparent)} {  // front face
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][2], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][3], });
                            faces += 1;
                        }
                        if if end[0] == x {true} else {(block_data[get_loc(x+1, y, z)].clone().transparent && !block.transparent)} {  // right face
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[3][3], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[3][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[3][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[3][2], });
                            faces += 1;
                        }
                        if if end[1] == y {true} else {(block_data[get_loc(x, y+1, z)].clone().transparent && !block.transparent)} {  // top face
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[0][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[0][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[0][2], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[0][3], });
                            faces += 1;
                        }
                        if if end[2] == z {true} else {(block_data[get_loc(x, y, z+1)].clone().transparent && !block.transparent)} {  // back face
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][2], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][3], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][0], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][1], });
                            faces += 1;
                        }

                        // if x < end[0] && y < end[1] && z < end[2] {  // Upper bound
                        //
                        //     if start[0] == x || (block_data[get_loc(x-1, y, z)].clone().transparent && !block.transparent) {  // left face
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[2][3], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[2][0], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[2][1], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[2][2], });
                        //         faces += 1;
                        //     }
                        //     if start[1] == y || (block_data[get_loc(x, y-1, z)].clone().transparent && !block.transparent) {  // bottom face
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[1][0], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[1][1], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[1][2], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[1][3], });
                        //         faces += 1;
                        //     }
                        //     if start[2] == z || (block_data[get_loc(x, y, z-1)].clone().transparent && !block.transparent) {  // front face
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][0], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][1], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][2], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][3], });
                        //         faces += 1;
                        //     }
                        // }
                        //
                        // if x > start[0] && y > start[1] && z > start[2] {  // Lower bound
                        //     if end[0] == x || (block_data[get_loc(x+1, y, z)].clone().transparent && !block.transparent) {  // right face
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[3][3], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[3][0], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[3][1], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[3][2], });
                        //         faces += 1;
                        //     }
                        //     if end[1] == y || (block_data[get_loc(x, y+1, z)].clone().transparent && !block.transparent) {  // top face
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[0][0], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[0][1], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[0][2], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[0][3], });
                        //         faces += 1;
                        //     }
                        //     if end[2] == z || (block_data[get_loc(x, y, z+1)].clone().transparent && !block.transparent) {  // back face
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][2], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][3], });
                        //         vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][0], });
                        //         vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][1], });
                        //         faces += 1;
                        //     }
                        // }

                        for _ in 0..faces {
                            if indices.is_empty() {
                                indices.append(
                                    &mut vec![
                                        0, 1, 2,
                                        0, 2, 3,
                                    ]
                                )
                            } else {
                                indices.append(
                                    &mut vec![
                                        0, 1, 2,
                                        0, 2, 3,
                                    ].iter().map(|&x| x+*indices.last().unwrap() as u32+1).collect()
                                )
                            }
                        }
                    }
                }
            }
        }

        self.chunk_data.push((chunk, vertices, indices));
    }

    fn offload_chunk(&self, chunk: &Chunk) {

    }

    fn retrieve_vert(&mut self, chunk_data: &Vec<Chunk>) -> Vec<Self::Vertex> {
        println!("Chunk datas in Cube Mesh: {:?}", self.chunk_data.len());
        let mut vtx_data = Vec::new();

        for (chunk, vertices, _indices) in self.chunk_data.iter() {
            if chunk_data[chunk.0 as usize].visible { // TODO: safety check and map ID's when offloading chunk is used
                vtx_data.extend(vertices.iter());
            }
        }
        println!("Vertices retrieved: {:?}", vtx_data.len());
        vtx_data
    }

    fn retrieve_ind(&mut self, chunk_data: &Vec<Chunk>) -> Vec<u32> {
        // TODO: index can be pre-computed on the run without hassling chaing the indexes since chunk visibility varies
        let mut ind_data: Vec<u32> = Vec::new();
        let mut index: u32 = 0;

        for (chunk, _vertices, indices) in self.chunk_data.iter() {
            if chunk_data[chunk.0 as usize].visible {
                if ind_data.is_empty() {
                    ind_data.extend(
                        indices.iter()
                    );
                } else {
                    let ind_index = (*ind_data.last().unwrap()).clone();
                    ind_data.extend(
                        indices.iter().map(|&x| x+ind_index+1)
                    );
                }
                index += 1;
            }
        }
        ind_data
    }
}
