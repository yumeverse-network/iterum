use bevy_ecs::prelude::*;
use glam::{DVec3, Quat, Vec3};

use crate::engine::nodes::light::PointLight;
use crate::engine::mesh::Mesh;
use crate::engine::transform::Transform;

pub fn spawn(world: &mut World) {
    // Spawn cube
    world.spawn((
        Transform {
            position: DVec3::new(0.0, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::cube(),
    ));

    // Spawn point light
    world.spawn((
        Transform {
            position: DVec3::new(3.0, 3.0, -5.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        PointLight {
            color: Vec3::ONE,
            intensity: 20.0,
            range: 100.0,
        },
    ));
}