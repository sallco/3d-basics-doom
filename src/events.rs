use std::f32::consts::PI;

use raylib::prelude::{KeyboardKey, RaylibHandle};

use crate::maze::Maze;
use crate::player::Player;

fn is_walkable(maze: &Maze, x: f32, y: f32, block_size: usize) -> bool {
    if x < 0.0 || y < 0.0 {
        return false;
    }

    let column = x as usize / block_size;
    let row = y as usize / block_size;

    matches!(
        maze.get(row).and_then(|maze_row| maze_row.get(column)),
        Some(' ' | 'p' | 'g')
    )
}

pub fn process_events(window: &RaylibHandle, player: &mut Player, maze: &Maze, block_size: usize) {
    const MOVE_SPEED: f32 = 5.0;
    const ROTATION_SPEED: f32 = PI / 40.0;

    if window.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a += ROTATION_SPEED;
    }

    let mut movement = 0.0;

    if window.is_key_down(KeyboardKey::KEY_UP) {
        movement += MOVE_SPEED;
    }

    if window.is_key_down(KeyboardKey::KEY_DOWN) {
        movement -= MOVE_SPEED;
    }

    if movement != 0.0 {
        let movement_x = movement * player.a.cos();
        let movement_y = movement * player.a.sin();
        let next_x = player.pos.x + movement_x;

        if is_walkable(maze, next_x, player.pos.y, block_size) {
            player.pos.x = next_x;
        }

        let next_y = player.pos.y + movement_y;

        if is_walkable(maze, player.pos.x, next_y, block_size) {
            player.pos.y = next_y;
        }
    }
}
