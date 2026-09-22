use bevy_ecs::prelude::*;
use glam::DVec3;

#[derive(Component)]
pub struct Planet {
    pub radius: f64,
    pub max_level: u32,
}