use bevy_ecs::prelude::*;
use glam::{DVec3, Quat, Vec3};

use crate::engine::mesh::Mesh;
use crate::engine::nodes::light::PointLight;
use crate::engine::nodes::planet::{Planet, Star};
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

    // Spawn star / sun
    world.spawn((
        Transform {
            position: DVec3::new(-2.0e9, 1.0e2, -1.0e7),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(6.96e8),
        },
        Mesh::icosphere(3),
        Star {
            color: Vec3::new(1.0, 0.95, 0.85),
            power: 4.0e18,
        },
    ));
    world.spawn((
        Transform {
            position: DVec3::new(2.0e9, 1.0e9, 1.0e9),
            rotation: Quat::IDENTITY,
            scale: Vec3::splat(4.0e8),
        },
        Mesh::icosphere(3),
        Star {
            color: Vec3::new(0.9, 0.1, 0.0),
            power: 2.0e18,
        },
    ));

    // Cube with ID so it can be modified
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

    world.spawn((
        Transform {
            position: DVec3::new(5.0, 7.5, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::uv_sphere(6, 6, 1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(0.0, 10.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::hemisphere(6, 6, 1.0)
    ));

    world.spawn((
        Transform {
            position: DVec3::new(-2.5, 1.0, -10.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        Mesh::capsule(6, 6, 1.0, 2.0)
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
            intensity: 1.0,
        }
    ));

    world.spawn((
        Transform {
            position: DVec3::new(0.0, -2.5, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::new(100.0, 1.0, 100.0),
        },
        Mesh::cube()
    ));
}