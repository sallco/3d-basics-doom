use std::error::Error;
use std::fmt::{Display, Formatter};

use raylib::prelude::*;

use crate::events::process_events;
use crate::framebuffer::Framebuffer;
use crate::level::{Level, LevelDefinition, LevelError, load_level};
use crate::player::Player;
use crate::renderer::render_level_3d;

pub const LOGICAL_WIDTH: u32 = 960;
pub const LOGICAL_HEIGHT: u32 = 540;
pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;

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
    Level(LevelError),
}

impl Display for GameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Level(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for GameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Level(error) => Some(error),
        }
    }
}

pub struct Game {
    screen: GameScreen,
    level_definition: &'static LevelDefinition,
    level: Level,
    player: Player,
    exit_unlocked: bool,
}

impl Game {
    pub fn new(level_definition: &'static LevelDefinition) -> Result<Self, GameError> {
        let level = load_level(level_definition.map_path).map_err(GameError::Level)?;
        let player = Player::new(level.player_spawn);

        Ok(Self {
            screen: GameScreen::Welcome,
            level_definition,
            level,
            player,
            exit_unlocked: false,
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

                process_events(
                    window,
                    &mut self.player,
                    &self.level.maze,
                    window.get_frame_time(),
                    self.exit_unlocked,
                );
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
            GameScreen::LevelSelect => render_level_select(framebuffer, self.level_definition.name),
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
        render_level_3d(
            framebuffer,
            &self.level.maze,
            self.player.pos,
            self.player.a,
            self.player.fov,
            self.exit_unlocked,
        );
    }
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

fn render_level_select(framebuffer: &mut Framebuffer, level_name: &str) {
    framebuffer.draw_centered_text("SELECCION DE NIVEL", 120, 46, Color::RAYWHITE);
    framebuffer.draw_centered_text(level_name, 260, 34, Color::GOLD);
    framebuffer.draw_centered_text("ENTER para jugar", 330, 22, Color::LIGHTGRAY);
}

fn render_message_screen(framebuffer: &mut Framebuffer, title: &str, subtitle: &str, color: Color) {
    framebuffer.draw_centered_text(title, 190, 56, color);
    framebuffer.draw_centered_text(subtitle, 310, 24, Color::RAYWHITE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::LEVEL_DEFINITIONS;

    #[test]
    fn starts_on_welcome_with_semantic_level_spawn() {
        let game = Game::new(&LEVEL_DEFINITIONS[0]).unwrap();

        assert_eq!(game.screen, GameScreen::Welcome);
        assert_eq!(game.player.pos, game.level.player_spawn);
        assert_eq!(game.level_definition.name, "Galería de ingreso");
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
