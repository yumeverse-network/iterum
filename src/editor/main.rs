/*use glam::DVec3;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use iterum::{
    AssetManager, Engine, camera::Camera, console::Console, emath::EMath, fps::FpsCounter,
    player::Player, renderer::Renderer, window::Window,
};
use std::time::Instant;

fn main() {
    let mut engine = Engine::new();
    let mut assets = AssetManager::new();
    let logger = Console::new();
    let math = EMath::new();
    let mut fps = FpsCounter::new();
    let mut winman = Window::new();
    let mut renderer = Renderer::new();
    let mut player = Player::new();
    let mut camera = Camera::new();

    let mut last_pos = DVec3::ZERO;

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

    assets.mount_iia("hi.iia");

    logger.break_line();

    winman.init_window();

    renderer.init(winman.window.as_ref().unwrap());

    while engine.is_running() {
        let frame_start = Instant::now();

        if !winman.update() {
            engine.quit();
            break;
        }

        // INPUT
        let speed = 130.0;
        let mut dx = 0.0;
        let mut dz = 0.0;

        if winman.is_key_down(iterum::Scancode::W) {
            dz = -speed;
        }
        if winman.is_key_down(iterum::Scancode::S) {
            dz = speed;
        }
        if winman.is_key_down(iterum::Scancode::A) {
            dx = -speed;
        }
        if winman.is_key_down(iterum::Scancode::D) {
            dx = speed;
        }
        player.velocity = DVec3::new(dx, 0.0, dz);

        let dt = engine.get_delta();
        player.update(dt);
        camera.follow(&player);

        // Continue updating
        engine.update();

        // RENDER THING
        let (screen_x, screen_y) = renderer.render_player(&player, &camera);
        renderer.render(winman.window.as_ref().unwrap());

        if player.position != last_pos {
            logger.log(&format!(
                "Player pos: ({:.2}, {:.2}, {:.2})",
                player.position.x, player.position.y, player.position.z
            ));
            last_pos = player.position;
        }

        fps.update(&logger);
        engine.limit_fps(frame_start);
        engine.update_delta();
    }
}
*/
