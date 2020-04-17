use crate::chunk::Chunk;
use crate::texture::TextureAtlas;
use crate::mesh::Cube;
use crate::mesh::CubeFace;
use crate::mesh::Mesh;
use crate::renderer::CubeVtx;
use crate::block::Block;

use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::pipeline::{GraphicsPipelineAbstract};
use vulkano::command_buffer::{CommandBufferExecFuture, AutoCommandBuffer};
use vulkano::sync::NowFuture;
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::image::{SwapchainImage};
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;
use vulkano::memory::MemoryPool;
use vulkano::descriptor::DescriptorSet;

use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

use winit::window::Window;

// world file
// where block storage and block creation happen
// and also where would the world state go
// and terrain generation (maybe separate file)

// height limit ~= 512 Block

pub struct World {
    name: String,
    cube_mesh: Rc<RefCell<Cube>>
}

impl World {
    // create a new world
    pub fn new(name: String, device: Arc<Device>, queue: Arc<Queue>) -> (Self, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
        let (txtr, future) = TextureAtlas::load(queue.clone(), include_bytes!("../resource/texture/texture1.png").to_vec(), 16);
        let cube_mesh = Rc::new(RefCell::new(Cube::new(device.clone(), txtr)));
        // cube_mesh.borrow_mut().update_vert()

        (
            World {
                name: name,
                cube_mesh: cube_mesh,
            },
            future
        )
    }

    // instantiate the world
    pub fn instantiate(&self, device: Arc<Device>) -> (Arc<CpuAccessibleBuffer<[CubeVtx]>>, Arc<CpuAccessibleBuffer<[u32]>>) {
        // skybox
        // sun/moon/stars

        // Cube Renderer
        // Flora Renderer

        // After affects
        // - fog
        // - vignette

        // try preloading the texture before
        // + also add defualt texture loading error when no texture is available
        //      ^- add default texture before the loop
        let mut dirt = Block::new(self.cube_mesh.clone(), String::from("dirt"), vec![[2,0], [2,0], [2,0], [2, 0], [2,0], [2,0]], 0);
        let mut grass = Block::new(self.cube_mesh.clone(), String::from("grass"), vec![[0,0], [2,0], [1,0], [1,0], [1,0], [1,0]], 0);

        let world_size = 32;

        for x in 0..world_size {
            for y in 0..world_size {
                for z in 0..world_size {
                    if y < world_size-1-1 {
                        dirt.create(x*world_size*world_size+y*world_size+z,
                                    [x as f32, y as f32, z as f32]);
                    } else {
                        grass.create(x*world_size*world_size+y*world_size+z,
                                    [x as f32, y as f32, z as f32]);
                    }
                }
            }
        }

        let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(),
                                                           BufferUsage::vertex_buffer(), false, self.cube_mesh.borrow_mut().vertex.clone().into_iter()).unwrap();

        let index_buffer = CpuAccessibleBuffer::from_iter(device.clone(),
                                                          BufferUsage::index_buffer(), false, self.cube_mesh.borrow_mut().index.clone().into_iter()).unwrap();

        (vertex_buffer, index_buffer)
    }

    // re-render the world
    pub fn render(&self, device: Arc<Device>,
                  render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                  images: &Vec<Arc<SwapchainImage<Window>>>) -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        self.cube_mesh.borrow_mut().pipeline(device, render_pass, images)
    }

    // TEMPORARY
    pub fn buffers<'a, U: Send+Sync+'a, A: MemoryPool+Send+Sync+'a>(&self, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>, sub_buf: &CpuBufferPoolSubbuffer<U, A>) -> Vec<Vec<Arc<dyn DescriptorSet+Send+Sync+'a>>>
        where <A as MemoryPool>::Alloc: Send+Sync
    {
        vec![self.cube_mesh.borrow_mut().descriptors(pipeline, sub_buf)]
    }

    // update the world
    pub fn update() {
        // block position update


        // lighting update
        // etc ...
    }
}
