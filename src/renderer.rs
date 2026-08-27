use raylib::prelude::{Color, Vector2};

use crate::assets::{
    AssetManager, Texture, WALL_ACCENT_PATH, WALL_BURGUNDY_PATH, WALL_GALLERY_PATH,
    WALL_SERVICE_PATH, decorative_path_for_tile,
};
use crate::framebuffer::Framebuffer;
use crate::level::{Level, Tile, WallMaterial};
use crate::raycasting::{RayHit, WallSide, cast_ray_dda};

const CEILING_COLOR: Color = Color::new(18, 22, 30, 255);
const FLOOR_COLOR: Color = Color::new(45, 43, 42, 255);
const TARGET_FRAME_COLOR: Color = Color::new(220, 180, 50, 255);

pub fn render_level_3d(
    framebuffer: &mut Framebuffer,
    level: &Level,
    camera_position: Vector2,
    camera_angle: f32,
    field_of_view: f32,
    exit_unlocked: bool,
    asset_manager: &AssetManager,
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
        let Some(hit) = cast_ray_dda(&level.maze, camera_position, ray_angle) else {
            continue;
        };

        let corrected_distance =
            (hit.distance * (ray_angle - camera_angle).cos()).max(f32::EPSILON);
        z_buffer[column as usize] = corrected_distance;

        let wall_height = projection_distance / corrected_distance;
        let wall_top_float = half_height - wall_height / 2.0;
        let wall_top = wall_top_float.max(0.0) as u32;
        let wall_bottom = (half_height + wall_height / 2.0).min(framebuffer.height as f32) as u32;

        let texture = get_tile_texture(&hit, level, asset_manager);
        let target_painting = if matches!(hit.tile, Tile::TargetPainting) {
            level
                .paintings
                .iter()
                .find(|p| p.map_position == hit.map_position)
        } else {
            None
        };

        for y in wall_top..wall_bottom {
            let v = (y as f32 - wall_top_float) / wall_height;
            let mut color = if let Some(painting) = target_painting {
                if let Some(splatter_color) = painting.splatter_color_at(hit.texture_u, v) {
                    splatter_color
                } else if v < 0.05 || v > 0.95 || hit.texture_u < 0.03 || hit.texture_u > 0.97 {
                    TARGET_FRAME_COLOR
                } else if let Some(texture) = texture {
                    texture.sample(hit.texture_u, v)
                } else {
                    tile_color(hit.tile, exit_unlocked)
                }
            } else if let Some(texture) = texture {
                texture.sample(hit.texture_u, v)
            } else {
                tile_color(hit.tile, exit_unlocked)
            };

            if matches!(hit.side, WallSide::Horizontal) {
                color = shade(color, 0.72);
            }

            framebuffer.set_current_color(color);
            framebuffer.point(column, y);
        }
    }

    z_buffer
}

pub fn get_tile_texture<'a>(
    hit: &RayHit,
    level: &Level,
    asset_manager: &'a AssetManager,
) -> Option<&'a Texture> {
    match hit.tile {
        Tile::Wall(WallMaterial::Gallery) => asset_manager.get_texture(WALL_GALLERY_PATH),
        Tile::Wall(WallMaterial::Burgundy) => asset_manager.get_texture(WALL_BURGUNDY_PATH),
        Tile::Wall(WallMaterial::Service) => asset_manager.get_texture(WALL_SERVICE_PATH),
        Tile::Wall(WallMaterial::Accent) => asset_manager.get_texture(WALL_ACCENT_PATH),
        Tile::DecorativePainting => {
            let path = decorative_path_for_tile(hit.map_position.0, hit.map_position.1);
            asset_manager.get_texture(path)
        }
        Tile::TargetPainting => {
            let target = level
                .paintings
                .iter()
                .find(|painting| painting.map_position == hit.map_position);
            if let Some(asset_path) = target.and_then(|p| p.asset_path) {
                asset_manager.get_texture(asset_path)
            } else {
                asset_manager.get_texture(WALL_GALLERY_PATH)
            }
        }
        Tile::Floor | Tile::Exit => None,
    }
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

pub fn tile_color(tile: Tile, exit_unlocked: bool) -> Color {
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

pub fn shade(color: Color, factor: f32) -> Color {
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
    use crate::level::{Guard, PaintingTarget};

    fn test_level() -> Level {
        let wall = Tile::Wall(WallMaterial::Gallery);
        let maze = vec![
            vec![wall; 5],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall; 5],
        ];
        Level {
            maze,
            player_spawn: Vector2::new(2.5, 2.5),
            exit: Vector2::new(2.5, 4.5),
            guards: vec![Guard {
                spawn: Vector2::new(2.5, 2.5),
                position: Vector2::new(2.5, 2.5),
            }],
            paintings: vec![PaintingTarget::new(
                (0, 2),
                Some("src/assets/museum/walls/with_artworks/one/1.jpg"),
            )],
        }
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
        let asset_manager = AssetManager::default();

        let z_buffer = render_level_3d(
            &mut framebuffer,
            &test_level(),
            Vector2::new(2.5, 2.5),
            0.0,
            std::f32::consts::FRAC_PI_3,
            false,
            &asset_manager,
        );

        assert_eq!(z_buffer.len(), framebuffer.width as usize);
        assert!(z_buffer.iter().all(|distance| distance.is_finite()));
        assert!(z_buffer.iter().all(|distance| *distance > 0.0));
    }
}
