use crate::sprite_constants::body_art;
use crate::App;
use lithicrivers_core::model::body::{Body, BodyPartState, BodyPartType};
use ratatui::{
    style::Color,
    style::Style,
    text::{Line, Span},
};

/// Parses a `#RRGGBB` string into a ratatui `Color`, or returns `None` if invalid.
pub fn parse_hex_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Some(Color::Rgb(r, g, b));
            }
        }
    }
    None
}

/// Renders 12x8 ASCII art for the tile at the given world position.
///
/// Loads the tile sprite from the "tiles" category, parses its `#RRGGBB` color,
/// and appends up to 8 styled `Line`s to `out`. If the sprite has no 12x8 art,
/// nothing is pushed.
///
/// # Arguments
/// - `app` — Game/app context used to read world tiles and load sprites.
/// - `pos` — World position (x, y, z) to sample the tile from.
/// - `out` — Target buffer to receive the rendered lines.
///
/// # Returns
/// `true` if any lines were appended; `false` otherwise.
///
/// # Panics
/// Does not panic; invalid colors are ignored and produce unstyled output.
pub fn block_art_12x8_lines_for_position(
    app: &mut App,
    pos: lithicrivers_core::components::Position,
    out: &mut Vec<Line<'static>>,
) -> bool {
    use lithicrivers_core::tiles::TileKind;

    let kind = app.game.res.world.get_tile_cached(pos.x, pos.y, pos.z);

    let sd = app.sprite_loader.load_sprite(kind.sprite_key(), "tiles");
    let color = parse_hex_color(&sd.color);

    if let Some(block) = sd.art12x8_sprites.first() {
        for row in block.split('\n') {
            let span = match color {
                Some(c) => Span::styled(row.to_string(), Style::default().fg(c)),
                None => Span::raw(row.to_string()),
            };
            out.push(Line::from(span));
        }
        return true;
    }
    false
}

/// Renders 12x8 ASCII art for the entity at the given world position.
///
/// First, queries for any entity at the position with a `SpriteRef`. If found,
/// loads the sprite and appends up to 8 styled `Line`s to `out`. If no entity
/// has a sprite, falls back to a generic entity sprite.
///
/// # Arguments
/// - `app` — Game/app context used to read world entities and load sprites.
/// - `pos` — World position (x, y, z) to sample the entity from.
/// - `out` — Target buffer to receive the rendered lines.
///
/// # Returns
/// `true` if any lines were appended; `false` otherwise.
///
/// # Panics
/// Does not panic; invalid colors are ignored and produce unstyled output.
pub fn entity_art_12x8_lines_for_position(
    app: &mut App,
    pos: lithicrivers_core::components::Position,
    out: &mut Vec<Line<'static>>,
) -> bool {
    // First, try any entity at this position with a SpriteRef
    for (_e, (e_pos, maybe_sr)) in app
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::SpriteRef>,
        )>()
        .iter()
    {
        if *e_pos != pos {
            continue;
        }
        if let Some(sr) = maybe_sr {
            let sd = app.sprite_loader.load_by_spriteref(sr);
            let color = parse_hex_color(&sd.color);
            if let Some(block) = sd.art12x8_sprites.first() {
                for row in block.split('\n') {
                    let span = match color {
                        Some(c) => Span::styled(row.to_string(), Style::default().fg(c)),
                        None => Span::raw(row.to_string()),
                    };
                    out.push(Line::from(span));
                }
                return true;
            }
        }
    }

    // Fallback: try EntityKind -> sprite mapping
    for (_e, (e_pos, maybe_kind)) in app
        .game
        .world
        .query::<(
            &lithicrivers_core::components::Position,
            Option<&lithicrivers_core::components::EntityKind>,
        )>()
        .iter()
    {
        if *e_pos != pos {
            continue;
        }
        let (category, name) = match maybe_kind.copied() {
            Some(lithicrivers_core::components::EntityKind::Player) => ("entities", "player"),
            Some(lithicrivers_core::components::EntityKind::Sheep) => ("entities", "sheep"),
            // Add specific mappings as you introduce more kinds
            _ => ("entities", "entity_generic"),
        };
        let sd = app.sprite_loader.load_sprite(name, category);
        let color = parse_hex_color(&sd.color);
        if let Some(block) = sd.art12x8_sprites.first() {
            for row in block.split('\n') {
                let span = match color {
                    Some(c) => Span::styled(row.to_string(), Style::default().fg(c)),
                    None => Span::raw(row.to_string()),
                };
                out.push(Line::from(span));
            }
            return true;
        }
    }
    false
}

/// Returns a vector of empty 12x8 lines, each containing 12 question marks.
pub fn empty_art_12x8_lines_for_position() -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    for _ in 0..8 {
        let mut line: String = String::new();
        for _ in 0..12 {
            line.push('?');
        }
        lines.push(Line::from(line));
    }
    return lines;
}

/// Renders a 13x13 ASCII art schematic of the given body.
pub fn build_body_ascii(body: &Body) -> Vec<Line<'static>> {
    // Simple 13x13 schematic using markers for parts:
    // H head, X torso, A/a arms, L/l legs, space background
    let art = body_art();

    // Helper to get state color by marker
    let color_for = |marker: char| -> Color {
        let (part_type, present) = match marker {
            'H' => (BodyPartType::Head, true),
            'X' => (BodyPartType::Torso, true),
            'A' => (BodyPartType::LeftArm, true),
            'a' => (BodyPartType::RightArm, true),
            'L' => (BodyPartType::LeftLeg, true),
            'l' => (BodyPartType::RightLeg, true),
            _ => (BodyPartType::Head, false),
        };
        if !present {
            return Color::DarkGray;
        }
        let state = body
            .parts
            .get(&part_type)
            .map(|p| p.state)
            .unwrap_or(BodyPartState::Missing);
        match state {
            BodyPartState::Missing => Color::Black,
            BodyPartState::Damaged => Color::Red,
            BodyPartState::Functional => Color::Green,
            BodyPartState::Enhanced => Color::Cyan,
        }
    };

    let mut out: Vec<Line> = Vec::new();
    for row in art {
        let mut spans: Vec<Span> = Vec::new();
        for ch in row.chars() {
            if ch == ' ' {
                spans.push(Span::raw(" "));
            } else {
                let color = color_for(ch);
                spans.push(Span::styled("█", Style::default().fg(color)));
            }
        }
        out.push(Line::from(spans));
    }
    out
}
