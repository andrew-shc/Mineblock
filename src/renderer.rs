use crate::mesh::MeshType;

use vulkano;
use std::fmt;
use std::sync::Arc;
use vulkano::device::{Device, Queue};
use vulkano::image::{AttachmentImage, SwapchainImage};
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract};
use vulkano::{sync, swapchain};
use vulkano::sync::{GpuFuture, FlushError};
use winit::window::Window;
use vulkano::swapchain::{Swapchain, SurfaceTransform, ColorSpace, PresentMode, FullscreenExclusive, SwapchainCreationError, AcquireError, Surface};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use crate::world::World;
use crate::mesh::vs;
use vulkano::format::Format;
use vulkano::framebuffer::Framebuffer;
use crate::camera::Camera;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipelineAbstract;


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


// a place where all the render data goes
pub struct Render {
    previous_frame: Option<Box<dyn GpuFuture>>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,  // swapchain images
    framebuffer: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    renderpass: Arc<dyn RenderPassAbstract + Send + Sync>,
    pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,

    recreate: bool, // recreate swapchain
    vertices: Arc<CpuAccessibleBuffer<[CubeVtx]>>,
    indices: Arc<CpuAccessibleBuffer<[u32]>>,

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

        let (world, future) = World::new(String::from("World 0"), device.clone(), queue.clone());
        let (vertex_buffer, index_buffer) = world.instantiate(device.clone());

        Self {
            previous_frame: Some(Box::new(sync::now(device.clone()).join(future)) as Box<dyn GpuFuture>),
            swapchain: swapchain,
            images: images.clone(),
            framebuffer: Self::frames(device.clone(), &images, renderpass.clone()),
            renderpass: renderpass.clone(),
            pipeline: world.render(device.clone(), renderpass.clone(), &images),

            recreate: false,
            vertices: vertex_buffer,
            indices: index_buffer,

            world: world,
            cam: Camera::new(device.clone(), 0.1, 0.125),
        }
    }

    pub fn update(&mut self, device: Arc<Device>, queue: Arc<Queue>, dimensions: [u32; 2],) {
        // cleans the buffer
        self.previous_frame.as_mut().unwrap().cleanup_finished();

        if self.recreate {
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
            self.pipeline = self.world.render(device.clone(), self.renderpass.clone(), &new_images);

            self.recreate = false;
        }

        // the closer the znear is to 0, the worse the depth buffering would perform
        // let proj = perspective (Rad::from(Deg(60.0)), dimensions[0] as f32/dimensions[1] as f32, 0.1 , 1000.0);
        // let mut view = Matrix4::from_angle_x(rotation.x) * Matrix4::from_angle_y(rotation.y) *
        //     Matrix4::look_at(Point3::new(position.x, position.y, -1.0+position.z), position, Vector3::new(0.0, -1.0, 0.0));
        // let mut world = Matrix4::identity();
        //
        // let sub_buf = matrix_buffer.next(
        //     vs::ty::Matrix {proj: proj.into(), view: view.into(), world: world.into()}
        // ).unwrap();

        let sub_buf = self.cam.mat_buf(dimensions);
        let buffers = self.world.buffers(self.pipeline.clone(), &sub_buf);

        let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(self.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                self.recreate = true;
                return;
            },
            Err(e) => panic!("Failed to acquire next image: {:?}", e)
        };

        if suboptimal { self.recreate = true; }

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(self.framebuffer[image_num].clone(), false, vec![[0.1, 0.3, 1.0, 1.0].into(), 1f32.into()]).unwrap()
            .draw_indexed(self.pipeline.clone(), &DynamicState::none(), vec!(self.vertices.clone()), self.indices.clone(), buffers[0].clone(), ()).unwrap()
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