use raylib::prelude::{Color, Vector2};

use crate::framebuffer::Framebuffer;
use crate::level::{LevelMap, Tile, WallMaterial};
use crate::raycasting::{WallSide, cast_ray_dda};

const CEILING_COLOR: Color = Color::new(18, 22, 30, 255);
const FLOOR_COLOR: Color = Color::new(45, 43, 42, 255);

#[allow(dead_code)] // Game lo utilizará al migrar al modelo Level.
pub fn render_level_3d(
    framebuffer: &mut Framebuffer,
    map: &LevelMap,
    camera_position: Vector2,
    camera_angle: f32,
    field_of_view: f32,
    exit_unlocked: bool,
) -> Vec<f32> {
    render_background(framebuffer);

    let mut z_buffer = vec![f32::INFINITY; framebuffer.width as usize];
    if framebuffer.width == 0
        || framebuffer.height == 0
        || !field_of_view.is_finite()
        || field_of_view <= 0.0
        || field_of_view >= std::f32::consts::PI
    {
        return z_buffer;
    }

    let half_width = framebuffer.width as f32 / 2.0;
    let half_height = framebuffer.height as f32 / 2.0;
    let projection_distance = half_width / (field_of_view / 2.0).tan();

    for column in 0..framebuffer.width {
        let ray_progress = (column as f32 + 0.5) / framebuffer.width as f32;
        let ray_angle = camera_angle - field_of_view / 2.0 + field_of_view * ray_progress;
        let Some(hit) = cast_ray_dda(map, camera_position, ray_angle) else {
            continue;
        };

        let corrected_distance =
            (hit.distance * (ray_angle - camera_angle).cos()).max(f32::EPSILON);
        z_buffer[column as usize] = corrected_distance;

        let wall_height = projection_distance / corrected_distance;
        let wall_top = (half_height - wall_height / 2.0).max(0.0) as u32;
        let wall_bottom = (half_height + wall_height / 2.0).min(framebuffer.height as f32) as u32;
        let mut color = tile_color(hit.tile, exit_unlocked);

        if matches!(hit.side, WallSide::Horizontal) {
            color = shade(color, 0.72);
        }

        framebuffer.set_current_color(color);
        for y in wall_top..wall_bottom {
            framebuffer.point(column, y);
        }
    }

    z_buffer
}

fn render_background(framebuffer: &mut Framebuffer) {
    let horizon = framebuffer.height / 2;

    framebuffer.set_current_color(CEILING_COLOR);
    for y in 0..horizon {
        for x in 0..framebuffer.width {
            framebuffer.point(x, y);
        }
    }

    framebuffer.set_current_color(FLOOR_COLOR);
    for y in horizon..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.point(x, y);
        }
    }
}

fn tile_color(tile: Tile, exit_unlocked: bool) -> Color {
    match tile {
        Tile::Floor => FLOOR_COLOR,
        Tile::Exit if exit_unlocked => Color::new(42, 190, 96, 255),
        Tile::Exit => Color::new(190, 45, 52, 255),
        Tile::Wall(WallMaterial::Gallery) => Color::new(148, 112, 80, 255),
        Tile::Wall(WallMaterial::Burgundy) => Color::new(104, 35, 55, 255),
        Tile::Wall(WallMaterial::Service) => Color::new(92, 98, 105, 255),
        Tile::Wall(WallMaterial::Accent) => Color::new(39, 82, 96, 255),
        Tile::TargetPainting => Color::new(205, 153, 48, 255),
        Tile::DecorativePainting => Color::new(166, 132, 94, 255),
    }
}

fn shade(color: Color, factor: f32) -> Color {
    Color::new(
        (color.r as f32 * factor) as u8,
        (color.g as f32 * factor) as u8,
        (color.b as f32 * factor) as u8,
        color.a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enclosed_map() -> LevelMap {
        let wall = Tile::Wall(WallMaterial::Gallery);
        vec![
            vec![wall; 5],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall; 5],
        ]
    }

    #[test]
    fn wall_materials_have_distinct_fallback_colors() {
        let colors = [
            tile_color(Tile::Wall(WallMaterial::Gallery), false),
            tile_color(Tile::Wall(WallMaterial::Burgundy), false),
            tile_color(Tile::Wall(WallMaterial::Service), false),
            tile_color(Tile::Wall(WallMaterial::Accent), false),
        ];

        for (index, color) in colors.iter().enumerate() {
            assert!(!colors[index + 1..].contains(color));
        }
    }

    #[test]
    fn exit_color_reflects_lock_state() {
        assert_ne!(tile_color(Tile::Exit, false), tile_color(Tile::Exit, true));
    }

    #[test]
    fn horizontal_walls_are_darker() {
        let base = tile_color(Tile::Wall(WallMaterial::Gallery), false);
        let shaded = shade(base, 0.72);

        assert!(shaded.r < base.r);
        assert!(shaded.g < base.g);
        assert!(shaded.b < base.b);
        assert_eq!(shaded.a, base.a);
    }

    #[test]
    fn rendering_builds_depth_for_every_column() {
        let mut framebuffer = Framebuffer::new(80, 60, Color::BLACK);

        let z_buffer = render_level_3d(
            &mut framebuffer,
            &enclosed_map(),
            Vector2::new(2.5, 2.5),
            0.0,
            std::f32::consts::FRAC_PI_3,
            false,
        );

        assert_eq!(z_buffer.len(), framebuffer.width as usize);
        assert!(z_buffer.iter().all(|distance| distance.is_finite()));
        assert!(z_buffer.iter().all(|distance| *distance > 0.0));
    }
}
