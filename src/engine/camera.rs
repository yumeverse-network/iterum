use glam::{DVec3, Quat};

use crate::engine::player::Player;

pub const CAM_NEAR: f32 = 0.1;
pub const CAM_FAR: f32 = 1.0e13;

pub struct Camera {
    pub position: DVec3,
    pub rotation: Quat,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: DVec3::ZERO,
            rotation: Quat::IDENTITY,
            yaw: 0.0,
            pitch: 0.0,
            fov: 80.0,
            near: CAM_NEAR,
            far: CAM_FAR,
        }
    }

    pub fn follow(&mut self, target: &Player) {
        self.position = target.position;
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw;
        self.pitch = (self.pitch + pitch).clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

        self.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }
}
