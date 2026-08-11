use std::f32::consts::PI;

use raylib::prelude::*;

use crate::framebuffer::Framebuffer;

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

    pub fn draw(&self, framebuffer: &mut Framebuffer) {
        framebuffer.set_current_color(Color::RED);
        framebuffer.point(self.pos.x as u32, self.pos.y as u32);
    }
}
