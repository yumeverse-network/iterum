use bevy_ecs::prelude::*;
use glam::Vec3;
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex, Copy, Clone)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

#[derive(Component)]
pub struct Mesh {
    pub vertices: Vec<MyVertex>,
}

impl Mesh {
    pub fn cube() -> Self {
        fn v(position: Vec3, normal: Vec3) -> MyVertex {
            MyVertex {
                position: position.to_array(),
                normal: normal.to_array(),
            }
        }

        Self {
            vertices: vec![
                // Front
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0,  1.0,  1.0), Vec3::Z),
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0,  1.0,  1.0), Vec3::Z),
                v(Vec3::new(-1.0,  1.0,  1.0), Vec3::Z),

                // Back
                v(Vec3::new( 1.0, -1.0, -1.0), Vec3::NEG_Z),
                v(Vec3::new(-1.0, -1.0, -1.0), Vec3::NEG_Z),
                v(Vec3::new(-1.0,  1.0, -1.0), Vec3::NEG_Z),
                v(Vec3::new( 1.0, -1.0, -1.0), Vec3::NEG_Z),
                v(Vec3::new(-1.0,  1.0, -1.0), Vec3::NEG_Z),
                v(Vec3::new( 1.0,  1.0, -1.0), Vec3::NEG_Z),

                // Left
                v(Vec3::new(-1.0, -1.0, -1.0), Vec3::NEG_X),
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::NEG_X),
                v(Vec3::new(-1.0,  1.0,  1.0), Vec3::NEG_X),
                v(Vec3::new(-1.0, -1.0, -1.0), Vec3::NEG_X),
                v(Vec3::new(-1.0,  1.0,  1.0), Vec3::NEG_X),
                v(Vec3::new(-1.0,  1.0, -1.0), Vec3::NEG_X),

                // Right
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::X),
                v(Vec3::new( 1.0, -1.0, -1.0), Vec3::X),
                v(Vec3::new( 1.0,  1.0, -1.0), Vec3::X),
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::X),
                v(Vec3::new( 1.0,  1.0, -1.0), Vec3::X),
                v(Vec3::new( 1.0,  1.0,  1.0), Vec3::X),

                // Top
                v(Vec3::new(-1.0,  1.0,  1.0), Vec3::Y),
                v(Vec3::new( 1.0,  1.0,  1.0), Vec3::Y),
                v(Vec3::new( 1.0,  1.0, -1.0), Vec3::Y),
                v(Vec3::new(-1.0,  1.0,  1.0), Vec3::Y),
                v(Vec3::new( 1.0,  1.0, -1.0), Vec3::Y),
                v(Vec3::new(-1.0,  1.0, -1.0), Vec3::Y),

                // Bottom
                v(Vec3::new(-1.0, -1.0, -1.0), Vec3::NEG_Y),
                v(Vec3::new( 1.0, -1.0, -1.0), Vec3::NEG_Y),
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::NEG_Y),
                v(Vec3::new(-1.0, -1.0, -1.0), Vec3::NEG_Y),
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::NEG_Y),
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::NEG_Y),
            ],
        }
    }
}