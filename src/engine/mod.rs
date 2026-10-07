pub mod asset_manager;
pub mod audio_manager;
pub mod camera;
pub mod console;
pub mod emath;
pub mod fps;
pub mod fs_utils;
pub mod mesh;
pub mod player;
pub mod transform;

pub mod render;
pub mod input;
pub mod nodes;
pub mod physics;

pub mod test_scene;

use bevy_ecs::world::World;
use glam::{DVec3, Vec3};

use std::time::{Duration, Instant};

pub use crate::engine::camera::Camera;
use crate::engine::physics::PhysicsWorld;
use crate::engine::physics::{sync_transforms, FIXED_DT};

pub use console::Console;

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

        self.logger.log_system(&format!("Entities: {}", self.world.entities().len()));
    }

    pub fn init(&self) {
        self.logger.log_system("-+ Iterum Engine +-");
        self.logger.log_system("Version: 1.0");
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

        self.step_physics();
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
    }

    pub fn get_delta(&self) -> f64 {
        self.delta_time.as_secs_f64()
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
}
