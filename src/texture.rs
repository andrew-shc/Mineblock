use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::command_buffer::{CommandBufferExecFuture, AutoCommandBuffer};
use vulkano::sync::NowFuture;
use vulkano::format::Format;
use vulkano::device::Queue;

use std::io::Cursor;
use std::sync::Arc;


pub struct TextureAtlas {
    pub texture: Arc<ImmutableImage<Format>>,
    quad_size: u16,
    dimensions: Dimensions,
}

impl TextureAtlas {
    pub fn load(queue: Arc<Queue>, img_bytes: Vec<u8>, quad_size: u16) -> (TextureAtlas, CommandBufferExecFuture<NowFuture, AutoCommandBuffer>) {
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

        (TextureAtlas {
            texture: texture,
            quad_size: quad_size,
            dimensions: dimensions,
        },
            tex_future
        )
    }

    pub fn texture_coord(&self, x: u16, y: u16) -> [[f32; 2]; 4] {
        let x_norm_start = (x*self.quad_size) as f32/self.dimensions.width() as f32;
        let y_norm_start = (y*self.quad_size) as f32/self.dimensions.height() as f32;
        let x_norm_end = ((x+1)*self.quad_size) as f32/self.dimensions.width() as f32;
        let y_norm_end = ((y+1)*self.quad_size) as f32/self.dimensions.height() as f32;

        [[x_norm_start, y_norm_start], [x_norm_end, y_norm_start], [x_norm_end, y_norm_end], [x_norm_start, y_norm_end]]
    }
}

use crate::mesh::CubeFace;

trait Texture {

}

struct CubeTexture {
    // each integer values correspond to the location of the quad in the UV atlas
    top: [u16; 2],
    bottom: [u16; 2],
    left: [u16; 2],
    right: [u16; 2],
    front: [u16; 2],
    back: [u16; 2],
}

impl CubeTexture {
    pub fn new(top: [u16; 2], bottom: [u16; 2], left: [u16; 2], right: [u16; 2], front: [u16; 2], back: [u16; 2]) -> Self {
        Self { top, bottom, left, right, front, back }
    }

    pub fn new_all(coord: [u16; 2]) -> Self {
        Self::new(coord, coord, coord, coord, coord, coord)
    }

    pub fn new_single(all: [u16; 2], unique: [u16; 2], face: CubeFace) -> Self {
        Self::new(
            if face == CubeFace::TOP {unique} else {all},
            if face == CubeFace::BOTTOM {unique} else {all},
            if face == CubeFace::LEFT {unique} else {all},
            if face == CubeFace::RIGHT {unique} else {all},
            if face == CubeFace::FRONT {unique} else {all},
            if face == CubeFace::BACK {unique} else {all}
        )
    }
}
