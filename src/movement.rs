use raylib::prelude::Vector2;

use crate::level::LevelMap;

#[allow(dead_code)] // Game lo utilizará durante la migración al modelo Level.
pub const PLAYER_RADIUS: f32 = 0.2;
const MAX_SUBSTEPS: usize = 1024;

#[allow(dead_code)] // Game lo utilizará durante la migración al modelo Level.
pub fn move_with_collisions(
    map: &LevelMap,
    position: &mut Vector2,
    displacement: Vector2,
    radius: f32,
    exit_unlocked: bool,
) {
    if !position.x.is_finite()
        || !position.y.is_finite()
        || !displacement.x.is_finite()
        || !displacement.y.is_finite()
        || !radius.is_finite()
        || radius <= 0.0
    {
        return;
    }

    let distance = displacement.x.hypot(displacement.y);
    if distance <= f32::EPSILON {
        return;
    }

    let max_step_length = (radius * 0.5).max(0.01);
    let substeps = ((distance / max_step_length).ceil() as usize).clamp(1, MAX_SUBSTEPS);
    let step = Vector2::new(
        displacement.x / substeps as f32,
        displacement.y / substeps as f32,
    );

    for _ in 0..substeps {
        let candidate_x = Vector2::new(position.x + step.x, position.y);
        if circle_is_walkable(map, candidate_x, radius, exit_unlocked) {
            position.x = candidate_x.x;
        }

        let candidate_y = Vector2::new(position.x, position.y + step.y);
        if circle_is_walkable(map, candidate_y, radius, exit_unlocked) {
            position.y = candidate_y.y;
        }
    }
}

fn circle_is_walkable(map: &LevelMap, position: Vector2, radius: f32, exit_unlocked: bool) -> bool {
    let min_column = (position.x - radius).floor() as i32;
    let max_column = (position.x + radius).floor() as i32;
    let min_row = (position.y - radius).floor() as i32;
    let max_row = (position.y + radius).floor() as i32;

    for row in min_row..=max_row {
        for column in min_column..=max_column {
            let Some(&tile) = map
                .get(row as usize)
                .and_then(|map_row| map_row.get(column as usize))
            else {
                return false;
            };

            if !tile.blocks_movement(exit_unlocked) {
                continue;
            }

            let closest_x = position.x.clamp(column as f32, column as f32 + 1.0);
            let closest_y = position.y.clamp(row as f32, row as f32 + 1.0);
            let distance_x = position.x - closest_x;
            let distance_y = position.y - closest_y;

            if distance_x * distance_x + distance_y * distance_y < radius * radius {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::level::{Tile, WallMaterial};

    use super::*;

    fn map_with_center_wall() -> LevelMap {
        let wall = Tile::Wall(WallMaterial::Gallery);
        vec![
            vec![wall; 5],
            vec![wall, Tile::Floor, wall, Tile::Floor, wall],
            vec![wall, Tile::Floor, wall, Tile::Floor, wall],
            vec![wall, Tile::Floor, wall, Tile::Floor, wall],
            vec![wall; 5],
        ]
    }

    #[test]
    fn large_displacement_cannot_cross_wall() {
        let mut position = Vector2::new(1.5, 2.5);

        move_with_collisions(
            &map_with_center_wall(),
            &mut position,
            Vector2::new(3.0, 0.0),
            PLAYER_RADIUS,
            false,
        );

        assert!(position.x <= 1.8 + f32::EPSILON);
        assert!((position.y - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn separated_axes_allow_sliding_along_wall() {
        let mut position = Vector2::new(1.5, 1.5);

        move_with_collisions(
            &map_with_center_wall(),
            &mut position,
            Vector2::new(2.0, 2.0),
            PLAYER_RADIUS,
            false,
        );

        assert!(position.x <= 1.8 + f32::EPSILON);
        assert!(position.y > 3.0);
    }

    #[test]
    fn map_boundaries_are_solid() {
        let wall = Tile::Wall(WallMaterial::Gallery);
        let map = vec![vec![wall; 3], vec![wall, Tile::Floor, wall], vec![wall; 3]];
        let mut position = Vector2::new(1.5, 1.5);

        move_with_collisions(
            &map,
            &mut position,
            Vector2::new(-5.0, 0.0),
            PLAYER_RADIUS,
            false,
        );

        assert!(position.x >= 1.2 - f32::EPSILON);
    }

    #[test]
    fn unlocked_exit_becomes_walkable() {
        let wall = Tile::Wall(WallMaterial::Gallery);
        let map = vec![
            vec![wall; 5],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, Tile::Exit],
            vec![wall, Tile::Floor, Tile::Floor, Tile::Floor, wall],
            vec![wall; 5],
        ];
        let mut locked_position = Vector2::new(3.5, 2.5);
        let mut unlocked_position = locked_position;

        move_with_collisions(
            &map,
            &mut locked_position,
            Vector2::new(1.0, 0.0),
            PLAYER_RADIUS,
            false,
        );
        move_with_collisions(
            &map,
            &mut unlocked_position,
            Vector2::new(1.0, 0.0),
            PLAYER_RADIUS,
            true,
        );

        assert!(locked_position.x <= 3.8 + f32::EPSILON);
        assert!(unlocked_position.x > 4.4);
    }
}
