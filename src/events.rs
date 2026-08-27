use raylib::prelude::{KeyboardKey, RaylibHandle, Vector2};

use crate::level::LevelMap;
use crate::movement::{PLAYER_RADIUS, move_with_collisions};
use crate::player::Player;

const MOVE_SPEED: f32 = 2.5;
const ROTATION_SPEED: f32 = 1.8;
const MAX_DELTA_TIME: f32 = 0.05;

pub fn process_events(
    window: &RaylibHandle,
    player: &mut Player,
    map: &LevelMap,
    delta_time: f32,
    exit_unlocked: bool,
) {
    let delta_time = sanitize_delta_time(delta_time);
    let rotation_axis = axis(
        window.is_key_down(KeyboardKey::KEY_RIGHT),
        window.is_key_down(KeyboardKey::KEY_LEFT),
    );
    player.a =
        (player.a + rotation_axis * ROTATION_SPEED * delta_time).rem_euclid(std::f32::consts::TAU);

    let forward_axis = axis(
        window.is_key_down(KeyboardKey::KEY_W) || window.is_key_down(KeyboardKey::KEY_UP),
        window.is_key_down(KeyboardKey::KEY_S) || window.is_key_down(KeyboardKey::KEY_DOWN),
    );
    let strafe_axis = axis(
        window.is_key_down(KeyboardKey::KEY_D),
        window.is_key_down(KeyboardKey::KEY_A),
    );
    let direction = movement_direction(player.a, forward_axis, strafe_axis);
    let displacement = Vector2::new(
        direction.x * MOVE_SPEED * delta_time,
        direction.y * MOVE_SPEED * delta_time,
    );

    move_with_collisions(
        map,
        &mut player.pos,
        displacement,
        PLAYER_RADIUS,
        exit_unlocked,
    );
}

fn axis(positive: bool, negative: bool) -> f32 {
    positive as u8 as f32 - negative as u8 as f32
}

fn sanitize_delta_time(delta_time: f32) -> f32 {
    if delta_time.is_finite() {
        delta_time.clamp(0.0, MAX_DELTA_TIME)
    } else {
        0.0
    }
}

fn movement_direction(angle: f32, forward_axis: f32, strafe_axis: f32) -> Vector2 {
    let mut direction = Vector2::new(
        angle.cos() * forward_axis - angle.sin() * strafe_axis,
        angle.sin() * forward_axis + angle.cos() * strafe_axis,
    );
    let length = direction.x.hypot(direction.y);

    if length > 1.0 {
        direction.x /= length;
        direction.y /= length;
    }

    direction
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_and_strafe_are_relative_to_camera() {
        let forward = movement_direction(0.0, 1.0, 0.0);
        let right = movement_direction(0.0, 0.0, 1.0);

        assert!((forward.x - 1.0).abs() < f32::EPSILON);
        assert!(forward.y.abs() < f32::EPSILON);
        assert!(right.x.abs() < f32::EPSILON);
        assert!((right.y - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn diagonal_input_does_not_increase_speed() {
        let direction = movement_direction(0.0, 1.0, 1.0);

        assert!((direction.x.hypot(direction.y) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn delta_time_is_clamped_after_pause() {
        assert_eq!(sanitize_delta_time(2.0), MAX_DELTA_TIME);
        assert_eq!(sanitize_delta_time(-1.0), 0.0);
        assert_eq!(sanitize_delta_time(f32::NAN), 0.0);
    }
}
