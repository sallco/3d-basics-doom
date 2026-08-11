mod framebuffer;
mod maze;

use framebuffer::Framebuffer;
use maze::{load_maze, render_maze};
use raylib::prelude::*;

const BLOCK_SIZE: usize = 20;
const SCALE: f32 = 2.0;

fn main() {
    let maze = load_maze("src/assets/maze.txt");

    let framebuffer_width = maze.iter().map(Vec::len).max().unwrap_or(0) * BLOCK_SIZE;
    let framebuffer_height = maze.len() * BLOCK_SIZE;
    let window_width = (framebuffer_width as f32 * SCALE) as i32;
    let window_height = (framebuffer_height as f32 * SCALE) as i32;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Doom Rust")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(
        framebuffer_width as u32,
        framebuffer_height as u32,
        Color::BLACK,
    );

    render_maze(&mut framebuffer, &maze, BLOCK_SIZE);

    while !window.window_should_close() {
        framebuffer.swap_buffers_scaled(&mut window, &raylib_thread, SCALE);
    }
}
