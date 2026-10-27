use MathUtils;
use bevy_ecs::prelude::*;
use glam::{DVec3, Vec3};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

pub struct MeshLod {
    pub indices: Vec<u32>,
    pub error: f32,
}

static NEXT_MESH_ID: AtomicU64 = AtomicU64::new(1);

#[derive(BufferContents, Vertex, Copy, Clone)]
#[repr(C)]
pub struct Vertices {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],
}

#[inline]
fn f32_key(f: f32) -> u32 {
    if f == 0.0 { 0 } else { f.to_bits() }
}

impl Vertices {
    #[inline]
    fn key(&self) -> [u32; 6] {
        [
            f32_key(self.position[0]),
            f32_key(self.position[1]),
            f32_key(self.position[2]),
            f32_key(self.normal[0]),
            f32_key(self.normal[1]),
            f32_key(self.normal[2]),
        ]
    }
}

impl PartialEq for Vertices {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Eq for Vertices {}

impl Hash for Vertices {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}

fn v(position: DVec3, normal: DVec3) -> Vertices {
    Vertices {
        position: position.as_vec3().to_array(),
        normal: normal.as_vec3().to_array(),
    }
}

#[derive(Copy, Clone)]
pub struct Bounds {
    pub center: Vec3,
    pub radius: f32,
}

impl Bounds {
    pub fn from_vertices(verts: &[Vertices]) -> Self {
        if verts.is_empty() {
            return Self {
                center: Vec3::ZERO,
                radius: 0.0,
            };
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for v in verts {
            let p = Vec3::from(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        let center = (min + max) * 0.5;
        let radius = verts
            .iter()
            .map(|v| (Vec3::from(v.position) - center).length())
            .fold(0.0, f32::max);
        Self { center, radius }
    }
}

#[derive(Component)]
pub struct Mesh {
    pub id: u64,
    pub vertices: Vec<Vertices>,
    pub indices: Vec<u32>,
    pub lods: Vec<MeshLod>,
    pub bounds: Bounds,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertices>) -> Self {
        let mut lookup: std::collections::HashMap<Vertices, u32> =
            std::collections::HashMap::with_capacity(vertices.len());
        let mut unique: Vec<Vertices> = Vec::new();
        let mut indices: Vec<u32> = Vec::with_capacity(vertices.len());

        for vert in vertices {
            let index = *lookup.entry(vert).or_insert_with(|| {
                unique.push(vert);
                (unique.len() - 1) as u32
            });
            indices.push(index);
        }

        unique.shrink_to_fit();
        
        let bounds = Bounds::from_vertices(&unique);
        
        let mut mesh = Self {
            id: NEXT_MESH_ID.fetch_add(1, Ordering::Relaxed),
            vertices: unique,
            indices,
            lods: Vec::new(),
            bounds,
        };
        
        if mesh.indices.len() / 3 > 100 {
            mesh.generate_lods()
                .expect("Failed to generate mesh LODs");
        }
        
        mesh
    }

    fn from_indexed(vertices: Vec<Vertices>, indices: Vec<u32>) -> Self {
        let bounds = Bounds::from_vertices(&vertices);
        let mut mesh = Self {
            id: NEXT_MESH_ID.fetch_add(1, Ordering::Relaxed),
            vertices,
            indices,
            lods: Vec::new(),
            bounds,
        };
        if mesh.indices.len() / 3 > 100 {
            if let Err(e) = mesh.generate_lods() {
                eprintln!("LOD generation skipped: {e:?}");
                mesh.lods.clear();
            }
        }
        mesh
    }

    pub fn generate_lods(&mut self) -> Result<(), meshoptimizer_rs::Error> {
        let positions: Vec<[f32; 3]> = self
            .vertices
            .iter()
            .map(|v| v.position)
            .collect();
    
        let mut workspace = meshoptimizer_rs::Workspace::default();
    
        self.lods.clear();

        // Amount of detail loss for the LOD subdivision
        for divisor in [16, 64, 256] {
            let target_index_count = (self.indices.len() / divisor / 3) * 3;
        
            let result = meshoptimizer_rs::simplify(
                &self.indices,
                meshoptimizer_rs::Positions::from_packed(&positions),
                meshoptimizer_rs::SimplifySettings {
                    target_index_count,
                    target_error: 0.042, // How much detail loss is acceptable
                    options: meshoptimizer_rs::SimplifyOptions::EMPTY,
                },
                &mut workspace,
            )?;
        
            println!(
                "LOD divisor {divisor}: target {} triangles, actual {} triangles, error {}",
                target_index_count / 3,
                result.indices.len() / 3,
                result.error,
            );

            let prev_len = self.lods.last().map(|l| l.indices.len()).unwrap_or(self.indices.len());
            if result.indices.len() >= prev_len {
                continue; // no reduction, don't store a duplicate LOD
            }
            self.lods.push(MeshLod { indices: result.indices, error: result.error });
        
            /*self.lods.push(MeshLod {
                indices: result.indices,
                error: result.error,
            });*/
        }
    
        Ok(())
    }

    // Cube
    pub fn cube() -> Self {
        Self::new(vec![
            // Front
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::Z),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::Z),
            v(DVec3::new(1.0, 1.0, 1.0), DVec3::Z),
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::Z),
            v(DVec3::new(1.0, 1.0, 1.0), DVec3::Z),
            v(DVec3::new(-1.0, 1.0, 1.0), DVec3::Z),
            // Back
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(-1.0, 1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(-1.0, 1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(1.0, 1.0, -1.0), DVec3::NEG_Z),
            // Left
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_X),
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::NEG_X),
            v(DVec3::new(-1.0, 1.0, 1.0), DVec3::NEG_X),
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_X),
            v(DVec3::new(-1.0, 1.0, 1.0), DVec3::NEG_X),
            v(DVec3::new(-1.0, 1.0, -1.0), DVec3::NEG_X),
            // Right
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::X),
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::X),
            v(DVec3::new(1.0, 1.0, -1.0), DVec3::X),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::X),
            v(DVec3::new(1.0, 1.0, -1.0), DVec3::X),
            v(DVec3::new(1.0, 1.0, 1.0), DVec3::X),
            // Top
            v(DVec3::new(-1.0, 1.0, 1.0), DVec3::Y),
            v(DVec3::new(1.0, 1.0, 1.0), DVec3::Y),
            v(DVec3::new(1.0, 1.0, -1.0), DVec3::Y),
            v(DVec3::new(-1.0, 1.0, 1.0), DVec3::Y),
            v(DVec3::new(1.0, 1.0, -1.0), DVec3::Y),
            v(DVec3::new(-1.0, 1.0, -1.0), DVec3::Y),
            // Bottom
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Y),
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::NEG_Y),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::NEG_Y),
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Y),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::NEG_Y),
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::NEG_Y),
        ])
    }

    // Icosphere
    pub fn icosphere(subdivisions: u32) -> Self {
        use std::collections::HashMap;

        let t = (1.0 + 5.0_f64.sqrt()) / 2.0;

        let mut positions = vec![
            DVec3::new(-1.0, t, 0.0),
            DVec3::new(1.0, t, 0.0),
            DVec3::new(-1.0, -t, 0.0),
            DVec3::new(1.0, -t, 0.0),
            DVec3::new(0.0, -1.0, t),
            DVec3::new(0.0, 1.0, t),
            DVec3::new(0.0, -1.0, -t),
            DVec3::new(0.0, 1.0, -t),
            DVec3::new(t, 0.0, -1.0),
            DVec3::new(t, 0.0, 1.0),
            DVec3::new(-t, 0.0, -1.0),
            DVec3::new(-t, 0.0, 1.0),
        ];

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

                let position = (positions[a as usize] + positions[b as usize]).normalize();

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

        /*let mut vertices = Vec::with_capacity(triangles.len() * 3);

        for [a, b, c] in triangles {
            for index in [a, b, c] {
                let position = positions[index as usize];
                vertices.push(v(position, position));
            }
        }

        Self::new(vertices)*/

        let vertices: Vec<Vertices> = positions.iter().map(|&p| v(p, p)).collect();
        let indices: Vec<u32> = triangles.iter().flatten().copied().collect();
        Self::from_indexed(vertices, indices)
    }

    // Plane
    pub fn plane() -> Self {
        Self::new(vec![
            v(DVec3::new(-1.0, -1.0, 0.0), DVec3::Z),
            v(DVec3::new(1.0, -1.0, 0.0), DVec3::Z),
            v(DVec3::new(1.0, 1.0, 0.0), DVec3::Z),
            v(DVec3::new(-1.0, -1.0, 0.0), DVec3::Z),
            v(DVec3::new(1.0, 1.0, 0.0), DVec3::Z),
            v(DVec3::new(-1.0, 1.0, 0.0), DVec3::Z),
        ])
    }

    // Triangle
    pub fn triangle() -> Self {
        Self::new(vec![
            v(DVec3::new(-1.0, -1.0, 0.0), DVec3::Z),
            v(DVec3::new(1.0, -1.0, 0.0), DVec3::Z),
            v(DVec3::new(0.0, 1.0, 0.0), DVec3::Z),
        ])
    }

    // Circle
    pub fn circle(segments: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        for i in 0..segments {
            let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
            let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(radius * theta0.cos(), radius * theta0.sin(), 0.0);
            let p1 = DVec3::new(radius * theta1.cos(), radius * theta1.sin(), 0.0);

            verts.push(v(DVec3::ZERO, DVec3::Z));
            verts.push(v(p0, DVec3::Z));
            verts.push(v(p1, DVec3::Z));
        }

        Self::new(verts)
    }

    // Cylinder
    pub fn cylinder(segments: i32, radius: f64, depth: f64) -> Self {
        let mut verts = Vec::new();

        let half_depth = depth;

        for i in 0..segments {
            let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
            let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(radius * theta0.cos(), radius * theta0.sin(), 0.0);
            let p1 = DVec3::new(radius * theta1.cos(), radius * theta1.sin(), 0.0);

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
            verts.push(v(p1_bottom, DVec3::NEG_Z));
            verts.push(v(p0_bottom, DVec3::NEG_Z));
        }

        Self::new(verts)
    }

    // Tube
    pub fn tube(segments: i32, radius: f64, depth: f64, thickness: f64) -> Self {
        let mut verts = Vec::new();

        let half_depth = depth;
        let inner_radius: f64 = radius - thickness;

        for i in 0..segments {
            let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
            let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(radius * theta0.cos(), radius * theta0.sin(), 0.0);
            let p1 = DVec3::new(radius * theta1.cos(), radius * theta1.sin(), 0.0);

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
            verts.push(v(p3_top, p3.normalize()));
            verts.push(v(p3_bottom, p3.normalize()));

            verts.push(v(p2_bottom, p2.normalize()));
            verts.push(v(p2_top, p2.normalize()));
            verts.push(v(p3_top, p3.normalize()));

            // Top ring
            verts.push(v(p0_top, DVec3::Z));
            verts.push(v(p1_top, DVec3::Z));
            verts.push(v(p2_top, DVec3::Z));

            verts.push(v(p1_top, DVec3::Z));
            verts.push(v(p3_top, DVec3::Z));
            verts.push(v(p2_top, DVec3::Z));

            // Bottom ring
            verts.push(v(p0_bottom, DVec3::NEG_Z));
            verts.push(v(p2_bottom, DVec3::NEG_Z));
            verts.push(v(p1_bottom, DVec3::NEG_Z));

            verts.push(v(p1_bottom, DVec3::NEG_Z));
            verts.push(v(p2_bottom, DVec3::NEG_Z));
            verts.push(v(p3_bottom, DVec3::NEG_Z));
        }

        Self::new(verts)
    }

    // Disc
    pub fn disc(segments: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        for i in 0..segments {
            let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
            let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

            let p0 = DVec3::new(radius * theta0.cos(), 0.0, radius * theta0.sin());
            let p1 = DVec3::new(radius * theta1.cos(), 0.0, radius * theta1.sin());

            verts.push(v(DVec3::ZERO, DVec3::Y));
            verts.push(v(p1, DVec3::Y));
            verts.push(v(p0, DVec3::Y));
        }

        Self::new(verts)
    }

    // Cone
    pub fn cone(segments: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        let apex = DVec3::new(0.0, 1.0, 0.0);
        let base_center = DVec3::new(0.0, -1.0, 0.0);
        let h = 2.0;
        let down = -DVec3::Y;

        for i in 0..segments {
            let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
            let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

            let dir0 = DVec3::new(theta0.cos(), 0.0, theta0.sin());
            let dir1 = DVec3::new(theta1.cos(), 0.0, theta1.sin());

            let p0 = DVec3::new(radius * dir0.x, -1.0, radius * dir0.z);
            let p1 = DVec3::new(radius * dir1.x, -1.0, radius * dir1.z);

            let n0 = (DVec3::Y * radius + dir0 * h).normalize();
            let n1 = (DVec3::Y * radius + dir1 * h).normalize();

            verts.push(v(apex, n0));
            verts.push(v(p1, n1));
            verts.push(v(p0, n0));

            verts.push(v(base_center, down));
            verts.push(v(p0, down));
            verts.push(v(p1, down));
        }

        Self::new(verts)
    }

    // Triangular prism
    pub fn prism(depth: f64) -> Self {
        let mut verts = Vec::new();

        let front = DVec3::new(0.0, 0.0, depth);
        let back = DVec3::new(0.0, 0.0, -depth);

        let a = DVec3::new(-1.0, -1.0, 0.0);
        let b = DVec3::new(1.0, -1.0, 0.0);
        let c = DVec3::new(0.0, 1.0, 0.0);

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

        // bottom
        verts.push(v(ab, DVec3::NEG_Y));
        verts.push(v(bb, DVec3::NEG_Y));
        verts.push(v(bf, DVec3::NEG_Y));
        verts.push(v(ab, DVec3::NEG_Y));
        verts.push(v(bf, DVec3::NEG_Y));
        verts.push(v(af, DVec3::NEG_Y));

        // right slope
        let n_right = DVec3::new(2.0, 1.0, 0.0).normalize();
        verts.push(v(bb, n_right));
        verts.push(v(cb, n_right));
        verts.push(v(cf, n_right));
        verts.push(v(bb, n_right));
        verts.push(v(cf, n_right));
        verts.push(v(bf, n_right));

        // left slope
        let n_left = DVec3::new(-2.0, 1.0, 0.0).normalize();
        verts.push(v(cb, n_left));
        verts.push(v(ab, n_left));
        verts.push(v(af, n_left));
        verts.push(v(cb, n_left));
        verts.push(v(af, n_left));
        verts.push(v(cf, n_left));

        Self::new(verts)
    }

    // Pyramid
    pub fn pyramid() -> Self {
        Self::new(vec![
            // Front
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::Z),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::Z),
            v(DVec3::new(0.0, 1.0, 0.0), DVec3::Z),
            // Back
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Z),
            v(DVec3::new(0.0, 1.0, 0.0), DVec3::NEG_Z),
            // Left
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_X),
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::NEG_X),
            v(DVec3::new(0.0, 1.0, 0.0), DVec3::NEG_X),
            // Right
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::X),
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::X),
            v(DVec3::new(0.0, 1.0, 0.0), DVec3::X),
            // Bottom
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Y),
            v(DVec3::new(1.0, -1.0, -1.0), DVec3::NEG_Y),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::NEG_Y),
            v(DVec3::new(-1.0, -1.0, -1.0), DVec3::NEG_Y),
            v(DVec3::new(1.0, -1.0, 1.0), DVec3::NEG_Y),
            v(DVec3::new(-1.0, -1.0, 1.0), DVec3::NEG_Y),
        ])
    }

    // UV Sphere
    pub fn uv_sphere(segments: i32, rings: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        for i in 0..rings {
            let phi0 = MathUtils::PI * i as f64 / rings as f64;
            let phi1 = MathUtils::PI * (i + 1) as f64 / rings as f64;

            for i in 0..segments {
                let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
                let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

                let p0 = DVec3::new(
                    radius * phi0.sin() * theta0.cos(),
                    radius * phi0.cos(),
                    radius * phi0.sin() * theta0.sin(),
                );
                let p1 = DVec3::new(
                    radius * phi0.sin() * theta1.cos(),
                    radius * phi0.cos(),
                    radius * phi0.sin() * theta1.sin(),
                );
                let p2 = DVec3::new(
                    radius * phi1.sin() * theta0.cos(),
                    radius * phi1.cos(),
                    radius * phi1.sin() * theta0.sin(),
                );
                let p3 = DVec3::new(
                    radius * phi1.sin() * theta1.cos(),
                    radius * phi1.cos(),
                    radius * phi1.sin() * theta1.sin(),
                );

                verts.push(v(p0, p0.normalize()));
                verts.push(v(p1, p1.normalize()));
                verts.push(v(p2, p2.normalize()));

                verts.push(v(p1, p1.normalize()));
                verts.push(v(p3, p3.normalize()));
                verts.push(v(p2, p2.normalize()));
            }
        }

        Self::new(verts)
    }

    // Hemisphere
    pub fn hemisphere(segments: i32, rings: i32, radius: f64) -> Self {
        let mut verts = Vec::new();

        for i in 0..rings {
            let phi0 = (MathUtils::PI / 2.0) * i as f64 / rings as f64;
            let phi1 = (MathUtils::PI / 2.0) * (i + 1) as f64 / rings as f64;

            for i in 0..segments {
                let theta0 = 2.0 * MathUtils::PI * i as f64 / segments as f64;
                let theta1 = 2.0 * MathUtils::PI * (i + 1) as f64 / segments as f64;

                let p0 = DVec3::new(
                    radius * phi0.sin() * theta0.cos(),
                    radius * phi0.cos() - 0.5,
                    radius * phi0.sin() * theta0.sin(),
                );
                let p1 = DVec3::new(
                    radius * phi0.sin() * theta1.cos(),
                    radius * phi0.cos() - 0.5,
                    radius * phi0.sin() * theta1.sin(),
                );
                let p2 = DVec3::new(
                    radius * phi1.sin() * theta0.cos(),
                    radius * phi1.cos() - 0.5,
                    radius * phi1.sin() * theta0.sin(),
                );
                let p3 = DVec3::new(
                    radius * phi1.sin() * theta1.cos(),
                    radius * phi1.cos() - 0.5,
                    radius * phi1.sin() * theta1.sin(),
                );

                verts.push(v(p0, p0.normalize()));
                verts.push(v(p1, p1.normalize()));
                verts.push(v(p2, p2.normalize()));

                verts.push(v(p1, p1.normalize()));
                verts.push(v(p3, p3.normalize()));
                verts.push(v(p2, p2.normalize()));
            }
        }

        let center = DVec3::new(0.0, -0.5, 0.0);
        let down = DVec3::new(0.0, -1.0, 0.0);

        for j in 0..segments {
            let theta0 = 2.0 * MathUtils::PI * j as f64 / segments as f64;
            let theta1 = 2.0 * MathUtils::PI * (j + 1) as f64 / segments as f64;

            let p0 = DVec3::new(radius * theta0.cos(), -0.5, radius * theta0.sin());
            let p1 = DVec3::new(radius * theta1.cos(), -0.5, radius * theta1.sin());

            verts.push(v(center, down));
            verts.push(v(p0, down));
            verts.push(v(p1, down));
        }

        Self::new(verts)
    }

    // Capsule
    pub fn capsule(segments: i32, rings: i32, radius: f64, height: f64) -> Self {
        let mut verts = Vec::new();

        let half_height = height / 2.0;

        for i in 0..rings {
            let phi0 = (MathUtils::PI / 2.0) * i as f64 / rings as f64;
            let phi1 = (MathUtils::PI / 2.0) * (i + 1) as f64 / rings as f64;
            let phi2 = MathUtils::PI / 2.0 + (MathUtils::PI / 2.0) * i as f64 / rings as f64;
            let phi3 = MathUtils::PI / 2.0 + (MathUtils::PI / 2.0) * (i + 1) as f64 / rings as f64;

            for j in 0..segments {
                let theta0 = 2.0 * MathUtils::PI * j as f64 / segments as f64;
                let theta1 = 2.0 * MathUtils::PI * (j + 1) as f64 / segments as f64;

                let n_b0 = DVec3::new(
                    phi2.sin() * theta0.cos(),
                    phi2.cos(),
                    phi2.sin() * theta0.sin(),
                );
                let n_b1 = DVec3::new(
                    phi2.sin() * theta1.cos(),
                    phi2.cos(),
                    phi2.sin() * theta1.sin(),
                );
                let n_b2 = DVec3::new(
                    phi3.sin() * theta0.cos(),
                    phi3.cos(),
                    phi3.sin() * theta0.sin(),
                );
                let n_b3 = DVec3::new(
                    phi3.sin() * theta1.cos(),
                    phi3.cos(),
                    phi3.sin() * theta1.sin(),
                );

                let n_t0 = DVec3::new(
                    phi0.sin() * theta0.cos(),
                    phi0.cos(),
                    phi0.sin() * theta0.sin(),
                );
                let n_t1 = DVec3::new(
                    phi0.sin() * theta1.cos(),
                    phi0.cos(),
                    phi0.sin() * theta1.sin(),
                );
                let n_t2 = DVec3::new(
                    phi1.sin() * theta0.cos(),
                    phi1.cos(),
                    phi1.sin() * theta0.sin(),
                );
                let n_t3 = DVec3::new(
                    phi1.sin() * theta1.cos(),
                    phi1.cos(),
                    phi1.sin() * theta1.sin(),
                );

                let b0 = DVec3::new(
                    radius * phi2.sin() * theta0.cos(),
                    radius * phi2.cos() - half_height,
                    radius * phi2.sin() * theta0.sin(),
                );
                let b1 = DVec3::new(
                    radius * phi2.sin() * theta1.cos(),
                    radius * phi2.cos() - half_height,
                    radius * phi2.sin() * theta1.sin(),
                );
                let b2 = DVec3::new(
                    radius * phi3.sin() * theta0.cos(),
                    radius * phi3.cos() - half_height,
                    radius * phi3.sin() * theta0.sin(),
                );
                let b3 = DVec3::new(
                    radius * phi3.sin() * theta1.cos(),
                    radius * phi3.cos() - half_height,
                    radius * phi3.sin() * theta1.sin(),
                );

                let t0 = DVec3::new(
                    radius * phi0.sin() * theta0.cos(),
                    radius * phi0.cos() + half_height,
                    radius * phi0.sin() * theta0.sin(),
                );
                let t1 = DVec3::new(
                    radius * phi0.sin() * theta1.cos(),
                    radius * phi0.cos() + half_height,
                    radius * phi0.sin() * theta1.sin(),
                );
                let t2 = DVec3::new(
                    radius * phi1.sin() * theta0.cos(),
                    radius * phi1.cos() + half_height,
                    radius * phi1.sin() * theta0.sin(),
                );
                let t3 = DVec3::new(
                    radius * phi1.sin() * theta1.cos(),
                    radius * phi1.cos() + half_height,
                    radius * phi1.sin() * theta1.sin(),
                );

                // Bottom
                verts.push(v(b0, n_b0));
                verts.push(v(b1, n_b1));
                verts.push(v(b2, n_b2));

                verts.push(v(b1, n_b1));
                verts.push(v(b3, n_b3));
                verts.push(v(b2, n_b2));

                // Top
                verts.push(v(t0, n_t0));
                verts.push(v(t1, n_t1));
                verts.push(v(t2, n_t2));

                verts.push(v(t1, n_t1));
                verts.push(v(t3, n_t3));
                verts.push(v(t2, n_t2));

                let s0 = DVec3::new(radius * theta0.cos(), -half_height, radius * theta0.sin());
                let s1 = DVec3::new(radius * theta1.cos(), -half_height, radius * theta1.sin());
                let s2 = DVec3::new(radius * theta0.cos(), half_height, radius * theta0.sin());
                let s3 = DVec3::new(radius * theta1.cos(), half_height, radius * theta1.sin());

                let n0 = DVec3::new(theta0.cos(), 0.0, theta0.sin());
                let n1 = DVec3::new(theta1.cos(), 0.0, theta1.sin());

                verts.push(v(s0, n0));
                verts.push(v(s2, n0));
                verts.push(v(s1, n1));

                verts.push(v(s1, n1));
                verts.push(v(s2, n0));
                verts.push(v(s3, n1));
            }
        }

        Self::new(verts)
    }
}