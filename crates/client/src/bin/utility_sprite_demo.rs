/*
 * this is a demo that loads every tile, item, entity, and structure
 * and lets you cycle through them. it should display their colors, 1x1/2x2/3x3, and 12x8 art.
 * for entities, it should also display their emotions.
 * for structures, it should also display their names and descriptions, their spawn chances,
 * and let you cycle up/down/left/right through them, as well as zoom in or out.
 */

use std::error::Error;
use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use lithicrivers_core::{
    components::{EntityKind, ItemKind, NPCMood, SpriteRef},
    structure::StructureDefinition,
    tiles::TileKind,
};
use strum::IntoEnumIterator;

// Import sprite_loader as a sibling module (like main.rs does)
#[path = "../sprite_loader.rs"]
mod sprite_loader;

use sprite_loader::{parse_color_string, sprite_block_for_tile, Scale, SpriteLoader};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Copy)]
enum SpriteCategory {
    Tiles,
    Items,
    Entities,
    Structures,
}

impl SpriteCategory {
    fn all() -> &'static [SpriteCategory] {
        &[
            SpriteCategory::Tiles,
            SpriteCategory::Items,
            SpriteCategory::Entities,
            SpriteCategory::Structures,
        ]
    }

    fn name(&self) -> &'static str {
        match self {
            SpriteCategory::Tiles => "Tiles",
            SpriteCategory::Items => "Items",
            SpriteCategory::Entities => "Entities",
            SpriteCategory::Structures => "Structures",
        }
    }
}

struct App {
    sprite_loader: SpriteLoader,
    category: usize,
    index: usize,
    scale: Scale,
    mood_index: usize,
    zoom_level: f32,
    structures: Vec<StructureDefinition>,
    item_kinds: Vec<ItemKind>,
    entity_kinds: Vec<EntityKind>,
}

impl App {
    fn new() -> Self {
        let mut sprite_loader = SpriteLoader::new(None);
        sprite_loader.preload_all();

        let structures = Self::discover_structures();
        let item_kinds = Self::discover_item_kinds();
        let entity_kinds = Self::discover_entity_kinds();

        Self {
            sprite_loader,
            category: 0,
            index: 0,
            scale: Scale::Small,
            mood_index: 0,
            zoom_level: 1.0,
            structures,
            item_kinds,
            entity_kinds,
        }
    }

    fn discover_structures() -> Vec<StructureDefinition> {
        let structure_names = [
            "small_ship.lrstructure",
            "small_temple.lrstructure",
            "giant_corpse.lrstructure",
            "starter_ship.lrstructure",
            "first-quest-sapiencorp.lrstructure",
        ];

        let mut structures = Vec::new();
        for name in &structure_names {
            match std::panic::catch_unwind(|| StructureDefinition::load_from_embedded(name)) {
                Ok(structure) => structures.push(structure),
                Err(_) => {
                    eprintln!("Warning: Could not load structure {}", name);
                }
            }
        }
        structures
    }

    fn discover_item_kinds() -> Vec<ItemKind> {
        // Use strum to automatically iterate over all ItemKind variants
        ItemKind::iter().collect()
    }

    fn discover_entity_kinds() -> Vec<EntityKind> {
        // Use strum to automatically iterate over all EntityKind variants
        EntityKind::iter().collect()
    }

    fn current_category(&self) -> SpriteCategory {
        SpriteCategory::all()[self.category]
    }

    fn get_current_count(&self) -> usize {
        match self.current_category() {
            SpriteCategory::Tiles => TileKind::all_variants().len(),
            SpriteCategory::Items => self.item_kinds.len(),
            SpriteCategory::Entities => self.entity_kinds.len(),
            SpriteCategory::Structures => self.structures.len(),
        }
    }

    fn next_category(&mut self) {
        self.category = (self.category + 1) % SpriteCategory::all().len();
        self.index = 0;
        self.mood_index = 0;
    }

    fn prev_category(&mut self) {
        if self.category == 0 {
            self.category = SpriteCategory::all().len() - 1;
        } else {
            self.category -= 1;
        }
        self.index = 0;
        self.mood_index = 0;
    }

    fn next_item(&mut self) {
        let count = self.get_current_count();
        if count > 0 {
            self.index = (self.index + 1) % count;
        }
    }

    fn prev_item(&mut self) {
        let count = self.get_current_count();
        if count > 0 {
            if self.index == 0 {
                self.index = count - 1;
            } else {
                self.index -= 1;
            }
        }
    }

    fn cycle_scale(&mut self) {
        self.scale = match self.scale {
            Scale::Small => Scale::Medium,
            Scale::Medium => Scale::Large,
            Scale::Large => Scale::Small,
        };
    }

    fn next_mood(&mut self) {
        self.mood_index = (self.mood_index + 1) % 4;
    }

    fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level * 1.2).min(5.0);
    }

    fn zoom_out(&mut self) {
        self.zoom_level = (self.zoom_level / 1.2).max(0.2);
    }

    fn get_current_mood(&self) -> NPCMood {
        match self.mood_index {
            0 => NPCMood::Happy,
            1 => NPCMood::Sad,
            2 => NPCMood::Neutral,
            _ => NPCMood::Weird,
        }
    }

    fn get_current_sprite_info(&mut self) -> Option<(String, Color, String, String, String)> {
        match self.current_category() {
            SpriteCategory::Tiles => {
                let tiles = TileKind::all_variants();
                if self.index < tiles.len() {
                    let tile = tiles[self.index];
                    if let Some((sprite, color)) =
                        sprite_block_for_tile(&mut self.sprite_loader, tile, self.scale)
                    {
                        let name = format!("{:?}", tile);
                        let sprite_key = tile.sprite_key();
                        let passable = if tile.is_passable() { "Yes" } else { "No" };
                        let info = format!("Sprite: {} | Passable: {}", sprite_key, passable);
                        return Some((sprite, color, name, info, String::new()));
                    }
                }
            }
            SpriteCategory::Items => {
                if self.index < self.item_kinds.len() {
                    let item = self.item_kinds[self.index];
                    let sprite_name = lithicrivers_core::components::itemkind_sprite_name(item);
                    let sprite_data = self.sprite_loader.load_sprite(sprite_name, "items");

                    let sprite = match self.scale {
                        Scale::Small => sprite_data
                            .sprites
                            .get(0)
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                        Scale::Medium => sprite_data
                            .sprites
                            .get(1)
                            .or(sprite_data.sprites.get(0))
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                        Scale::Large => sprite_data
                            .sprites
                            .get(2)
                            .or(sprite_data.sprites.get(1))
                            .or(sprite_data.sprites.get(0))
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                    };

                    let color = parse_color_string(&sprite_data.color).unwrap_or(Color::White);
                    let name = lithicrivers_core::components::itemkind_name(item).to_string();
                    let info = format!(
                        "Sprite: {} | Description: {}",
                        sprite_name, sprite_data.description
                    );
                    let art12x8 = sprite_data
                        .art12x8_sprites
                        .get(0)
                        .cloned()
                        .unwrap_or_else(|| "No art".to_string());

                    return Some((sprite, color, name, info, art12x8));
                }
            }
            SpriteCategory::Entities => {
                if self.index < self.entity_kinds.len() {
                    let entity = self.entity_kinds[self.index];
                    let current_mood = self.get_current_mood();
                    let (category, sprite_name) =
                        self.sprite_loader.sprite_path_for_entitykind(entity);
                    let sprite_data = self.sprite_loader.load_sprite(&sprite_name, &category);

                    let sprite = match self.scale {
                        Scale::Small => sprite_data
                            .sprites
                            .get(0)
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                        Scale::Medium => sprite_data
                            .sprites
                            .get(1)
                            .or(sprite_data.sprites.get(0))
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                        Scale::Large => sprite_data
                            .sprites
                            .get(2)
                            .or(sprite_data.sprites.get(1))
                            .or(sprite_data.sprites.get(0))
                            .cloned()
                            .unwrap_or_else(|| "?".to_string()),
                    };

                    let color = parse_color_string(&sprite_data.color).unwrap_or(Color::White);
                    let name = format!("{:?}", entity);
                    let mood_str = format!("{:?}", current_mood);
                    let info = format!(
                        "Sprite: {} | Mood: {} | Description: {}",
                        sprite_name, mood_str, sprite_data.description
                    );

                    let sprite_ref = SpriteRef::new(&category, &sprite_name);
                    let art12x8 = self
                        .sprite_loader
                        .get_mood_portrait(&sprite_ref, current_mood)
                        .unwrap_or_else(|| "No mood art".to_string());

                    return Some((sprite, color, name, info, art12x8));
                }
            }
            SpriteCategory::Structures => {
                if self.index < self.structures.len() {
                    let structure = &self.structures[self.index];
                    let layer_display = if !structure.layers.is_empty() {
                        structure.layers[0].clone()
                    } else {
                        "Empty structure".to_string()
                    };

                    let name = structure.name.clone();
                    let info = format!(
                        "Layers: {} | Biomes: {} | Spawn Chance: {:.1}%",
                        structure.layers.len(),
                        structure.gen_biomes,
                        structure.gen_chance * 100.0
                    );

                    return Some((layer_display, Color::Gray, name, info, String::new()));
                }
            }
        }
        None
    }
}

fn main() -> Result<()> {
    // Set up panic handler to restore terminal (like main.rs does)
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen);
        eprintln!("\n\nPanic: {info}");
    }));

    let mut terminal = setup_terminal()?;
    let res = run_app(&mut terminal);
    restore_terminal(terminal)?;
    if let Err(err) = res {
        eprintln!("{err:?}");
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Tab => app.next_category(),
                    KeyCode::BackTab => app.prev_category(),
                    KeyCode::Right | KeyCode::Char('l') => app.next_item(),
                    KeyCode::Left | KeyCode::Char('h') => app.prev_item(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                    KeyCode::Char('s') => app.cycle_scale(),
                    KeyCode::Char('m') => app.next_mood(),
                    KeyCode::Char('+') | KeyCode::Char('=') => app.zoom_in(),
                    KeyCode::Char('-') => app.zoom_out(),
                    _ => {}
                }
            }
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Main border
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Sprite Demo - All Tiles, Items, Entities & Structures ")
        .title_alignment(Alignment::Center);
    let inner = block.inner(size);
    f.render_widget(block, size);

    // Layout: info area + sprite display + art + controls
    let chunks = Layout::vertical([
        Constraint::Length(3), // Info
        Constraint::Min(8),    // Main content
        Constraint::Length(4), // Controls
    ])
    .split(inner);

    // Category and item info
    let category_name = app.current_category().name();
    let count = app.get_current_count();
    let info_text = format!(
        "Category: {} ({}/{}) | Scale: {}x{} | Zoom: {:.1}x",
        category_name,
        app.index + 1,
        count,
        app.scale.as_u32(),
        app.scale.as_u32(),
        app.zoom_level
    );

    let info_block = Paragraph::new(info_text)
        .block(Block::default().borders(Borders::BOTTOM))
        .alignment(Alignment::Center);
    f.render_widget(info_block, chunks[0]);

    // Main content area
    let content_chunks = Layout::horizontal([
        Constraint::Percentage(40), // Sprite display
        Constraint::Percentage(60), // Art and details
    ])
    .split(chunks[1]);

    if let Some((sprite, color, name, info, art12x8)) = app.get_current_sprite_info() {
        // Sprite display
        let sprite_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", name));
        let sprite_inner = sprite_block.inner(content_chunks[0]);
        f.render_widget(sprite_block, content_chunks[0]);

        let sprite_para = Paragraph::new(sprite.clone())
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .block(Block::default());

        // Center the sprite in the available space
        let sprite_rect = center_rect_for_content(&sprite, sprite_inner);
        f.render_widget(sprite_para, sprite_rect);

        // Art and details area
        let art_chunks = Layout::vertical([
            Constraint::Length(3), // Info
            Constraint::Min(8),    // Art
        ])
        .split(content_chunks[1]);

        // Item details
        let details_para = Paragraph::new(info)
            .block(Block::default().borders(Borders::ALL).title(" Details "))
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(details_para, art_chunks[0]);

        // 12x8 Art display
        if !art12x8.is_empty() && art12x8 != "No art" && art12x8 != "No mood art" {
            let art_para = Paragraph::new(art12x8)
                .style(Style::default().fg(color))
                .block(Block::default().borders(Borders::ALL).title(" 12x8 Art "))
                .alignment(Alignment::Center);
            f.render_widget(art_para, art_chunks[1]);
        } else {
            let no_art_para = Paragraph::new("No artwork available")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title(" 12x8 Art "))
                .alignment(Alignment::Center);
            f.render_widget(no_art_para, art_chunks[1]);
        }
    }

    // Controls
    let controls = vec![
        "Tab/Shift+Tab: Switch category",
        "←/→ or h/l: Navigate items",
        "↑/↓ or k/j: Navigate items",
        "s: Cycle scale (1x1/2x2/3x3)",
        "m: Cycle mood (entities)",
        "+/-: Zoom in/out",
        "q: Quit",
    ];

    let controls_text = controls.join(" | ");
    let controls_para = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::TOP))
        .wrap(ratatui::widgets::Wrap { trim: true })
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(controls_para, chunks[2]);
}

fn center_rect_for_content(content: &str, area: Rect) -> Rect {
    let lines: Vec<&str> = content.lines().collect();
    let height = lines.len() as u16;
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;

    let centered_width = width.min(area.width);
    let centered_height = height.min(area.height);

    let x = area.x + (area.width.saturating_sub(centered_width)) / 2;
    let y = area.y + (area.height.saturating_sub(centered_height)) / 2;

    Rect {
        x,
        y,
        width: centered_width,
        height: centered_height,
    }
}
