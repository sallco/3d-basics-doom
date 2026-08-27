use std::error::Error;
use std::fmt::{Display, Formatter};

use raylib::prelude::*;

use crate::caster::{cast_ray, render_3d};
use crate::events::process_events;
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, load_maze, render_maze};
use crate::player::Player;
use crate::textures::TextureManager;

pub const LOGICAL_WIDTH: u32 = 960;
pub const LOGICAL_HEIGHT: u32 = 540;
pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;
const BLOCK_SIZE: usize = 36;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameScreen {
    Welcome,
    LevelSelect,
    Playing,
    #[allow(dead_code)] // Se activará al implementar la condición de salida.
    Success,
    #[allow(dead_code)] // Se activará al implementar las vidas del jugador.
    GameOver,
}

#[derive(Debug)]
pub enum GameError {
    Maze(std::io::Error),
    MissingPlayerSpawn,
}

impl Display for GameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Maze(error) => write!(formatter, "no se pudo cargar el mapa: {error}"),
            Self::MissingPlayerSpawn => {
                write!(
                    formatter,
                    "el mapa no contiene una posición de jugador ('p')"
                )
            }
        }
    }
}

impl Error for GameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Maze(error) => Some(error),
            Self::MissingPlayerSpawn => None,
        }
    }
}

pub struct Game {
    screen: GameScreen,
    maze: Maze,
    player: Player,
    textures: TextureManager,
    show_3d: bool,
}

impl Game {
    pub fn new(maze_path: &str) -> Result<Self, GameError> {
        let maze = load_maze(maze_path).map_err(GameError::Maze)?;
        let (player_row, player_column) = find_player_spawn(&maze)?;

        let textures = TextureManager::load_defaults().unwrap_or_else(|error| {
            eprintln!("Advertencia: {error}. Se usarán colores de respaldo.");
            TextureManager::new()
        });

        Ok(Self {
            screen: GameScreen::Welcome,
            maze,
            player: Player::new(Vector2::new(
                (player_column * BLOCK_SIZE + BLOCK_SIZE / 2) as f32,
                (player_row * BLOCK_SIZE + BLOCK_SIZE / 2) as f32,
            )),
            textures,
            show_3d: true,
        })
    }

    pub fn update(&mut self, window: &RaylibHandle) {
        match self.screen {
            GameScreen::Welcome => {
                if confirm_pressed(window) {
                    self.screen = GameScreen::LevelSelect;
                }
            }
            GameScreen::LevelSelect => {
                if confirm_pressed(window) {
                    self.screen = GameScreen::Playing;
                }
            }
            GameScreen::Playing => {
                if window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    self.screen = GameScreen::LevelSelect;
                    return;
                }

                process_events(window, &mut self.player, &self.maze, BLOCK_SIZE);

                if window.is_key_pressed(KeyboardKey::KEY_M) {
                    self.show_3d = !self.show_3d;
                }
            }
            GameScreen::Success | GameScreen::GameOver => {
                if confirm_pressed(window) {
                    self.screen = GameScreen::LevelSelect;
                }
            }
        }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear();

        match self.screen {
            GameScreen::Welcome => render_welcome(framebuffer),
            GameScreen::LevelSelect => render_level_select(framebuffer),
            GameScreen::Playing => self.render_playing(framebuffer),
            GameScreen::Success => render_message_screen(
                framebuffer,
                "NIVEL COMPLETADO",
                "Enter para volver al selector",
                Color::GREEN,
            ),
            GameScreen::GameOver => render_message_screen(
                framebuffer,
                "GAME OVER",
                "Enter para volver al selector",
                Color::RED,
            ),
        }
    }

    fn render_playing(&self, framebuffer: &mut Framebuffer) {
        if self.show_3d {
            render_3d(
                framebuffer,
                &self.maze,
                &self.player,
                BLOCK_SIZE,
                &self.textures,
            );
            return;
        }

        render_maze(framebuffer, &self.maze, BLOCK_SIZE);

        for ray_index in 0..framebuffer.width {
            let current_ray = ray_index as f32 / framebuffer.width as f32;
            let ray_angle = self.player.a - self.player.fov / 2.0 + self.player.fov * current_ray;

            cast_ray(
                framebuffer,
                &self.maze,
                &self.player,
                ray_angle,
                BLOCK_SIZE,
                true,
            );
        }

        self.player.draw(framebuffer);
    }
}

fn find_player_spawn(maze: &Maze) -> Result<(usize, usize), GameError> {
    maze.iter()
        .enumerate()
        .find_map(|(row_index, row)| {
            row.iter()
                .position(|&cell| cell == 'p')
                .map(|column_index| (row_index, column_index))
        })
        .ok_or(GameError::MissingPlayerSpawn)
}

fn confirm_pressed(window: &RaylibHandle) -> bool {
    window.is_key_pressed(KeyboardKey::KEY_ENTER) || window.is_key_pressed(KeyboardKey::KEY_SPACE)
}

fn render_welcome(framebuffer: &mut Framebuffer) {
    framebuffer.draw_centered_text("MUSEO NOCTURNO", 145, 58, Color::RAYWHITE);
    framebuffer.draw_centered_text(
        "Interviene las obras y escapa antes de que te atrapen.",
        235,
        24,
        Color::LIGHTGRAY,
    );
    framebuffer.draw_centered_text("ENTER para comenzar", 385, 28, Color::GOLD);
}

fn render_level_select(framebuffer: &mut Framebuffer) {
    framebuffer.draw_centered_text("SELECCION DE NIVEL", 120, 46, Color::RAYWHITE);
    framebuffer.draw_centered_text("Galeria original", 260, 34, Color::GOLD);
    framebuffer.draw_centered_text(
        "Por ahora hay un nivel disponible - ENTER para jugar",
        330,
        22,
        Color::LIGHTGRAY,
    );
}

fn render_message_screen(framebuffer: &mut Framebuffer, title: &str, subtitle: &str, color: Color) {
    framebuffer.draw_centered_text(title, 190, 56, color);
    framebuffer.draw_centered_text(subtitle, 310, 24, Color::RAYWHITE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_player_in_rectangular_maze() {
        let maze = vec![vec!['+', '-', '+'], vec!['|', 'p', '|']];

        assert_eq!(find_player_spawn(&maze).unwrap(), (1, 1));
    }

    #[test]
    fn reports_missing_player_without_panicking() {
        let maze = vec![vec!['+', '-', '+']];

        assert!(matches!(
            find_player_spawn(&maze),
            Err(GameError::MissingPlayerSpawn)
        ));
    }

    #[test]
    fn all_planned_screens_are_constructible() {
        let screens = [
            GameScreen::Welcome,
            GameScreen::LevelSelect,
            GameScreen::Playing,
            GameScreen::Success,
            GameScreen::GameOver,
        ];

        assert_eq!(screens.len(), 5);
    }
}
