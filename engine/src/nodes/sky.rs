use bevy_ecs::prelude::*;
use glam::Vec3;

#[derive(Component)]
pub struct GlobalSky {
    pub sky_top: Vec3,
    pub sky_bottom: Vec3,
}

#[derive(Clone, Copy)]
pub enum SkyShape {
    Sphere,
    Cube,
}

#[derive(Component)]
pub struct LocalSky {
    pub sky_top: Vec3,
    pub sky_bottom: Vec3,
    pub radius: f64,
    pub shape: SkyShape,
}

#[derive(Component)]
pub struct GlobalSpace {
    pub density: f32,
    pub brightness: f32,
    pub seed: f32,
}