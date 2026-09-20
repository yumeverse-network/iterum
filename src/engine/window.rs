use sdl3::EventPump;
use sdl3::event::Event;

pub struct Window {
    pub sdl_context: Option<sdl3::Sdl>,
    pub window: Option<sdl3::video::Window>,
    pub event_pump: Option<EventPump>,
}

impl Window {
    pub fn new() -> Self {
        Window {
            sdl_context: None,
            window: None,
            event_pump: None,
        }
    }

    pub fn init_window(&mut self) {
        let sdl_context = sdl3::init().unwrap();
        let video = sdl_context.video().unwrap();

        let window = video
            .window("Iterum", 800, 600)
            .vulkan()
            .resizable()
            .position_centered()
            .build()
            .unwrap();

        self.window = Some(window);
        self.event_pump = Some(sdl_context.event_pump().unwrap());
        self.sdl_context = Some(sdl_context);
    }

    pub fn set_relative_mouse_mode(&self, enabled: bool) {
        let window = self.window.as_ref().unwrap();
        let context = self.sdl_context.as_ref().unwrap();
        context.mouse().set_relative_mouse_mode(window, enabled);
    }

    pub fn warp_mouse(&self, x: f32, y: f32) {
        let window = self.window.as_ref().unwrap();
        let context = self.sdl_context.as_ref().unwrap();
        context.mouse().warp_mouse_in_window(window, x, y);
    }

    pub fn update(&mut self) -> Vec<Event> {
        if let Some(event_pump) = &mut self.event_pump {
            return event_pump.poll_iter().collect();
        }

        Vec::new()
    }
}
