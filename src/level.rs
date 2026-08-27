use raylib::prelude::Vector2;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

#[allow(dead_code)] // El renderer semántico se implementará en un paso posterior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallMaterial {
    Gallery,
    Burgundy,
    Service,
    Accent,
}

#[allow(dead_code)] // El renderer semántico se implementará en un paso posterior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tile {
    Floor,
    Exit,
    Wall(WallMaterial),
    TargetPainting,
    DecorativePainting,
}

impl Tile {
    pub fn is_solid(self) -> bool {
        !matches!(self, Self::Floor)
    }

    pub fn blocks_movement(self, exit_unlocked: bool) -> bool {
        match self {
            Self::Floor => false,
            Self::Exit => !exit_unlocked,
            _ => true,
        }
    }
}

pub type LevelMap = Vec<Vec<Tile>>;

#[allow(dead_code)] // El cargador consumirá los marcadores y conservará únicamente Tile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MapSymbol {
    Tile(Tile),
    PlayerSpawn,
    GuardSpawn,
}

#[allow(dead_code)] // Será el error de validación cuando el cargador use MapSymbol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidMapSymbol(pub char);

impl Display for InvalidMapSymbol {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "símbolo de mapa desconocido: {:?}", self.0)
    }
}

impl Error for InvalidMapSymbol {}

impl TryFrom<char> for MapSymbol {
    type Error = InvalidMapSymbol;

    fn try_from(symbol: char) -> Result<Self, Self::Error> {
        let parsed = match symbol {
            ' ' => Self::Tile(Tile::Floor),
            'p' => Self::PlayerSpawn,
            'g' => Self::Tile(Tile::Exit),
            'e' => Self::GuardSpawn,
            '1' => Self::Tile(Tile::Wall(WallMaterial::Gallery)),
            '2' => Self::Tile(Tile::Wall(WallMaterial::Burgundy)),
            '3' => Self::Tile(Tile::Wall(WallMaterial::Service)),
            '4' => Self::Tile(Tile::Wall(WallMaterial::Accent)),
            'T' => Self::Tile(Tile::TargetPainting),
            'd' => Self::Tile(Tile::DecorativePainting),
            unknown => return Err(InvalidMapSymbol(unknown)),
        };

        Ok(parsed)
    }
}

impl MapSymbol {
    fn closes_boundary(self) -> bool {
        matches!(
            self,
            Self::Tile(
                Tile::Exit | Tile::Wall(_) | Tile::TargetPainting | Tile::DecorativePainting
            )
        )
    }
}

pub type ParsedMap = Vec<Vec<MapSymbol>>;

pub fn parse_map(contents: &str) -> Result<ParsedMap, InvalidMapSymbol> {
    contents
        .lines()
        .map(|line| line.chars().map(MapSymbol::try_from).collect())
        .collect()
}

#[allow(dead_code)] // Se usará cuando Game cargue LevelDefinition.
#[derive(Debug)]
pub enum LevelError {
    Io(std::io::Error),
    InvalidSymbol(InvalidMapSymbol),
    EmptyMap,
    EmptyRow {
        row: usize,
    },
    NonRectangular {
        row: usize,
        expected: usize,
        actual: usize,
    },
    OpenBoundary {
        row: usize,
        column: usize,
    },
    InvalidPlayerCount {
        found: usize,
    },
    InvalidExitCount {
        found: usize,
    },
    MissingPaintingTarget,
}

impl Display for LevelError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "no se pudo leer el mapa: {error}"),
            Self::InvalidSymbol(error) => Display::fmt(error, formatter),
            Self::EmptyMap => write!(formatter, "el mapa está vacío"),
            Self::EmptyRow { row } => write!(formatter, "la fila {} está vacía", row + 1),
            Self::NonRectangular {
                row,
                expected,
                actual,
            } => write!(
                formatter,
                "la fila {} tiene {actual} celdas; se esperaban {expected}",
                row + 1
            ),
            Self::OpenBoundary { row, column } => write!(
                formatter,
                "el mapa está abierto en la fila {}, columna {}",
                row + 1,
                column + 1
            ),
            Self::InvalidPlayerCount { found } => write!(
                formatter,
                "el mapa debe contener exactamente un jugador; se encontraron {found}"
            ),
            Self::InvalidExitCount { found } => write!(
                formatter,
                "el mapa debe contener exactamente una salida; se encontraron {found}"
            ),
            Self::MissingPaintingTarget => {
                write!(
                    formatter,
                    "el mapa debe contener al menos una pintura objetivo"
                )
            }
        }
    }
}

impl Error for LevelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::InvalidSymbol(error) => Some(error),
            Self::EmptyMap
            | Self::EmptyRow { .. }
            | Self::NonRectangular { .. }
            | Self::OpenBoundary { .. }
            | Self::InvalidPlayerCount { .. }
            | Self::InvalidExitCount { .. }
            | Self::MissingPaintingTarget => None,
        }
    }
}

pub fn validate_map_shape(map: &ParsedMap) -> Result<(), LevelError> {
    let Some(first_row) = map.first() else {
        return Err(LevelError::EmptyMap);
    };
    let expected_width = first_row.len();

    for (row_index, row) in map.iter().enumerate() {
        if row.is_empty() {
            return Err(LevelError::EmptyRow { row: row_index });
        }

        if row.len() != expected_width {
            return Err(LevelError::NonRectangular {
                row: row_index,
                expected: expected_width,
                actual: row.len(),
            });
        }
    }

    Ok(())
}

pub fn validate_closed_boundaries(map: &ParsedMap) -> Result<(), LevelError> {
    validate_map_shape(map)?;

    let last_row = map.len() - 1;
    let last_column = map[0].len() - 1;

    for (row_index, row) in map.iter().enumerate() {
        for (column_index, symbol) in row.iter().enumerate() {
            let is_boundary = row_index == 0
                || row_index == last_row
                || column_index == 0
                || column_index == last_column;

            if is_boundary && !symbol.closes_boundary() {
                return Err(LevelError::OpenBoundary {
                    row: row_index,
                    column: column_index,
                });
            }
        }
    }

    Ok(())
}

pub fn validate_required_entities(map: &ParsedMap) -> Result<(), LevelError> {
    let mut player_count = 0;
    let mut exit_count = 0;
    let mut painting_count = 0;

    for symbol in map.iter().flatten() {
        match symbol {
            MapSymbol::PlayerSpawn => player_count += 1,
            MapSymbol::Tile(Tile::Exit) => exit_count += 1,
            MapSymbol::Tile(Tile::TargetPainting) => painting_count += 1,
            _ => {}
        }
    }

    if player_count != 1 {
        return Err(LevelError::InvalidPlayerCount {
            found: player_count,
        });
    }

    if exit_count != 1 {
        return Err(LevelError::InvalidExitCount { found: exit_count });
    }

    if painting_count == 0 {
        return Err(LevelError::MissingPaintingTarget);
    }

    Ok(())
}

#[allow(dead_code)] // Expuesto para pruebas sin asignación explícita de assets.
pub fn build_level(map: ParsedMap) -> Result<Level, LevelError> {
    build_level_with_assets(map, &[])
}

pub fn build_level_with_assets(
    map: ParsedMap,
    painting_assets: &[&'static str],
) -> Result<Level, LevelError> {
    validate_closed_boundaries(&map)?;
    validate_required_entities(&map)?;

    let mut maze = Vec::with_capacity(map.len());
    let mut player_spawn = Vector2::zero();
    let mut exit = Vector2::zero();
    let mut guards = Vec::new();
    let mut paintings = Vec::new();

    for (row_index, row) in map.into_iter().enumerate() {
        let mut level_row = Vec::with_capacity(row.len());

        for (column_index, symbol) in row.into_iter().enumerate() {
            let position = Vector2::new(column_index as f32 + 0.5, row_index as f32 + 0.5);
            let tile = match symbol {
                MapSymbol::PlayerSpawn => {
                    player_spawn = position;
                    Tile::Floor
                }
                MapSymbol::GuardSpawn => {
                    guards.push(Guard {
                        spawn: position,
                        position,
                    });
                    Tile::Floor
                }
                MapSymbol::Tile(Tile::Exit) => {
                    exit = position;
                    Tile::Exit
                }
                MapSymbol::Tile(Tile::TargetPainting) => {
                    let asset_path = painting_assets.get(paintings.len()).copied();
                    paintings.push(PaintingTarget {
                        map_position: (row_index, column_index),
                        hits: 0,
                        asset_path,
                    });
                    Tile::TargetPainting
                }
                MapSymbol::Tile(tile) => tile,
            };

            level_row.push(tile);
        }

        maze.push(level_row);
    }

    Ok(Level {
        maze,
        player_spawn,
        exit,
        guards,
        paintings,
    })
}

fn load_parsed_map(path: impl AsRef<Path>) -> Result<ParsedMap, LevelError> {
    let contents = fs::read_to_string(path).map_err(LevelError::Io)?;
    let map = parse_map(&contents).map_err(LevelError::InvalidSymbol)?;
    validate_closed_boundaries(&map)?;
    validate_required_entities(&map)?;
    Ok(map)
}

#[allow(dead_code)] // Expuesto para pruebas y carga sin assets explícitos.
pub fn load_level(path: impl AsRef<Path>) -> Result<Level, LevelError> {
    load_level_with_assets(path, &[])
}

pub fn load_level_with_assets(
    path: impl AsRef<Path>,
    painting_assets: &[&'static str],
) -> Result<Level, LevelError> {
    let map = load_parsed_map(path)?;
    build_level_with_assets(map, painting_assets)
}

pub fn load_level_definition(definition: &LevelDefinition) -> Result<Level, LevelError> {
    load_level_with_assets(definition.map_path, definition.painting_assets)
}

#[allow(dead_code)] // Se integrará con el selector y el cargador en pasos posteriores.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelDefinition {
    pub name: &'static str,
    pub map_path: &'static str,
    pub painting_assets: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelSummary {
    pub width: usize,
    pub height: usize,
    pub guards_count: usize,
    pub paintings_count: usize,
}

#[allow(dead_code)] // Se construirá cuando el cargador semántico esté implementado.
#[derive(Debug)]
pub struct Level {
    pub maze: LevelMap,
    pub player_spawn: Vector2,
    pub exit: Vector2,
    pub guards: Vec<Guard>,
    pub paintings: Vec<PaintingTarget>,
}

impl Level {
    #[allow(dead_code)] // Utilizado para inspeccionar y resumir niveles cargados.
    pub fn summary(&self) -> LevelSummary {
        LevelSummary {
            width: self.maze.first().map_or(0, |row| row.len()),
            height: self.maze.len(),
            guards_count: self.guards.len(),
            paintings_count: self.paintings.len(),
        }
    }
}

pub fn summarize_level(path: impl AsRef<Path>) -> Result<LevelSummary, LevelError> {
    let map = load_parsed_map(path)?;
    let width = map.first().map_or(0, |row| row.len());
    let height = map.len();
    let mut guards_count = 0;
    let mut paintings_count = 0;

    for symbol in map.iter().flatten() {
        match symbol {
            MapSymbol::GuardSpawn => guards_count += 1,
            MapSymbol::Tile(Tile::TargetPainting) => paintings_count += 1,
            _ => {}
        }
    }

    Ok(LevelSummary {
        width,
        height,
        guards_count,
        paintings_count,
    })
}

#[allow(dead_code)] // Sus estados y comportamiento pertenecen a una etapa posterior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guard {
    pub spawn: Vector2,
    pub position: Vector2,
}

#[allow(dead_code)] // Su progreso se conectará al sistema de disparos posteriormente.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaintingTarget {
    pub map_position: (usize, usize),
    pub hits: u8,
    pub asset_path: Option<&'static str>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_map_path(label: &str) -> PathBuf {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        std::env::temp_dir().join(format!(
            "doom-rust-{label}-{}-{unique_suffix}.txt",
            std::process::id()
        ))
    }

    #[test]
    fn converts_every_supported_map_symbol() {
        let cases = [
            (' ', MapSymbol::Tile(Tile::Floor)),
            ('p', MapSymbol::PlayerSpawn),
            ('g', MapSymbol::Tile(Tile::Exit)),
            ('e', MapSymbol::GuardSpawn),
            ('1', MapSymbol::Tile(Tile::Wall(WallMaterial::Gallery))),
            ('2', MapSymbol::Tile(Tile::Wall(WallMaterial::Burgundy))),
            ('3', MapSymbol::Tile(Tile::Wall(WallMaterial::Service))),
            ('4', MapSymbol::Tile(Tile::Wall(WallMaterial::Accent))),
            ('T', MapSymbol::Tile(Tile::TargetPainting)),
            ('d', MapSymbol::Tile(Tile::DecorativePainting)),
        ];

        for (symbol, expected) in cases {
            assert_eq!(MapSymbol::try_from(symbol), Ok(expected));
        }
    }

    #[test]
    fn rejects_unknown_map_symbol() {
        assert_eq!(MapSymbol::try_from('?'), Err(InvalidMapSymbol('?')));
    }

    #[test]
    fn parses_text_into_rows_and_columns() {
        let parsed = parse_map("1pT\r\n2eg\r\n").unwrap();

        assert_eq!(
            parsed,
            vec![
                vec![
                    MapSymbol::Tile(Tile::Wall(WallMaterial::Gallery)),
                    MapSymbol::PlayerSpawn,
                    MapSymbol::Tile(Tile::TargetPainting),
                ],
                vec![
                    MapSymbol::Tile(Tile::Wall(WallMaterial::Burgundy)),
                    MapSymbol::GuardSpawn,
                    MapSymbol::Tile(Tile::Exit),
                ],
            ]
        );
    }

    #[test]
    fn parser_propagates_unknown_symbols() {
        assert_eq!(parse_map("111\n1?1"), Err(InvalidMapSymbol('?')));
    }

    #[test]
    fn loads_and_parses_map_file() {
        let path = temporary_map_path("valid-map");
        let contents = "1111\n1pT1\n1 e1\n11g1\n";
        std::fs::write(&path, contents).unwrap();

        let loaded = load_parsed_map(&path);
        std::fs::remove_file(path).unwrap();

        assert_eq!(loaded.unwrap(), parse_map(contents).unwrap());
    }

    #[test]
    fn reports_io_error_for_missing_map_file() {
        let path = temporary_map_path("missing-map");

        assert!(matches!(load_level(path), Err(LevelError::Io(_))));
    }

    #[test]
    fn loads_complete_level_from_file() {
        let path = temporary_map_path("complete-level");
        let contents = "11111\n1p e1\n1 T 1\n11g11\n";
        std::fs::write(&path, contents).unwrap();

        let level = load_level(&path);
        std::fs::remove_file(path).unwrap();
        let level = level.unwrap();

        assert_eq!(level.player_spawn, Vector2::new(1.5, 1.5));
        assert_eq!(level.exit, Vector2::new(2.5, 3.5));
        assert_eq!(level.guards.len(), 1);
        assert_eq!(level.paintings.len(), 1);
    }

    #[test]
    fn accepts_rectangular_map_shape() {
        let map = parse_map("1111\n1pT1\n1g11").unwrap();

        assert!(validate_map_shape(&map).is_ok());
    }

    #[test]
    fn rejects_empty_map() {
        let map = parse_map("").unwrap();

        assert!(matches!(
            validate_map_shape(&map),
            Err(LevelError::EmptyMap)
        ));
    }

    #[test]
    fn rejects_map_with_empty_row() {
        let map = parse_map("111\n\n111").unwrap();

        assert!(matches!(
            validate_map_shape(&map),
            Err(LevelError::EmptyRow { row: 1 })
        ));
    }

    #[test]
    fn rejects_non_rectangular_map() {
        let map = parse_map("1111\n1p1\n1111").unwrap();

        assert!(matches!(
            validate_map_shape(&map),
            Err(LevelError::NonRectangular {
                row: 1,
                expected: 4,
                actual: 3,
            })
        ));
    }

    #[test]
    fn accepts_closed_map_with_exit_on_boundary() {
        let map = parse_map("11111\n1p T1\n1 e 1\n11g11").unwrap();

        assert!(validate_closed_boundaries(&map).is_ok());
    }

    #[test]
    fn rejects_walkable_cell_on_boundary() {
        let map = parse_map("11 11\n1p T1\n1 e 1\n11g11").unwrap();

        assert!(matches!(
            validate_closed_boundaries(&map),
            Err(LevelError::OpenBoundary { row: 0, column: 2 })
        ));
    }

    #[test]
    fn accepts_required_level_entities() {
        let map = parse_map("1p eTg").unwrap();

        assert!(validate_required_entities(&map).is_ok());
    }

    #[test]
    fn rejects_map_without_player() {
        let map = parse_map("1 Tg").unwrap();

        assert!(matches!(
            validate_required_entities(&map),
            Err(LevelError::InvalidPlayerCount { found: 0 })
        ));
    }

    #[test]
    fn rejects_map_with_multiple_players() {
        let map = parse_map("ppTg").unwrap();

        assert!(matches!(
            validate_required_entities(&map),
            Err(LevelError::InvalidPlayerCount { found: 2 })
        ));
    }

    #[test]
    fn rejects_map_without_exit() {
        let map = parse_map("1p T").unwrap();

        assert!(matches!(
            validate_required_entities(&map),
            Err(LevelError::InvalidExitCount { found: 0 })
        ));
    }

    #[test]
    fn rejects_map_with_multiple_exits() {
        let map = parse_map("pTgg").unwrap();

        assert!(matches!(
            validate_required_entities(&map),
            Err(LevelError::InvalidExitCount { found: 2 })
        ));
    }

    #[test]
    fn rejects_map_without_target_painting() {
        let map = parse_map("p dg").unwrap();

        assert!(matches!(
            validate_required_entities(&map),
            Err(LevelError::MissingPaintingTarget)
        ));
    }

    #[test]
    fn builds_level_entities_and_replaces_spawn_markers_with_floor() {
        let map = parse_map("111111\n1p e 1\n1 T T1\n1 e  1\n11g111").unwrap();

        let level = build_level(map).unwrap();

        assert_eq!(level.player_spawn, Vector2::new(1.5, 1.5));
        assert_eq!(level.exit, Vector2::new(2.5, 4.5));
        assert_eq!(level.maze[1][1], Tile::Floor);
        assert_eq!(level.maze[1][3], Tile::Floor);
        assert_eq!(level.maze[3][2], Tile::Floor);
        assert_eq!(level.maze[4][2], Tile::Exit);
        assert_eq!(level.guards.len(), 2);
        assert_eq!(level.guards[0].spawn, Vector2::new(3.5, 1.5));
        assert_eq!(level.guards[1].spawn, Vector2::new(2.5, 3.5));
        assert_eq!(
            level
                .paintings
                .iter()
                .map(|painting| (painting.map_position, painting.hits))
                .collect::<Vec<_>>(),
            vec![((2, 2), 0), ((2, 4), 0)]
        );
    }

    #[test]
    fn summarizes_level_file_correctly() {
        let path = temporary_map_path("summary-level");
        let contents = "111111\n1p e 1\n1 T T1\n1 e  1\n11g111\n";
        std::fs::write(&path, contents).unwrap();

        let summary = summarize_level(&path).unwrap();
        std::fs::remove_file(path).unwrap();

        assert_eq!(
            summary,
            LevelSummary {
                width: 6,
                height: 5,
                guards_count: 2,
                paintings_count: 2,
            }
        );
    }

    #[test]
    fn level_summary_method_matches_built_level() {
        let map = parse_map("111111\n1p e 1\n1 T T1\n1 e  1\n11g111").unwrap();
        let level = build_level(map).unwrap();

        assert_eq!(
            level.summary(),
            LevelSummary {
                width: 6,
                height: 5,
                guards_count: 2,
                paintings_count: 2,
            }
        );
    }

    #[test]
    fn assigns_painting_assets_in_reading_order() {
        let map = parse_map("111111\n1p T 1\n1   T1\n11g111").unwrap();
        let assets = ["asset_one.jpg", "asset_two.jpg"];
        let level = build_level_with_assets(map, &assets).unwrap();

        assert_eq!(level.paintings.len(), 2);
        assert_eq!(level.paintings[0].map_position, (1, 3));
        assert_eq!(level.paintings[0].asset_path, Some("asset_one.jpg"));
        assert_eq!(level.paintings[1].map_position, (2, 4));
        assert_eq!(level.paintings[1].asset_path, Some("asset_two.jpg"));
    }
}
