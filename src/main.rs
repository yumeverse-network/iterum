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
pub mod input;
pub mod nodes;
pub mod physics;
pub mod render;
pub mod editor;

pub mod test_scene;

use bevy_ecs::world::World;
use std::time::{Duration, Instant};

pub use crate::camera::Camera;
use crate::physics::PhysicsWorld;
use crate::physics::{FIXED_DT, sync_transforms};

pub use console::Console;

use glam::{DQuat, DVec3};
use mimalloc::MiMalloc;
use winit::{event_loop::EventLoop, platform::pump_events::PumpStatus};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use crate::{
    input::{Action, Binding, Key, MouseButton},
    render::window::Win,
};

fn main() {
    let mut engine = Engine::new();
    let mut assets = asset_manager::AssetManager::new();
    let logger = console::Console::new();
    let math = emath::EMath::new();
    let mut fps = fps::FpsCounter::new();
    let mut event_loop = EventLoop::new().unwrap();
    let mut winman = Win::new();
    let mut renderer = render::renderer::Renderer::new();
    let mut player = player::Player::new();
    let mut eui = editor::ui::editor::EditorUi::new();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

    engine.init();
    engine.init_world();

    logger.log("Helloooo");

    logger.break_line();

    logger.log_error("Boop");
    logger.log_system("Bap");
    logger.log_warning("Boom");

    logger.break_line();

    logger.log(math.normalize(43.21, 0));
    logger.log(math.to_str(21));

    logger.break_line();

    assets._mount_iia("hi.iia");

    logger.break_line();

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    winman.update(&mut event_loop);
    renderer.init(winman.window.as_ref().unwrap());
    renderer.init_ui(&mut imgui);
    eui.viewport_tex_id = Some(renderer.viewport_tex_id());

    // INPUT ACTIONS
    winman
        .input
        .actions
        .bind("mv_forward", Action::new(vec![Binding::Key(Key::W)]));

    winman
        .input
        .actions
        .bind("mv_backward", Action::new(vec![Binding::Key(Key::S)]));

    winman
        .input
        .actions
        .bind("mv_left", Action::new(vec![Binding::Key(Key::A)]));

    winman
        .input
        .actions
        .bind("mv_right", Action::new(vec![Binding::Key(Key::D)]));

    winman
        .input
        .actions
        .bind("mv_up", Action::new(vec![Binding::Key(Key::E)]));

    winman
        .input
        .actions
        .bind("mv_down", Action::new(vec![Binding::Key(Key::Q)]));

    winman
        .input
        .actions
        .bind("speedup", Action::new(vec![Binding::Key(Key::Shift)]));

    let mut drag_start: Option<[f32; 2]> = None;

    while engine.is_running() {
        let frame_start = Instant::now();

        winman.input.begin_frame();

        if let PumpStatus::Exit(_) = winman.update(&mut event_loop) {
            logger.log_system("Exiting...");
            engine.quit();
            break;
        }

        // WINDOW / INPUT EVENTS
        if winman.resized {
            renderer.request_swapchain_recreation();
            winman.resized = false;
        }

        {
            let io = imgui.io_mut();
            for ev in winman.ui_events.drain(..) {
                match ev {
                    render::window::UiEvent::MousePos(p) => io.add_mouse_pos_event(p),
                    render::window::UiEvent::MouseButton(b, down) => {
                        io.add_mouse_button_event(b, down)
                    }
                    render::window::UiEvent::Wheel(w) => io.add_mouse_wheel_event(w),
                    render::window::UiEvent::Key(k, down) => io.add_key_event(k, down),
                    render::window::UiEvent::Char(c) => io.add_input_character(c),
                    render::window::UiEvent::Mods { ctrl, shift, alt, sup } => {
                        io.add_key_event(imgui::Key::ModCtrl, ctrl);
                        io.add_key_event(imgui::Key::ModShift, shift);
                        io.add_key_event(imgui::Key::ModAlt, alt);
                        io.add_key_event(imgui::Key::ModSuper, sup);
                    }
                }
            }
        }

        // INPUT
        let speed;
        let mut dx = 0.0;
        let mut dz = 0.0;
        let mut dy = 0.0;

        if winman.input.is_action_pressed("speedup") {
            speed = 2.0e10; // 9.46e17 is max speed tested
        } else {
            speed = 10.0;
        }

        let [mdx, mdy] = winman.input.mouse_delta();
        let m_sens = 0.02;

        if eui.hovering_viewport
            && winman
                .input
                .is_mouse_button_just_pressed(MouseButton::Right)
        {
            drag_start = winman.last_mouse_pos;
            winman.set_relative_mouse_mode(true);
        }

        if eui.hovering_viewport && winman.input.is_mouse_button_pressed(MouseButton::Right) {
            engine.camera.rotate(-mdx * m_sens, -mdy * m_sens);

            if winman.input.is_action_pressed("mv_forward") {
                dz = -speed;
            }

            if winman.input.is_action_pressed("mv_backward") {
                dz = speed;
            }

            if winman.input.is_action_pressed("mv_left") {
                dx = -speed;
            }

            if winman.input.is_action_pressed("mv_right") {
                dx = speed;
            }

            if winman.input.is_action_pressed("mv_up") {
                dy = speed;
            }

            if winman.input.is_action_pressed("mv_down") {
                dy = -speed;
            }
        }
        if winman
            .input
            .is_mouse_button_just_released(MouseButton::Right)
        {
            winman.set_relative_mouse_mode(false);

            if let Some([sx, sy]) = drag_start {
                winman.warp_mouse(sx, sy);
            }
            drag_start = None;
        }

        let q = engine.camera.rotation;
        let q = DQuat::from_xyzw(q.x as f64, q.y as f64, q.z as f64, q.w as f64);

        let mv_dir: DVec3 = q * DVec3::new(dx, dy, dz);
        player.velocity = mv_dir;

        let dt = engine.get_delta();

        player.update(dt);
        engine.camera.follow(&player);
        engine.update();

        let window = winman.window.as_ref().unwrap();
        let ext = render::renderer::logical_extent(window);

        let io = imgui.io_mut();
        io.display_size = [ext[0] as f32, ext[1] as f32];
        io.display_framebuffer_scale = [1.0, 1.0];
        io.update_delta_time(std::time::Duration::from_secs_f32((dt as f32).max(0.0001)));

        let ui = imgui.new_frame();
        eui.draw(&ui, &mut engine, &mut player, &fps);
        renderer.set_viewport_size(eui.viewport_size);
        let draw_data = imgui.render();

        renderer.render(window, &mut engine, Some(draw_data));

        fps.update();
        engine.limit_fps(frame_start);
        engine.update_delta();
    }
}

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
