use std::collections::HashMap;
use std::path::Path;

use raylib::prelude::{Color, Image};

pub const WALL_GALLERY_PATH: &str = "src/assets/museum/walls/empty/single_spotlight.jpg";
pub const WALL_BURGUNDY_PATH: &str = "src/assets/museum/walls/empty/triple_spotlight.jpg";
pub const WALL_SERVICE_PATH: &str = "src/assets/museum/walls/empty/single_spotlight.jpg";
pub const WALL_ACCENT_PATH: &str = "src/assets/museum/walls/empty/triple_spotlight.jpg";
pub const WALL_DECORATIVE_PATH: &str = "src/assets/museum/walls/with_artworks/three/2.jpg";

pub const DECORATIVE_ARTWORK_PATHS: [&str; 6] = [
    "src/assets/museum/walls/with_artworks/one/16.jpg",
    "src/assets/museum/walls/with_artworks/one/17.jpg",
    "src/assets/museum/walls/with_artworks/one/18.jpg",
    "src/assets/museum/walls/with_artworks/one/19.jpg",
    "src/assets/museum/walls/with_artworks/two/21.jpg",
    "src/assets/museum/walls/with_artworks/three/2.jpg",
];

pub fn decorative_path_for_tile(row: usize, column: usize) -> &'static str {
    let index = (row * 7 + column * 13) % DECORATIVE_ARTWORK_PATHS.len();
    DECORATIVE_ARTWORK_PATHS[index]
}

#[derive(Clone, Debug, PartialEq)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Color>,
}

impl Texture {
    pub fn solid(width: u32, height: u32, color: Color) -> Self {
        let w = width.max(1);
        let h = height.max(1);
        let size = (w * h) as usize;
        Self {
            width: w,
            height: h,
            pixels: vec![color; size],
        }
    }

    pub fn from_image(image: &Image) -> Self {
        let width = image.width().max(1) as u32;
        let height = image.height().max(1) as u32;
        let colors = image.get_image_data();
        let pixels = colors.to_vec();

        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path_ref = path.as_ref();
        let path_str = path_ref
            .to_str()
            .ok_or_else(|| "ruta de archivo no válida como texto UTF-8".to_string())?;
        let image = Image::load_image(path_str)
            .map_err(|error| format!("no se pudo cargar la imagen '{path_str}': {error}"))?;
        Ok(Self::from_image(&image))
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        if self.width == 0 || self.height == 0 || self.pixels.is_empty() {
            return Color::WHITE;
        }

        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = v.clamp(0.0, 1.0);

        let x = ((u_clamped * (self.width as f32 - 1.0)).round() as u32).min(self.width - 1);
        let y = ((v_clamped * (self.height as f32 - 1.0)).round() as u32).min(self.height - 1);

        let index = (y * self.width + x) as usize;
        self.pixels.get(index).copied().unwrap_or(Color::WHITE)
    }
}

#[derive(Debug, Default)]
pub struct AssetManager {
    textures: HashMap<String, Texture>,
}

impl AssetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            textures: HashMap::new(),
        };
        manager.preload_standard_textures();
        manager
    }

    pub fn preload_standard_textures(&mut self) {
        let standard_paths = [
            WALL_GALLERY_PATH,
            WALL_BURGUNDY_PATH,
            WALL_SERVICE_PATH,
            WALL_ACCENT_PATH,
            WALL_DECORATIVE_PATH,
        ];

        for path in standard_paths {
            self.load_texture(path);
        }

        for path in DECORATIVE_ARTWORK_PATHS {
            self.load_texture(path);
        }
    }

    pub fn preload_level_assets(&mut self, painting_assets: &[&str]) {
        for path in painting_assets {
            self.load_texture(path);
        }
    }

    pub fn load_texture(&mut self, path: &str) -> &Texture {
        if !self.textures.contains_key(path) {
            match Texture::load(path) {
                Ok(texture) => {
                    self.textures.insert(path.to_string(), texture);
                }
                Err(error) => {
                    eprintln!("Advertencia de asset: {error}; usando textura de respaldo.");
                    let fallback = Texture::solid(64, 64, Color::new(120, 120, 120, 255));
                    self.textures.insert(path.to_string(), fallback);
                }
            }
        }

        &self.textures[path]
    }

    pub fn get_texture(&self, path: &str) -> Option<&Texture> {
        self.textures.get(path)
    }

    #[allow(dead_code)] // Expuesto para pruebas de carga y fallbacks.
    pub fn loaded_texture_count(&self) -> usize {
        self.textures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_texture_samples_uniform_color() {
        let texture = Texture::solid(4, 4, Color::RED);
        assert_eq!(texture.sample(0.0, 0.0), Color::RED);
        assert_eq!(texture.sample(0.5, 0.5), Color::RED);
        assert_eq!(texture.sample(1.0, 1.0), Color::RED);
        assert_eq!(texture.sample(-0.5, 1.5), Color::RED);
    }

    #[test]
    fn texture_sample_interpolates_coordinates() {
        let pixels = vec![Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW];
        let texture = Texture {
            width: 2,
            height: 2,
            pixels,
        };

        assert_eq!(texture.sample(0.0, 0.0), Color::RED);
        assert_eq!(texture.sample(1.0, 0.0), Color::GREEN);
        assert_eq!(texture.sample(0.0, 1.0), Color::BLUE);
        assert_eq!(texture.sample(1.0, 1.0), Color::YELLOW);
    }

    #[test]
    fn asset_manager_loads_and_caches_real_wall_asset() {
        let mut manager = AssetManager::new();
        assert!(manager.get_texture(WALL_GALLERY_PATH).is_some());

        let count_before = manager.loaded_texture_count();
        let _ = manager.load_texture(WALL_GALLERY_PATH);
        assert_eq!(manager.loaded_texture_count(), count_before);
    }

    #[test]
    fn asset_manager_handles_missing_file_with_fallback() {
        let mut manager = AssetManager::default();
        let fallback = manager.load_texture("non_existent_texture.jpg");

        assert_eq!(fallback.width, 64);
        assert_eq!(fallback.height, 64);
        assert_eq!(fallback.sample(0.5, 0.5), Color::new(120, 120, 120, 255));
    }
}
