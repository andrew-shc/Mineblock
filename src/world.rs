use crate::chunk::{Chunk, ChunkID};
use crate::chunk::CHUNK_SIZE;
use crate::camera::Camera;
use crate::camera::CHUNK_RADIUS;
use crate::texture::TextureAtlas;
use crate::renderer::CubeVtx;
use crate::terrain::Terrain;
use crate::mesh::mesh::Meshes;

use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::pipeline::{GraphicsPipelineAbstract};
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
    pub name: String,
    meshes: Rc<RefCell<Meshes>>,
    terrain: Terrain,
    chunks: Vec<Chunk>,
    loaded_chunks: Vec<ChunkID>,
}

impl World {
    // create a new world
    pub fn new(name: String, device: Arc<Device>, queue: Arc<Queue>, txtr: Rc<TextureAtlas>) -> Self {
        World {
            name: name,
            meshes: Rc::new(RefCell::new(Meshes::new(device.clone(), txtr.clone()))),
            terrain: Terrain::new(txtr.clone()),

            chunks: Vec::new(),
            loaded_chunks: Vec::new(),
        }
    }

    // instantiate the world
    pub fn instantiate(&mut self) {
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

        // let ply_pos: [u32; 3] = camera.chunk_pos().into();
        //
        // for x in 0..CHUNK_RADIUS as u32*2 {
        //     for y in 0..CHUNK_RADIUS as u32*2 {
        //         for z in 0..CHUNK_RADIUS as u32*2 {
        //             let pos = [
        //                 ply_pos[0]+x-CHUNK_RADIUS as u32,
        //                 ply_pos[1]+y-CHUNK_RADIUS as u32,
        //                 ply_pos[2]+z-CHUNK_RADIUS as u32,
        //             ];
        //             if pos[0] > 0 && pos[1] > 0 && pos[2] > 0 {
        //                 self.load_chunk(pos);
        //             }
        //         }
        //     }
        // }
    }

    // update the world
    pub fn update<T>(&mut self, camera: &Camera<T>) -> Option<u32> {
        // block position update

        let ply_pos = [camera.chunk_pos()[0] as u32, camera.chunk_pos()[1] as u32, camera.chunk_pos()[2] as u32];

        let mut chunk_loaded = 0;
        for x in 0..CHUNK_RADIUS as u32*2 {
            for y in 0..CHUNK_RADIUS as u32*2 {
                for z in 0..CHUNK_RADIUS as u32*2 {
                    if  ply_pos[0] as i32+x as i32-CHUNK_RADIUS as i32 > 0 &&
                        ply_pos[1] as i32+y as i32-CHUNK_RADIUS as i32 > 0 &&
                        ply_pos[2] as i32+z as i32-CHUNK_RADIUS as i32 > 0 {
                        if self.load_chunk([
                            (ply_pos[0]+x-CHUNK_RADIUS as u32),
                            (ply_pos[1]+y-CHUNK_RADIUS as u32),
                            (ply_pos[2]+z-CHUNK_RADIUS as u32),
                        ]) {
                            chunk_loaded += 1;
                        }
                    }
                }
            }
        }

        // lighting update
        // etc ...
        if chunk_loaded == 0 {
            None
        } else {
            Some(chunk_loaded)
        }
    }

    pub fn load_chunk(&mut self, chunk_pos: [u32; 3]) -> bool {  // returns if the chunk loaded successfully
        let new_id = ChunkID(chunk_pos[0],chunk_pos[1],chunk_pos[2]);
        if !self.loaded_chunks.contains(&new_id) {
            let position = [chunk_pos[0]*CHUNK_SIZE as u32, chunk_pos[1]*CHUNK_SIZE as u32, chunk_pos[2]*CHUNK_SIZE as u32];
            let chunk  = Chunk::new(new_id, position, self.terrain.generate( &position, CHUNK_SIZE));  // &[0,0,0] <- to repeat same terrain generation @ [0,0,0] for each chunk
            chunk.render(self.meshes.clone());

            self.loaded_chunks.push(chunk.id);
            self.chunks.push(chunk);
            true
        } else {
            false
        }
    }

    pub fn offload_chunk() {

    }


    pub fn mesh_datas(&mut self, device: Arc<Device>) -> Vec<(Arc<CpuAccessibleBuffer<[CubeVtx]>>, Arc<CpuAccessibleBuffer<[u32]>>)> {
        (*self.meshes).borrow_mut().retrieve_data(device.clone(), &self.chunks)
    }

    pub fn mesh_pipelines(&mut self,
                          device: Arc<Device>,
                          render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                          dimensions: [u32; 2]
    ) -> Vec<Arc<dyn GraphicsPipelineAbstract + Send + Sync>> {
        // TODO: this code smells
        (*self.meshes).borrow_mut().retrieve_pipeline(device.clone(), render_pass.clone(), dimensions)
    }

    pub fn cube_sets<'b, U: Send+Sync+'b, A: MemoryPool+Sync+'b>(&self, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>, sub_buf: &CpuBufferPoolSubbuffer<U, A>) -> Vec<Arc<dyn DescriptorSet+Send+Sync+'b>>
        where <A as MemoryPool>::Alloc: Send+Sync {
        (*self.meshes).borrow_mut().cube_sets(pipeline.clone(), &sub_buf.clone())
    }
}

/*
PROGRAM - BEGIN INITIALIZATION
PROGRAM - BEGIN MAIN PROGRAM
Input size constant pre-check: 32 Blocks
Terrain size allocated: 32768 Blocks
CHUNK VISIBLE true
Vertices retrieved: 24576
CHUNK VISIBLE IND: true, BASE INDEX: 0
PROGRAM - START MAIN LOOP
Number of vertices rendering: 24576
 */