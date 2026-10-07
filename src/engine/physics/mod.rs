use std::f64::consts::FRAC_PI_2;

use bevy_ecs::prelude::{Component, Entity, World};
use glam::{DVec3, Quat};
use rapier3d_f64::prelude::{
    CCDSolver, ColliderBuilder, ColliderHandle, ColliderSet, DefaultBroadPhase, ImpulseJointSet,
    IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet, RigidBodyType, SoftBodySet,
};

use crate::engine::mesh::Mesh;
use crate::engine::transform::Transform;

pub const FIXED_DT: f64 = 1.0 / 60.0;

#[derive(Clone, Debug)]
pub enum Shape {
    Cuboid { half_extents: DVec3 },
    Ball { radius: f64 },
    Capsule { radius: f64, height: f64 },
    Cylinder { radius: f64, half_depth: f64 },
    Cone { radius: f64 },
    ConvexHull,
}

impl Shape {
    pub const CUBE: Shape = Shape::Cuboid { half_extents: DVec3::ONE };
    pub const SPHERE: Shape = Shape::Ball { radius: 1.0 };
    pub const PLANE: Shape = Shape::Cuboid { half_extents: DVec3::new(1.0, 1.0, 0.01) };

    fn build(&self, scale: DVec3, mesh: &Mesh) -> ColliderBuilder {
        let s = scale.abs();
        match self {
            Shape::Cuboid { half_extents } => {
                let h = *half_extents * s;
                ColliderBuilder::cuboid(h.x, h.y, h.z)
            }
            Shape::Ball { radius } => ColliderBuilder::ball(radius * s.max_element()),
            Shape::Capsule { radius, height } => {
                ColliderBuilder::capsule_y(height * 0.5 * s.y, radius * s.x.max(s.z))
            }
            Shape::Cylinder { radius, half_depth } => {
                ColliderBuilder::cylinder(half_depth * s.z, radius * s.x.max(s.y))
                    .rotation(DVec3::new(FRAC_PI_2, 0.0, 0.0))
            }
            Shape::Cone { radius } => ColliderBuilder::cone(s.y, radius * s.x.max(s.z)),
            Shape::ConvexHull => {
                let pts: Vec<DVec3> = mesh
                    .vertices
                    .iter()
                    .map(|v| {
                        DVec3::new(
                            v.position[0] as f64 * s.x,
                            v.position[1] as f64 * s.y,
                            v.position[2] as f64 * s.z,
                        )
                    })
                    .collect();
                ColliderBuilder::convex_hull(&pts).unwrap_or_else(|| {
                    ColliderBuilder::ball(mesh.bounds.radius as f64 * s.max_element())
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Physics {
    kind: RigidBodyType,
    shape: Shape,
    restitution: f64,
    friction: f64,
    density: f64,
}

impl Physics {
    fn new(kind: RigidBodyType, shape: Shape) -> Self {
        Self { kind, shape, restitution: 0.2, friction: 0.7, density: 1.0 }
    }
    pub fn dynamic(shape: Shape) -> Self { Self::new(RigidBodyType::Dynamic, shape) }
    pub fn fixed(shape: Shape) -> Self { Self::new(RigidBodyType::Fixed, shape) }
    pub fn kinematic(shape: Shape) -> Self {
        Self::new(RigidBodyType::KinematicPositionBased, shape)
    }

    pub fn restitution(mut self, v: f64) -> Self { self.restitution = v; self }
    pub fn friction(mut self, v: f64) -> Self { self.friction = v; self }
    pub fn density(mut self, v: f64) -> Self { self.density = v; self }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct PhysicsBody(pub RigidBodyHandle);

#[derive(Component, Debug, Clone, Copy)]
pub struct PhysicsCollider(pub ColliderHandle);

pub struct PhysicsWorld {
    pub gravity: DVec3,
    pub integration_parameters: IntegrationParameters,
    pub pipeline: PhysicsPipeline,
    pub islands: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub impulse_joints: ImpulseJointSet,
    pub multibody_joints: MultibodyJointSet,
    pub soft_bodies: SoftBodySet,
    pub ccd_solver: CCDSolver,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            gravity: DVec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters {
                dt: FIXED_DT,
                ..Default::default()
            },
            pipeline: PhysicsPipeline::new(),
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            impulse_joints: ImpulseJointSet::new(),
            multibody_joints: MultibodyJointSet::new(),
            soft_bodies: SoftBodySet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    pub fn step(&mut self) {
        self.pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.soft_bodies,
            &mut self.ccd_solver,
            &(),
            &(),
        );
    }

    pub fn spawn(
        &mut self,
        world: &mut World,
        transform: Transform,
        mesh: Mesh,
        physics: Physics,
    ) -> Entity {
        let axis_angle = transform.rotation.to_scaled_axis().as_dvec3();

        let body = RigidBodyBuilder::new(physics.kind)
            .translation(transform.position)
            .rotation(axis_angle)
            .build();

        let collider = physics
            .shape
            .build(transform.scale.as_dvec3(), &mesh)
            .restitution(physics.restitution)
            .friction(physics.friction)
            .density(physics.density)
            .build();

        let body_handle = self.bodies.insert(body);
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, body_handle, &mut self.bodies);

        world
            .spawn((
                transform,
                mesh,
                PhysicsBody(body_handle),
                PhysicsCollider(collider_handle),
            ))
            .id()
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self { Self::new() }
}

pub fn sync_transforms(world: &mut World, physics: &PhysicsWorld) {
    let mut query = world.query::<(&PhysicsBody, &mut Transform)>();
    for (body_ref, mut transform) in query.iter_mut(world) {
        if let Some(body) = physics.bodies.get(body_ref.0) {
            let t = body.translation();
            transform.position = DVec3::new(t.x, t.y, t.z);

            let r = body.rotation();
            transform.rotation = Quat::from_xyzw(r.x as f32, r.y as f32, r.z as f32, r.w as f32);
        }
    }
}