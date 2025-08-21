use crate::sprite_constants::body_art;
use crate::App;
use lithicrivers_core::model::body::{Body, BodyPartState, BodyPartType};
use ratatui::{
    style::Color,
    style::Style,
    text::{Line, Span},
};

// Helper: parse #RRGGBB strings into ratatui Color
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

pub fn block_art_12x8_lines_for_position(app: &mut App, pos: lithicrivers_core::components::Position, out: &mut Vec<Line<'static>>) ->bool{
    
    //todo implement this, for now static todo
    for _ in 0..8 {
        let mut line: String = String::new();
        for _ in 0..3 {
            line.push('T');
            line.push('O');
            line.push('D');
            line.push('O');
        }
        out.push(Line::from(line));
    }
    return true;
}

// Build 12x8 art lines for entity or dropped item at this position. Returns true if any art was added.
pub fn art_12x8_lines_for_position(
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

    // If there is any entity at this position but without SpriteRef, render a generic entity sprite
    let mut any_entity = false;
    for (_e, (e_pos,)) in app
        .game
        .world
        .query::<(&lithicrivers_core::components::Position,)>()
        .iter()
    {
        if *e_pos == pos {
            any_entity = true;
            break;
        }
    }
    if any_entity {
        let sd = app.sprite_loader.load_sprite("entity_generic", "entities");
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
