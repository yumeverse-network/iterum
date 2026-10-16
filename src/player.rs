use bevy_ecs::prelude::*;
use glam::{DVec3, Vec3, Quat};

#[derive(Component)]
pub struct Player {
    pub position: DVec3,
    pub velocity: DVec3,
    pub _size: Vec3,
    pub _rotation: Quat,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: DVec3::ZERO,
            velocity: DVec3::ZERO,
            _size: Vec3::new(32.0, 32.0, 32.0),
            _rotation: Quat::IDENTITY,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.position += self.velocity * dt;
    }
}
