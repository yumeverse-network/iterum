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

    // Spawn icosphere
    world.spawn((
        Transform {
            position: DVec3::new(2.5, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::icosphere(1),
    ));

    // Spawn plane
    world.spawn((
        Transform {
            position: DVec3::new(5.0, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::plane(),
    ));

    // Spawn triangle
    world.spawn((
        Transform {
            position: DVec3::new(7.5, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::triangle(),
    ));

    // Spawn circle
    world.spawn((
        Transform {
            position: DVec3::new(-2.5, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::circle(8, 1.0),
    ));

    // Spawn cylinder
    world.spawn((
        Transform {
            position: DVec3::new(0.0, 2.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::cylinder(8, 1.0, 1.0),
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