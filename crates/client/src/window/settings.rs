//! The game window's text size and fullscreen choice, and whether the Display window is
//! shown. The Display window and Options change these; the game window applies them.

use crate::app::client_app::App;
use lithicrivers_core::config::ConfigManager;

pub const MIN_FONT_PX: u32 = 12;
pub const MAX_FONT_PX: u32 = 48;
pub const FONT_PX_STEP: u32 = 2;
pub const DEFAULT_FONT_PX: u32 = 18;

/// Where the Display panel (text size and fullscreen buttons) is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// A strip along the right edge of the game window
    Docked,
    /// A small separate window
    Floating,
    Hidden,
}

impl DisplayMode {
    pub fn name(self) -> &'static str {
        match self {
            DisplayMode::Docked => "docked",
            DisplayMode::Floating => "floating",
            DisplayMode::Hidden => "hidden",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        [Self::Docked, Self::Floating, Self::Hidden]
            .into_iter()
            .find(|m| m.name() == name)
    }

    /// The next mode in Options order, wrapping around.
    pub fn next(self) -> Self {
        match self {
            DisplayMode::Docked => DisplayMode::Floating,
            DisplayMode::Floating => DisplayMode::Hidden,
            DisplayMode::Hidden => DisplayMode::Docked,
        }
    }

    pub fn previous(self) -> Self {
        self.next().next()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSettings {
    /// Text height in pixels
    pub font_px: u32,
    pub fullscreen: bool,
    /// Where the Display panel is shown
    pub display: DisplayMode,
    /// Whether the on-screen touch controls are wanted (only ever shown on mobile)
    pub touch_controls: bool,
    /// Whether the game is running in its own window (false in terminal and headless runs)
    pub active: bool,
    /// Whether this is a phone or tablet build; desktop builds assume a keyboard and
    /// never show touch controls
    pub mobile: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            font_px: DEFAULT_FONT_PX,
            fullscreen: false,
            display: DisplayMode::Docked,
            touch_controls: true,
            active: false,
            mobile: false,
        }
    }
}

impl WindowSettings {
    pub fn from_config(config: &ConfigManager) -> Self {
        let defaults = Self::default();
        let get = |key| config.get_setting("ui", key);
        Self {
            font_px: get("WINDOW_FONT_PX")
                .and_then(|v| v.as_u64())
                .map(|px| (px as u32).clamp(MIN_FONT_PX, MAX_FONT_PX))
                .unwrap_or(defaults.font_px),
            fullscreen: get("WINDOW_FULLSCREEN")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.fullscreen),
            display: get("WINDOW_DISPLAY")
                .and_then(|v| v.as_str().and_then(DisplayMode::from_name))
                .unwrap_or(defaults.display),
            touch_controls: get("TOUCH_CONTROLS")
                .and_then(|v| v.as_bool())
                .unwrap_or(defaults.touch_controls),
            active: false,
            mobile: false,
        }
    }

    /// Whether the game window draws the on-screen pad and takes touch input.
    pub fn shows_touch_controls(&self) -> bool {
        self.active && self.mobile && self.touch_controls
    }

    pub fn bigger(&mut self) {
        self.font_px = (self.font_px + FONT_PX_STEP).min(MAX_FONT_PX);
    }

    pub fn smaller(&mut self) {
        self.font_px = self.font_px.saturating_sub(FONT_PX_STEP).max(MIN_FONT_PX);
    }
}

/// The screen a phone's first text size leaves the game, in cells.
const MOBILE_GAME_COLS: u32 = 80;
const MOBILE_GAME_ROWS: u32 = 30;

/// The text size a phone or tablet starts with: the largest that still leaves the game
/// 80x30 cells (the touch pad floats over them).
pub fn mobile_font_px((width, height): (u32, u32)) -> u32 {
    let rows = MOBILE_GAME_ROWS;
    (MIN_FONT_PX..=MAX_FONT_PX)
        .rev()
        .step_by(FONT_PX_STEP as usize)
        .find(|&px| {
            let (cell_w, cell_h) = super::fonts::cell_size(px);
            width / cell_w >= MOBILE_GAME_COLS && height / cell_h >= rows
        })
        .unwrap_or(MIN_FONT_PX)
}

/// Whether the player has chosen a window setting (it's in the config file).
pub fn chosen(app: &App, key: &str) -> bool {
    app.core.config_manager.get_setting("ui", key).is_some()
}

/// Settings for a phone or tablet: touch controls, and no Display panel unless the
/// player chose one (the pad needs the space, and Options has Text size). The text size
/// is picked once the window's size is known (`mobile_font_px`).
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub fn use_mobile_defaults(app: &mut App) {
    app.ui.window.mobile = true;
    if !chosen(app, "WINDOW_DISPLAY") {
        app.ui.window.display = DisplayMode::Hidden;
    }
}

/// Change the window settings and save them to the config file (outside headless runs).
pub fn change(app: &mut App, f: impl FnOnce(&mut WindowSettings)) {
    f(&mut app.ui.window);
    let settings = app.ui.window;
    let ui = &mut app.core.config_manager.data.settings["ui"];
    ui["WINDOW_FONT_PX"] = serde_json::json!(settings.font_px);
    ui["WINDOW_FULLSCREEN"] = serde_json::json!(settings.fullscreen);
    ui["WINDOW_DISPLAY"] = serde_json::json!(settings.display.name());
    ui["TOUCH_CONTROLS"] = serde_json::json!(settings.touch_controls);
    if app.persist_config {
        if let Err(e) = app.core.config_manager.save() {
            tracing::warn!(target: "game", "could not save config: {e:#}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_size_stays_within_limits() {
        let mut s = WindowSettings::default();
        for _ in 0..20 {
            s.bigger();
        }
        assert_eq!(s.font_px, MAX_FONT_PX);
        for _ in 0..20 {
            s.smaller();
        }
        assert_eq!(s.font_px, MIN_FONT_PX);
        s.bigger();
        assert_eq!(s.font_px, MIN_FONT_PX + FONT_PX_STEP);
    }

    #[test]
    fn display_modes_cycle_both_ways() {
        let mut mode = DisplayMode::Docked;
        for _ in 0..3 {
            mode = mode.next();
        }
        assert_eq!(mode, DisplayMode::Docked);
        assert_eq!(DisplayMode::Docked.previous(), DisplayMode::Hidden);
    }

    #[test]
    fn settings_round_trip_through_the_config() {
        let mut runner = crate::headless::chain_commands(&["skip-splash"]);
        change(&mut runner.app, |w| {
            w.bigger();
            w.fullscreen = true;
            w.display = DisplayMode::Floating;
            w.touch_controls = false;
        });
        let loaded = WindowSettings::from_config(&runner.app.core.config_manager);
        assert_eq!(loaded.font_px, DEFAULT_FONT_PX + FONT_PX_STEP);
        assert!(loaded.fullscreen);
        assert_eq!(loaded.display, DisplayMode::Floating);
        assert!(!loaded.touch_controls);
        assert!(!loaded.active, "being in a window is never saved");
        assert!(!loaded.mobile, "the platform is never saved");
    }

    #[test]
    fn phones_start_with_text_as_big_as_fits() {
        // Pixel 8, landscape
        let px = mobile_font_px((2400, 1080));
        assert_eq!(px, 36);
        let (w, h) = super::super::fonts::cell_size(px);
        assert!(2400 / w >= MOBILE_GAME_COLS);
        assert!(1080 / h >= MOBILE_GAME_ROWS);
        assert!(
            1080 / (h + FONT_PX_STEP) < MOBILE_GAME_ROWS,
            "the next size won't fit"
        );
        // A big tablet gets the largest size; a tiny screen the smallest
        assert_eq!(mobile_font_px((3840, 2160)), MAX_FONT_PX);
        assert_eq!(mobile_font_px((320, 240)), MIN_FONT_PX);
    }

    #[test]
    fn mobile_hides_the_display_panel_unless_chosen() {
        let mut runner = crate::headless::chain_commands(&["skip-splash"]);
        use_mobile_defaults(&mut runner.app);
        assert!(runner.app.ui.window.mobile);
        assert_eq!(runner.app.ui.window.display, DisplayMode::Hidden);
        assert!(!chosen(&runner.app, "WINDOW_FONT_PX"));
        change(&mut runner.app, |w| w.display = DisplayMode::Docked);
        use_mobile_defaults(&mut runner.app);
        assert_eq!(runner.app.ui.window.display, DisplayMode::Docked);
        assert!(chosen(&runner.app, "WINDOW_FONT_PX"));
    }

    #[test]
    fn touch_controls_only_on_mobile() {
        let mut s = WindowSettings {
            active: true,
            ..WindowSettings::default()
        };
        assert!(!s.shows_touch_controls(), "desktop assumes a keyboard");
        s.mobile = true;
        assert!(s.shows_touch_controls());
        s.touch_controls = false;
        assert!(!s.shows_touch_controls());
    }
}
