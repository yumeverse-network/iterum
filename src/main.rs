use glam::{DQuat, DVec3};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use crate::engine::input::{Action, Binding, InputEvent, InputManager, Key, MouseButton};
use engine::Engine;
use sdl3::event::Event;
mod engine;

use std::time::Instant;

fn main() {
    let mut engine = Engine::new();
    let mut assets = engine::asset_manager::AssetManager::new();
    let logger = engine::console::Console::new();
    let math = engine::emath::EMath::new();
    let mut fps = engine::fps::FpsCounter::new();
    let mut winman = engine::window::Window::new();
    let mut renderer = engine::renderer::Renderer::new();
    let mut input = InputManager::new();
    let mut player = engine::player::Player::new();

    engine.init();

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

    winman.init_window();

    renderer.init(winman.window.as_ref().unwrap());

    // INPUT ACTIONS
    input
        .actions
        .bind("mv_forward", Action::new(vec![Binding::Key(Key::W)]));

    input
        .actions
        .bind("mv_backward", Action::new(vec![Binding::Key(Key::S)]));

    input
        .actions
        .bind("mv_left", Action::new(vec![Binding::Key(Key::A)]));

    input
        .actions
        .bind("mv_right", Action::new(vec![Binding::Key(Key::D)]));

    input
        .actions
        .bind("mv_up", Action::new(vec![Binding::Key(Key::E)]));

    input
        .actions
        .bind("mv_down", Action::new(vec![Binding::Key(Key::Q)]));

    input
        .actions
        .bind("speedup", Action::new(vec![Binding::Key(Key::Shift)]));

    let mut drag_start: Option<[f32; 2]> = None;

    while engine.is_running() {
        let frame_start = Instant::now();
        
        input.begin_frame();

        // WINDOW / INPUT EVENTS
        for event in winman.update() {
            match event {
                Event::Window {
                    win_event: sdl3::event::WindowEvent::Resized(_, _),
                    ..
                } => {
                    renderer.request_swapchain_recreation();
                }

                sdl3::event::Event::Quit { .. } => {
                    engine.quit();
                }

                sdl3::event::Event::KeyDown {
                    scancode: Some(sc),
                    repeat,
                    ..
                } => {
                    if let Some(key) = Key::from_scancode(sc) {
                        input.process_event(InputEvent::Key {
                            key,
                            pressed: true,
                            repeat,
                        });
                    }
                }

                sdl3::event::Event::MouseButtonDown { mouse_btn, .. } => {
                    input.process_event(InputEvent::MouseButton {
                        button: MouseButton::from_sdl(mouse_btn),
                        pressed: true,
                    });
                }

                sdl3::event::Event::MouseButtonUp { mouse_btn, .. } => {
                    input.process_event(InputEvent::MouseButton {
                        button: MouseButton::from_sdl(mouse_btn),
                        pressed: false,
                    });
                }

                //Event::MouseMotion { timestamp, window_id, which, mousestate, x, y, xrel, yrel }
                sdl3::event::Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    input.process_event(InputEvent::MouseMotion {
                        position: [x, y],
                        delta: [xrel, yrel],
                    });
                }

                sdl3::event::Event::KeyUp {
                    scancode: Some(sc), ..
                } => {
                    if let Some(key) = Key::from_scancode(sc) {
                        input.process_event(InputEvent::Key {
                            key,
                            pressed: false,
                            repeat: false,
                        });
                    }
                }

                _ => {}
            }
        }

        // INPUT
        let speed;
        let mut dx = 0.0;
        let mut dz = 0.0;
        let mut dy = 0.0;

        if input.is_action_pressed("speedup") {
            speed = 1.0e7;
        } else {
            speed = 10.0;
        }

        //let [mx, my] = input.mouse_position();
        let [mdx, mdy] = input.mouse_delta();
        let m_sens = 0.02;

        if input.is_mouse_button_just_pressed(MouseButton::Right) {
            drag_start = Some(input.mouse_position());
            winman.set_relative_mouse_mode(true);
        }

        if input.is_mouse_button_pressed(MouseButton::Right) {
            engine.camera.rotate(-mdx * m_sens, -mdy * m_sens);

            if input.is_action_pressed("mv_forward") {
                dz = -speed;
            }

            if input.is_action_pressed("mv_backward") {
                dz = speed;
            }

            if input.is_action_pressed("mv_left") {
                dx = -speed;
            }

            if input.is_action_pressed("mv_right") {
                dx = speed;
            }

            if input.is_action_pressed("mv_up") {
                dy = speed;
            }

            if input.is_action_pressed("mv_down") {
                dy = -speed;
            }
        }
        if input.is_mouse_button_just_released(MouseButton::Right) {
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

        renderer.render(winman.window.as_ref().unwrap(), &mut engine);

        /*if player.position != last_pos {
            logger.log(&format!(
                "Player pos: ({:.2}, {:.2}, {:.2})",
                player.position.x, player.position.y, player.position.z
            ));

            last_pos = player.position;
        }*/

        fps.update(&logger);
        engine.limit_fps(frame_start);
        engine.update_delta();
    }
}
