use raylib::prelude::{Color, Vector2};

use crate::framebuffer::Framebuffer;
use crate::level::{Level, Tile};

pub const MINIMAP_BOX_WIDTH: i32 = 210;
pub const MINIMAP_BOX_HEIGHT: i32 = 145;
pub const MINIMAP_MARGIN_RIGHT: i32 = 12;
pub const MINIMAP_MARGIN_TOP: i32 = 52;

pub const MINIMAP_BG_COLOR: Color = Color::new(10, 14, 22, 215);
pub const MINIMAP_BORDER_COLOR: Color = Color::new(55, 70, 90, 255);
pub const MINIMAP_WALL_COLOR: Color = Color::new(60, 72, 88, 255);
pub const MINIMAP_FLOOR_COLOR: Color = Color::new(20, 24, 32, 255);
pub const MINIMAP_TARGET_PENDING_COLOR: Color = Color::new(255, 215, 0, 255);
pub const MINIMAP_TARGET_DONE_COLOR: Color = Color::new(0, 230, 255, 255);
pub const MINIMAP_DECORATIVE_COLOR: Color = Color::new(130, 170, 200, 255);
pub const MINIMAP_EXIT_LOCKED_COLOR: Color = Color::new(235, 55, 65, 255);
pub const MINIMAP_EXIT_OPEN_COLOR: Color = Color::new(50, 240, 120, 255);
pub const MINIMAP_GUARD_COLOR: Color = Color::new(255, 80, 50, 255);
pub const MINIMAP_PLAYER_COLOR: Color = Color::new(255, 255, 255, 255);
pub const MINIMAP_HEADING_COLOR: Color = Color::new(255, 230, 80, 255);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinimapTransform {
    pub origin_x: f32,
    pub origin_y: f32,
    pub cell_size: f32,
    pub map_width: usize,
    pub map_height: usize,
}

impl MinimapTransform {
    pub fn compute(
        box_x: i32,
        box_y: i32,
        box_width: i32,
        box_height: i32,
        map_width: usize,
        map_height: usize,
    ) -> Self {
        let padding = 8.0;
        let available_w = (box_width as f32 - padding * 2.0).max(1.0);
        let available_h = (box_height as f32 - padding * 2.0).max(1.0);

        let cell_w = available_w / map_width.max(1) as f32;
        let cell_h = available_h / map_height.max(1) as f32;
        let cell_size = cell_w.min(cell_h).max(2.0);

        let total_map_w = map_width as f32 * cell_size;
        let total_map_h = map_height as f32 * cell_size;

        let origin_x = box_x as f32 + (box_width as f32 - total_map_w) / 2.0;
        let origin_y = box_y as f32 + (box_height as f32 - total_map_h) / 2.0;

        Self {
            origin_x,
            origin_y,
            cell_size,
            map_width,
            map_height,
        }
    }

    pub fn world_to_minimap(&self, world_pos: Vector2) -> Vector2 {
        Vector2::new(
            self.origin_x + world_pos.x * self.cell_size,
            self.origin_y + world_pos.y * self.cell_size,
        )
    }
}

pub fn render_minimap(
    framebuffer: &mut Framebuffer,
    level: &Level,
    player_pos: Vector2,
    player_angle: f32,
    exit_unlocked: bool,
) {
    let box_x = framebuffer.width as i32 - MINIMAP_MARGIN_RIGHT - MINIMAP_BOX_WIDTH;
    let box_y = MINIMAP_MARGIN_TOP;

    framebuffer.draw_rectangle(
        box_x,
        box_y,
        MINIMAP_BOX_WIDTH,
        MINIMAP_BOX_HEIGHT,
        MINIMAP_BG_COLOR,
    );
    framebuffer.draw_rectangle_lines(
        box_x,
        box_y,
        MINIMAP_BOX_WIDTH,
        MINIMAP_BOX_HEIGHT,
        1,
        MINIMAP_BORDER_COLOR,
    );

    let map_height = level.maze.len();
    let map_width = level.maze.first().map_or(0, |row| row.len());
    if map_width == 0 || map_height == 0 {
        return;
    }

    let transform = MinimapTransform::compute(
        box_x,
        box_y,
        MINIMAP_BOX_WIDTH,
        MINIMAP_BOX_HEIGHT,
        map_width,
        map_height,
    );

    for (row_idx, row) in level.maze.iter().enumerate() {
        for (col_idx, tile) in row.iter().enumerate() {
            let cell_x = (transform.origin_x + col_idx as f32 * transform.cell_size) as i32;
            let cell_y = (transform.origin_y + row_idx as f32 * transform.cell_size) as i32;
            let cell_sz = (transform.cell_size.ceil() as i32).max(1);

            let color = match tile {
                Tile::Wall(_) => MINIMAP_WALL_COLOR,
                Tile::Floor => MINIMAP_FLOOR_COLOR,
                Tile::DecorativePainting => MINIMAP_DECORATIVE_COLOR,
                Tile::TargetPainting => {
                    let is_completed = level
                        .paintings
                        .iter()
                        .find(|p| p.map_position == (row_idx, col_idx))
                        .is_some_and(|p| p.hits >= 3);
                    if is_completed {
                        MINIMAP_TARGET_DONE_COLOR
                    } else {
                        MINIMAP_TARGET_PENDING_COLOR
                    }
                }
                Tile::Exit => {
                    if exit_unlocked {
                        MINIMAP_EXIT_OPEN_COLOR
                    } else {
                        MINIMAP_EXIT_LOCKED_COLOR
                    }
                }
            };

            framebuffer.draw_rectangle(cell_x, cell_y, cell_sz, cell_sz, color);
        }
    }

    for guard in &level.guards {
        let g_pos = transform.world_to_minimap(guard.position);
        let radius = (transform.cell_size * 0.35).clamp(2.0, 4.0) as i32;
        framebuffer.draw_circle(g_pos.x as i32, g_pos.y as i32, radius, MINIMAP_GUARD_COLOR);
    }

    let p_pos = transform.world_to_minimap(player_pos);
    let p_radius = (transform.cell_size * 0.4).clamp(2.0, 4.0) as i32;
    framebuffer.draw_circle(
        p_pos.x as i32,
        p_pos.y as i32,
        p_radius,
        MINIMAP_PLAYER_COLOR,
    );

    let line_len = (transform.cell_size * 1.3).clamp(5.0, 10.0);
    let end_x = (p_pos.x + player_angle.cos() * line_len) as i32;
    let end_y = (p_pos.y + player_angle.sin() * line_len) as i32;
    framebuffer.draw_line(
        p_pos.x as i32,
        p_pos.y as i32,
        end_x,
        end_y,
        MINIMAP_HEADING_COLOR,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::{Guard, PaintingTarget, WallMaterial};

    fn sample_level(w: usize, h: usize) -> Level {
        let wall = Tile::Wall(WallMaterial::Gallery);
        let mut maze = vec![vec![Tile::Floor; w]; h];
        maze[0].fill(wall);
        maze[h - 1].fill(wall);
        for row in &mut maze {
            row[0] = wall;
            row[w - 1] = wall;
        }
        maze[1][2] = Tile::TargetPainting;
        maze[1][3] = Tile::DecorativePainting;
        maze[h - 2][w - 2] = Tile::Exit;

        Level {
            maze,
            player_spawn: Vector2::new(1.5, 1.5),
            exit: Vector2::new((w - 2) as f32 + 0.5, (h - 2) as f32 + 0.5),
            guards: vec![Guard::new(Vector2::new(3.5, 3.5))],
            paintings: vec![PaintingTarget::new((1, 2), None)],
        }
    }

    #[test]
    fn minimap_transform_scales_and_centers_correctly() {
        let transform_16x12 = MinimapTransform::compute(700, 50, 200, 140, 16, 12);
        assert!(transform_16x12.cell_size > 0.0);
        assert!(transform_16x12.origin_x >= 700.0);
        assert!(transform_16x12.origin_y >= 50.0);

        let p1 = transform_16x12.world_to_minimap(Vector2::new(0.0, 0.0));
        assert_eq!(p1.x, transform_16x12.origin_x);
        assert_eq!(p1.y, transform_16x12.origin_y);

        let transform_32x20 = MinimapTransform::compute(700, 50, 200, 140, 32, 20);
        assert!(transform_32x20.cell_size > 0.0);
        assert!(transform_32x20.cell_size < transform_16x12.cell_size);
    }

    #[test]
    fn minimap_renders_cleanly_for_all_map_sizes() {
        let mut fb = Framebuffer::new(960, 540, Color::BLACK);
        let level1 = sample_level(16, 12);
        let level2 = sample_level(24, 16);
        let level3 = sample_level(32, 20);

        render_minimap(&mut fb, &level1, Vector2::new(1.5, 1.5), 0.0, false);
        render_minimap(&mut fb, &level2, Vector2::new(2.5, 2.5), 1.0, true);
        render_minimap(&mut fb, &level3, Vector2::new(3.5, 3.5), -1.5, false);
    }
}
