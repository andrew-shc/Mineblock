use crate::renderer::CubeVtx;
use crate::texture::TextureAtlas;
use crate::chunk::{CHUNK_SIZE, ChunkID};
use crate::chunk::Chunk;
use crate::renderer;
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
use std::borrow::BorrowMut;


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

        for x in start[0]..=end[0] {
            for y in start[1]..=end[1] {
                for z in start[2]..=end[2] {
                    let index = (x%CHUNK_SIZE)*CHUNK_SIZE*CHUNK_SIZE+(y%CHUNK_SIZE)*CHUNK_SIZE+(z%CHUNK_SIZE);  // the block location on the data
                    let block = block_data[index].clone();

                    if block.mesh == MeshType::Cube {
                        if start[0] == x {
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[2][3], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[2][0], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[2][1], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[2][2], });
                        }
                        if start[1] == y {
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[1][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[1][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[1][2], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[1][3], });
                        }
                        if start[2] == z {
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][2], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[4][3], });
                        }
                        if end[0] == x {
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[3][3], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[3][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[3][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[3][2], });
                        }
                        if end[1] == y {
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[0][0], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[0][1], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[0][2], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,0.0+z as f32], txtr_crd: block.texture_coord[0][3], });
                        }
                        if end[2] == z {
                            vertices.push(Self::Vertex { position: [0.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][2], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,0.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][3], });
                            vertices.push(Self::Vertex { position: [1.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][0], });
                            vertices.push(Self::Vertex { position: [0.0+x as f32,1.0+y as f32,1.0+z as f32], txtr_crd: block.texture_coord[5][1], });
                        }

                        indices.append(
                            &mut vec![
                                 0, 1, 2, 0, 2, 3,
                                 4, 5, 6, 4, 6, 7,
                                 8, 9,10, 8,10,11,
                                12,13,14,12,14,15,
                                16,17,18,16,18,19,
                                20,21,22,20,22,23,].iter().map(|&x| x+(index*VERT_PER_CUBE as usize) as u32).collect()
                        );
                    }
                }
            }
        }

        self.chunk_data.push((chunk, vertices, indices));
    }

    // // index refers to the overall polygon
    // fn onload_ind(&mut self, index: u32) {
    //     // println!("#> {:?}", self.index);
    //     self.index.append(
    //         &mut vec![
    //              0, 1, 2, 0, 2, 3,
    //              4, 5, 6, 4, 6, 7,
    //              8, 9,10, 8,10,11,
    //             12,13,14,12,14,15,
    //             16,17,18,16,18,19,
    //             20,21,22,20,22,23,].iter().map(|&x| x+(index*6*4)).collect()
    //     )
    // }

    fn offload_chunk(&self, chunk: &Chunk) {

    }

    fn retrieve_vert(&mut self, chunk_data: &Vec<Chunk>) -> Vec<Self::Vertex> {
        println!("Chunk datas in Cube Mesh: {:?}", self.chunk_data.len());
        let mut vtx_data = Vec::new();

        for (chunk, vertices, _indices) in self.chunk_data.iter() {
            println!("CHUNK VISIBLE {:?}", chunk_data[chunk.0 as usize].visible);
            if chunk_data[chunk.0 as usize].visible { // TODO: safety check and map ID's when offloading chunk is used
                println!("===== {:?}", vtx_data.len());
                println!("===== {:?}", vertices.len());
                // vtx_data.append(&mut vertices.clone());
                vtx_data.extend(vertices.iter());
                println!("----- {:?}", vtx_data.len());
            }
        }
        println!("Vertices retrieved: {:?}", vtx_data.len());
        vtx_data
    }

    fn retrieve_ind(&mut self, chunk_data: &Vec<Chunk>) -> Vec<u32> {
        // TODO: index can be pre-computed on the run without hassling chaing the indexes since chunk visibility varies
        let mut ind_data = Vec::new();
        let mut index: u32 = 0;

        for (chunk, _vertices, indices) in self.chunk_data.iter() {
            // println!("CHUNK VISIBLE IND: {:?}, BASE INDEX: {:?}", chunk_data[chunk.0 as usize].visible, &index*CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE*VERT_PER_CUBE as usize);
            if chunk_data[chunk.0 as usize].visible {
                // ind_data.append(
                //     &mut indices.iter().map(|&x| x+(&index*CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE*VERT_PER_CUBE as usize) as u32).collect()
                // );
                ind_data.extend(
                    indices.iter().map(|&x| x+(index*VERT_PER_CUBE))
                );
                index += 1;
            }
        }
        ind_data
    }
}
