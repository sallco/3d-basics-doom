mod caster;
mod framebuffer;
mod maze;
mod player;

use caster::cast_ray;
use framebuffer::Framebuffer;
use maze::{load_maze, render_maze};
use player::Player;
use raylib::prelude::*;

const BLOCK_SIZE: usize = 20;
const SCALE: f32 = 2.0;

fn main() {
    let maze = load_maze("src/assets/maze.txt");

    let (player_row, player_column) = maze
        .iter()
        .enumerate()
        .find_map(|(row_index, row)| {
            row.iter()
                .position(|&cell| cell == 'p')
                .map(|column_index| (row_index, column_index))
        })
        .expect("The maze must contain a player position marked with 'p'");

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

    let player = Player::new(Vector2::new(
        (player_column * BLOCK_SIZE + BLOCK_SIZE / 2) as f32,
        (player_row * BLOCK_SIZE + BLOCK_SIZE / 2) as f32,
    ));

    render_maze(&mut framebuffer, &maze, BLOCK_SIZE);
    cast_ray(&mut framebuffer, &maze, &player, BLOCK_SIZE);
    player.draw(&mut framebuffer);

    while !window.window_should_close() {
        framebuffer.swap_buffers_scaled(&mut window, &raylib_thread, SCALE);
    }
}
