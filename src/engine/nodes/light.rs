use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component)]
pub struct PointLight {
    pub color: Vec3,
    pub intensity: f32,
    pub range: f32,
}