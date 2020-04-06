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
    image::{Dimensions, StorageImage, ImmutableImage, swapchain::SwapchainImage, attachment::AttachmentImage},
    format::{Format, ClearValue},
    framebuffer::{Framebuffer, Subpass, FramebufferAbstract, RenderPassAbstract},
    swapchain::{Swapchain, SurfaceTransform, PresentMode, AcquireError, FullscreenExclusive, ColorSpace, SwapchainCreationError},
    swapchain,
    sampler::{Sampler, Filter, MipmapMode, SamplerAddressMode},
};

use std::sync::Arc;
use std::panic;
use std::io::Cursor;
use std::iter;

use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder, Fullscreen};

use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Vector3, Deg, Rad, Euler, Angle};
use cgmath::perspective;

use png;
use vulkano::buffer::CpuBufferPool;
use vulkano::descriptor::descriptor_set::DescriptorSetDesc;
use winit::dpi::{LogicalPosition, PhysicalPosition};

mod renderer;


fn main() {
    let speed = 0.05;  // normalized relative to the screen size
    let mut maximized = false;

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

    // finish setup

    let matrix_buffer = CpuBufferPool::uniform_buffer(device.clone());

    let vertex = vec![
        renderer::Vertex { position: [0.0,0.0,0.0], txtr_crd: [0.0, 0.0], },
        renderer::Vertex { position: [1.0,0.0,0.0], txtr_crd: [1.0, 0.0], },
        renderer::Vertex { position: [1.0,1.0,0.0], txtr_crd: [1.0, 1.0], },
        renderer::Vertex { position: [0.0,1.0,0.0], txtr_crd: [0.0, 1.0], },

        renderer::Vertex { position: [0.0,0.0,1.0], txtr_crd: [0.0, 0.0], },
        renderer::Vertex { position: [1.0,0.0,1.0], txtr_crd: [1.0, 0.0], },
        renderer::Vertex { position: [1.0,1.0,1.0], txtr_crd: [1.0, 1.0], },
        renderer::Vertex { position: [0.0,1.0,1.0], txtr_crd: [1.0, 1.0], },

        renderer::Vertex { position: [0.0,0.0,0.0], txtr_crd: [0.0, 0.0], },
        renderer::Vertex { position: [1.0,0.0,0.0], txtr_crd: [1.0, 0.0], },
        renderer::Vertex { position: [1.0,0.0,1.0], txtr_crd: [1.0, 1.0], },
        renderer::Vertex { position: [0.0,0.0,1.0], txtr_crd: [1.0, 1.0], },

        renderer::Vertex { position: [0.0,1.0,0.0], txtr_crd: [0.0, 0.0], },
        renderer::Vertex { position: [1.0,1.0,0.0], txtr_crd: [1.0, 0.0], },
        renderer::Vertex { position: [1.0,1.0,1.0], txtr_crd: [1.0, 1.0], },
        renderer::Vertex { position: [0.0,1.0,1.0], txtr_crd: [1.0, 1.0], },

        renderer::Vertex { position: [0.0,0.0,0.0], txtr_crd: [0.0, 0.0], },
        renderer::Vertex { position: [0.0,1.0,0.0], txtr_crd: [1.0, 0.0], },
        renderer::Vertex { position: [0.0,1.0,1.0], txtr_crd: [1.0, 1.0], },
        renderer::Vertex { position: [0.0,0.0,1.0], txtr_crd: [1.0, 1.0], },

        renderer::Vertex { position: [1.0,0.0,0.0], txtr_crd: [0.0, 0.0], },
        renderer::Vertex { position: [1.0,1.0,0.0], txtr_crd: [1.0, 0.0], },
        renderer::Vertex { position: [1.0,1.0,1.0], txtr_crd: [1.0, 1.0], },
        renderer::Vertex { position: [1.0,0.0,1.0], txtr_crd: [1.0, 1.0], },
    ];

    let index: Vec<u16> = vec![ 0, 1, 2, 0, 2, 3,
                                4, 5, 6, 4, 6, 7,
                                8, 9,10, 8,10,11,
                               12,13,14,12,14,15,
                               16,17,18,16,18,19,
                               20,21,22,20,22,23,];

    let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(),
                                                       BufferUsage::all(), false, vertex.into_iter()).unwrap();

    let index_buffer = CpuAccessibleBuffer::from_iter(device.clone(),
                                                       BufferUsage::all(), false, index.into_iter()).unwrap();

    let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
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

    let mut dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [swapchain.dimensions()[0] as f32, swapchain.dimensions()[1] as f32],
            depth_range: 0.0 .. 1.0
        }]),
        .. DynamicState::none()
    };

    let (texture, tex_future) = {
        let png_bytes = include_bytes!("../resource/stone.png").to_vec();
        let cursor = Cursor::new(png_bytes);
        let decoder = png::Decoder::new(cursor);
        let (info, mut reader) = decoder.read_info().unwrap();
        let dimensions = Dimensions::Dim2d { width: info.width, height: info.height };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        ImmutableImage::from_iter(
            image_data.iter().cloned(),
            dimensions,
            Format::R8G8B8A8Srgb,
            queue.clone()
        ).unwrap()
    };

    // Filter::Nearest for rendering each pixel instead of "smudging" between the adjacent pixels
    let sampler = Sampler::new(device.clone(), Filter::Nearest, Filter::Nearest,
                               MipmapMode::Nearest, SamplerAddressMode::Repeat, SamplerAddressMode::Repeat,
                               SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();

    let mut framebuffer = frames(device.clone(), &images, render_pass.clone(), &mut dynamic_state);

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let mut previous_frame_end = Some(Box::new(tex_future) as Box<dyn GpuFuture>);  // store previous submission of frame
    let mut recreate_swapchain = false;  // recreating the swapchain if the swapchain's screen was changed

    use winit::event_loop::ControlFlow;
    use winit::event::{Event, WindowEvent, DeviceEvent, VirtualKeyCode as K, KeyboardInput, ElementState};
    use winit::monitor::{MonitorHandle, VideoMode};
    use winit::window::{Fullscreen};
    use winit::dpi::Position;

    let mut position = Point3::new(0.0, 0.0, 0.0);  // the position of the player's camera
    let mut rotation = Euler::new(Deg(0.0 as f32), Deg(0.0), Deg(0.0));  // the rotation of the player's camera in Radian
    let mut pressed: Vec<K> = Vec::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        match event {
            Event::WindowEvent { event: event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {*control_flow = ControlFlow::Exit},
                    WindowEvent::KeyboardInput { input: input, ..} => {
                        match input {
                            KeyboardInput { virtual_keycode: key, state: ElementState::Pressed, ..} => {
                                match key.unwrap() {
                                    K::Escape => {*control_flow = ControlFlow::Exit},
                                    K::F11 => {
                                        maximized = !maximized;
                                        surface.window().set_maximized(maximized);
                                    },
                                    K::A => { if !pressed.contains(&K::A) {pressed.push(K::A);} },
                                    K::D => { if !pressed.contains(&K::D) {pressed.push(K::D);} },
                                    K::W => { if !pressed.contains(&K::W) {pressed.push(K::W);} },
                                    K::S => { if !pressed.contains(&K::D) {pressed.push(K::S);} },
                                    K::LShift => { if !pressed.contains(&K::LShift) {pressed.push(K::LShift);} },
                                    K::Space =>  { if !pressed.contains(&K::Space) {pressed.push(K::Space);} },
                                    _ => {}
                                }
                            },
                            KeyboardInput { virtual_keycode: key, state: ElementState::Released, ..} => {
                                match key.unwrap() {
                                    K::A => { if pressed.contains(&K::A) {pressed.retain(|i| i != &K::A);} },
                                    K::D => { if pressed.contains(&K::D) {pressed.retain(|i| i != &K::D);} },
                                    K::W => { if pressed.contains(&K::W) {pressed.retain(|i| i != &K::W);} },
                                    K::S => { if pressed.contains(&K::S) {pressed.retain(|i| i != &K::S);} },
                                    K::LShift => { if pressed.contains(&K::LShift) {pressed.retain(|i| i != &K::LShift);} },
                                    K::Space => { if pressed.contains(&K::Space) {pressed.retain(|i| i != &K::Space);} },
                                    _ => {}
                                }
                            }
                        }
                    },
                    _ => {}
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta: delta }, .. } => {
                // println!("{} {}", delta.0, delta.1);
                rotation.x -= Deg(delta.1 as f32/10.0);
                rotation.y += Deg(delta.0 as f32/10.0);

                surface.window().set_cursor_position(
                    Position::Physical(PhysicalPosition{ x: dimensions[0] as i32/2, y: dimensions[1] as i32/2 })
                );

                // Position::Logical(LogicalPosition{ x: dimensions[0] as f64/2.0, y: dimensions[1] as f64/2.0 })
            },
            // this calls last after all the event finishes emitting
            // and only calls once, which is great for updating mutable variables since it'll be uniform
            Event::MainEventsCleared => {
                // only translating relative from the x rotation
                if pressed.contains(&K::A) {position.x -= speed * Rad(rotation.y).0.cos(); position.z += speed * Rad(rotation.y).0.sin()}
                if pressed.contains(&K::D) {position.x += speed * Rad(rotation.y).0.cos(); position.z -= speed * Rad(rotation.y).0.sin()}
                if pressed.contains(&K::W) {position.z += speed * Rad(rotation.y).0.cos(); position.x += speed * Rad(rotation.y).0.sin()}
                if pressed.contains(&K::S) {position.z -= speed * Rad(rotation.y).0.cos(); position.x -= speed * Rad(rotation.y).0.sin()}
                if pressed.contains(&K::LShift) {position.y -= speed}
                if pressed.contains(&K::Space)  {position.y += speed}
            },
            Event::RedrawEventsCleared => {
                // cleans the buffer
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                let mut proj = perspective (Rad::from(Deg(60.0)), dimensions[0] as f32/dimensions[1] as f32, 0.2 , 1000.0);
                let mut view =
                    Matrix4::from_angle_x(rotation.x) * Matrix4::from_angle_y(rotation.y) *
                    Matrix4::look_at(Point3::new(position.x, position.y, -1.0+position.z),
                                                position, Vector3::new(0.0, -1.0, 0.0));
                let mut world = Matrix4::identity();


                let pipeline = Arc::new(GraphicsPipeline::start()
                    .vertex_input_single_buffer::<renderer::Vertex>()
                    .vertex_shader(vs.main_entry_point(), ())
                    .triangle_list()
                    .viewports_dynamic_scissors_irrelevant(1)
                    .viewports(iter::once(Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                        depth_range: 0.0 .. 1.0,
                    }))
                    .fragment_shader(fs.main_entry_point(), ())
                    .depth_stencil_simple_depth()
                    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                    .build(device.clone()).unwrap());

                let sub_buf = matrix_buffer.next(
                    vs::ty::Matrix {proj: proj.into(), view: view.into(), world: world.into()}
                ).unwrap();

                let layout0 = pipeline.descriptor_set_layout(0).unwrap();
                let set0 = Arc::new(PersistentDescriptorSet::start(layout0.clone())
                    .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
                    .build().unwrap()
                );

                let layout1 = pipeline.descriptor_set_layout(1).unwrap();
                let set1 = Arc::new(PersistentDescriptorSet::start(layout1.clone())
                    .add_buffer(sub_buf.clone()).unwrap()
                    .build().unwrap()
                );

                // println!("{} {}", set.num_bindings(), mset.num_bindings());

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
                    framebuffer = frames(device.clone(), &images, render_pass.clone(), &mut dynamic_state);
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

                if suboptimal {
                    recreate_swapchain = true;
                }

                let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
                    .begin_render_pass(framebuffer[image_num].clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into(), 1f32.into()]).unwrap()
                    .draw_indexed(pipeline.clone(), &DynamicState::none(), vertex_buffer.clone(), index_buffer.clone(), (set0.clone(), set1.clone()), ()).unwrap()
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
            _ => {},
        }
    });
}

fn frames(device: Arc<Device>,
          images: &Vec<Arc<SwapchainImage<Window>>>,
          render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
          dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

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

mod vs { vulkano_shaders::shader!{ty: "vertex", path: "src/cube.vert",} }
mod fs { vulkano_shaders::shader!{ty: "fragment", path: "src/cube.frag",} }
