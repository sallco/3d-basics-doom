use raylib::prelude::Vector2;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::maze::Maze;

#[allow(dead_code)] // Se usará cuando Maze migre de caracteres a celdas semánticas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallMaterial {
    Gallery,
    Burgundy,
    Service,
    Accent,
}

#[allow(dead_code)] // Se usará cuando Maze migre de caracteres a celdas semánticas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tile {
    Floor,
    Exit,
    Wall(WallMaterial),
    TargetPainting,
    DecorativePainting,
}

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

#[allow(dead_code)] // Se integrará con el selector y el cargador en pasos posteriores.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelDefinition {
    pub name: &'static str,
    pub map_path: &'static str,
    pub painting_assets: &'static [&'static str],
}

#[allow(dead_code)] // Se construirá cuando el cargador semántico esté implementado.
#[derive(Debug)]
pub struct Level {
    pub maze: Maze,
    pub player_spawn: Vector2,
    pub exit: Vector2,
    pub guards: Vec<Guard>,
    pub paintings: Vec<PaintingTarget>,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
