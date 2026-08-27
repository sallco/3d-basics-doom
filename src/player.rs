use std::f32::consts::PI;

use raylib::prelude::*;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32,
}

impl Player {
    pub fn new(pos: Vector2) -> Self {
        Self {
            pos,
            a: PI / 3.0,
            fov: PI / 3.0,
        }
    }
}
