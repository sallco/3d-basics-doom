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
    render_background(framebuffer, camera_position, camera_angle, field_of_view);

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

        let fog = (1.0 / (1.0 + corrected_distance * 0.05)).clamp(0.28, 1.0);
        let side_factor = if matches!(hit.side, WallSide::Horizontal) {
            0.76
        } else {
            1.0
        };

        for y in wall_top..wall_bottom {
            let v = (y as f32 - wall_top_float) / wall_height;
            let mut color = if let Some(painting) = target_painting {
                if let Some(splatter_color) = painting.splatter_color_at(hit.texture_u, v) {
                    splatter_color
                } else if v < 0.06 || v > 0.94 || hit.texture_u < 0.04 || hit.texture_u > 0.96 {
                    TARGET_FRAME_COLOR
                } else if let Some(texture) = texture {
                    texture.sample(hit.texture_u, v)
                } else {
                    tile_color(hit.tile, exit_unlocked)
                }
            } else if matches!(hit.tile, Tile::DecorativePainting) {
                if v < 0.04 || v > 0.96 || hit.texture_u < 0.03 || hit.texture_u > 0.97 {
                    Color::new(60, 42, 28, 255)
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

            color = shade(color, side_factor * fog);

            framebuffer.set_current_color(color);
            framebuffer.point(column, y);
        }
    }

    render_guards_3d(
        framebuffer,
        level,
        camera_position,
        camera_angle,
        field_of_view,
        &z_buffer,
        asset_manager,
    );

    z_buffer
}

pub fn render_guards_3d(
    framebuffer: &mut Framebuffer,
    level: &Level,
    camera_position: Vector2,
    camera_angle: f32,
    field_of_view: f32,
    z_buffer: &[f32],
    asset_manager: &AssetManager,
) {
    if framebuffer.width == 0
        || framebuffer.height == 0
        || !field_of_view.is_finite()
        || field_of_view <= 0.0
        || field_of_view >= std::f32::consts::PI
    {
        return;
    }

    let half_width = framebuffer.width as f32 / 2.0;
    let half_height = framebuffer.height as f32 / 2.0;
    let projection_distance = half_width / (field_of_view / 2.0).tan();

    let mut guard_entries: Vec<(&crate::level::Guard, f32)> = level
        .guards
        .iter()
        .map(|guard| {
            let dx = guard.position.x - camera_position.x;
            let dy = guard.position.y - camera_position.y;
            let dist_sq = dx * dx + dy * dy;
            (guard, dist_sq)
        })
        .collect();

    guard_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    for (guard, dist_sq) in guard_entries {
        let distance = dist_sq.sqrt();
        if distance < 0.2 {
            continue;
        }

        let dx = guard.position.x - camera_position.x;
        let dy = guard.position.y - camera_position.y;

        let guard_angle = dy.atan2(dx);
        let mut angle_diff = guard_angle - camera_angle;

        while angle_diff > std::f32::consts::PI {
            angle_diff -= std::f32::consts::TAU;
        }
        while angle_diff < -std::f32::consts::PI {
            angle_diff += std::f32::consts::TAU;
        }

        if angle_diff.abs() > std::f32::consts::FRAC_PI_2 + field_of_view / 2.0 {
            continue;
        }

        let corrected_distance = distance * angle_diff.cos();
        if corrected_distance < 0.2 {
            continue;
        }

        let screen_x = half_width + (angle_diff / (field_of_view / 2.0)) * half_width;

        let sprite_height =
            (projection_distance / corrected_distance).min(framebuffer.height as f32 * 2.0);
        let sprite_width = sprite_height * 0.75;

        let sprite_top = half_height - sprite_height / 2.0;
        let sprite_bottom = half_height + sprite_height / 2.0;
        let sprite_left = screen_x - sprite_width / 2.0;
        let sprite_right = screen_x + sprite_width / 2.0;

        let start_x = (sprite_left.max(0.0).floor() as usize).min(framebuffer.width as usize);
        let end_x = (sprite_right.min(framebuffer.width as f32).ceil() as usize)
            .min(framebuffer.width as usize);

        let sprite_path = match guard.state {
            crate::level::GuardState::Patrol | crate::level::GuardState::Resetting => {
                crate::assets::GUARD_PATROL_PATH
            }
            crate::level::GuardState::Alerted => crate::assets::GUARD_IDLE_PATH,
            crate::level::GuardState::Chase => crate::assets::GUARD_CHASE_PATH,
            crate::level::GuardState::Slowed => crate::assets::GUARD_ANGRY_PATH,
        };

        let Some(texture) = asset_manager.get_texture(sprite_path) else {
            continue;
        };

        for (col, &depth) in z_buffer.iter().enumerate().take(end_x).skip(start_x) {
            if corrected_distance >= depth {
                continue;
            }

            let u = (col as f32 - sprite_left) / sprite_width;
            if !(0.0..=1.0).contains(&u) {
                continue;
            }

            let start_y = (sprite_top.max(0.0).floor() as u32).min(framebuffer.height);
            let end_y = (sprite_bottom.min(framebuffer.height as f32).ceil() as u32)
                .min(framebuffer.height);

            for y in start_y..end_y {
                let v = (y as f32 - sprite_top) / sprite_height;
                if !(0.0..=1.0).contains(&v) {
                    continue;
                }

                let pixel_color = texture.sample(u, v);
                if pixel_color.a < 32
                    || (pixel_color.r == 0
                        && pixel_color.g == 0
                        && pixel_color.b == 0
                        && pixel_color.a == 0)
                {
                    continue;
                }

                let final_color = if guard.splattered {
                    Color::new(
                        ((pixel_color.r as u32 * 255 + 255 * 80) / 335).min(255) as u8,
                        ((pixel_color.g as u32 * 50) / 255).min(255) as u8,
                        ((pixel_color.b as u32 * 150 + 200 * 80) / 335).min(255) as u8,
                        pixel_color.a,
                    )
                } else {
                    pixel_color
                };

                framebuffer.set_current_color(final_color);
                framebuffer.point(col as u32, y);
            }
        }
    }
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

fn render_background(
    framebuffer: &mut Framebuffer,
    camera_position: Vector2,
    camera_angle: f32,
    field_of_view: f32,
) {
    let horizon = framebuffer.height / 2;
    let half_height = horizon as f32;
    let projection_distance = (framebuffer.width as f32 / 2.0) / (field_of_view / 2.0).tan();

    let mut ray_cos = Vec::with_capacity(framebuffer.width as usize);
    let mut ray_sin = Vec::with_capacity(framebuffer.width as usize);
    for column in 0..framebuffer.width {
        let ray_progress = (column as f32 + 0.5) / framebuffer.width as f32;
        let ray_angle = camera_angle - field_of_view / 2.0 + field_of_view * ray_progress;
        ray_cos.push(ray_angle.cos());
        ray_sin.push(ray_angle.sin());
    }

    // 1. Techo arquitectónico de galería con vigas y artesonado
    for y in 0..horizon {
        let row_dy = (half_height - y as f32).max(1.0);
        let row_distance = projection_distance / row_dy;
        let fog = (1.0 / (1.0 + row_distance * 0.08)).clamp(0.20, 1.0);

        for x in 0..framebuffer.width {
            let world_x = camera_position.x + row_distance * ray_cos[x as usize];
            let world_y = camera_position.y + row_distance * ray_sin[x as usize];

            let u = (world_x - world_x.floor()).abs();
            let v = (world_y - world_y.floor()).abs();

            let is_beam = u < 0.08 || u > 0.92 || v < 0.08 || v > 0.92;
            let base_color = if is_beam {
                Color::new(28, 34, 46, 255)
            } else {
                CEILING_COLOR
            };

            let pixel = shade(base_color, fog);
            framebuffer.set_current_color(pixel);
            framebuffer.point(x, y);
        }
    }

    // 2. Piso temático de museo: Parquet de madera de roble cálido con juntas y vetas
    for y in horizon..framebuffer.height {
        let row_dy = (y as f32 - half_height).max(1.0);
        let row_distance = projection_distance / row_dy;
        let fog = (1.0 / (1.0 + row_distance * 0.07)).clamp(0.20, 1.0);

        for x in 0..framebuffer.width {
            let world_x = camera_position.x + row_distance * ray_cos[x as usize];
            let world_y = camera_position.y + row_distance * ray_sin[x as usize];

            let u = (world_x - world_x.floor()).abs();
            let v = (world_y - world_y.floor()).abs();

            let plank_idx = (v * 4.0).floor() as u32;
            let plank_offset = if plank_idx.is_multiple_of(2) {
                0.5
            } else {
                0.0
            };
            let plank_u = ((u + plank_offset) * 2.0).fract();
            let plank_v = (v * 4.0).fract();

            let is_joint = plank_v < 0.06 || plank_u < 0.04;
            let wood_color = if is_joint {
                Color::new(42, 26, 16, 255)
            } else {
                let grain = ((plank_u * 14.0).sin() * 0.5 + 0.5) * 0.12;
                match plank_idx % 3 {
                    0 => Color::new(
                        (88.0 * (1.0 + grain)).min(255.0) as u8,
                        (56.0 * (1.0 + grain)).min(255.0) as u8,
                        (36.0 * (1.0 + grain)).min(255.0) as u8,
                        255,
                    ),
                    1 => Color::new(
                        (78.0 * (1.0 + grain)).min(255.0) as u8,
                        (48.0 * (1.0 + grain)).min(255.0) as u8,
                        (30.0 * (1.0 + grain)).min(255.0) as u8,
                        255,
                    ),
                    _ => Color::new(
                        (98.0 * (1.0 + grain)).min(255.0) as u8,
                        (64.0 * (1.0 + grain)).min(255.0) as u8,
                        (42.0 * (1.0 + grain)).min(255.0) as u8,
                        255,
                    ),
                }
            };

            let pixel = shade(wood_color, fog);
            framebuffer.set_current_color(pixel);
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
            guards: vec![Guard::new(Vector2::new(2.5, 2.5))],
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

    #[test]
    fn render_guards_3d_runs_safely_with_all_states() {
        let mut framebuffer = Framebuffer::new(80, 60, Color::BLACK);
        let mut level = test_level();
        let mut guard = Guard::new(Vector2::new(2.5, 3.5));
        guard.splattered = true;
        guard.state = crate::level::GuardState::Slowed;
        level.guards = vec![guard];

        let asset_manager = AssetManager::new();
        let z_buf = vec![10.0; 80];

        render_guards_3d(
            &mut framebuffer,
            &level,
            Vector2::new(2.5, 2.0),
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::FRAC_PI_3,
            &z_buf,
            &asset_manager,
        );
    }
}
