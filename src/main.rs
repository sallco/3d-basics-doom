mod ai;
mod assets;
mod events;
mod framebuffer;
mod game;
mod level;
mod levels;
mod minimap;
mod movement;
mod player;
mod raycasting;
mod renderer;

use framebuffer::Framebuffer;
use game::{Game, LOGICAL_HEIGHT, LOGICAL_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use levels::LEVEL_DEFINITIONS;
use raylib::prelude::*;

fn main() {
    if let Err(error) = run() {
        eprintln!("No se pudo iniciar Doom Rust: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (mut window, raylib_thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("Doom Rust")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(LOGICAL_WIDTH, LOGICAL_HEIGHT, Color::BLACK);
    let mut presentation_texture = window
        .load_texture_from_image(&raylib_thread, &framebuffer.color_buffer)
        .map_err(|error| format!("no se pudo crear la textura de presentación: {error}"))?;
    presentation_texture.set_texture_filter(&raylib_thread, TextureFilter::TEXTURE_FILTER_POINT);

    let mut game = Game::new(&LEVEL_DEFINITIONS)?;

    while !window.window_should_close() {
        game.update(&mut window);
        game.render(&mut framebuffer);
        framebuffer.present(&mut window, &raylib_thread, &mut presentation_texture)?;
    }

    Ok(())
}
