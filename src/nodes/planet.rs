use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component)]
pub struct Planet {
    pub radius: f64,
    pub max_level: u32,
}

#[derive(Component)]
pub struct Star {
    pub color: Vec3,
    pub power: f32,
}