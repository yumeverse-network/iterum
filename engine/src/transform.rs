use bevy_ecs::prelude::*;
use glam::{DVec3, Vec3, Quat};

#[derive(Component)]
pub struct Transform {
    pub position: DVec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

pub trait WorldPositionExt {
    fn relative_to(self, origin: glam::DVec3) -> glam::DVec3;
    fn camera_relative_f32(self, camera_position: glam::DVec3) -> glam::Vec3;
}

impl WorldPositionExt for glam::DVec3 {
    fn relative_to(self, origin: glam::DVec3) -> glam::DVec3 {
        self - origin
    }

    fn camera_relative_f32(self, camera_position: glam::DVec3) -> glam::Vec3 {
        self.relative_to(camera_position).as_vec3()
    }
}
