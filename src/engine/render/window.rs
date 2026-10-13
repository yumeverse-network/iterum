use winit::{
    application::ApplicationHandler, dpi::{LogicalPosition, PhysicalPosition}, event::{DeviceEvent, ElementState, WindowEvent}, event_loop::{ActiveEventLoop, EventLoop}, window::{Window, WindowId},
};
use std::time::Duration;
use winit::platform::pump_events::{
    EventLoopExtPumpEvents, PumpStatus,
};

use crate::engine::input::{InputEvent, InputManager, Key, MouseButton};

#[derive(Default)]
pub struct Win {
    pub window: Option<winit::window::Window>,
    pub resized: bool,
    pub input: InputManager,
    pub last_mouse_pos: Option<[f32; 2]>,
}

impl ApplicationHandler for Win {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let win_attributes = Window::default_attributes()
            .with_title("Iterum")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)) // 800, 600
            .with_resizable(true);

        self.window = Some(
            event_loop.create_window(win_attributes).unwrap()
        );
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    )
    {
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
        window_id: WindowId,
        event: WindowEvent,
    )
    {
        match event {
            WindowEvent::Resized(_) => {
                self.resized = true;
            }
            
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            
            WindowEvent::KeyboardInput { event, .. } =>
            if let winit::keyboard::PhysicalKey::Code(code) = event.physical_key {
                if let Some(key) = Key::from_scancode(code) {
                    self.input.process_event(InputEvent::Key {
                        key,
                        pressed: event.state == ElementState::Pressed,
                        repeat: false
                    });
                }
            }
            
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.process_event(InputEvent::MouseButton {
                    button: MouseButton::from_winit(button),
                    pressed: state == ElementState::Pressed,
                });
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.last_mouse_pos = Some([
                    position.x as f32,
                    position.y as f32,
                ]);
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
        }
    }

    pub fn set_relative_mouse_mode(&self, enabled: bool) {
        let window = self.window.as_ref().unwrap();

        window.set_cursor_grab(if enabled {
            winit::window::CursorGrabMode::Locked
        } else {
            winit::window::CursorGrabMode::None
        }).unwrap();
        
        window.set_cursor_visible(!enabled);
    }

    pub fn warp_mouse(&self, x: f32, y: f32) {
        if let Some(window) = &self.window {
            let _ = window.set_cursor_position(
                PhysicalPosition::new(x as f64, y as f64)
            );
        }
    }

    pub fn update(&mut self, event_loop: &mut EventLoop<()>) -> PumpStatus {
        event_loop.pump_app_events(Some(Duration::ZERO), self)
    }
}
