use raylib::prelude::Vector2;

use crate::level::{LevelMap, Tile, WallMaterial};

#[allow(dead_code)] // El renderer semántico lo utilizará en el siguiente bloque.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallSide {
    Vertical,
    Horizontal,
}

#[allow(dead_code)] // El renderer semántico lo utilizará en el siguiente bloque.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RayHit {
    pub distance: f32,
    pub tile: Tile,
    pub map_position: (usize, usize),
    pub texture_u: f32,
    pub side: WallSide,
}

#[allow(dead_code)] // Sustituirá al raycaster incremental al migrar el renderer.
pub fn cast_ray_dda(map: &LevelMap, origin: Vector2, angle: f32) -> Option<RayHit> {
    let height = map.len();
    let width = map.first()?.len();

    if width == 0
        || !origin.x.is_finite()
        || !origin.y.is_finite()
        || !angle.is_finite()
        || origin.x < 0.0
        || origin.y < 0.0
        || origin.x >= width as f32
        || origin.y >= height as f32
    {
        return None;
    }

    let direction_x = angle.cos();
    let direction_y = angle.sin();
    let mut map_x = origin.x.floor() as i32;
    let mut map_y = origin.y.floor() as i32;

    let delta_x = if direction_x.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        (1.0 / direction_x).abs()
    };
    let delta_y = if direction_y.abs() < f32::EPSILON {
        f32::INFINITY
    } else {
        (1.0 / direction_y).abs()
    };

    let (step_x, mut side_distance_x) = if direction_x < 0.0 {
        (-1, (origin.x - map_x as f32) * delta_x)
    } else {
        (1, (map_x as f32 + 1.0 - origin.x) * delta_x)
    };
    let (step_y, mut side_distance_y) = if direction_y < 0.0 {
        (-1, (origin.y - map_y as f32) * delta_y)
    } else {
        (1, (map_y as f32 + 1.0 - origin.y) * delta_y)
    };

    let max_steps = width.saturating_add(height).saturating_add(1);

    for _ in 0..max_steps {
        let (side, distance) = if side_distance_x < side_distance_y {
            map_x += step_x;
            let distance = side_distance_x;
            side_distance_x += delta_x;
            (WallSide::Vertical, distance)
        } else {
            map_y += step_y;
            let distance = side_distance_y;
            side_distance_y += delta_y;
            (WallSide::Horizontal, distance)
        };

        let Some(&tile) = map
            .get(map_y as usize)
            .and_then(|row| row.get(map_x as usize))
        else {
            return Some(RayHit {
                distance,
                tile: Tile::Wall(WallMaterial::Service),
                map_position: (
                    map_y.clamp(0, height as i32 - 1) as usize,
                    map_x.clamp(0, width as i32 - 1) as usize,
                ),
                texture_u: texture_coordinate(origin, direction_x, direction_y, distance, side),
                side,
            });
        };

        if tile.is_solid() {
            return Some(RayHit {
                distance,
                tile,
                map_position: (map_y as usize, map_x as usize),
                texture_u: texture_coordinate(origin, direction_x, direction_y, distance, side),
                side,
            });
        }
    }

    None
}

fn texture_coordinate(
    origin: Vector2,
    direction_x: f32,
    direction_y: f32,
    distance: f32,
    side: WallSide,
) -> f32 {
    let wall_position = match side {
        WallSide::Vertical => origin.y + distance * direction_y,
        WallSide::Horizontal => origin.x + distance * direction_x,
    };
    let mut texture_u = wall_position.rem_euclid(1.0);

    if matches!(side, WallSide::Vertical) && direction_x > 0.0
        || matches!(side, WallSide::Horizontal) && direction_y < 0.0
    {
        texture_u = 1.0 - texture_u;
    }

    texture_u.rem_euclid(1.0)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;

    fn enclosed_map() -> LevelMap {
        let gallery_wall = Tile::Wall(WallMaterial::Gallery);
        vec![
            vec![gallery_wall; 5],
            vec![
                gallery_wall,
                Tile::Floor,
                Tile::Floor,
                Tile::Floor,
                gallery_wall,
            ],
            vec![
                gallery_wall,
                Tile::Floor,
                Tile::Floor,
                Tile::Floor,
                Tile::Wall(WallMaterial::Accent),
            ],
            vec![
                gallery_wall,
                Tile::Floor,
                Tile::Floor,
                Tile::Floor,
                gallery_wall,
            ],
            vec![Tile::Wall(WallMaterial::Burgundy); 5],
        ]
    }

    #[test]
    fn hits_vertical_wall_with_distance_and_uv() {
        let hit = cast_ray_dda(&enclosed_map(), Vector2::new(2.5, 2.5), 0.0).unwrap();

        assert_eq!(hit.map_position, (2, 4));
        assert_eq!(hit.tile, Tile::Wall(WallMaterial::Accent));
        assert_eq!(hit.side, WallSide::Vertical);
        assert!((hit.distance - 1.5).abs() < 0.0001);
        assert!((hit.texture_u - 0.5).abs() < 0.0001);
    }

    #[test]
    fn hits_horizontal_wall_with_distance_and_uv() {
        let hit = cast_ray_dda(&enclosed_map(), Vector2::new(2.5, 2.5), FRAC_PI_2).unwrap();

        assert_eq!(hit.map_position, (4, 2));
        assert_eq!(hit.tile, Tile::Wall(WallMaterial::Burgundy));
        assert_eq!(hit.side, WallSide::Horizontal);
        assert!((hit.distance - 1.5).abs() < 0.0001);
        assert!((hit.texture_u - 0.5).abs() < 0.0001);
    }

    #[test]
    fn reports_exact_target_painting_cell() {
        let mut map = enclosed_map();
        map[2][3] = Tile::TargetPainting;

        let hit = cast_ray_dda(&map, Vector2::new(1.5, 2.5), 0.0).unwrap();

        assert_eq!(hit.map_position, (2, 3));
        assert_eq!(hit.tile, Tile::TargetPainting);
        assert!((hit.distance - 1.5).abs() < 0.0001);
    }

    #[test]
    fn treats_out_of_bounds_as_solid() {
        let map = vec![vec![Tile::Floor; 2]; 2];

        let hit = cast_ray_dda(&map, Vector2::new(0.5, 0.5), PI).unwrap();

        assert_eq!(hit.map_position, (0, 0));
        assert_eq!(hit.tile, Tile::Wall(WallMaterial::Service));
        assert_eq!(hit.side, WallSide::Vertical);
        assert!((hit.distance - 0.5).abs() < 0.0001);
    }

    #[test]
    fn rejects_origin_outside_map() {
        assert!(cast_ray_dda(&enclosed_map(), Vector2::new(-0.5, 1.5), 0.0).is_none());
    }
}
