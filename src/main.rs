use glam::{DQuat, DVec3};
use mimalloc::MiMalloc;
use winit::{event_loop::EventLoop, platform::pump_events::PumpStatus};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use crate::engine::{
    input::{Action, Binding, Key, MouseButton},
    render::window::Win,
};
use engine::Engine;
mod engine;

use std::time::Instant;

fn main() {
    let mut engine = Engine::new();
    let mut assets = engine::asset_manager::AssetManager::new();
    let logger = engine::console::Console::new();
    let math = engine::emath::EMath::new();
    let mut fps = engine::fps::FpsCounter::new();
    let mut event_loop = EventLoop::new().unwrap();
    let mut winman = Win::new();
    let mut renderer = engine::render::renderer::Renderer::new();
    let mut player = engine::player::Player::new();
    let mut eui = engine::ui::editor::EditorUi::new();

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
                    engine::render::window::UiEvent::MousePos(p) => io.add_mouse_pos_event(p),
                    engine::render::window::UiEvent::MouseButton(b, down) => {
                        io.add_mouse_button_event(b, down)
                    }
                    engine::render::window::UiEvent::Wheel(w) => io.add_mouse_wheel_event(w),
                    engine::render::window::UiEvent::Key(k, down) => io.add_key_event(k, down),
                    engine::render::window::UiEvent::Char(c) => io.add_input_character(c),
                    engine::render::window::UiEvent::Mods { ctrl, shift, alt, sup } => {
                        io.add_key_event(imgui::Key::ModCtrl, ctrl);
                        io.add_key_event(imgui::Key::ModShift, shift);
                        io.add_key_event(imgui::Key::ModAlt, alt);
                        io.add_key_event(imgui::Key::ModSuper, sup);
                    }
                }
            }
        }

        let ui_wants_mouse = imgui.io().want_capture_mouse;

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
        let ext = engine::render::renderer::logical_extent(window);

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
