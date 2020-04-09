use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::command_buffer::{CommandBufferExecFuture, AutoCommandBuffer};
use vulkano::sync::NowFuture;
use vulkano::format::Format;
use vulkano::device::Queue;

use std::io::Cursor;
use std::sync::Arc;
use vulkano::descriptor::descriptor::DescriptorDescSupersetError::DimensionsMismatch;
use std::ops::Mul;


pub struct TextureAtlas {
    pub texture: Arc<ImmutableImage<Format>>,
    pub future: CommandBufferExecFuture<NowFuture, AutoCommandBuffer>,
    quad_size: u16,
    dimensions: Dimensions,
}

impl TextureAtlas {
    pub fn load(queue: Arc<Queue>, img_bytes: Vec<u8>, quad_size: u16) -> TextureAtlas {
        let cursor = Cursor::new(img_bytes);
        let decoder = png::Decoder::new(cursor);
        let (info, mut reader) = decoder.read_info().unwrap();
        let dimensions = Dimensions::Dim2d { width: info.width, height: info.height };
        let mut image_data = Vec::new();
        image_data.resize((info.width * info.height * 4) as usize, 0);
        reader.next_frame(&mut image_data).unwrap();

        let (texture, tex_future) = ImmutableImage::from_iter(
            image_data.iter().cloned(), dimensions, Format::R8G8B8A8Unorm, queue.clone()
            ).unwrap();

        TextureAtlas {
            texture: texture,
            future: tex_future,
            quad_size: quad_size,
            dimensions: dimensions,
        }
    }

    pub fn texture_coord(&self, x: u16, y: u16) -> [[f32; 2]; 4] {
        let x_norm_start = (x*self.quad_size) as f32/self.dimensions.width() as f32;
        let y_norm_start = (y*self.quad_size) as f32/self.dimensions.height() as f32;
        let x_norm_end = ((x+1)*self.quad_size) as f32/self.dimensions.width() as f32;
        let y_norm_end = ((y+1)*self.quad_size) as f32/self.dimensions.height() as f32;

        [[x_norm_start, y_norm_start], [x_norm_end, y_norm_start], [x_norm_end, y_norm_end], [x_norm_start, y_norm_end]]
    }
}
