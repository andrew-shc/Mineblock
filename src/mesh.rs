use crate::texture::TextureAtlas;

pub trait Renderable {
    type Vertex;

    fn vert_data(&self, texture: &TextureAtlas, position: [f32; 3]) -> Vec<Self::Vertex>;  // returns the vertex data
    fn ind_data(&self) -> Vec<u32>;  // returns the index data
}
