//! The client's `App` state, its menu tabs and splash/front-end states.

use crate::app_state::*;
use crate::clock;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SplashState {
    Logo,
    GameTitle,
    /// Title menu screens (see `app::title_menu`)
    Title,
    WorldList,
    WorldActions,
    ConfirmDelete,
    CreateWorld,
    Options,
    BootMessage,
    MainUI,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuTab {
    World,
    Tutorial,
    GlobalMap,
    Quests,
    Body,
    Inventory,
    Crafting,
    Menu,
    Help,
    Credits,
    Quit,
}

pub(crate) const BOOT_MESSAGE_TYPEWRITER_MS: u64 = 100;

impl MenuTab {
    #[allow(dead_code)]
    const COUNT: usize = 10;

    pub(crate) fn next(self, tutorial_visible: bool) -> Self {
        match self {
            MenuTab::World => {
                if tutorial_visible {
                    MenuTab::Tutorial
                } else {
                    MenuTab::GlobalMap
                }
            }
            MenuTab::Tutorial => MenuTab::GlobalMap,
            MenuTab::GlobalMap => MenuTab::Quests,
            MenuTab::Quests => MenuTab::Body,
            MenuTab::Body => MenuTab::Inventory,
            MenuTab::Inventory => MenuTab::Crafting,
            MenuTab::Crafting => MenuTab::Menu,
            MenuTab::Menu => MenuTab::Help,
            MenuTab::Help => MenuTab::Credits,
            MenuTab::Credits => MenuTab::Quit,
            MenuTab::Quit => MenuTab::World,
        }
    }

    pub(crate) fn prev(self, tutorial_visible: bool) -> Self {
        match self {
            MenuTab::World => MenuTab::Quit,
            MenuTab::Tutorial => MenuTab::World,
            MenuTab::GlobalMap => {
                if tutorial_visible {
                    MenuTab::Tutorial
                } else {
                    MenuTab::World
                }
            }
            MenuTab::Body => MenuTab::Quests,
            MenuTab::Quests => MenuTab::GlobalMap,
            MenuTab::Inventory => MenuTab::Body,
            MenuTab::Crafting => MenuTab::Inventory,
            MenuTab::Menu => MenuTab::Crafting,
            MenuTab::Help => MenuTab::Menu,
            MenuTab::Credits => MenuTab::Help,
            MenuTab::Quit => MenuTab::Credits,
        }
    }

    pub(crate) fn as_index(&self, tutorial_visible: bool) -> usize {
        match self {
            MenuTab::World => 0,
            MenuTab::Tutorial => {
                if tutorial_visible {
                    1
                } else {
                    0
                }
            } // Should not be selected if not visible
            MenuTab::GlobalMap => {
                if tutorial_visible {
                    2
                } else {
                    1
                }
            }
            MenuTab::Quests => {
                if tutorial_visible {
                    3
                } else {
                    2
                }
            }
            MenuTab::Body => {
                if tutorial_visible {
                    4
                } else {
                    3
                }
            }
            MenuTab::Inventory => {
                if tutorial_visible {
                    5
                } else {
                    4
                }
            }
            MenuTab::Crafting => {
                if tutorial_visible {
                    6
                } else {
                    5
                }
            }
            MenuTab::Menu => {
                if tutorial_visible {
                    7
                } else {
                    6
                }
            }
            MenuTab::Help => {
                if tutorial_visible {
                    8
                } else {
                    7
                }
            }
            MenuTab::Credits => {
                if tutorial_visible {
                    9
                } else {
                    8
                }
            }
            MenuTab::Quit => {
                if tutorial_visible {
                    10
                } else {
                    9
                }
            }
        }
    }
}

/// Main application state - organized into logical subsystems for better maintainability
pub(crate) struct App {
    /// Core game engine systems
    pub core: CoreState,
    /// UI framework state
    pub ui: UiState,
    /// Audio system
    #[allow(dead_code)]
    pub audio: AudioState,
    /// Logging configuration
    pub logging: LoggingState,
    /// Combat system
    pub combat: CombatUiState,
    /// UI panel states
    pub panels: PanelStates,
    /// Splash screen system
    pub splash: SplashScreenState,
    /// Last time we advanced a combat tick (for 10 ticks/second)
    pub last_combat_tick: Option<std::time::Instant>,
    /// Time source for all time-dependent behavior and rendering
    pub clock: clock::Clock,
    /// Seed for worlds started without the create-world screen (`skip-splash`)
    pub default_seed: u64,
    /// Folder holding one subfolder per world
    pub saves_dir: std::path::PathBuf,
    /// The world being played, if any
    pub current_world: Option<crate::app::world::CurrentWorld>,
    /// Title menu selections and forms
    pub title: crate::app::title_menu::TitleMenuState,
    /// Whether settings changed in Options are written to the config file
    pub persist_config: bool,
}

impl App {
    pub(crate) fn on_tick(&mut self) {
        // Handle splash screen timing
        if let Some(start_time) = self.splash.start_time {
            match self.splash.state {
                SplashState::Logo => {
                    if self.clock.since(start_time) >= Duration::from_secs(3) {
                        self.splash.state = SplashState::GameTitle;
                        self.splash.start_time = Some(self.clock.now());
                    }
                }
                SplashState::GameTitle => {
                    if self.clock.since(start_time) >= Duration::from_secs(3) {
                        self.show_title();
                    }
                }
                SplashState::BootMessage => {
                    // Handle typewriter effect
                    if !self.splash.boot_complete {
                        let elapsed = self.clock.since(self.splash.last_line_time);
                        if elapsed >= Duration::from_millis(BOOT_MESSAGE_TYPEWRITER_MS) {
                            // ~(1000/x) characters per second
                            self.splash.last_line_time = self.clock.now();

                            // Get all text as a single string with newlines
                            let full_text = self.splash.boot_message_lines.join("\n");

                            if self.splash.boot_line_index < full_text.len() {
                                // Move to next line
                                if let Some(next_newline) =
                                    full_text[self.splash.boot_line_index..].find('\n')
                                {
                                    self.splash.boot_line_index += next_newline + 1;
                                } else {
                                    self.splash.boot_line_index = full_text.len();
                                }
                                self.splash.boot_display_text =
                                    full_text[..self.splash.boot_line_index].to_string();
                            } else if !self.splash.boot_complete {
                                self.splash.boot_complete = true;
                                // Set a minimum display time after completion
                                self.splash.start_time = Some(self.clock.now());
                            }
                        }
                    } else if self.clock.since(start_time) >= Duration::from_secs(2) {
                        // 2 seconds after completion, move to main UI
                        self.splash.state = SplashState::MainUI;
                    }
                }
                _ => {}
            }
        }
    }

    /// Apply a game tick's combat transitions to the UI. Every client path that advances the
    /// game must call this, so combat can't start without the combat panel (or outlive it).
    pub(crate) fn apply_tick_result(&mut self, result: &lithicrivers_core::game::GameTickResult) {
        use lithicrivers_core::game::GameTickResult;
        if result.contains(GameTickResult::CombatTriggered) {
            self.open_combat_ui();
        }
        if result.contains(GameTickResult::CombatEnded) {
            self.combat = CombatUiState::None;
        }
    }

    /// Show the combat panel with the first move and enemy selected.
    pub(crate) fn open_combat_ui(&mut self) {
        // Only the World tab draws combat
        self.ui.current_tab = MenuTab::World;
        self.combat = CombatUiState::Active {
            current_move: 0,
            current_enemy: 0,
            enemy_timers: vec![], // Deprecated - using ActionQueue now
            move_scroll_offset: 0,
        };
    }

    pub(crate) fn snap_view_to_player_z(&mut self) {
        if let Some(e) = self.core.game.get_player_entity() {
            if let Ok(pos) = self
                .core
                .game
                .world
                .get::<&lithicrivers_core::components::Position>(e)
            {
                // Snap entire viewport center and Z slice to player
                self.ui.view_x = pos.x;
                self.ui.view_y = pos.y;
                self.ui.view_z = pos.z;
            }
        }
    }

    pub(crate) fn activate_menu(&mut self) {
        match self.ui.current_tab {
            MenuTab::World => {
                // World (already active view)
                self.core.game.res.log("World map active");
            }
            MenuTab::Tutorial => {
                // Tutorial
                self.core
                    .game
                    .res
                    .log("Tutorial panel active (F1 to toggle)");
            }
            MenuTab::GlobalMap => {
                // Global Map
                self.core.game.res.log("Global Map panel active");
            }
            MenuTab::Quests => {
                // Quests
                self.core.game.res.log("Quests panel active");
            }
            MenuTab::Body => {
                // Body
                self.core.game.res.log("Body panel active");
            }
            MenuTab::Inventory => {
                // Inventory (placeholder)
                self.core.game.res.log("Inventory panel (WIP)");
            }
            MenuTab::Menu => {
                // Menu
                self.core
                    .game
                    .res
                    .log("Menu panel active (press S to Save, L to Load)");
            }
            MenuTab::Help => {
                // Help
                self.core.game.res.log("Help panel active");
            }
            MenuTab::Credits => {
                // Credits
                self.core
                    .game
                    .res
                    .log("Credits panel active (Up/Down to scroll)");
            }
            MenuTab::Quit => {
                // Quit
                self.core.game.res.log("Quit requested (menu)");
                tracing::info!(target: "game", "quit_requested input=menu tick={}", self.core.game.res.time.tick);
                self.core.should_quit = true;
            }
            MenuTab::Crafting => {
                // Crafting
                self.core.game.res.log("Crafting panel active");
            }
        }
    }
}
