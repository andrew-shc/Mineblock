#[allow(unused_imports)]
use vulkano::{
    instance::{Instance, InstanceExtensions, PhysicalDevice},
    device::{Device, DeviceExtensions, Features},
    buffer::{CpuAccessibleBuffer, BufferUsage},
    command_buffer::{AutoCommandBufferBuilder, CommandBuffer, AutoCommandBuffer, DynamicState},
    pipeline::{ComputePipeline, GraphicsPipeline, viewport::Viewport},
    sync::{GpuFuture, FlushError},
    sync,
    descriptor::{
        descriptor_set::PersistentDescriptorSet,
        pipeline_layout::{PipelineLayoutDesc, PipelineLayoutAbstract}
    },
    image::{Dimensions, StorageImage, ImageAccess, swapchain::SwapchainImage},
    format::{Format, ClearValue},
    framebuffer::{Framebuffer, Subpass, FramebufferAbstract, RenderPassAbstract},
    swapchain::{Swapchain, SurfaceTransform, PresentMode, AcquireError, FullscreenExclusive, ColorSpace, SwapchainCreationError},
    swapchain,

};

use std::sync::Arc;
use std::panic;

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use cgmath::prelude::*;
use cgmath::PerspectiveFov;
use cgmath::Deg;
use cgmath::Rad;

#[derive(Default, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position);


fn main() {
    // setup
    let instance= {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create instance")
    };

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");
    let (device, mut queues) = {
        let device_ext = vulkano::device::DeviceExtensions {
            khr_swapchain: true,
            .. vulkano::device::DeviceExtensions::none()
        };
        Device::new(physical, physical.supported_features(), &device_ext,
                    [(queue_family, 0.5)].iter().cloned()).expect("failed to create device")
    };
    let queue = queues.next().unwrap();

    let caps = surface.capabilities(physical)
        .expect("failed to get surface capabilities");

    let dimensions = caps.current_extent.unwrap_or([1280, 1280]);
    let alpha = caps.supported_composite_alpha.iter().next().unwrap();
    let format = caps.supported_formats[0].0;
    let (mut swapchain, images) = Swapchain::new(
        device.clone(), surface.clone(), caps.min_image_count, format, dimensions, 1,
        caps.supported_usage_flags, &queue, SurfaceTransform::Identity, alpha, PresentMode::Fifo,
        FullscreenExclusive::Default,  true, ColorSpace::SrgbNonLinear)
        .expect("failed to create swapchain");

    let proj = PerspectiveFov {fovy: Rad::from(Deg(60.0)), aspect: 16.0/9.0, near: 0.001, far: 1000.0};

    let vertex = vec![
        Vertex { position: [-0.5, -0.5, 1.0] },
        Vertex { position: [ 0.5, -0.5, 1.0] },
        Vertex { position: [ 0.5,  0.5, 1.0] },
        Vertex { position: [-0.5,  0.5, 1.0] }];

    let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(),
    false, vertex.into_iter()).unwrap();

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

    let mut dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [swapchain.dimensions()[0] as f32, swapchain.dimensions()[1] as f32],
            depth_range: 0.0 .. 1.0
        }]),
        .. DynamicState::none()
    };

    let mut framebuffer = frames(images, render_pass.clone(), &mut dynamic_state);

    mod vs {
        vulkano_shaders::shader!{ty: "vertex", path: "src/cube.vert",}
    }

    mod fs {
        vulkano_shaders::shader!{ty: "fragment", path: "src/cube.frag",}
    }

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer::<Vertex>()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_fan()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone()).unwrap());

    use winit::event_loop::ControlFlow;
    use winit::event::WindowEvent;
    use winit::event::Event;

    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);  // store previous submission of frame
    let mut recreate_swapchain = false;  // recreating the swapchain if the swapchain's screen was changed

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {*control_flow = ControlFlow::Exit},
            Event::RedrawEventsCleared => {
                // cleans the buffer
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                if recreate_swapchain {
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) = match swapchain.recreate_with_dimensions(dimensions) {
                        Ok(r) => r,
                        // This error tends to happen when the user is manually resizing the window.
                        // Simply restarting the loop is the easiest way to fix this issue.
                        Err(SwapchainCreationError::UnsupportedDimensions) => return,
                        Err(e) => panic!("Failed to recreate swapchain: {:?}", e)
                    };
                    swapchain = new_swapchain;

                    // recreate the framebuffer after recreating swapchain
                    framebuffer = frames(new_images, render_pass.clone(), &mut dynamic_state);
                    recreate_swapchain = false;
                }

                let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    },
                    Err(e) => panic!("Failed to acquire next image: {:?}", e)
                };

                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
                    .begin_render_pass(framebuffer[image_num].clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()]).unwrap()
                    .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ()).unwrap()
                    .end_render_pass().unwrap()
                    .build().unwrap();

                let future = previous_frame_end.take().unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer).unwrap()
                    // submits present command to the GPU to the end of queue
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(Box::new(future) as Box<_>);
                    },
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
                    }
                }


            },
            _ => {*control_flow = ControlFlow::Wait},
        }
    });
}

fn frames(images: Vec<Arc<SwapchainImage<Window>>>,
             render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
             dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions.width() as f32, dimensions.height() as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}
