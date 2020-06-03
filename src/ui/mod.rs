use crate::renderer::{Vertex, UIVtx};

use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::viewport::Viewport;
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use vulkano::device::Device;
use vulkano::buffer::{CpuAccessibleBuffer, BufferUsage};

use std::sync::Arc;
use std::iter;
use cgmath::vec1;
use std::ops::Add;
use winit::event::Event;
use vulkano::descriptor::DescriptorSet;
use std::rc::Rc;
use std::cell::RefCell;

pub mod text;


pub mod vs { vulkano_shaders::shader!{ty: "vertex", path: "resource/shaders/ui.vert",} }
pub mod fs { vulkano_shaders::shader!{ty: "fragment", path: "resource/shaders/ui.frag",} }


// a UI context for rendering GUI/HUD components for the game
pub struct UIContext {
    vert_shd: vs::Shader,
    frag_shd: fs::Shader,
    widgets: Vec<Rc<RefCell<dyn Widget>>>
}

impl UIContext {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            vert_shd: vs::Shader::load(device.clone()).expect("failed to load ui vertex shaders module"),
            frag_shd: fs::Shader::load(device.clone()).expect("failed to load ui fragment shaders module"),
            widgets: Vec::new(),
        }
    }

    pub fn add_widget<T: 'static + Widget>(&mut self, w: T) -> Rc<RefCell<T>> {
        let wdg = Rc::new(RefCell::new(w));
        self.widgets.push(wdg.clone());
        wdg.clone()
    }

    pub fn update(&mut self, event: &Event<()>) {
        for w in &self.widgets {
            (*w).borrow_mut().update(&event)
        }
    }

    pub fn render(&self, device: Arc<Device>) -> (Arc<CpuAccessibleBuffer<[UIVtx]>>, Arc<CpuAccessibleBuffer<[u32]>>) {
        let mut canvas = UICanvas::new();
        for w in &self.widgets {
            (*w).borrow().render(&mut canvas)
        }
        let (vtx, ind) = canvas.flush();

        (CpuAccessibleBuffer::from_iter(device.clone(),
                                        BufferUsage::vertex_buffer(),
                                        false,
                                        vtx.into_iter()).unwrap(),
         CpuAccessibleBuffer::from_iter(device.clone(),
                                        BufferUsage::index_buffer(),
                                        false,
                                        ind.into_iter()).unwrap()
        )
    }

    pub fn pipeline(&self,
                    device: Arc<Device>,
                    dimensions: [u32; 2],
                    renderpass: Arc<dyn RenderPassAbstract + Send + Sync>)
        -> Arc<dyn GraphicsPipelineAbstract + Send + Sync> {
        let spec_consts = vs::SpecializationConstants {
            aspect_ratio: dimensions[0] as f32 / dimensions[1] as f32
        };

        Arc::new(
            GraphicsPipeline::start()
                .vertex_input_single_buffer::<UIVtx>()
                .vertex_shader(self.vert_shd.main_entry_point(), spec_consts)
                .triangle_list()
                .viewports_dynamic_scissors_irrelevant(1)
                .viewports(iter::once(Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }))
                .fragment_shader(self.frag_shd.main_entry_point(), ())
                .cull_mode_front()
                .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
                .build(device.clone()).unwrap()
        )
    }
}

pub struct UICanvas {
    vtx: Vec<UIVtx>,  // vertices
    ind: Vec<u32>,  // indices
}

impl UICanvas {
    fn new() -> Self {
        Self {
            vtx: Vec::new(),
            ind: Vec::new(),
        }
    }

    fn flush(self) -> (Vec<UIVtx>, Vec<u32>) {
        (self.vtx, self.ind)
    }

    fn add_square(&mut self, pos: [f32; 2], size: f32, color: [f32; 4]) {
        self.vtx.push(UIVtx {position: [pos[0]     , pos[1]+size], color} ); // bottom right
        self.vtx.push(UIVtx {position: [pos[0]     , pos[1]     ], color} ); // top right
        self.vtx.push(UIVtx {position: [pos[0]+size, pos[1]     ], color} ); // top left
        self.vtx.push(UIVtx {position: [pos[0]+size, pos[1]+size], color} ); // bottom left

        if self.ind.is_empty() {
            self.ind.append(&mut vec![0, 1, 2, 0, 2, 3]);
        } else {
            self.ind.append(&mut vec![0, 1, 2, 0, 2, 3].iter().map(|x| x+self.ind.last().unwrap()+1).collect());
        }
    }
}

// All UI Components must implement this trait
pub trait Widget {
    fn update(&mut self, e: &Event<()>); // updates the widget with states; &mut self
    fn render(&self, ctx: &mut UICanvas); // renders the widget states; handles rendering and user input; &self
}

// TODO
// All animation must implement this trait to be used with the Widgets
trait Animate {

}

// TODO
// All layouts must implement this to be used within the Widgets for organizations
trait Layout {

}
