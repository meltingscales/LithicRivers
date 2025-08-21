use crate::sprite_constants::body_art;
use lithicrivers_core::model::body::{Body, BodyPartState, BodyPartType};
use ratatui::{
    style::Color,
    style::Style,
    text::{Line, Span},
};
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
