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
use std::sync::Arc;
use std::iter;
use winit::window::Window;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;



// Flora Mesh
// - 2 x shape
// - both of the two shapes are perpendicular to each other
// - rotated 45 deg to differentiate itself from blocks

pub mod vs { vulkano_shaders::shader!{ty: "vertex", path: "resource/shaders/cube.vert",} }
pub mod fs { vulkano_shaders::shader!{ty: "fragment", path: "resource/shaders/cube.frag",} }

pub struct Flora {
    pub texture: TextureAtlas,  // texture image
    chunk_data: Vec<(ChunkID, Vec<<Flora as Mesh>::Vertex>, Vec<u32>)>,
    // pub index: Vec<u32>,
    sampler: Arc<Sampler>,  // texture sampler
    vtx_shader: vs::Shader,
    frg_shader: fs::Shader,
}

impl Flora {
    pub fn new(device: Arc<Device>, texture: TextureAtlas) -> Flora {
        // Filter::Nearest for rendering each pixel instead of "smudging" between the adjacent pixels
        let sampler = Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest,
                                   MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                                   SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();

        Flora { texture: texture, sampler: sampler, chunk_data: Vec::new(),
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

impl Mesh for Flora {
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
        let start = position.clone();
        let end = [position[0]+CHUNK_SIZE as f32-1.0, position[1]+CHUNK_SIZE as f32-1.0, position[2]+CHUNK_SIZE as f32-1.0];

        let mut vertices = Vec::with_capacity(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE*6*4);
        let mut indices: Vec<u32> = Vec::with_capacity(CHUNK_SIZE*CHUNK_SIZE*CHUNK_SIZE*6*6);

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let index = x*CHUNK_SIZE*CHUNK_SIZE+y*CHUNK_SIZE+z;  // the block location on the data
                    let block = block_data[index];
                    if block.mesh == MeshType::Cube {
                        let top = self.texture.texture_coord(block.texture[0][0],block.texture[0][1]);
                        let bottom = self.texture.texture_coord(block.texture[1][0],block.texture[1][1]);
                        let left = self.texture.texture_coord(block.texture[2][0],block.texture[2][1]);
                        let right = self.texture.texture_coord(block.texture[3][0],block.texture[3][1]);
                        let front = self.texture.texture_coord(block.texture[4][0],block.texture[4][1]);
                        let back = self.texture.texture_coord(block.texture[5][0],block.texture[5][1]);

                        if start[0] == position[0] {
                            vertices.push(Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: left[3], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: left[0], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: left[1], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: left[2], });
                        }
                        if start[1] == position[1] {
                            vertices.push(Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: bottom[0], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: bottom[1], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: bottom[2], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: bottom[3], });
                        }
                        if start[2] == position[2] {
                            vertices.push(Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: front[0], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: front[1], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: front[2], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: front[3], });
                        }
                        if end[0] == position[0] {
                            vertices.push(Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: right[3], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: right[0], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: right[1], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: right[2], });
                        }
                        if end[1] == position[1] {
                            vertices.push(Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: top[0], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: top[1], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: top[2], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: top[3], });
                        }
                        if end[2] == position[2] {
                            vertices.push(Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: back[2], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: back[3], });
                            vertices.push(Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: back[0], });
                            vertices.push(Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: back[1], });
                        }

                        indices.append(
                            &mut vec![
                                0, 1, 2, 0, 2, 3,
                                4, 5, 6, 4, 6, 7,
                                8, 9,10, 8,10,11,
                                12,13,14,12,14,15,
                                16,17,18,16,18,19,
                                20,21,22,20,22,23,].iter().map(|&x| x+(index*6*4) as u32).collect()
                        );
                    }
                }
            }
        }

        self.chunk_data.push((chunk, vertices, indices));
    }

    fn offload_chunk(&self, chunk: &Chunk) {

    }

    fn retrieve_vert(&mut self, chunk_data: &Vec<Chunk>) -> Vec<Self::Vertex> {
        let mut vtx_data = Vec::new();

        for (chunk, vertices, _indices) in self.chunk_data.iter() {
            if chunk_data[chunk.0 as usize].visible {
                vtx_data.append(&mut vertices.clone());
            }
        }
        vtx_data
    }

    fn retrieve_ind(&mut self, chunk_data: &Vec<Chunk>) -> Vec<u32> {
        // TODO: index can be pre-computed on the run without hassling chaing the indexes since chunk visibility varies
        let mut ind_data = Vec::new();
        let mut index = 0;

        for (chunk, _vertices, indices) in self.chunk_data.iter() {
            if chunk_data[chunk.0 as usize].visible {
                ind_data.append(
                    &mut indices.iter().map(|&x| x+(&index*6*4)).collect()
                );
                index += 1;
            }
        }
        ind_data
    }
}
