use iterum_engine::{
    render::{renderer::Renderer, window::Win},
    Engine,
};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    platform::pump_events::PumpStatus,
};

fn main() {
    let mut engine = Engine::new();
    engine.init();
    engine.init_world();

    let mut event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut window_manager = Win::new();
    window_manager.update(&mut event_loop);

    let Some(window) = window_manager.window.as_ref() else {
        return;
    };
    let mut renderer = Renderer::new();
    if let Err(error) = pollster::block_on(renderer.init(
        std::sync::Arc::clone(window),
        event_loop.owned_display_handle(),
    )) {
        eprintln!("renderer initialization failed: {error:#}");
        return;
    }

    while engine.is_running() {
        let frame_start = std::time::Instant::now();
        if let PumpStatus::Exit(_) = window_manager.update(&mut event_loop) {
            break;
        }

        if window_manager.resized {
            let size = window_manager.window.as_ref().unwrap().inner_size();
            renderer.resize(size.width, size.height);
            window_manager.resized = false;
        }

        engine.update_delta();
        engine.update();
        let size = window_manager.window.as_ref().unwrap().inner_size();
        renderer.set_viewport_size([size.width, size.height]);
        renderer.render_scene(&mut engine, None);

        let frame = match renderer.surface().get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                renderer.resize(size.width, size.height);
                continue;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                continue;
            }
            wgpu::CurrentSurfaceTexture::Validation => break,
        };
        renderer.present_viewport(frame);
        engine.limit_fps(frame_start);
    }
}
