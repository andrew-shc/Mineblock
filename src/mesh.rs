use crate::texture::TextureAtlas;
use crate::renderer::CubeVtx;

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


pub enum MeshType {
    Cube,  // 6 side cube
    Flora,  // x-shape
    // Particle
    // Custom
}

pub trait Mesh {
    type Vertex: 'static;

    fn pipeline(&self, device: Arc<Device>,
                render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
                images: &Vec<Arc<SwapchainImage<Window>>>)
        -> Arc<dyn GraphicsPipelineAbstract + Send + Sync>;  // returns the graphic pipeline of that mesh
    fn update_vert(&mut self, txtr_coord: &Vec<[u16; 2]>, position: [f32; 3]);  // updates the vertex data
    fn update_ind(&mut self, index: u32);  // updates the index data
}


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
    pub texture: TextureAtlas,  // texture image
    pub vertex: Vec<<Cube as Mesh>::Vertex>,
    pub index: Vec<u32>,
    sampler: Arc<Sampler>,  // texture sampler
    vtx_shader: vs::Shader,
    frg_shader: fs::Shader,
}

impl Cube {
    pub fn new(device: Arc<Device>, texture: TextureAtlas) -> Cube {
        // Filter::Nearest for rendering each pixel instead of "smudging" between the adjacent pixels
        let sampler = Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest,
                                   MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                                   SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();

        Cube { texture: texture, sampler: sampler, vertex: Vec::new(), index: Vec::new(),
            vtx_shader: vs::Shader::load(device.clone()).expect("failed to create cube vertex shaders module"),
            frg_shader: fs::Shader::load(device.clone()).expect("failed to create cube fragment shaders module")
        }
    }

    pub fn descriptors<'a, U: Send+Sync+'a, A: MemoryPool+Sync+'a>(&self, pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>, sub_buf: &CpuBufferPoolSubbuffer<U, A>) -> Vec<Arc<dyn DescriptorSet+Send+Sync+'a>>
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

    fn update_vert(&mut self, txtr_coord: &Vec<[u16; 2]>, position: [f32; 3]) {
        let top = self.texture.texture_coord(txtr_coord[0][0],txtr_coord[0][1]);
        let bottom = self.texture.texture_coord(txtr_coord[1][0],txtr_coord[1][1]);
        let left = self.texture.texture_coord(txtr_coord[2][0],txtr_coord[2][1]);
        let right = self.texture.texture_coord(txtr_coord[3][0],txtr_coord[3][1]);
        let front = self.texture.texture_coord(txtr_coord[4][0],txtr_coord[4][1]);
        let back = self.texture.texture_coord(txtr_coord[5][0],txtr_coord[5][1]);

        self.vertex.append(
            &mut vec![
                Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: front[0], },
                Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: front[1], },
                Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: front[2], },
                Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: front[3], },

                Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: back[2], },
                Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: back[3], },
                Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: back[0], },
                Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: back[1], },

                Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: bottom[0], },
                Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: bottom[1], },
                Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: bottom[2], },
                Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: bottom[3], },

                Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: top[0], },
                Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: top[1], },
                Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: top[2], },
                Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: top[3], },

                Self::Vertex { position: [0.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: left[3], },
                Self::Vertex { position: [0.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: left[0], },
                Self::Vertex { position: [0.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: left[1], },
                Self::Vertex { position: [0.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: left[2], },

                Self::Vertex { position: [1.0+position[0],0.0+position[1],0.0+position[2]], txtr_crd: right[3], },
                Self::Vertex { position: [1.0+position[0],1.0+position[1],0.0+position[2]], txtr_crd: right[0], },
                Self::Vertex { position: [1.0+position[0],1.0+position[1],1.0+position[2]], txtr_crd: right[1], },
                Self::Vertex { position: [1.0+position[0],0.0+position[1],1.0+position[2]], txtr_crd: right[2], },
            ]
        );
    }

    // index refers to the overall polygon
    fn update_ind(&mut self, index: u32) {
        self.index.append(
            &mut vec![  0, 1, 2, 0, 2, 3,
                   4, 5, 6, 4, 6, 7,
                   8, 9,10, 8,10,11,
                   12,13,14,12,14,15,
                   16,17,18,16,18,19,
                   20,21,22,20,22,23,].iter().map(|&x| x+(index*6*4)).collect()
        )
    }
}
