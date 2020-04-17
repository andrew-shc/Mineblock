use vulkano::buffer::{CpuBufferPool};
use vulkano::device::Device;
use cgmath::prelude::*;
use cgmath::{Matrix4, Vector3, Point3, Euler, Deg, Rad};
use cgmath::perspective;
use std::sync::Arc;
use vulkano::buffer::cpu_pool::CpuBufferPoolSubbuffer;

use crate::mesh::vs;
use vulkano::memory::pool::StdMemoryPool;

pub struct Camera<T> {
    rot_speed: f32,
    trans_speed: f32,  // normalized relative to the screen size
    fov: Deg<f32>,
    mat_buf: CpuBufferPool<T>,  // matrix buffer
    position: Point3<f32>,
    rotation: Euler<Deg<f32>>,
}

impl Camera<vs::ty::Matrix> {
    pub fn new(device: Arc<Device>, rot_speed: f32, trans_speed: f32) -> Self {
        Self {
            rot_speed: rot_speed,
            trans_speed: trans_speed,
            fov: Deg(60.0),
            mat_buf: CpuBufferPool::uniform_buffer(device.clone()),
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Euler::new(Deg(0.0 as f32), Deg(0.0), Deg(0.0)),
        }
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position.x += x * self.trans_speed;
        self.position.y += y * self.trans_speed;
        self.position.z += z * self.trans_speed;
    }

    pub fn rotate(&mut self, x: f32, y: f32) {
        // z-rot stays constant; you don't want the camera spinning sideways
        self.rotation.x += Deg(x * -self.rot_speed);
        self.rotation.y += Deg(y * self.rot_speed);
    }

    pub fn mat_buf(&self, dimensions: [u32; 2]) -> CpuBufferPoolSubbuffer<vs::ty::Matrix, Arc<StdMemoryPool>> {
        // retrieves the matrix buffer from the camera

        // the closer the znear is to 0, the worse the depth buffering would perform
        let proj = perspective (Rad::from(self.fov), dimensions[0] as f32/dimensions[1] as f32, 0.1 , 1000.0);
        let view = Matrix4::from_angle_x(self.rotation.x) * Matrix4::from_angle_y(self.rotation.y) *
            Matrix4::look_at(Point3::new(self.position.x, self.position.y, -1.0+self.position.z), self.position, Vector3::new(0.0, -1.0, 0.0));
        let world = Matrix4::identity();

        self.mat_buf.next(
            vs::ty::Matrix {proj: proj.into(), view: view.into(), world: world.into()}
        ).unwrap()
    }
}