pub mod asset_manager;
pub mod audio_manager;
pub mod camera;
pub mod console;
pub mod emath;
pub mod fps;
pub mod fs_utils;
pub mod input;
pub mod mesh;
pub mod nodes;
pub mod physics;
pub mod player;
pub mod profiler;
pub mod render;
pub mod transform;

pub mod test_scene;

use bevy_ecs::world::World;
use std::time::{Duration, Instant};

pub use crate::camera::Camera;
use crate::physics::PhysicsWorld;
use crate::physics::{FIXED_DT, sync_transforms};

pub use console::Console;

use glam::DVec3;

pub enum FrameLimit {
    _Unlimited,
    _Capped(u32),
}

pub struct Engine {
    running: bool,
    pub camera: Camera,
    pub frame_limit: FrameLimit,
    pub delta_time: Duration,
    last_frame: Instant,
    pub _on_update: Option<Box<dyn FnMut()>>,
    pub _on_fixed_update: Option<Box<dyn FnMut()>>,
    logger: Console,
    last: Instant,

    // Profiler
    ram: u64,
    vram: Option<u64>,

    // ECS
    pub world: World,

    // Physics
    pub physics_world: PhysicsWorld,
    physics_accumulator: f64,
}

impl Engine {
    pub fn new() -> Self {
        let mut camera = Camera::new();
        camera.position = DVec3::new(2.0, 3.0, 5.0);

        Engine {
            running: true,
            camera,
            world: World::new(),
            frame_limit: FrameLimit::_Capped(0),
            delta_time: Duration::ZERO,
            last_frame: Instant::now(),
            _on_update: None,
            _on_fixed_update: None,
            logger: Console::new(),
            physics_world: PhysicsWorld::new(),
            physics_accumulator: 0.0,
            last: Instant::now(),
            ram: profiler::get_ram_usage().unwrap(),
            vram: profiler::get_vram_usage().unwrap(),
        }
    }

    pub fn init_world(&mut self) {
        self.load_scene("test");
    }

    pub fn load_scene(&mut self, name: &str) {
        self.world.clear_entities();
        self.physics_world = PhysicsWorld::new();

        match name {
            "test" => test_scene::spawn(&mut self.world, &mut self.physics_world),
            other => self.logger.log_system(&format!("Unknown scene: {other}")),
        }

        self.logger
            .log_system(&format!("Entities: {}", self.world.entities().len()));
    }

    pub fn init(&self) {
        let ver = option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown");

        self.logger.log_system("-+ Iterum Engine +-");
        self.logger.log_system(format!("Version: {}", ver));
        self.logger.break_line();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn update(&mut self) {
        if let Some(ref mut callback) = self._on_update {
            callback();
        }

        // Once a second update
        let elapsed = self.last.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.last = Instant::now();

            self.ram = profiler::get_ram_usage().unwrap();
            self.vram = profiler::get_vram_usage().unwrap();
        }
    }

    fn step_physics(&mut self) {
        const MAX_STEPS: u32 = 5; // avoids the "spiral of death" after a lag spike
        self.physics_accumulator += self.get_delta();

        let mut steps = 0;
        while self.physics_accumulator >= FIXED_DT && steps < MAX_STEPS {
            if let Some(ref mut callback) = self._on_fixed_update {
                callback();
            }
            self.physics_world.step();
            self.physics_accumulator -= FIXED_DT;
            steps += 1;
        }
        if steps == MAX_STEPS {
            self.physics_accumulator = 0.0;
        }

        sync_transforms(&mut self.world, &self.physics_world);
    }

    pub fn _fixed_update(&mut self) {
        if let Some(ref mut callback) = self._on_fixed_update {
            callback();
        }
    }

    pub fn update_delta(&mut self) {
        let now = Instant::now();
        let dt = now - self.last_frame;
        self.delta_time = dt.min(Duration::from_secs_f64(0.1));
        self.last_frame = now;

        self.step_physics();
    }

    pub fn get_delta(&self) -> f64 {
        self.delta_time.as_secs_f64()
    }

    pub fn get_ram_usage(&self) -> u64 {
        self.ram
    }

    pub fn get_vram_usage(&self) -> Option<u64> {
        self.vram
    }

    pub fn limit_fps(&self, frame_start: Instant) {
        match self.frame_limit {
            FrameLimit::_Unlimited => {}

            FrameLimit::_Capped(0) => {}

            FrameLimit::_Capped(fps) => {
                let target = Duration::from_secs_f64(1.0 / fps as f64);

                while frame_start.elapsed() < target {
                    let remaining = target
                        .checked_sub(frame_start.elapsed())
                        .unwrap_or_default();

                    if remaining > Duration::from_millis(2) {
                        std::thread::sleep(Duration::from_millis(1));
                    } else {
                        std::hint::spin_loop();
                    }
                }
            }
        }
    }

    pub fn load_image(&self, bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let img = image::load_from_memory(bytes).unwrap().to_rgba8();
        (img.width(), img.height(), img.into_raw())
    }
}
