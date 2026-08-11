use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, background_color: Color) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, background_color);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(
            self.width as i32,
            self.height as i32,
            self.background_color,
        );
    }

    pub fn point(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer
                .draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn get_color(&self, x: u32, y: u32) -> Color {
        if x < self.width && y < self.height {
            self.color_buffer.get_color(x as i32, y as i32)
        } else {
            self.background_color
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }

    pub fn swap_buffers(&self, window: &mut RaylibHandle, raylib_thread: &RaylibThread) {
        self.swap_buffers_scaled(window, raylib_thread, 1.0);
    }

    pub fn swap_buffers_scaled(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        scale: f32,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            texture.set_texture_filter(raylib_thread, TextureFilter::TEXTURE_FILTER_POINT);

            let mut renderer = window.begin_drawing(raylib_thread);

            renderer.draw_texture_ex(
                &texture,
                Vector2::new(0.0, 0.0),
                0.0,
                scale,
                Color::WHITE,
            );
        }
    }
}
