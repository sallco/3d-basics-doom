use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    let mut distance = 0.0;
    let direction_x = player.a.cos();
    let direction_y = player.a.sin();

    framebuffer.set_current_color(Color::WHITESMOKE);

    loop {
        let pixel_x = player.pos.x + distance * direction_x;
        let pixel_y = player.pos.y + distance * direction_y;

        if pixel_x < 0.0
            || pixel_y < 0.0
            || pixel_x >= framebuffer.width as f32
            || pixel_y >= framebuffer.height as f32
        {
            break;
        }

        let x = pixel_x as usize;
        let y = pixel_y as usize;
        let column = x / block_size;
        let row = y / block_size;

        let Some(&cell) = maze.get(row).and_then(|maze_row| maze_row.get(column)) else {
            break;
        };

        if matches!(cell, '+' | '-' | '|') {
            break;
        }

        framebuffer.point(x as u32, y as u32);
        distance += 5.0;
    }
}
