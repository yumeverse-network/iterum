use bevy_ecs::prelude::*;
use glam::{DVec3, Quat, Vec3};

use crate::engine::mesh::Mesh;
use crate::engine::nodes::light::PointLight;
use crate::engine::nodes::planet::Planet;
use crate::engine::transform::Transform;

pub fn spawn(world: &mut World) {
    // Spawn planet
    world.spawn((
        Transform {
            position: DVec3::new(0.0, 0.0, -1.0e9),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Planet {
            radius: 6.371e8,
            max_level: 20,
        },
    ));

    world.spawn((
        Transform {
            position: DVec3::new(0.0, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::cube()
    ));

    // Spawn point light
    world.spawn((
        Transform {
            position: DVec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        PointLight {
            color: Vec3::ONE,
            intensity: 1.0e22,
        }
    ));
}