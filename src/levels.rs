use crate::level::LevelDefinition;

const GALLERY_PAINTINGS: [&str; 3] = [
    "src/assets/museum/walls/with_artworks/one/1.jpg",
    "src/assets/museum/walls/with_artworks/two/3.jpg",
    "src/assets/museum/walls/with_artworks/three/4.jpg",
];

const MODERN_WING_PAINTINGS: [&str; 5] = [
    "src/assets/museum/walls/with_artworks/one/5.jpg",
    "src/assets/museum/walls/with_artworks/one/8.jpg",
    "src/assets/museum/walls/with_artworks/two/6.jpg",
    "src/assets/museum/walls/with_artworks/two/9.jpg",
    "src/assets/museum/walls/with_artworks/three/7.jpg",
];

const NIGHT_ARCHIVE_PAINTINGS: [&str; 7] = [
    "src/assets/museum/walls/with_artworks/one/11.jpg",
    "src/assets/museum/walls/with_artworks/one/14.jpg",
    "src/assets/museum/walls/with_artworks/one/15.jpg",
    "src/assets/museum/walls/with_artworks/two/12.jpg",
    "src/assets/museum/walls/with_artworks/two/20.jpg",
    "src/assets/museum/walls/with_artworks/three/10.jpg",
    "src/assets/museum/walls/with_artworks/three/13.jpg",
];

pub static LEVEL_DEFINITIONS: [LevelDefinition; 3] = [
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

    #[test]
    fn all_levels_have_reachable_exit() {
        use crate::ai::bfs_path;

        for definition in &LEVEL_DEFINITIONS {
            let level = load_level(definition.map_path).unwrap();
            let start = (
                level.player_spawn.y.floor() as usize,
                level.player_spawn.x.floor() as usize,
            );
            let goal = (level.exit.y.floor() as usize, level.exit.x.floor() as usize);

            // Temporarily treat exit cell as walkable for reachability test
            let mut maze_with_exit_walkable = level.maze.clone();
            maze_with_exit_walkable[goal.0][goal.1] = crate::level::Tile::Floor;

            let path = bfs_path(&maze_with_exit_walkable, start, goal);
            assert!(
                path.is_some(),
                "Exit must be reachable from player spawn in {}",
                definition.name
            );
        }
    }
}
