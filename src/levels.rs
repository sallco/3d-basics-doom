use crate::level::LevelDefinition;

const GALLERY_PAINTINGS: [&str; 3] = [
    "src/assets/museum/walls/with_artworks/one/1.jpg",
    "src/assets/museum/walls/with_artworks/one/5.jpg",
    "src/assets/museum/walls/with_artworks/one/8.jpg",
];

const MODERN_WING_PAINTINGS: [&str; 5] = [
    "src/assets/museum/walls/with_artworks/one/11.jpg",
    "src/assets/museum/walls/with_artworks/one/14.jpg",
    "src/assets/museum/walls/with_artworks/one/15.jpg",
    "src/assets/museum/walls/with_artworks/one/16.jpg",
    "src/assets/museum/walls/with_artworks/one/17.jpg",
];

const NIGHT_ARCHIVE_PAINTINGS: [&str; 7] = [
    "src/assets/museum/walls/with_artworks/one/18.jpg",
    "src/assets/museum/walls/with_artworks/one/19.jpg",
    "src/assets/museum/walls/with_artworks/two/3.jpg",
    "src/assets/museum/walls/with_artworks/two/6.jpg",
    "src/assets/museum/walls/with_artworks/two/9.jpg",
    "src/assets/museum/walls/with_artworks/two/12.jpg",
    "src/assets/museum/walls/with_artworks/two/20.jpg",
];

#[allow(dead_code)] // El selector de niveles lo utilizará al migrar Game.
pub const LEVEL_DEFINITIONS: [LevelDefinition; 3] = [
    LevelDefinition {
        name: "Galería de ingreso",
        map_path: "src/assets/maps/gallery_entrance.txt",
        painting_assets: &GALLERY_PAINTINGS,
    },
    LevelDefinition {
        name: "Ala moderna",
        map_path: "src/assets/maps/modern_wing.txt",
        painting_assets: &MODERN_WING_PAINTINGS,
    },
    LevelDefinition {
        name: "Archivo nocturno",
        map_path: "src/assets/maps/night_archive.txt",
        painting_assets: &NIGHT_ARCHIVE_PAINTINGS,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::load_level;

    #[test]
    fn catalog_levels_match_planned_dimensions_and_entities() {
        let expectations = [(16, 12, 2, 3), (24, 16, 4, 5), (32, 20, 6, 7)];

        for (definition, (width, height, guards, paintings)) in
            LEVEL_DEFINITIONS.iter().zip(expectations)
        {
            let level = load_level(definition.map_path).unwrap();

            assert_eq!(level.maze.len(), height, "{}", definition.name);
            assert!(
                level.maze.iter().all(|row| row.len() == width),
                "{}",
                definition.name
            );
            assert_eq!(level.guards.len(), guards, "{}", definition.name);
            assert_eq!(level.paintings.len(), paintings, "{}", definition.name);
            assert_eq!(definition.painting_assets.len(), paintings);
        }
    }
}
