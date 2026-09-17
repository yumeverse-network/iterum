use glam::{DVec3, Quat};

use crate::engine::player::Player;

pub struct Camera {
    pub position: DVec3,
    pub rotation: Quat,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: DVec3::ZERO,
            rotation: Quat::IDENTITY,
            fov: 80.0,
            near: 0.01,
            far: 10000.0,
        }
    }

    pub fn follow(&mut self, target: &Player) {
        self.position = target.position;
    }
}
