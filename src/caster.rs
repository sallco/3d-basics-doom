use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, cell_color};
use crate::player::Player;
use crate::textures::TextureManager;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    ray_angle: f32,
    block_size: usize,
    draw_line: bool,
) -> Intersect {
    let mut distance = 0.0;
    let direction_x = ray_angle.cos();
    let direction_y = ray_angle.sin();

    framebuffer.set_current_color(Color::WHITESMOKE);

    loop {
        let pixel_x = player.pos.x + distance * direction_x;
        let pixel_y = player.pos.y + distance * direction_y;

        if pixel_x < 0.0
            || pixel_y < 0.0
            || pixel_x >= framebuffer.width as f32
            || pixel_y >= framebuffer.height as f32
        {
            return Intersect {
                distance,
                impact: '#',
            };
        }

        let x = pixel_x as usize;
        let y = pixel_y as usize;
        let column = x / block_size;
        let row = y / block_size;

        let Some(&cell) = maze.get(row).and_then(|maze_row| maze_row.get(column)) else {
            return Intersect {
                distance,
                impact: '#',
            };
        };

        if matches!(cell, '+' | '-' | '|') {
            return Intersect {
                distance,
                impact: cell,
            };
        }

        if draw_line {
            framebuffer.point(x as u32, y as u32);
        }

        distance += 0.1;
    }
}

pub fn render_3d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    textures: &TextureManager,
) {
    let horizon = framebuffer.height / 2;

    framebuffer.set_current_color(Color::SKYBLUE);
    for y in 0..horizon {
        for x in 0..framebuffer.width {
            framebuffer.point(x, y);
        }
    }

    framebuffer.set_current_color(Color::BROWN);
    for y in horizon..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.point(x, y);
        }
    }

    let num_rays = framebuffer.width;
    let half_width = framebuffer.width as f32 / 2.0;
    let half_height = framebuffer.height as f32 / 2.0;
    let distance_to_projection_plane = half_width / (player.fov / 2.0).tan();

    framebuffer.set_current_color(Color::WHITESMOKE);

    for column in 0..num_rays {
        let current_ray = column as f32 / num_rays as f32;
        let ray_angle = player.a - player.fov / 2.0 + player.fov * current_ray;
        let intersect = cast_ray(framebuffer, maze, player, ray_angle, block_size, false);

        let impact_x = player.pos.x + intersect.distance * ray_angle.cos();
        let impact_y = player.pos.y + intersect.distance * ray_angle.sin();
        let impact_column = (impact_x.max(0.0) as usize) / block_size;
        let impact_row = (impact_y.max(0.0) as usize) / block_size;

        let distance_to_wall = (intersect.distance * (ray_angle - player.a).cos()).max(0.0001);
        let stake_height = (block_size as f32 / distance_to_wall) * distance_to_projection_plane;
        let stake_top = (half_height - stake_height / 2.0).max(0.0) as u32;
        let stake_bottom = (half_height + stake_height / 2.0).min(framebuffer.height as f32) as u32;

        let fallback_color = cell_color(intersect.impact, impact_row, impact_column);
        let texture_dimensions = textures.dimensions(intersect.impact);
        let wall_offset = if intersect.impact == '|' {
            impact_y.rem_euclid(block_size as f32)
        } else {
            impact_x.rem_euclid(block_size as f32)
        };
        let stake_length = (stake_bottom - stake_top).max(1);

        for y in stake_top..stake_bottom {
            let color = texture_dimensions
                .and_then(|(texture_width, texture_height)| {
                    let texture_x = (wall_offset / block_size as f32 * texture_width as f32) as u32;
                    let texture_y = ((y - stake_top) as f32 / stake_length as f32
                        * texture_height as f32) as u32;

                    textures.get_pixel_color(intersect.impact, texture_x, texture_y)
                })
                .unwrap_or(fallback_color);

            framebuffer.set_current_color(color);
            framebuffer.point(column, y);
        }
    }
}
