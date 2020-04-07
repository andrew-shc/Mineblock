pub trait Renderable {
    fn vert_data<T>() -> Vec<T>;  // returns the vertex data
    fn ind_data() -> Vec<u32>;  // returns the index data
}


pub trait Quad {
    fn volume() -> i32;
}

impl Renderable for Quad {
    fn vert_data<T>() -> Vec<T>;  // returns the vertex data
    fn ind_data() -> Vec<u32>;  // returns the index data
}