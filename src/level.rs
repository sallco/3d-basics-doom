use raylib::prelude::Vector2;

use crate::maze::Maze;

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
