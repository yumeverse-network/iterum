pub mod asset_manager;
pub mod audio_manager;
pub mod camera;
pub mod console;
pub mod emath;
pub mod fps;
pub mod mesh;
pub mod player;
pub mod renderer;
pub mod window;
pub mod fs_utils;
pub mod shaders;
pub mod transform;

pub mod planet;

pub mod input;
pub mod nodes;

pub mod test_scene;

use std::time::{Duration, Instant};
use bevy_ecs::world::World;

pub use crate::engine::camera::Camera;
pub use console::Console;

pub enum FrameLimit {
    _Unlimited,
    _Capped(u32),
}

pub struct Engine {
    running: bool,
    pub camera: Camera,
    pub world: World,
    pub frame_limit: FrameLimit,
    pub delta_time: Duration,
    last_frame: Instant,
    pub _on_update: Option<Box<dyn FnMut()>>,
    pub _on_fixed_update: Option<Box<dyn FnMut()>>,
    logger: Console,
}

impl Engine {
    pub fn new() -> Self {
        let mut camera = Camera::new();
        camera.position.z = 5.0;
        camera.position.x = 2.0;
        camera.position.y = 3.0;

        let mut world = World::new();

        // Temporary test object for lighting.
        test_scene::spawn(&mut world);

        Engine {
            running: true,
            camera,
            world,
            frame_limit: FrameLimit::_Capped(60),
            delta_time: Duration::ZERO,
            last_frame: Instant::now(),
            _on_update: None,
            _on_fixed_update: None,
            logger: Console::new(),
        }
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