use bevy_ecs::prelude::*;
use glam::Vec3;
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex, Copy, Clone)]
#[repr(C)]
pub struct Vertices {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

#[derive(Component)]
pub struct Mesh {
    pub vertices: Vec<Vertices>,
}

impl Mesh {
    // Cube
    pub fn cube() -> Self {
        fn v(position: Vec3, normal: Vec3) -> Vertices {
            Vertices {
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

    // Icosphere
    pub fn icosphere(subdivisions: u32) -> Self {
        use std::collections::HashMap;

        let t = (1.0 + 5.0_f32.sqrt()) / 2.0;

        let mut positions = vec![
            Vec3::new(-1.0,  t,  0.0),
            Vec3::new( 1.0,  t,  0.0),
            Vec3::new(-1.0, -t,  0.0),
            Vec3::new( 1.0, -t,  0.0),

            Vec3::new( 0.0, -1.0,  t),
            Vec3::new( 0.0,  1.0,  t),
            Vec3::new( 0.0, -1.0, -t),
            Vec3::new( 0.0,  1.0, -t),

            Vec3::new( t,  0.0, -1.0),
            Vec3::new( t,  0.0,  1.0),
            Vec3::new(-t,  0.0, -1.0),
            Vec3::new(-t,  0.0,  1.0),
        ];

        // Normalize the initial icosahedron onto the unit sphere.
        for position in &mut positions {
            *position = position.normalize();
        }

        let mut triangles: Vec<[u32; 3]> = vec![
            [0, 11, 5],
            [0, 5, 1],
            [0, 1, 7],
            [0, 7, 10],
            [0, 10, 11],

            [1, 5, 9],
            [5, 11, 4],
            [11, 10, 2],
            [10, 7, 6],
            [7, 1, 8],

            [3, 9, 4],
            [3, 4, 2],
            [3, 2, 6],
            [3, 6, 8],
            [3, 8, 9],

            [4, 9, 5],
            [2, 4, 11],
            [6, 2, 10],
            [8, 6, 7],
            [9, 8, 1],
        ];

        // Subdivide every triangle.
        for _ in 0..subdivisions {
            let mut next_triangles = Vec::with_capacity(triangles.len() * 4);
            let mut midpoint_cache = HashMap::<(u32, u32), u32>::new();

            fn midpoint(
                a: u32,
                b: u32,
                positions: &mut Vec<Vec3>,
                cache: &mut HashMap<(u32, u32), u32>,
            ) -> u32 {
                let key = if a < b { (a, b) } else { (b, a) };

                if let Some(&index) = cache.get(&key) {
                    return index;
                }

                let position =
                    (positions[a as usize] + positions[b as usize]).normalize();

                let index = positions.len() as u32;
                positions.push(position);
                cache.insert(key, index);

                index
            }

            for [a, b, c] in triangles {
                let ab = midpoint(a, b, &mut positions, &mut midpoint_cache);
                let bc = midpoint(b, c, &mut positions, &mut midpoint_cache);
                let ca = midpoint(c, a, &mut positions, &mut midpoint_cache);

                next_triangles.push([a, ab, ca]);
                next_triangles.push([b, bc, ab]);
                next_triangles.push([c, ca, bc]);
                next_triangles.push([ab, bc, ca]);
            }

            triangles = next_triangles;
        }

        // Convert indexed triangles into your current vertex format.
        let mut vertices = Vec::with_capacity(triangles.len() * 3);

        for [a, b, c] in triangles {
            for index in [a, b, c] {
                let position = positions[index as usize];

                vertices.push(Vertices {
                    position: position.to_array(),
                    normal: position.to_array(),
                });
            }
        }

        Self { vertices }
    }

    // Plane
    pub fn plane() -> Self {
        fn v(position: Vec3, normal: Vec3) -> Vertices {
            Vertices {
                position: position.to_array(),
                normal: normal.to_array(),
            }
        }

        Self {
            vertices: vec![
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0,  1.0,  1.0), Vec3::Z),
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0,  1.0,  1.0), Vec3::Z),
                v(Vec3::new(-1.0,  1.0,  1.0), Vec3::Z),
            ],
        }
    }

    // Triangle
    pub fn triangle() -> Self {
        fn v(position: Vec3, normal: Vec3) -> Vertices {
            Vertices {
                position: position.to_array(),
                normal: normal.to_array(),
            }
        }

        Self {
            vertices: vec![
                v(Vec3::new(-1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 1.0, -1.0,  1.0), Vec3::Z),
                v(Vec3::new( 0.0, 1.0,  1.0), Vec3::Z),
            ],
        }
    }

    // Circle
    pub fn circle(segments: i32, radius: f32) -> Self {
        let mut verts = Vec::new();
    
        fn v(position: Vec3, normal: Vec3) -> Vertices {
            Vertices {
                position: position.to_array(),
                normal: normal.to_array(),
            }
        }
    
        for i in 0..segments {
            let theta0 =
                2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let theta1 =
                2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;
    
            let p0 = Vec3::new(
                radius * theta0.cos(),
                radius * theta0.sin(),
                0.0,
            );
    
            let p1 = Vec3::new(
                radius * theta1.cos(),
                radius * theta1.sin(),
                0.0,
            );
    
            // One triangle of the fan
            verts.push(v(Vec3::ZERO, Vec3::Z));
            verts.push(v(p0, Vec3::Z));
            verts.push(v(p1, Vec3::Z));
        }
    
        Self {
            vertices: verts,
        }
    }
}