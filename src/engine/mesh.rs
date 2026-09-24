use bevy_ecs::prelude::*;
use glam::DVec3;
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
        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        Self {
            vertices: vec![
                // Front
                v(DVec3::new(-1.0, -1.0,  1.0), DVec3::Z),
                v(DVec3::new( 1.0, -1.0,  1.0), DVec3::Z),
                v(DVec3::new( 1.0,  1.0,  1.0), DVec3::Z),
                v(DVec3::new(-1.0, -1.0,  1.0), DVec3::Z),
                v(DVec3::new( 1.0,  1.0,  1.0), DVec3::Z),
                v(DVec3::new(-1.0,  1.0,  1.0), DVec3::Z),

                // Back
                v(DVec3::new( 1.0, -1.0, -1.0), DVec3::NEG_Z),
                v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Z),
                v(DVec3::new(-1.0,  1.0, -1.0), DVec3::NEG_Z),
                v(DVec3::new( 1.0, -1.0, -1.0), DVec3::NEG_Z),
                v(DVec3::new(-1.0,  1.0, -1.0), DVec3::NEG_Z),
                v(DVec3::new( 1.0,  1.0, -1.0), DVec3::NEG_Z),

                // Left
                v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_X),
                v(DVec3::new(-1.0, -1.0,  1.0), DVec3::NEG_X),
                v(DVec3::new(-1.0,  1.0,  1.0), DVec3::NEG_X),
                v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_X),
                v(DVec3::new(-1.0,  1.0,  1.0), DVec3::NEG_X),
                v(DVec3::new(-1.0,  1.0, -1.0), DVec3::NEG_X),

                // Right
                v(DVec3::new( 1.0, -1.0,  1.0), DVec3::X),
                v(DVec3::new( 1.0, -1.0, -1.0), DVec3::X),
                v(DVec3::new( 1.0,  1.0, -1.0), DVec3::X),
                v(DVec3::new( 1.0, -1.0,  1.0), DVec3::X),
                v(DVec3::new( 1.0,  1.0, -1.0), DVec3::X),
                v(DVec3::new( 1.0,  1.0,  1.0), DVec3::X),

                // Top
                v(DVec3::new(-1.0,  1.0,  1.0), DVec3::Y),
                v(DVec3::new( 1.0,  1.0,  1.0), DVec3::Y),
                v(DVec3::new( 1.0,  1.0, -1.0), DVec3::Y),
                v(DVec3::new(-1.0,  1.0,  1.0), DVec3::Y),
                v(DVec3::new( 1.0,  1.0, -1.0), DVec3::Y),
                v(DVec3::new(-1.0,  1.0, -1.0), DVec3::Y),

                // Bottom
                v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Y),
                v(DVec3::new( 1.0, -1.0, -1.0), DVec3::NEG_Y),
                v(DVec3::new( 1.0, -1.0,  1.0), DVec3::NEG_Y),
                v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Y),
                v(DVec3::new( 1.0, -1.0,  1.0), DVec3::NEG_Y),
                v(DVec3::new(-1.0, -1.0,  1.0), DVec3::NEG_Y),
            ],
        }
    }

    // Icosphere
    pub fn icosphere(subdivisions: u32) -> Self {
        use std::collections::HashMap;

        let t = (1.0 + 5.0_f64.sqrt()) / 2.0;

        let mut positions = vec![
            DVec3::new(-1.0,  t,  0.0),
            DVec3::new( 1.0,  t,  0.0),
            DVec3::new(-1.0, -t,  0.0),
            DVec3::new( 1.0, -t,  0.0),

            DVec3::new( 0.0, -1.0,  t),
            DVec3::new( 0.0,  1.0,  t),
            DVec3::new( 0.0, -1.0, -t),
            DVec3::new( 0.0,  1.0, -t),

            DVec3::new( t,  0.0, -1.0),
            DVec3::new( t,  0.0,  1.0),
            DVec3::new(-t,  0.0, -1.0),
            DVec3::new(-t,  0.0,  1.0),
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
                positions: &mut Vec<DVec3>,
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
                    position: position.as_vec3().to_array(),
                    normal: position.as_vec3().to_array(),
                });
            }
        }

        Self { vertices }
    }

    // Plane
    pub fn plane() -> Self {
        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        Self {
            vertices: vec![
                v(DVec3::new(-1.0, -1.0,  0.0), DVec3::Z),
                v(DVec3::new( 1.0, -1.0,  0.0), DVec3::Z),
                v(DVec3::new( 1.0,  1.0,  0.0), DVec3::Z),
                v(DVec3::new(-1.0, -1.0,  0.0), DVec3::Z),
                v(DVec3::new( 1.0,  1.0,  0.0), DVec3::Z),
                v(DVec3::new(-1.0,  1.0,  0.0), DVec3::Z),
            ],
        }
    }

    // Triangle
    pub fn triangle() -> Self {
        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        Self {
            vertices: vec![
                v(DVec3::new(-1.0, -1.0,  0.0), DVec3::Z),
                v(DVec3::new( 1.0, -1.0,  0.0), DVec3::Z),
                v(DVec3::new( 0.0,  1.0,  0.0), DVec3::Z),
            ],
        }
    }

    // Circle
    pub fn circle(segments: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        for i in 0..segments {
            let theta0 =
                2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            let theta1 =
                2.0 * std::f64::consts::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(
                radius * theta0.cos(),
                radius * theta0.sin(),
                0.0,
            );

            let p1 = DVec3::new(
                radius * theta1.cos(),
                radius * theta1.sin(),
                0.0,
            );

            verts.push(v(DVec3::ZERO, DVec3::Z));
            verts.push(v(p0, DVec3::Z));
            verts.push(v(p1, DVec3::Z));
        }

        Self {
            vertices: verts,
        }
    }

    // Cylinder
    pub fn cylinder(segments: i32, radius: f64, depth: f64) -> Self {
        let mut verts = Vec::new();

        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        let half_depth = depth;

        for i in 0..segments {
            let theta0 =
                2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            let theta1 =
                2.0 * std::f64::consts::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(
                radius * theta0.cos(),
                radius * theta0.sin(),
                0.0,
            );

            let p1 = DVec3::new(
                radius * theta1.cos(),
                radius * theta1.sin(),
                0.0,
            );

            let p0_top = p0 + DVec3::Z * half_depth;
            let p1_top = p1 + DVec3::Z * half_depth;

            let p0_bottom = p0 - DVec3::Z * half_depth;
            let p1_bottom = p1 - DVec3::Z * half_depth;

            verts.push(v(p0_bottom, p0.normalize()));
            verts.push(v(p1_bottom, p1.normalize()));
            verts.push(v(p1_top, p1.normalize()));

            verts.push(v(p0_bottom, p0.normalize()));
            verts.push(v(p1_top, p1.normalize()));
            verts.push(v(p0_top, p0.normalize()));

            verts.push(v(DVec3::new(0.0, 0.0, half_depth), DVec3::Z));
            verts.push(v(p0_top, DVec3::Z));
            verts.push(v(p1_top, DVec3::Z));

            verts.push(v(DVec3::new(0.0, 0.0, -half_depth), DVec3::NEG_Z));
            verts.push(v(p0_bottom, DVec3::NEG_Z));
            verts.push(v(p1_bottom, DVec3::NEG_Z));
        }

        Self {
            vertices: verts,
        }
    }

    // Tube
    pub fn tube(segments: i32, radius: f64, depth: f64, thickness: f64) -> Self {
        let mut verts = Vec::new();

        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        let half_depth = depth;
        let inner_radius: f64 = radius - thickness;

        for i in 0..segments {
            let theta0 =
                2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            let theta1 =
                2.0 * std::f64::consts::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(
                radius * theta0.cos(),
                radius * theta0.sin(),
                0.0,
            );

            let p1 = DVec3::new(
                radius * theta1.cos(),
                radius * theta1.sin(),
                0.0,
            );

            let p2 = DVec3::new(
                inner_radius * theta0.cos(),
                inner_radius * theta0.sin(),
                0.0,
            );

            let p3 = DVec3::new(
                inner_radius * theta1.cos(),
                inner_radius * theta1.sin(),
                0.0,
            );

            let p0_top = p0 + DVec3::Z * half_depth;
            let p1_top = p1 + DVec3::Z * half_depth;

            let p0_bottom = p0 - DVec3::Z * half_depth;
            let p1_bottom = p1 - DVec3::Z * half_depth;

            let p2_top = p2 + DVec3::Z * half_depth;
            let p3_top = p3 + DVec3::Z * half_depth;

            let p2_bottom = p2 - DVec3::Z * half_depth;
            let p3_bottom = p3 - DVec3::Z * half_depth;

            // Outer
            verts.push(v(p0_bottom, p0.normalize()));
            verts.push(v(p1_bottom, p1.normalize()));
            verts.push(v(p1_top, p1.normalize()));

            verts.push(v(p0_bottom, p0.normalize()));
            verts.push(v(p1_top, p1.normalize()));
            verts.push(v(p0_top, p0.normalize()));

            // Inner
            verts.push(v(p2_bottom, p2.normalize()));
            verts.push(v(p3_bottom, p3.normalize()));
            verts.push(v(p3_top, p3.normalize()));

            verts.push(v(p2_bottom, p2.normalize()));
            verts.push(v(p3_top, p3.normalize()));
            verts.push(v(p2_top, p2.normalize()));

            // Inner to outer
            verts.push(v(p0_top, DVec3::Z));
            verts.push(v(p1_top, DVec3::Z));
            verts.push(v(p2_top, DVec3::Z));
            verts.push(v(p1_top, DVec3::Z));
            verts.push(v(p2_top, DVec3::Z));
            verts.push(v(p3_top, DVec3::Z));

            verts.push(v(p0_bottom, DVec3::Z));
            verts.push(v(p1_bottom, DVec3::Z));
            verts.push(v(p2_bottom, DVec3::Z));
            verts.push(v(p1_bottom, DVec3::Z));
            verts.push(v(p2_bottom, DVec3::Z));
            verts.push(v(p3_bottom, DVec3::Z));
        }

        Self {
            vertices: verts,
        }
    }

    // Disc
    pub fn disc(segments: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }

        for i in 0..segments {
            let theta0 =
                2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            let theta1 =
                2.0 * std::f64::consts::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(
                radius * theta0.cos(),
                0.0,
                radius * theta0.sin(),
            );

            let p1 = DVec3::new(
                radius * theta1.cos(),
                0.0,
                radius * theta1.sin(),
            );

            verts.push(v(DVec3::ZERO, DVec3::Y));
            verts.push(v(p0, DVec3::Y));
            verts.push(v(p1, DVec3::Y));
        }

        Self {
            vertices: verts,
        }
    }

    // Cone
    pub fn cone(segments: i32, radius: f64) -> Self {
        let mut verts = Vec::new();
    
        fn v(position: DVec3, normal: DVec3) -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        }
    
        let apex = DVec3::new(0.0, 1.0, 0.0);
        let base_center = DVec3::new(0.0, -1.0, 0.0);
        let h = 2.0;
        let down = -DVec3::Y;
    
        for i in 0..segments {
            let theta0 = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
            let theta1 = 2.0 * std::f64::consts::PI * (i + 1) as f64 / segments as f64;
    
            let dir0 = DVec3::new(theta0.cos(), 0.0, theta0.sin());
            let dir1 = DVec3::new(theta1.cos(), 0.0, theta1.sin());
    
            let p0 = DVec3::new(radius * dir0.x, -1.0, radius * dir0.z);
            let p1 = DVec3::new(radius * dir1.x, -1.0, radius * dir1.z);
    
            let n0 = (DVec3::Y * radius + dir0 * h).normalize();
            let n1 = (DVec3::Y * radius + dir1 * h).normalize();
    
            verts.push(v(apex, n0));
            verts.push(v(p0, n0));
            verts.push(v(p1, n1));
    
            verts.push(v(base_center, down));
            verts.push(v(p1, down));
            verts.push(v(p0, down));
        }
    
        Self { vertices: verts }
    }

    // Triangular prism
    pub fn prism(depth: f64) -> Self {
        let mut verts = Vec::new();
    
        let v = |position: DVec3, normal: DVec3| -> Vertices {
            Vertices {
                position: position.as_vec3().to_array(),
                normal: normal.as_vec3().to_array(),
            }
        };
    
        let h = depth * 0.5;
    
        let front = DVec3::new(0.0, 0.0,  h);
        let back  = DVec3::new(0.0, 0.0, -h);
    
        let a = DVec3::new(-1.0, -1.0, 0.0);
        let b = DVec3::new( 1.0, -1.0, 0.0);
        let c = DVec3::new( 0.0,  1.0, 0.0);
    
        let af = a + front;
        let bf = b + front;
        let cf = c + front;
    
        let ab = a + back;
        let bb = b + back;
        let cb = c + back;
    
        // front
        verts.push(v(af, DVec3::Z));
        verts.push(v(bf, DVec3::Z));
        verts.push(v(cf, DVec3::Z));
    
        // back
        verts.push(v(ab, DVec3::NEG_Z));
        verts.push(v(cb, DVec3::NEG_Z));
        verts.push(v(bb, DVec3::NEG_Z));
    
        verts.push(v(ab, DVec3::NEG_Y));
        verts.push(v(bb, DVec3::NEG_Y));
        verts.push(v(bf, DVec3::NEG_Y));
        verts.push(v(ab, DVec3::NEG_Y));
        verts.push(v(bf, DVec3::NEG_Y));
        verts.push(v(af, DVec3::NEG_Y));
    
        verts.push(v(bb, DVec3::new( 2.0,  1.0, 0.0).normalize()));
        verts.push(v(cb, DVec3::new( 2.0,  1.0, 0.0).normalize()));
        verts.push(v(cf, DVec3::new( 2.0,  1.0, 0.0).normalize()));
        verts.push(v(bb, DVec3::new( 2.0,  1.0, 0.0).normalize()));
        verts.push(v(cf, DVec3::new( 2.0,  1.0, 0.0).normalize()));
        verts.push(v(bf, DVec3::new( 2.0,  1.0, 0.0).normalize()));
    
        verts.push(v(cb, DVec3::new(-2.0, -1.0, 0.0).normalize()));
        verts.push(v(ab, DVec3::new(-2.0, -1.0, 0.0).normalize()));
        verts.push(v(af, DVec3::new(-2.0, -1.0, 0.0).normalize()));
        verts.push(v(cb, DVec3::new(-2.0, -1.0, 0.0).normalize()));
        verts.push(v(af, DVec3::new(-2.0, -1.0, 0.0).normalize()));
        verts.push(v(cf, DVec3::new(-2.0, -1.0, 0.0).normalize()));
    
        Self { vertices: verts }
    }
}
