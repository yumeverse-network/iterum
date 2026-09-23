use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Planet {
    pub radius: f64,
    pub max_level: u32,
}