/*
A file to store all kinds of data types used in UI's and the game itself.
 */

pub struct Pos {
    pub x: f32,
    pub y: f32,
    pub z: f32, // optional
}

pub struct Size {
    pub w: f32, // width (x)
    pub h: f32, // height (y)
    pub d: f32, // depth (z); optional
}

pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32, // optional
}

impl Pos {
    pub fn new3D(x: f32, y: f32, z: f32) -> Self {
        Self {x, y, z}
    }

    pub fn new2D(x: f32, y: f32) -> Self {
        Self {x, y, z: 0.0}
    }
}