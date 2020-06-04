use crate::world::World;
use crate::mesh::cube::vs;
use crate::camera::Camera;
use crate::texture::TextureAtlas;
use crate::ui::UIContext;

use vulkano;
use vulkano::device::{Device, Queue};
use vulkano::image::{AttachmentImage, SwapchainImage};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::{sync, swapchain};
use vulkano::sync::{GpuFuture, FlushError};
use winit::window::Window;
use vulkano::swapchain::{Swapchain, SurfaceTransform, ColorSpace, PresentMode, FullscreenExclusive, SwapchainCreationError, AcquireError, Surface};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use vulkano::buffer::{CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipelineAbstract;

use std::{fmt, thread};
use std::rc::Rc;
use std::sync::{Arc, mpsc};


pub trait Vertex {}


#[derive(Default, Copy, Clone)]
pub struct CubeVtx {  // rename: TxtrVtx
    pub position: [f32; 3],
    pub txtr_crd: [f32; 2],
}

#[derive(Default, Copy, Clone)]
pub struct UIVtx {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex for CubeVtx {}
impl Vertex for UIVtx {}


impl fmt::Debug for CubeVtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CubeVtx")
            .field("position", &self.position)
            .field("txtr_crd", &self.txtr_crd)
            .finish()
    }
}

impl fmt::Debug for UIVtx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextVtx")
            .field("position", &self.position)
            .field("color", &self.color)
            .finish()
    }
}

vulkano::impl_vertex!(CubeVtx, position, txtr_crd);
vulkano::impl_vertex!(UIVtx, position, color);


// a place where all the render data goes
pub struct Render {
    previous_frame: Option<Box<dyn GpuFuture>>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,  // swapchain images
    framebuffer: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pipeline: Vec<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>,

    recreate: bool, // recreate swapchain
    vertices: Arc<CpuAccessibleBuffer<[CubeVtx]>>,
    indices: Arc<CpuAccessibleBuffer<[u32]>>,
    pub ui: UIContext,

    textures: Vec<Rc<TextureAtlas>>,

    pub world: World,
    pub cam: Camera<vs::ty::Matrix>,
}

impl Render {
    pub fn new(physical: PhysicalDevice, device: Arc<Device>, queue: Arc<Queue>, surface: Arc<Surface<Window>>) -> Self {
        let caps = surface.capabilities(physical)
            .expect("failed to get surface capabilities");

        let dimensions = caps.current_extent.unwrap_or([1280, 1280]);
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let (swapchain, images) = Swapchain::new(
            device.clone(), surface.clone(), caps.min_image_count, format, dimensions, 1,
            caps.supported_usage_flags, &queue, SurfaceTransform::Identity, alpha, PresentMode::Fifo,
            FullscreenExclusive::Default,  true, ColorSpace::SrgbNonLinear)
            .expect("failed to create swapchain");

        let renderpass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        ).unwrap());

        let (txtr, future) = TextureAtlas::load(queue.clone(), include_bytes!("../resource/texture/texture2.png").to_vec(), 16);

        let cam = Camera::new(device.clone(), 0.1, 0.125);
        let mut world = World::new(String::from("World 0"), device.clone(), queue.clone(), txtr.clone());
        // world.instantiate();

        let mut mesh_data = world.mesh_datas(device.clone());
        let (v, i) = mesh_data.pop().unwrap();

        Self {
            previous_frame: Some(Box::new(sync::now(device.clone()).join(future)) as Box<dyn GpuFuture>),
            swapchain: swapchain,
            images: images.clone(),
            framebuffer: Self::frames(device.clone(), &images, renderpass.clone()),
            renderpass: renderpass.clone(),

            recreate: false,
            pipeline: world.mesh_pipelines(device.clone(), renderpass.clone(), dimensions),
            vertices: v,
            indices: i,
            ui: UIContext::new(device.clone()),

            textures: vec![
                (txtr.clone()),
            ],

            world: world,
            cam: cam,
        }
    }

    pub fn update(&mut self, device: Arc<Device>, queue: Arc<Queue>, dimensions: [u32; 2]) {
        // cleans the buffer
        self.previous_frame.as_mut().unwrap().cleanup_finished();

        if let Some(chunk_loaded) = self.world.update(&self.cam) {
            let mut mesh_data = self.world.mesh_datas(device.clone());
            let (v, i) = mesh_data.pop().unwrap();
            self.vertices = v;
            self.indices = i;
        }

        if self.recreate {
            println!("CREATE AGAIN {:?}", dimensions);

            let (new_swapchain, new_images) = match self.swapchain.recreate_with_dimensions(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e)
            };
            self.swapchain = new_swapchain;

            // recreate the framebuffer after recreating swapchain
            self.framebuffer = Self::frames(device.clone(), &new_images, self.renderpass.clone());
            self.pipeline = self.world.mesh_pipelines(device.clone(), self.renderpass.clone(), dimensions);

            self.recreate = false;
        }

        let sub_buf = self.cam.mat_buf(dimensions);
        let sets = self.world.cube_sets(self.pipeline[0].clone(), &sub_buf);

        let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate = true;
                return;
            },
            Err(e) => panic!("Failed to acquire next image: {:?}", e)
        };

        if suboptimal { self.recreate = true; }

        println!("Number of vertices rendering: {:?}", self.vertices.clone().len());

        let (vbo, ibo) = self.ui.render(device.clone());
        let ui_pipeline = self.ui.pipeline(device.clone(), dimensions, self.renderpass.clone());

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(self.framebuffer[image_num].clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_indexed(self.pipeline[0].clone(), &DynamicState::none(), vec!(self.vertices.clone()), self.indices.clone(), sets.clone(), ()).unwrap()
            .draw_indexed(ui_pipeline.clone(), &DynamicState::none(), vec!(vbo.clone()), ibo.clone(), (), ()).unwrap()
            .end_render_pass().unwrap()
            .build().unwrap();

        let future = self.previous_frame.take().unwrap()
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            // submits present command to the GPU to the end of queue
            .then_swapchain_present(queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();


        match future {
            Ok(fut) => {
                self.previous_frame = Some(Box::new(fut) as Box<_>);
            },
            Err(FlushError::OutOfDate) => {
                self.recreate = true;
                self.previous_frame = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
        }
    }

    fn frames(device: Arc<Device>,
              images: &Vec<Arc<SwapchainImage<Window>>>,
              render_pass: Arc<dyn RenderPassAbstract + Send + Sync>
    ) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
        let dimensions = images[0].dimensions();

        let depth_buffer = AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

        images.iter().map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone()).unwrap()
                    .add(depth_buffer.clone()).unwrap()
                    .build().unwrap()
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        }).collect::<Vec<_>>()
    }
}


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