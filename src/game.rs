use std::error::Error;
use std::fmt::{Display, Formatter};

use raylib::prelude::*;

use crate::assets::AssetManager;
use crate::events::process_events;
use crate::framebuffer::Framebuffer;
use crate::level::{
    Level, LevelDefinition, LevelError, LevelSummary, PaintSplatter, Tile, WallMaterial,
    load_level_definition, summarize_level,
};
use crate::player::Player;
use crate::raycasting::cast_ray_dda;
use crate::renderer::render_level_3d;

pub const LOGICAL_WIDTH: u32 = 960;
pub const LOGICAL_HEIGHT: u32 = 540;
pub const WINDOW_WIDTH: i32 = 1280;
pub const WINDOW_HEIGHT: i32 = 720;

pub const SHOT_COOLDOWN: f32 = 0.30;
pub const REQUIRED_HITS_PER_PAINTING: u8 = 3;

const CARD_START_X: f32 = 40.0;
const CARD_Y: f32 = 120.0;
const CARD_WIDTH: f32 = 280.0;
const CARD_HEIGHT: f32 = 330.0;
const CARD_GAP: f32 = 20.0;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShotHitResult {
    Miss,
    Wall(WallMaterial),
    DecorativePainting,
    TargetPainting {
        painting_index: usize,
        hits: u8,
        completed: bool,
    },
}

#[derive(Debug)]
pub enum GameError {
    EmptyCatalog,
    InvalidLevelIndex { index: usize, total: usize },
    Level(LevelError),
}

impl Display for GameError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCatalog => write!(formatter, "el catálogo de niveles está vacío"),
            Self::InvalidLevelIndex { index, total } => {
                write!(
                    formatter,
                    "índice de nivel inválido: {index}; total disponible: {total}"
                )
            }
            Self::Level(error) => Display::fmt(error, formatter),
        }
    }
}

impl Error for GameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyCatalog | Self::InvalidLevelIndex { .. } => None,
            Self::Level(error) => Some(error),
        }
    }
}

pub struct Game {
    screen: GameScreen,
    level_definitions: &'static [LevelDefinition],
    level_summaries: Vec<LevelSummary>,
    selected_level_index: usize,
    level: Level,
    player: Player,
    exit_unlocked: bool,
    shot_cooldown: f32,
    asset_manager: AssetManager,
}

impl Game {
    pub fn new(level_definitions: &'static [LevelDefinition]) -> Result<Self, GameError> {
        if level_definitions.is_empty() {
            return Err(GameError::EmptyCatalog);
        }

        let mut level_summaries = Vec::with_capacity(level_definitions.len());
        let mut asset_manager = AssetManager::new();

        for definition in level_definitions {
            let summary = summarize_level(definition.map_path).map_err(GameError::Level)?;
            level_summaries.push(summary);
            asset_manager.preload_level_assets(definition.painting_assets);
        }

        let level = load_level_definition(&level_definitions[0]).map_err(GameError::Level)?;
        let player = Player::new(level.player_spawn);

        Ok(Self {
            screen: GameScreen::Welcome,
            level_definitions,
            level_summaries,
            selected_level_index: 0,
            level,
            player,
            exit_unlocked: false,
            shot_cooldown: 0.0,
            asset_manager,
        })
    }

    #[allow(dead_code)] // Expuesto para pruebas e integración de estados posteriores.
    pub fn screen(&self) -> GameScreen {
        self.screen
    }

    #[allow(dead_code)] // Expuesto para pruebas y transiciones directas.
    pub fn set_screen(&mut self, screen: GameScreen) {
        self.screen = screen;
    }

    #[allow(dead_code)] // Expuesto para inspección y pruebas del selector.
    pub fn selected_level_index(&self) -> usize {
        self.selected_level_index
    }

    #[allow(dead_code)] // Expuesto para HUD y pruebas.
    pub fn selected_level_definition(&self) -> &'static LevelDefinition {
        &self.level_definitions[self.selected_level_index]
    }

    #[allow(dead_code)] // Expuesto para inspección del catálogo.
    pub fn level_definitions(&self) -> &'static [LevelDefinition] {
        self.level_definitions
    }

    #[allow(dead_code)] // Expuesto para el selector y pruebas.
    pub fn level_summaries(&self) -> &[LevelSummary] {
        &self.level_summaries
    }

    #[allow(dead_code)] // Expuesto para el selector y HUD.
    pub fn selected_level_summary(&self) -> LevelSummary {
        self.level_summaries[self.selected_level_index]
    }

    pub fn select_level(&mut self, index: usize) -> Result<(), GameError> {
        if index >= self.level_definitions.len() {
            return Err(GameError::InvalidLevelIndex {
                index,
                total: self.level_definitions.len(),
            });
        }
        self.selected_level_index = index;
        Ok(())
    }

    pub fn select_next_level(&mut self) {
        if !self.level_definitions.is_empty() {
            self.selected_level_index =
                (self.selected_level_index + 1) % self.level_definitions.len();
        }
    }

    pub fn select_previous_level(&mut self) {
        if !self.level_definitions.is_empty() {
            if self.selected_level_index == 0 {
                self.selected_level_index = self.level_definitions.len() - 1;
            } else {
                self.selected_level_index -= 1;
            }
        }
    }

    pub fn start_selected_level(&mut self) -> Result<(), GameError> {
        let definition = self
            .level_definitions
            .get(self.selected_level_index)
            .ok_or(GameError::InvalidLevelIndex {
                index: self.selected_level_index,
                total: self.level_definitions.len(),
            })?;
        let level = load_level_definition(definition).map_err(GameError::Level)?;
        self.player = Player::new(level.player_spawn);
        self.level = level;
        self.exit_unlocked = false;
        self.shot_cooldown = 0.0;
        self.screen = GameScreen::Playing;
        Ok(())
    }

    fn try_start_selected_level(&mut self) {
        if let Err(error) = self.start_selected_level() {
            eprintln!("Error al cargar nivel: {error}");
        }
    }

    pub fn shoot(&mut self) -> Option<ShotHitResult> {
        if self.shot_cooldown > 0.0 {
            return None;
        }
        self.shot_cooldown = SHOT_COOLDOWN;

        let Some(hit) = cast_ray_dda(&self.level.maze, self.player.pos, self.player.a) else {
            return Some(ShotHitResult::Miss);
        };

        match hit.tile {
            Tile::TargetPainting => {
                let mut shot_result = None;
                if let Some((index, painting)) = self
                    .level
                    .paintings
                    .iter_mut()
                    .enumerate()
                    .find(|(_, p)| p.map_position == hit.map_position)
                {
                    let completed_before = painting.hits >= REQUIRED_HITS_PER_PAINTING;
                    if !completed_before {
                        painting.hits += 1;
                        let splatter = generate_splatter(painting.hits, hit.texture_u);
                        painting.splatters.push(splatter);
                    }

                    shot_result = Some(ShotHitResult::TargetPainting {
                        painting_index: index,
                        hits: painting.hits,
                        completed: painting.hits >= REQUIRED_HITS_PER_PAINTING,
                    });
                }

                if let Some(result) = shot_result {
                    self.update_exit_unlock_state();
                    Some(result)
                } else {
                    Some(ShotHitResult::Miss)
                }
            }
            Tile::Wall(material) => Some(ShotHitResult::Wall(material)),
            Tile::DecorativePainting => Some(ShotHitResult::DecorativePainting),
            Tile::Floor | Tile::Exit => Some(ShotHitResult::Miss),
        }
    }

    pub fn update_exit_unlock_state(&mut self) {
        if !self.level.paintings.is_empty()
            && self
                .level
                .paintings
                .iter()
                .all(|p| p.hits >= REQUIRED_HITS_PER_PAINTING)
        {
            self.exit_unlocked = true;
        }
    }

    #[allow(dead_code)] // Expuesto para HUD y pruebas.
    pub fn shot_cooldown(&self) -> f32 {
        self.shot_cooldown
    }

    #[allow(dead_code)] // Expuesto para HUD y pruebas.
    pub fn completed_paintings_count(&self) -> usize {
        self.level
            .paintings
            .iter()
            .filter(|p| p.hits >= REQUIRED_HITS_PER_PAINTING)
            .count()
    }

    #[allow(dead_code)] // Expuesto para HUD y pruebas.
    pub fn total_paintings_count(&self) -> usize {
        self.level.paintings.len()
    }

    pub fn check_exit_reached(&self) -> bool {
        let player_cell_x = self.player.pos.x.floor() as usize;
        let player_cell_y = self.player.pos.y.floor() as usize;
        let exit_cell_x = self.level.exit.x.floor() as usize;
        let exit_cell_y = self.level.exit.y.floor() as usize;

        if player_cell_x == exit_cell_x && player_cell_y == exit_cell_y {
            return true;
        }

        let dx = self.player.pos.x - self.level.exit.x;
        let dy = self.player.pos.y - self.level.exit.y;
        dx * dx + dy * dy <= 0.64
    }

    #[allow(dead_code)] // Expuesto para inspección y pruebas del jugador.
    pub fn player(&self) -> &Player {
        &self.player
    }

    #[allow(dead_code)] // Expuesto para inspección y pruebas del nivel.
    pub fn level(&self) -> &Level {
        &self.level
    }

    #[allow(dead_code)] // Expuesto para inspección de la condición de victoria.
    pub fn exit_unlocked(&self) -> bool {
        self.exit_unlocked
    }

    #[allow(dead_code)] // Expuesto para inspección y pruebas del gestor de recursos.
    pub fn asset_manager(&self) -> &AssetManager {
        &self.asset_manager
    }

    pub fn update(&mut self, window: &mut RaylibHandle) {
        match self.screen {
            GameScreen::Welcome => {
                window.enable_cursor();
                if confirm_pressed(window) {
                    self.screen = GameScreen::LevelSelect;
                }
            }
            GameScreen::LevelSelect => {
                window.enable_cursor();
                if window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    self.screen = GameScreen::Welcome;
                    return;
                }

                if window.is_key_pressed(KeyboardKey::KEY_LEFT)
                    || window.is_key_pressed(KeyboardKey::KEY_A)
                {
                    self.select_previous_level();
                }
                if window.is_key_pressed(KeyboardKey::KEY_RIGHT)
                    || window.is_key_pressed(KeyboardKey::KEY_D)
                {
                    self.select_next_level();
                }

                if window.is_key_pressed(KeyboardKey::KEY_ONE)
                    || window.is_key_pressed(KeyboardKey::KEY_KP_1)
                {
                    let _ = self.select_level(0);
                }
                if window.is_key_pressed(KeyboardKey::KEY_TWO)
                    || window.is_key_pressed(KeyboardKey::KEY_KP_2)
                {
                    let _ = self.select_level(1);
                }
                if window.is_key_pressed(KeyboardKey::KEY_THREE)
                    || window.is_key_pressed(KeyboardKey::KEY_KP_3)
                {
                    let _ = self.select_level(2);
                }

                if let Some(mouse) = mouse_to_logical(window, LOGICAL_WIDTH, LOGICAL_HEIGHT) {
                    for (i, _) in self.level_definitions.iter().enumerate() {
                        let card_x = CARD_START_X + i as f32 * (CARD_WIDTH + CARD_GAP);
                        if mouse.x >= card_x
                            && mouse.x <= card_x + CARD_WIDTH
                            && mouse.y >= CARD_Y
                            && mouse.y <= CARD_Y + CARD_HEIGHT
                            && window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                        {
                            if self.selected_level_index == i {
                                self.try_start_selected_level();
                            } else {
                                let _ = self.select_level(i);
                            }
                            return;
                        }
                    }
                }

                if confirm_pressed(window) {
                    self.try_start_selected_level();
                }
            }
            GameScreen::Playing => {
                window.disable_cursor();

                if window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    window.enable_cursor();
                    self.screen = GameScreen::LevelSelect;
                    return;
                }

                let delta_time = window.get_frame_time();
                if self.shot_cooldown > 0.0 {
                    self.shot_cooldown = (self.shot_cooldown - delta_time).max(0.0);
                }

                if window.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                    || window.is_key_pressed(KeyboardKey::KEY_SPACE)
                {
                    self.shoot();
                }

                process_events(
                    window,
                    &mut self.player,
                    &self.level.maze,
                    delta_time,
                    self.exit_unlocked,
                );

                if self.exit_unlocked && self.check_exit_reached() {
                    window.enable_cursor();
                    self.screen = GameScreen::Success;
                }
            }
            GameScreen::Success | GameScreen::GameOver => {
                window.enable_cursor();
                if confirm_pressed(window) || window.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    self.screen = GameScreen::LevelSelect;
                }
            }
        }
    }

    pub fn render(&self, framebuffer: &mut Framebuffer) {
        framebuffer.clear();

        match self.screen {
            GameScreen::Welcome => render_welcome(framebuffer),
            GameScreen::LevelSelect => render_level_select(
                framebuffer,
                self.level_definitions,
                &self.level_summaries,
                self.selected_level_index,
            ),
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
            &self.level,
            self.player.pos,
            self.player.a,
            self.player.fov,
            self.exit_unlocked,
            &self.asset_manager,
        );
    }
}

fn generate_splatter(hit_number: u8, hit_u: f32) -> PaintSplatter {
    let (u_offset, v, color) = match hit_number {
        1 => (0.0, 0.38, Color::new(255, 30, 120, 255)),
        2 => (0.18, 0.62, Color::new(0, 230, 255, 255)),
        _ => (-0.18, 0.50, Color::new(255, 215, 0, 255)),
    };
    let u = (hit_u + u_offset).clamp(0.15, 0.85);
    let radius = 0.12;

    PaintSplatter {
        u,
        v,
        radius,
        color,
    }
}

fn confirm_pressed(window: &RaylibHandle) -> bool {
    window.is_key_pressed(KeyboardKey::KEY_ENTER) || window.is_key_pressed(KeyboardKey::KEY_SPACE)
}

fn mouse_to_logical(
    window: &RaylibHandle,
    framebuffer_width: u32,
    framebuffer_height: u32,
) -> Option<Vector2> {
    let window_width = window.get_screen_width() as f32;
    let window_height = window.get_screen_height() as f32;
    if window_width <= 0.0 || window_height <= 0.0 {
        return None;
    }
    let scale =
        (window_width / framebuffer_width as f32).min(window_height / framebuffer_height as f32);
    if scale <= 0.0 {
        return None;
    }
    let destination_width = framebuffer_width as f32 * scale;
    let destination_height = framebuffer_height as f32 * scale;
    let offset_x = (window_width - destination_width) / 2.0;
    let offset_y = (window_height - destination_height) / 2.0;

    let mouse = window.get_mouse_position();
    let logical_x = (mouse.x - offset_x) / scale;
    let logical_y = (mouse.y - offset_y) / scale;

    if logical_x >= 0.0
        && logical_x < framebuffer_width as f32
        && logical_y >= 0.0
        && logical_y < framebuffer_height as f32
    {
        Some(Vector2::new(logical_x, logical_y))
    } else {
        None
    }
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

fn render_level_select(
    framebuffer: &mut Framebuffer,
    definitions: &[LevelDefinition],
    summaries: &[LevelSummary],
    selected_index: usize,
) {
    framebuffer.draw_centered_text("SELECCION DE NIVEL", 32, 42, Color::RAYWHITE);
    framebuffer.draw_centered_text(
        "Elige el sector del museo que deseas intervenir",
        80,
        18,
        Color::LIGHTGRAY,
    );

    for (i, definition) in definitions.iter().enumerate() {
        let is_selected = i == selected_index;
        let card_x = (CARD_START_X + i as f32 * (CARD_WIDTH + CARD_GAP)) as i32;
        let card_y = CARD_Y as i32;
        let card_w = CARD_WIDTH as i32;
        let card_h = CARD_HEIGHT as i32;

        let bg_color = if is_selected {
            Color::new(32, 28, 20, 255)
        } else {
            Color::new(18, 18, 22, 255)
        };
        framebuffer.draw_rectangle(card_x, card_y, card_w, card_h, bg_color);

        if is_selected {
            framebuffer.draw_rectangle_lines(card_x, card_y, card_w, card_h, 3, Color::GOLD);
        } else {
            framebuffer.draw_rectangle_lines(card_x, card_y, card_w, card_h, 1, Color::DARKGRAY);
        }

        let mission_tag = format!("NIVEL {:02}", i + 1);
        let tag_color = if is_selected {
            Color::GOLD
        } else {
            Color::GRAY
        };
        framebuffer.draw_text(&mission_tag, card_x + 16, card_y + 18, 16, tag_color);

        let name_color = if is_selected {
            Color::RAYWHITE
        } else {
            Color::LIGHTGRAY
        };
        framebuffer.draw_text(definition.name, card_x + 16, card_y + 42, 20, name_color);

        let divider_color = if is_selected {
            Color::GOLD
        } else {
            Color::DARKGRAY
        };
        framebuffer.draw_rectangle(card_x + 16, card_y + 74, card_w - 32, 2, divider_color);

        if let Some(summary) = summaries.get(i) {
            let dim_text = format!("Dimensiones: {}x{}", summary.width, summary.height);
            framebuffer.draw_text(&dim_text, card_x + 16, card_y + 92, 16, Color::RAYWHITE);

            let guard_text = format!("Guardias: {}", summary.guards_count);
            framebuffer.draw_text(&guard_text, card_x + 16, card_y + 122, 16, Color::RAYWHITE);

            let obj_text = format!("Objetivos: {} pinturas", summary.paintings_count);
            framebuffer.draw_text(&obj_text, card_x + 16, card_y + 152, 16, Color::RAYWHITE);

            framebuffer.draw_text(
                "Impactos por obra: 3",
                card_x + 16,
                card_y + 182,
                14,
                Color::LIGHTGRAY,
            );

            let (diff_text, diff_color) = match i {
                0 => ("Dificultad: Inicial", Color::GREEN),
                1 => ("Dificultad: Media", Color::YELLOW),
                _ => ("Dificultad: Alta", Color::RED),
            };
            framebuffer.draw_text(diff_text, card_x + 16, card_y + 220, 16, diff_color);
        }

        let button_y = card_y + card_h - 54;
        if is_selected {
            framebuffer.draw_rectangle(card_x + 16, button_y, card_w - 32, 38, Color::GOLD);
            framebuffer.draw_text(
                "[ ENTER ] JUGAR",
                card_x + 46,
                button_y + 10,
                18,
                Color::BLACK,
            );
        } else {
            framebuffer.draw_rectangle(
                card_x + 16,
                button_y,
                card_w - 32,
                38,
                Color::new(28, 28, 34, 255),
            );
            framebuffer.draw_text(
                "Seleccionar",
                card_x + 78,
                button_y + 11,
                16,
                Color::DARKGRAY,
            );
        }
    }

    framebuffer.draw_centered_text(
        "Flechas / A-D / 1-3: Elegir    |    ENTER / Clic: Jugar    |    ESC: Volver",
        485,
        18,
        Color::GOLD,
    );
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
        let game = Game::new(&LEVEL_DEFINITIONS).unwrap();

        assert_eq!(game.screen(), GameScreen::Welcome);
        assert_eq!(game.selected_level_index(), 0);
        assert_eq!(game.player().pos, game.level().player_spawn);
        assert_eq!(game.selected_level_definition().name, "Galería de ingreso");
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

    #[test]
    fn rejects_empty_level_catalog() {
        let empty_definitions: &'static [LevelDefinition] = &[];
        assert!(matches!(
            Game::new(empty_definitions),
            Err(GameError::EmptyCatalog)
        ));
    }

    #[test]
    fn level_selection_cycles_forward_and_backward() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        assert_eq!(game.selected_level_index(), 0);

        game.select_next_level();
        assert_eq!(game.selected_level_index(), 1);
        assert_eq!(game.selected_level_definition().name, "Ala moderna");

        game.select_next_level();
        assert_eq!(game.selected_level_index(), 2);
        assert_eq!(game.selected_level_definition().name, "Archivo nocturno");

        game.select_next_level();
        assert_eq!(game.selected_level_index(), 0);

        game.select_previous_level();
        assert_eq!(game.selected_level_index(), 2);

        game.select_previous_level();
        assert_eq!(game.selected_level_index(), 1);
    }

    #[test]
    fn direct_level_selection_validates_bounds() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();

        assert!(game.select_level(2).is_ok());
        assert_eq!(game.selected_level_index(), 2);

        assert!(matches!(
            game.select_level(3),
            Err(GameError::InvalidLevelIndex { index: 3, total: 3 })
        ));
        assert_eq!(game.selected_level_index(), 2);
    }

    #[test]
    fn starting_selected_level_updates_game_state_and_resets_player() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        game.select_level(1).unwrap();

        assert_eq!(game.screen(), GameScreen::Welcome);
        game.start_selected_level().unwrap();

        assert_eq!(game.screen(), GameScreen::Playing);
        assert_eq!(game.player().pos, game.level().player_spawn);
        assert_eq!(game.level().maze.len(), 16);
        assert_eq!(game.level().guards.len(), 4);
        assert_eq!(game.level().paintings.len(), 5);
        assert!(!game.exit_unlocked());
    }

    #[test]
    fn summaries_match_all_catalog_levels() {
        let game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        let expected = [
            LevelSummary {
                width: 16,
                height: 12,
                guards_count: 2,
                paintings_count: 3,
            },
            LevelSummary {
                width: 24,
                height: 16,
                guards_count: 4,
                paintings_count: 5,
            },
            LevelSummary {
                width: 32,
                height: 20,
                guards_count: 6,
                paintings_count: 7,
            },
        ];

        assert_eq!(game.level_summaries(), &expected);
    }

    #[test]
    fn screen_and_summary_accessors_reflect_state() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();

        assert_eq!(game.level_definitions().len(), 3);
        assert_eq!(
            game.selected_level_summary(),
            LevelSummary {
                width: 16,
                height: 12,
                guards_count: 2,
                paintings_count: 3,
            }
        );

        game.set_screen(GameScreen::LevelSelect);
        assert_eq!(game.screen(), GameScreen::LevelSelect);

        game.select_level(2).unwrap();
        assert_eq!(
            game.selected_level_summary(),
            LevelSummary {
                width: 32,
                height: 20,
                guards_count: 6,
                paintings_count: 7,
            }
        );
    }

    #[test]
    fn game_preloads_assets_and_associates_paintings_with_definition() {
        let game = Game::new(&LEVEL_DEFINITIONS).unwrap();

        assert!(game.asset_manager().loaded_texture_count() >= 5);
        for painting in &game.level().paintings {
            assert!(painting.asset_path.is_some());
            let path = painting.asset_path.unwrap();
            assert!(game.asset_manager().get_texture(path).is_some());
        }
    }

    #[test]
    fn shoot_hits_target_painting_and_advances_hits_up_to_max() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        game.start_selected_level().unwrap();

        // Level 1: target painting is at (1, 3) (or in view), let's place player in front aiming at painting (1, 3)
        let target_pos = game.level().paintings[0].map_position;
        game.player.pos = Vector2::new(target_pos.1 as f32 + 0.5, target_pos.0 as f32 + 1.5);
        game.player.a = -std::f32::consts::FRAC_PI_2; // Aiming north towards row - 1

        let result1 = game.shoot().unwrap();
        assert_eq!(
            result1,
            ShotHitResult::TargetPainting {
                painting_index: 0,
                hits: 1,
                completed: false,
            }
        );
        assert_eq!(game.level().paintings[0].hits, 1);
        assert_eq!(game.level().paintings[0].splatters.len(), 1);

        // Reset cooldown for test
        game.shot_cooldown = 0.0;
        let result2 = game.shoot().unwrap();
        assert_eq!(
            result2,
            ShotHitResult::TargetPainting {
                painting_index: 0,
                hits: 2,
                completed: false,
            }
        );

        game.shot_cooldown = 0.0;
        let result3 = game.shoot().unwrap();
        assert_eq!(
            result3,
            ShotHitResult::TargetPainting {
                painting_index: 0,
                hits: 3,
                completed: true,
            }
        );

        // 4th shot caps at 3 hits
        game.shot_cooldown = 0.0;
        let result4 = game.shoot().unwrap();
        assert_eq!(
            result4,
            ShotHitResult::TargetPainting {
                painting_index: 0,
                hits: 3,
                completed: true,
            }
        );
        assert_eq!(game.level().paintings[0].hits, 3);
    }

    #[test]
    fn shooting_normal_walls_does_not_advance_paintings() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        game.start_selected_level().unwrap();

        // Aim at outer boundary wall (0, 1) from (1.5, 1.5) aiming north
        game.player.pos = Vector2::new(1.5, 1.5);
        game.player.a = -std::f32::consts::FRAC_PI_2;

        let result = game.shoot().unwrap();
        assert!(matches!(result, ShotHitResult::Wall(_)));
        assert_eq!(game.completed_paintings_count(), 0);
    }

    #[test]
    fn completing_all_paintings_unlocks_exit_and_reaching_exit_triggers_success() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        game.start_selected_level().unwrap();
        assert!(!game.exit_unlocked());

        // Complete each painting by setting hits to 3
        for painting in &mut game.level.paintings {
            painting.hits = 3;
            painting.splatters.push(generate_splatter(1, 0.5));
            painting.splatters.push(generate_splatter(2, 0.5));
            painting.splatters.push(generate_splatter(3, 0.5));
        }

        game.update_exit_unlock_state();
        assert!(game.exit_unlocked());

        // Move player to exit
        game.player.pos = game.level.exit;
        assert!(game.check_exit_reached());
    }

    #[test]
    fn shot_cooldown_delays_consecutive_shots() {
        let mut game = Game::new(&LEVEL_DEFINITIONS).unwrap();
        game.start_selected_level().unwrap();

        let first = game.shoot();
        assert!(first.is_some());
        assert!(game.shot_cooldown() > 0.0);

        let second = game.shoot();
        assert!(second.is_none());
    }
}
