use raylib::prelude::{KeyboardKey, RaylibHandle, Vector2};

use crate::level::LevelMap;
use crate::movement::{PLAYER_RADIUS, move_with_collisions};
use crate::player::Player;

pub const MOVE_SPEED: f32 = 2.5;
pub const ROTATION_SPEED: f32 = 1.8;
pub const MOUSE_SENSITIVITY: f32 = 0.003;
pub const MAX_DELTA_TIME: f32 = 0.05;

pub fn process_events(
    window: &RaylibHandle,
    player: &mut Player,
    map: &LevelMap,
    delta_time: f32,
    exit_unlocked: bool,
) {
    let delta_time = sanitize_delta_time(delta_time);
    let keyboard_axis = axis(
        window.is_key_down(KeyboardKey::KEY_RIGHT),
        window.is_key_down(KeyboardKey::KEY_LEFT),
    );
    let mouse_delta_x = window.get_mouse_delta().x;
    let rotation_delta = compute_rotation_delta(
        keyboard_axis,
        ROTATION_SPEED,
        delta_time,
        mouse_delta_x,
        MOUSE_SENSITIVITY,
    );
    player.a = apply_rotation(player.a, rotation_delta);

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

pub fn compute_rotation_delta(
    keyboard_axis: f32,
    rotation_speed: f32,
    delta_time: f32,
    mouse_delta_x: f32,
    mouse_sensitivity: f32,
) -> f32 {
    keyboard_axis * rotation_speed * delta_time + mouse_delta_x * mouse_sensitivity
}

pub fn apply_rotation(current_angle: f32, rotation_delta: f32) -> f32 {
    (current_angle + rotation_delta).rem_euclid(std::f32::consts::TAU)
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
    use std::f32::consts::{FRAC_PI_2, PI, TAU};

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

    #[test]
    fn rotation_combines_keyboard_and_mouse_delta() {
        let keyboard_only =
            compute_rotation_delta(1.0, ROTATION_SPEED, 0.016, 0.0, MOUSE_SENSITIVITY);
        assert!((keyboard_only - 1.8 * 0.016).abs() < 1e-6);

        let mouse_only =
            compute_rotation_delta(0.0, ROTATION_SPEED, 0.016, 10.0, MOUSE_SENSITIVITY);
        assert!((mouse_only - 10.0 * 0.003).abs() < 1e-6);

        let combined = compute_rotation_delta(1.0, ROTATION_SPEED, 0.016, 10.0, MOUSE_SENSITIVITY);
        assert!((combined - (keyboard_only + mouse_only)).abs() < 1e-6);
    }

    #[test]
    fn apply_rotation_wraps_around_tau_circle() {
        let wrapped_positive = apply_rotation(TAU - 0.1, 0.2);
        assert!((wrapped_positive - 0.1).abs() < 1e-6);

        let wrapped_negative = apply_rotation(0.1, -0.2);
        assert!((wrapped_negative - (TAU - 0.1)).abs() < 1e-6);

        let quarter_turn = apply_rotation(0.0, FRAC_PI_2);
        assert!((quarter_turn - FRAC_PI_2).abs() < 1e-6);

        let half_turn = apply_rotation(PI, PI);
        assert!(half_turn.abs() < 1e-6 || (half_turn - TAU).abs() < 1e-6);
    }
}
