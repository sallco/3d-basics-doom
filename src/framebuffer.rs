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
        self.color_buffer.clear_background(self.background_color);
    }

    pub fn point(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer
                .draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn draw_centered_text(&mut self, text: &str, y: i32, font_size: i32, color: Color) {
        // La fuente predeterminada de Raylib tiene glifos de aproximadamente media em.
        // Esta estimación evita acoplar el framebuffer lógico al handle de la ventana.
        let width = text.chars().count() as i32 * font_size / 2;
        let x = (self.width as i32 - width) / 2;
        self.color_buffer.draw_text(text, x, y, font_size, color);
    }

    pub fn present(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        texture: &mut Texture2D,
    ) -> Result<(), String> {
        let colors = self.color_buffer.get_image_data();
        // `raylib::Color` es #[repr(C)] y contiene exactamente cuatro canales u8.
        // Reinterpretar la vista evita una segunda copia al actualizar la textura GPU.
        let pixels = unsafe {
            std::slice::from_raw_parts(
                colors.as_ptr().cast::<u8>(),
                std::mem::size_of_val(colors.as_ref()),
            )
        };
        texture
            .update_texture(pixels)
            .map_err(|error| format!("no se pudo actualizar el framebuffer: {error}"))?;

        let window_width = window.get_screen_width() as f32;
        let window_height = window.get_screen_height() as f32;
        let scale = (window_width / self.width as f32).min(window_height / self.height as f32);
        let destination_width = self.width as f32 * scale;
        let destination_height = self.height as f32 * scale;
        let destination = Rectangle::new(
            (window_width - destination_width) / 2.0,
            (window_height - destination_height) / 2.0,
            destination_width,
            destination_height,
        );

        let mut renderer = window.begin_drawing(raylib_thread);
        renderer.clear_background(Color::BLACK);
        renderer.draw_texture_pro(
            texture,
            Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
            destination,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );

        Ok(())
    }
}
