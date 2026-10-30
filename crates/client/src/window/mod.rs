//! The game's own window (`game-window` spec): the same ratatui screen drawn on the GPU
//! with a bundled font, instead of in a terminal emulator, plus the small Display window
//! with text size and fullscreen buttons.

pub mod display;
pub mod fonts;
pub mod input;
pub mod settings;
pub mod touch;

use crate::app::client_app::App;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui::Terminal;
use ratatui_wgpu::{Builder, Dimensions, Font, Fonts, WgpuBackend};
use settings::DisplayMode;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId, WindowLevel};

/// Time between frames: at most 60 a second. Game time comes from `app.clock`, so the
/// frame rate never changes how fast the game runs.
const FRAME: Duration = Duration::from_millis(16);

/// Colors for text and background the UI leaves at the terminal default.
const DEFAULT_FG: Color = Color::Rgb(204, 204, 204);
const DEFAULT_BG: Color = Color::Rgb(0, 0, 0);

/// The fewest columns the game keeps when the Display panel is docked beside it; a
/// narrower window shows no docked panel.
const MIN_GAME_COLS: u16 = 40;

/// Split a screen into the game's area (top left, so game coordinates are unchanged) and
/// the docked Display strip along the right edge, or `None` if the screen is too narrow.
fn dock_split(area: Rect) -> Option<(Rect, Rect)> {
    let dock_cols = display::COLS + 1; // one column for the separator
    (area.width >= MIN_GAME_COLS + dock_cols).then(|| {
        let game_cols = area.width - dock_cols;
        (
            Rect::new(0, 0, game_cols, area.height),
            Rect::new(game_cols, 0, dock_cols, area.height),
        )
    })
}

/// Where things go in the game window: the game at the top left, the docked Display
/// panel along the right edge, and the touch pad along the bottom of the game's column.
#[derive(Debug, PartialEq, Eq)]
struct Layout {
    game: Rect,
    dock: Option<Rect>,
    pad: Option<Rect>,
}

fn layout(area: Rect, settings: &settings::WindowSettings) -> Layout {
    let (column, dock) = match (settings.display == DisplayMode::Docked)
        .then(|| dock_split(area))
        .flatten()
    {
        Some((column, dock)) => (column, Some(dock)),
        None => (area, None),
    };
    let (game, pad) = match settings
        .shows_touch_controls()
        .then(|| touch::split(column))
        .flatten()
    {
        Some((game, pad)) => (game, Some(pad)),
        None => (column, None),
    };
    Layout { game, dock, pad }
}

/// One window and the terminal drawn into it.
struct Surface {
    window: Arc<Window>,
    terminal: Terminal<WgpuBackend<'static, 'static>>,
    /// The text height its fonts were built with
    font_px: u32,
    /// The pointer's position in pixels
    pointer: (f64, f64),
}

impl Surface {
    fn open(
        event_loop: &ActiveEventLoop,
        attributes: WindowAttributes,
        font_px: u32,
    ) -> Result<Self, String> {
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|e| format!("cannot create a window: {e}"))?,
        );
        let inner = window.inner_size();
        let backend = futures_lite::future::block_on(
            Builder::from_font(Font::new(fonts::MONO).expect("bundled font parses"))
                .with_font_size_px(font_px)
                .with_fg_color(DEFAULT_FG)
                .with_bg_color(DEFAULT_BG)
                .with_width_and_height(Dimensions {
                    width: NonZeroU32::new(inner.width.max(1)).expect("nonzero"),
                    height: NonZeroU32::new(inner.height.max(1)).expect("nonzero"),
                })
                .build_with_target(window.clone()),
        )
        .map_err(|e| format!("cannot start the GPU renderer: {e}"))?;
        let terminal =
            Terminal::new(backend).map_err(|e| format!("cannot start the renderer: {e}"))?;
        Ok(Self {
            window,
            terminal,
            font_px,
            pointer: (0.0, 0.0),
        })
    }

    fn cell(&self) -> (u16, u16) {
        self.cell_at(self.pointer)
    }

    fn cell_at(&self, (x, y): (f64, f64)) -> (u16, u16) {
        input::cell_at(x, y, fonts::cell_size(self.font_px))
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.terminal.backend_mut().resize(size.width, size.height);
        let _ = self.terminal.autoresize();
    }

    /// Rebuild the fonts at a new text height; the grid follows on the next draw.
    fn set_font_px(&mut self, font_px: u32) {
        let fonts = Fonts::new(
            Font::new(fonts::MONO).expect("bundled font parses"),
            font_px,
        );
        self.terminal.backend_mut().update_fonts(fonts);
        self.font_px = font_px;
        let _ = self.terminal.autoresize();
    }
}

/// Run the game in its own window until it quits. Fails without having shown anything
/// if no window or GPU renderer can be created, so the caller can fall back.
pub fn run(app: &mut App, event_loop: EventLoop<()>) -> Result<(), String> {
    app.ui.window.active = true;
    let mut handler = WindowApp {
        app,
        game: None,
        display: None,
        game_screen: None,
        dock_panel: None,
        pad_buttons: Vec::new(),
        touches: touch::Touches::default(),
        fullscreen: false,
        modifiers: ModifiersState::empty(),
        pointer_cell: None,
        error: None,
    };
    let result = event_loop.run_app(&mut handler);
    handler.app.ui.window.active = false;
    result.map_err(|e| format!("window event loop failed: {e}"))?;
    match handler.error {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

struct WindowApp<'a> {
    app: &'a mut App,
    game: Option<Surface>,
    /// The floating Display window
    display: Option<Surface>,
    /// In-memory screen the game draws into when the Display panel is docked or the
    /// touch pad is shown, copied into the top left of the game window
    game_screen: Option<Terminal<TestBackend>>,
    /// Where the docked panel's buttons were drawn last frame, for clicks
    dock_panel: Option<Rect>,
    /// The touch pad's buttons as drawn last frame (empty when it isn't shown)
    pad_buttons: Vec<touch::PadButton>,
    /// Fingers on the screen outside the pad (mobile only)
    touches: touch::Touches,
    /// Whether the game window is fullscreen now
    fullscreen: bool,
    modifiers: ModifiersState,
    /// The game window cell the pointer was last reported on, so moves are sent once
    /// per cell
    pointer_cell: Option<(u16, u16)>,
    /// Why the game window couldn't be created, if it couldn't
    error: Option<String>,
}

fn fullscreen_mode(on: bool) -> Option<Fullscreen> {
    on.then_some(Fullscreen::Borderless(None))
}

impl WindowApp<'_> {
    fn open_game(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let settings = self.app.ui.window;
        let (cell_w, cell_h) = fonts::cell_size(settings.font_px);
        // Room for 120x36 cells to start with
        let attributes = Window::default_attributes()
            .with_title("LithicRivers")
            .with_inner_size(PhysicalSize::new(cell_w * 120, cell_h * 36))
            .with_fullscreen(fullscreen_mode(settings.fullscreen));
        self.game = Some(Surface::open(event_loop, attributes, settings.font_px)?);
        self.fullscreen = settings.fullscreen;
        Ok(())
    }

    fn open_display(&mut self, event_loop: &ActiveEventLoop) {
        let (cell_w, cell_h) = fonts::cell_size(display::FONT_PX);
        let attributes = Window::default_attributes()
            .with_title("LithicRivers Display")
            .with_inner_size(PhysicalSize::new(
                cell_w * display::COLS as u32,
                cell_h * display::ROWS as u32,
            ))
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop);
        match Surface::open(event_loop, attributes, display::FONT_PX) {
            Ok(surface) => self.display = Some(surface),
            Err(e) => {
                // The game works without it; Options can change the same settings
                tracing::warn!(target: "game", "display window unavailable: {e}");
                self.app.ui.window.display = DisplayMode::Hidden;
            }
        }
    }

    /// Bring both windows in line with the settings the Display window or Options changed.
    fn apply_settings(&mut self, event_loop: &ActiveEventLoop) {
        let settings = self.app.ui.window;
        if let Some(game) = self.game.as_mut() {
            if game.font_px != settings.font_px {
                game.set_font_px(settings.font_px);
            }
            if self.fullscreen != settings.fullscreen {
                game.window
                    .set_fullscreen(fullscreen_mode(settings.fullscreen));
                self.fullscreen = settings.fullscreen;
            }
        }
        match (
            settings.display == DisplayMode::Floating,
            self.display.is_some(),
        ) {
            (true, false) => self.open_display(event_loop),
            (false, true) => self.display = None,
            _ => {}
        }
    }

    /// A game window cell inside the docked panel, as a cell relative to the panel.
    fn in_dock(&self, (col, row): (u16, u16)) -> Option<(u16, u16)> {
        let panel = self.dock_panel?;
        let dock_left = panel.x.saturating_sub(1); // include the separator column
        (col >= dock_left).then(|| (col.saturating_sub(panel.x), row.saturating_sub(panel.y)))
    }

    fn shift(&self) -> bool {
        self.modifiers.shift_key()
    }

    fn game_event(&mut self, event: WindowEvent) {
        let Some(game) = self.game.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => self.app.core.should_quit = true,
            WindowEvent::Resized(size) => game.resize(size),
            WindowEvent::RedrawRequested => {
                let settings = self.app.ui.window;
                let area = game.terminal.size().map(Rect::from).unwrap_or_default();
                let Layout {
                    game: game_area,
                    dock,
                    pad,
                } = layout(area, &settings);
                let app = &mut *self.app;
                let result = if game_area == area {
                    self.dock_panel = None;
                    self.pad_buttons.clear();
                    game.terminal.draw(|f| crate::ui(f, app)).map(|_| ())
                } else {
                    let screen = self.game_screen.get_or_insert_with(|| {
                        Terminal::new(TestBackend::new(game_area.width, game_area.height))
                            .expect("in-memory screen")
                    });
                    if screen.size().ok() != Some(game_area.as_size()) {
                        screen
                            .backend_mut()
                            .resize(game_area.width, game_area.height);
                        let _ = screen.autoresize();
                    }
                    let _ = screen.draw(|f| crate::ui(f, app));
                    let drawn = screen.backend().buffer();
                    let separator = Block::default().borders(Borders::LEFT);
                    self.dock_panel = dock.map(|dock| separator.inner(dock));
                    self.pad_buttons = pad.map(touch::layout).unwrap_or_default();
                    let (dock_panel, pad_buttons) = (self.dock_panel, &self.pad_buttons);
                    game.terminal
                        .draw(|f| {
                            let buf = f.buffer_mut();
                            for y in 0..game_area.height {
                                for x in 0..game_area.width {
                                    buf[(x, y)] = drawn[(x, y)].clone();
                                }
                            }
                            if let (Some(dock), Some(panel)) = (dock, dock_panel) {
                                separator.render(dock, buf);
                                display::render(panel, buf, &settings);
                            }
                            touch::render(pad_buttons, buf, app);
                        })
                        .map(|_| ())
                };
                if let Err(e) = result {
                    tracing::warn!(target: "game", "window draw failed: {e}");
                }
            }
            WindowEvent::Touch(touch) if self.app.ui.window.mobile => {
                let cell = game.cell_at((touch.location.x, touch.location.y));
                match touch.phase {
                    TouchPhase::Started => {
                        if let Some((col, row)) = self.in_dock(cell) {
                            if let Some(button) = display::button_at(col, row, &self.app.ui.window)
                            {
                                settings::change(self.app, |w| display::press(button, w));
                            }
                        } else if let Some(press) = touch::button_at(&self.pad_buttons, cell) {
                            if let Some(code) = touch::key_code(self.app, press) {
                                if let Err(e) = crate::handle_input(self.app, code) {
                                    tracing::warn!(target: "game", "input error: {e}");
                                }
                            }
                        } else {
                            self.touches.started(touch.id, cell);
                        }
                    }
                    TouchPhase::Moved => {
                        for scroll in self.touches.moved(touch.id, cell) {
                            crate::handle_mouse_event(self.app, scroll);
                        }
                    }
                    TouchPhase::Ended => {
                        for event in self.touches.ended(touch.id) {
                            crate::handle_mouse_event(self.app, event);
                        }
                    }
                    TouchPhase::Cancelled => self.touches.cancelled(touch.id),
                }
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let text = event.text.as_ref().map(|t| t.as_str());
                let shift = self.modifiers.shift_key();
                if let Some(code) = input::key_code(&event.logical_key, text, shift) {
                    if let Err(e) = crate::handle_input(self.app, code) {
                        tracing::warn!(target: "game", "input error: {e}");
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                game.pointer = (position.x, position.y);
                let cell = game.cell();
                if self.in_dock(cell).is_some() {
                    return;
                }
                if self.pointer_cell != Some(cell) {
                    self.pointer_cell = Some(cell);
                    let shift = self.shift();
                    crate::handle_mouse_event(self.app, input::moved(cell, shift));
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                let cell = game.cell();
                if let Some((col, row)) = self.in_dock(cell) {
                    let settings = self.app.ui.window;
                    if button == winit::event::MouseButton::Left {
                        if let Some(button) = display::button_at(col, row, &settings) {
                            settings::change(self.app, |w| display::press(button, w));
                        }
                    }
                    return;
                }
                if let Some(click) = input::click(button, cell, self.shift()) {
                    crate::handle_mouse_event(self.app, click);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => {
                        (p.y / fonts::cell_size(game.font_px).1 as f64) as f32
                    }
                };
                let cell = game.cell();
                if self.in_dock(cell).is_some() {
                    return;
                }
                for scroll in input::scrolls(lines, cell, self.shift()) {
                    crate::handle_mouse_event(self.app, scroll);
                }
            }
            _ => {}
        }
    }

    fn display_event(&mut self, event: WindowEvent) {
        let Some(display) = self.display.as_mut() else {
            return;
        };
        match event {
            WindowEvent::CloseRequested => {
                settings::change(self.app, |w| w.display = DisplayMode::Hidden);
            }
            WindowEvent::Resized(size) => display.resize(size),
            WindowEvent::RedrawRequested => {
                let settings = self.app.ui.window;
                if let Err(e) = display.terminal.draw(|f| display::draw(f, &settings)) {
                    tracing::warn!(target: "game", "display window draw failed: {e}");
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                display.pointer = (position.x, position.y);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                let (col, row) = display.cell();
                if let Some(button) = display::button_at(col, row, &self.app.ui.window) {
                    settings::change(self.app, |w| display::press(button, w));
                }
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for WindowApp<'_> {
    /// The app went to the background (on Android, it may be closed there without
    /// warning, and its surfaces are destroyed): save, and drop the surfaces. `resumed`
    /// opens them again.
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.app.save_on_suspend();
        self.game = None;
        self.display = None;
        self.dock_panel = None;
        self.pad_buttons.clear();
        self.touches = touch::Touches::default();
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.game.is_some() {
            return;
        }
        if let Err(e) = self.open_game(event_loop) {
            self.error = Some(e);
            event_loop.exit();
            return;
        }
        if self.app.ui.window.display == DisplayMode::Floating {
            self.open_display(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let WindowEvent::ModifiersChanged(modifiers) = &event {
            self.modifiers = modifiers.state();
        }
        if self.display.as_ref().is_some_and(|d| d.window.id() == id) {
            self.display_event(event);
        } else {
            self.game_event(event);
        }
        if self.app.core.should_quit {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        crate::step(self.app);
        if self.app.core.should_quit {
            event_loop.exit();
            return;
        }
        self.apply_settings(event_loop);
        for surface in [&self.game, &self.display].into_iter().flatten() {
            surface.window.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + FRAME));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docking_keeps_the_game_at_the_top_left() {
        let (game, dock) = dock_split(Rect::new(0, 0, 120, 36)).unwrap();
        assert_eq!(game, Rect::new(0, 0, 120 - display::COLS - 1, 36));
        assert_eq!(dock.x, game.width);
        assert_eq!(dock.width, display::COLS + 1);
        // Too narrow: no docked panel, the game keeps the whole window
        assert!(dock_split(Rect::new(0, 0, MIN_GAME_COLS + display::COLS, 36)).is_none());
    }

    #[test]
    fn the_touch_pad_only_shows_on_mobile() {
        let area = Rect::new(0, 0, 120, 40);
        let mut settings = settings::WindowSettings {
            active: true,
            display: DisplayMode::Hidden,
            ..Default::default()
        };
        assert_eq!(
            layout(area, &settings),
            Layout {
                game: area,
                dock: None,
                pad: None
            },
            "desktop: the game keeps the whole window"
        );
        settings.mobile = true;
        let shown = layout(area, &settings);
        assert_eq!(shown.game, Rect::new(0, 0, 120, 40 - touch::PAD_ROWS));
        assert_eq!(shown.pad.unwrap().y, 40 - touch::PAD_ROWS);
        // Docked too: the pad spans the game's column, not the panel
        settings.display = DisplayMode::Docked;
        let both = layout(area, &settings);
        assert_eq!(both.pad.unwrap().width, both.dock.unwrap().x);
        assert_eq!(both.game.width, both.dock.unwrap().x);
        // Hidden in Options: the game takes the space back
        settings.touch_controls = false;
        assert_eq!(layout(area, &settings).pad, None);
    }
}
