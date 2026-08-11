use std::collections::HashMap;

use raylib::prelude::*;

pub struct TextureManager {
    images: HashMap<char, Image>,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
        }
    }

    pub fn load(&mut self, character: char, path: &str) -> Result<(), String> {
        let image = Image::load_image(path)
            .map_err(|error| format!("Failed to load image '{path}': {error}"))?;

        self.images.insert(character, image);
        Ok(())
    }

    pub fn load_defaults() -> Result<Self, String> {
        let mut manager = Self::new();
        let texture_files = [
            ('+', "src/assets/wall4.png"),
            ('-', "src/assets/wall2.png"),
            ('|', "src/assets/wall1.png"),
            ('g', "src/assets/wall5.png"),
            ('#', "src/assets/wall3.png"),
        ];

        for (character, path) in texture_files {
            manager.load(character, path)?;
        }

        Ok(manager)
    }

    fn get_image(&self, character: char) -> Option<&Image> {
        self.images
            .get(&character)
            .or_else(|| self.images.get(&'#'))
    }

    pub fn dimensions(&self, character: char) -> Option<(u32, u32)> {
        let image = self.get_image(character)?;

        if image.width <= 0 || image.height <= 0 {
            return None;
        }

        Some((image.width as u32, image.height as u32))
    }

    pub fn get_pixel_color(&self, character: char, x: u32, y: u32) -> Option<Color> {
        let image = self.get_image(character)?;

        if image.width <= 0 || image.height <= 0 {
            return None;
        }

        let texture_x = x.min(image.width as u32 - 1) as i32;
        let texture_y = y.min(image.height as u32 - 1) as i32;

        Some(image.get_color(texture_x, texture_y))
    }
}
