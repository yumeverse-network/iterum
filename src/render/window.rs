use std::time::Duration;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{DeviceEvent, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

use crate::input::{InputEvent, InputManager, Key, MouseButton};

pub enum UiEvent {
    MousePos([f32; 2]),
    MouseButton(imgui::MouseButton, bool),
    Wheel([f32; 2]),
    Key(imgui::Key, bool),
    Char(char),
    Mods { ctrl: bool, shift: bool, alt: bool, sup: bool },
}

fn map_key(code: winit::keyboard::KeyCode) -> Option<imgui::Key> {
    use winit::keyboard::KeyCode as K;
    Some(match code {
        K::Backspace => imgui::Key::Backspace,
        K::Delete => imgui::Key::Delete,
        K::Enter => imgui::Key::Enter,
        K::Escape => imgui::Key::Escape,
        K::Tab => imgui::Key::Tab,
        K::ArrowLeft => imgui::Key::LeftArrow,
        K::ArrowRight => imgui::Key::RightArrow,
        K::ArrowUp => imgui::Key::UpArrow,
        K::ArrowDown => imgui::Key::DownArrow,
        K::Home => imgui::Key::Home,
        K::End => imgui::Key::End,
        K::KeyA => imgui::Key::A,
        K::KeyC => imgui::Key::C,
        K::KeyV => imgui::Key::V,
        K::KeyX => imgui::Key::X,
        K::KeyY => imgui::Key::Y,
        K::KeyZ => imgui::Key::Z,
        _ => return None,
    })
}

#[derive(Default)]
pub struct Win {
    pub window: Option<winit::window::Window>,
    pub resized: bool,
    pub input: InputManager,
    pub last_mouse_pos: Option<[f32; 2]>,
    pub ui_events: Vec<UiEvent>,
}

impl ApplicationHandler for Win {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attributes = Window::default_attributes()
            .with_title("Iterum")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)) // 800, 600
            .with_resizable(true);

        self.window = Some(event_loop.create_window(win_attributes).unwrap());
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input.process_event(InputEvent::MouseMotion {
                position: self.input.mouse_position(),
                delta: [delta.0 as f32, delta.1 as f32],
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(_) => {
                self.resized = true;
            }

            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;

                if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                    if let Some(k) = map_key(code) {
                        self.ui_events.push(UiEvent::Key(k, pressed));
                    }
                    if let Some(key) = Key::from_scancode(code) {
                        self.input.process_event(InputEvent::Key {
                            key,
                            pressed,
                            repeat: false,
                        });
                    }
                }

                if pressed {
                    if let Some(text) = &event.text {
                        for c in text.chars() {
                            self.ui_events.push(UiEvent::Char(c));
                        }
                    }
                }
            }

            WindowEvent::ModifiersChanged(m) => {
                let s = m.state();
                self.ui_events.push(UiEvent::Mods {
                    ctrl: s.control_key(),
                    shift: s.shift_key(),
                    alt: s.alt_key(),
                    sup: s.super_key(),
                });
            }

            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = state == ElementState::Pressed;

                let ui_button = match button {
                    winit::event::MouseButton::Left => Some(imgui::MouseButton::Left),
                    winit::event::MouseButton::Right => Some(imgui::MouseButton::Right),
                    winit::event::MouseButton::Middle => Some(imgui::MouseButton::Middle),
                    _ => None,
                };
                if let Some(b) = ui_button {
                    self.ui_events.push(UiEvent::MouseButton(b, pressed));
                }

                self.input.process_event(InputEvent::MouseButton {
                    button: MouseButton::from_winit(button),
                    pressed,
                });
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(p) => {
                        (p.x as f32 / 20.0, p.y as f32 / 20.0)
                    }
                };
                self.ui_events.push(UiEvent::Wheel([x, y]));
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_mouse_pos = Some([position.x as f32, position.y as f32]);

                if let Some(w) = &self.window {
                    let p = position.to_logical::<f32>(w.scale_factor());
                    self.ui_events.push(UiEvent::MousePos([p.x, p.y]));
                }
            }

            _ => {}
        }
    }
}

impl Win {
    pub fn new() -> Self {
        Win {
            window: None,
            resized: false,
            input: InputManager::new(),
            last_mouse_pos: None,
            ui_events: Vec::new(),
        }
    }

    pub fn set_relative_mouse_mode(&self, enabled: bool) {
        let window = self.window.as_ref().unwrap();

        window
            .set_cursor_grab(if enabled {
                winit::window::CursorGrabMode::Locked
            } else {
                winit::window::CursorGrabMode::None
            })
            .unwrap();

        window.set_cursor_visible(!enabled);
    }

    pub fn warp_mouse(&self, x: f32, y: f32) {
        if let Some(window) = &self.window {
            let _ = window.set_cursor_position(PhysicalPosition::new(x as f64, y as f64));
        }
    }

    pub fn update(&mut self, event_loop: &mut EventLoop<()>) -> PumpStatus {
        event_loop.pump_app_events(Some(Duration::ZERO), self)
    }
}
