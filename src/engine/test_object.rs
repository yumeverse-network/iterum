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

    world.spawn((
        Transform {
            position: DVec3::new(2.5, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::icosphere(6)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(5.0, 0.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::plane()
    ));

    world.spawn((
        Transform {
            position: DVec3::new(0.0, 2.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::triangle()
    ));

    world.spawn((
        Transform {
            position: DVec3::new(2.5, 2.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::circle(6, 1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(5.0, 2.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::cylinder(6, 1.0, 1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(0.0, 5.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::tube(6, 1.0, 1.0, 0.2)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(2.5, 5.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::disc(6, 1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(5.0, 5.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::cone(6, 1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(0.0, 7.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::prism(1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(2.5, 7.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::pyramid()
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