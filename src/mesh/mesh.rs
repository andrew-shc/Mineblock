use crate::renderer;
use crate::renderer::CubeVtx;
use crate::block::Block;
use crate::chunk::{Chunk, ChunkID};

use vulkano::device::Device;
use vulkano::image::{SwapchainImage};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, cpu_pool::CpuBufferPoolSubbuffer};
use vulkano::descriptor::DescriptorSet;
use vulkano::memory::MemoryPool;
use winit::window::Window;

use std::rc::Rc;
use std::sync::Arc;


#[derive(Eq, PartialEq, Copy, Clone)]
pub enum MeshType {
    Cube,  // 6 side cube
    Flora,  // x-shape
    // Particle
    // Custom
}

pub trait Mesh {
    type Vertex: 'static;

    // TODO: move pipeline to the world struct
    fn pipeline(&self, device: Arc<Device>,
                render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                images: &Vec<Arc<SwapchainImage<Window>>>)
        -> Arc<dyn GraphicsPipelineAbstract + Send + Sync>;  // returns the graphic pipeline of that mesh
    fn onload_data(&mut self, chunk: ChunkID, position: [f32; 3], block_data: &Vec<Block>);  // updates the vertex data
    fn offload_chunk(&self, chunk: &Chunk);
    fn retrieve_vert(&mut self, chunk_data: &Vec<Chunk>) -> Vec<Self::Vertex>;
    fn retrieve_ind(&mut self, chunk_data: &Vec<Chunk>) -> Vec<u32>;
}

use crate::mesh::cube::Cube;
// use crate::mesh::flora::Flora;
use crate::texture::TextureAtlas;

pub struct Meshes {
    cube: Cube,
    // flora: M,
}

impl Meshes {
    pub fn new(device: Arc<Device>, txtr: Rc<TextureAtlas>) -> Self {
        Self {
            cube: Cube::new(device.clone(), txtr.clone()),
            // flora: Flora,
        }
    }

    pub fn onload_data(&mut self, chunk: ChunkID, position: [f32; 3], block_data: &Vec<Block>) {
        println!("ONLOADED {:?}", position);
        self.cube.onload_data(chunk, position, block_data)
    }

    pub fn retrieve_data(&mut self, device: Arc<Device>, chunk_data: &Vec<Chunk>) -> Vec<(Arc<CpuAccessibleBuffer<[CubeVtx]>>, Arc<CpuAccessibleBuffer<[u32]>>)> {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(),
                                                           BufferUsage::vertex_buffer(), false, self.cube.retrieve_vert(chunk_data).into_iter()).unwrap();

        let index_buffer = CpuAccessibleBuffer::from_iter(device.clone(),
                                                          BufferUsage::index_buffer(), false, self.cube.retrieve_ind(chunk_data).into_iter()).unwrap();

        vec![(vertex_buffer, index_buffer)]
    }

    pub fn retrieve_pipeline(&self,
                             device: Arc<Device>,
                             render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                             images: &Vec<Arc<SwapchainImage<Window>>>
    ) -> Vec<Arc<dyn GraphicsPipelineAbstract + Send + Sync>> {
        vec![self.cube.pipeline(device.clone(),render_pass.clone(), images)]
    }

    pub fn cube_sets<'b, U: Send+Sync+'b, A: MemoryPool+Sync+'b>(&self, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>, sub_buf: &CpuBufferPoolSubbuffer<U, A>) -> Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>
    where <A as MemoryPool>::Alloc: Send+Sync {
        self.cube.descriptors(pipeline.clone(), &sub_buf.clone())
    }
}
