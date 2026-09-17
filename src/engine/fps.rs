use crate::engine::console::Console;
use std::time::{Duration, Instant};

pub struct FpsCounter {
    frames: u32,
    last: Instant,
    fps: f32,
}

impl FpsCounter {
    pub fn new() -> Self {
        FpsCounter {
            frames: 0,
            last: Instant::now(),
            fps: 0.0,
        }
    }

    pub fn update(&mut self, logger: &Console) {
        self.frames += 1;

        let elapsed = self.last.elapsed();
        if elapsed >= Duration::from_secs(1) {
            self.fps = self.frames as f32 / self.last.elapsed().as_secs_f32();
            self.frames = 0;
            self.last = Instant::now();

            logger.log_debug(format_args!("{} FPS", self.fps.round()));
        }
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }
}
