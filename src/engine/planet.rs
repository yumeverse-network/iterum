use glam::DVec3;
use crate::engine::mesh::Vertices;

#[derive(Clone, Copy)]
pub struct Chunk {
    pub face: u8,
    pub level: u32,
    pub x: u32,
    pub y: u32,
    pub center_dir: DVec3, // unit vector
    pub world_size: f64,   // approximate
}

pub struct PlanetChunkMesh {
    pub center_dir: DVec3,  // for camera-relative translation
    pub world_size: f64,
    pub vertices: Vec<Vertices>,
}

pub fn cube_face_point(face: u8, u: f64, v: f64) -> DVec3 {
    let (x, y, z) = match face {
        0 => ( 1.0,  v,  -u),
        1 => (-1.0,  v,   u),
        2 => ( u,   1.0, -v),
        3 => ( u,  -1.0,  v),
        4 => ( u,   v,   1.0),
        5 => (-u,   v,  -1.0),
        _ => unreachable!(),
    };
    DVec3::new(x, y, z).normalize()
}

/*fn chunk_center_world(planet_center: DVec3, radius: f64, chunk: &Chunk) -> DVec3 {
    planet_center + chunk.center_dir * radius
}*/

pub fn select_chunks(
    planet_center: DVec3,
    radius: f64,
    camera_pos: DVec3,
    max_level: u32,
    pixel_threshold: f64,  // 8.0 --- target chunk size in pixels
    fov_radians: f64,
    screen_height: f64,
) -> Vec<Chunk> {
    let mut out = Vec::new();
    for face in 0..6u8 {
        recurse(face, 0, 0, 0, planet_center, radius, camera_pos,
                max_level, pixel_threshold, fov_radians, screen_height, &mut out);
    }
    out
}

fn recurse(
    face: u8, level: u32, x: u32, y: u32,
    planet_center: DVec3, radius: f64, camera_pos: DVec3,
    max_level: u32, px_thresh: f64, fov: f64, screen_h: f64,
    out: &mut Vec<Chunk>,
) {
    let n = 1u32 << level;
    // u, v in [-1, 1] so each face covers its full square.
    let u = ((x as f64 + 0.5) / n as f64) * 2.0 - 1.0;
    let v = ((y as f64 + 0.5) / n as f64) * 2.0 - 1.0;
    let center_dir = cube_face_point(face, u, v);

    let chunk_world_center = planet_center + center_dir * radius;
    let dist = (chunk_world_center - camera_pos).length();

    // Approx angular size of one chunk at this level.
    let face_arc = std::f64::consts::FRAC_PI_2 * radius;  // arc length of one cube face
    let chunk_world_size = face_arc / n as f64;
    let angular = chunk_world_size / dist.max(1.0);

    let pixels = angular * (screen_h / (2.0 * (fov / 2.0).tan()));

    if pixels < 0.5 {
        return;
    }

    if pixels > px_thresh && level < max_level {
        for dy in 0..2 {
            for dx in 0..2 {
                recurse(face, level + 1, x * 2 + dx, y * 2 + dy,
                        planet_center, radius, camera_pos,
                        max_level, px_thresh, fov, screen_h, out);
            }
        }
    } else {
        out.push(Chunk { face, level, x, y, center_dir, world_size: chunk_world_size });
    }
}

pub fn build_chunk_mesh(
    chunk: &Chunk,
    planet_center: DVec3,
    radius: f64,
    _camera_pos: DVec3,
    resolution: u32,  // verts per side, e.g. 16
) -> PlanetChunkMesh {
    let n = 1u32 << chunk.level;
    // u, v in [-1, 1] so each face covers its full square.
    let u0 = (chunk.x as f64 / n as f64) * 2.0 - 1.0;
    let v0 = (chunk.y as f64 / n as f64) * 2.0 - 1.0;
    let du = 2.0 / n as f64;
    let dv = 2.0 / n as f64;

    let mut vertices = Vec::with_capacity(((resolution + 1) * (resolution + 1) * 6) as usize);

    // Grid verts -> camera-relative f32
    let mut grid: Vec<Vertices> = Vec::new();
    for j in 0..=resolution {
        for i in 0..=resolution {
            let u = u0 + du * (i as f64 / resolution as f64);
            let v = v0 + dv * (j as f64 / resolution as f64);
            let dir = cube_face_point(chunk.face, u, v);
            let world = planet_center + dir * radius;
            let rel = (world - planet_center).as_vec3();
            grid.push(Vertices {
                position: rel.to_array(),
                normal: dir.as_vec3().to_array(),
            });
        }
    }

    // Triangulate grid
    let w = (resolution + 1) as usize;
    for j in 0..resolution as usize {
        for i in 0..resolution as usize {
            let a = grid[j * w + i];
            let b = grid[j * w + i + 1];
            let c = grid[(j + 1) * w + i + 1];
            let d = grid[(j + 1) * w + i];
            vertices.push(a); vertices.push(b); vertices.push(c);
            vertices.push(a); vertices.push(c); vertices.push(d);
        }
    }

    PlanetChunkMesh {
        center_dir: chunk.center_dir,
        world_size: chunk.world_size,
        vertices,
    }
}