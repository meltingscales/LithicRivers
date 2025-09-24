use crate::App;
use lithicrivers_core::resources::QuestMarkerType;
use lithicrivers_core::world::{CHUNK_SIZE, CHUNK_SIZE_Z};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the global map panel
pub fn render_global_map_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(" Global Map ")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Get player position for centering the map
    let (player_chunk_x, player_chunk_y, player_chunk_z) =
        if let Some(player_pos) = app.core.game.get_player_position() {
            (
                player_pos.x.div_euclid(CHUNK_SIZE),
                player_pos.y.div_euclid(CHUNK_SIZE),
                player_pos.z.div_euclid(CHUNK_SIZE_Z),
            )
        } else {
            (0, 0, 0)
        };

    // Calculate map dimensions based on the available area
    let map_width = inner_area.width.saturating_sub(2) as i64;
    let map_height = inner_area.height.saturating_sub(6) as i64; // Leave space for legend

    // Calculate the range of chunks to show
    let half_width = map_width / 2;
    let half_height = map_height / 2;

    let min_chunk_x = player_chunk_x - half_width;
    let max_chunk_x = player_chunk_x + half_width;
    let min_chunk_y = player_chunk_y - half_height;
    let max_chunk_y = player_chunk_y + half_height;

    // Build the map display
    let mut map_lines = Vec::new();

    for chunk_y in min_chunk_y..=max_chunk_y {
        let mut line_spans = Vec::new();

        for chunk_x in min_chunk_x..=max_chunk_x {
            let char_to_display;
            let style;

            // Check if this is the player's current chunk
            if chunk_x == player_chunk_x && chunk_y == player_chunk_y {
                char_to_display = '@';
                style = Style::default().fg(Color::Yellow);
            }
            // Check if there's a quest marker here
            else if let Some(quest_marker) =
                app.core.game.res.quest_markers.iter().find(|marker| {
                    let marker_chunk_x = marker.x.div_euclid(CHUNK_SIZE);
                    let marker_chunk_y = marker.y.div_euclid(CHUNK_SIZE);
                    marker_chunk_x == chunk_x && marker_chunk_y == chunk_y
                })
            {
                char_to_display = match quest_marker.marker_type {
                    QuestMarkerType::MainQuest => '!',
                    QuestMarkerType::SideQuest => '?',
                    QuestMarkerType::Location => 'L',
                    QuestMarkerType::Treasure => '$',
                };
                style = Style::default().fg(Color::Red);
            }
            // Check if this chunk has been explored
            else if app
                .core
                .game
                .res
                .is_chunk_explored(chunk_x, chunk_y, player_chunk_z)
            {
                char_to_display = '·';
                style = Style::default().fg(Color::Gray);
            }
            // Unknown/unexplored chunk
            else {
                char_to_display = ' ';
                style = Style::default().fg(Color::DarkGray);
            }

            line_spans.push(Span::styled(char_to_display.to_string(), style));
        }

        map_lines.push(Line::from(line_spans));
    }

    // Render the map
    let map_paragraph = Paragraph::new(map_lines).alignment(Alignment::Left);

    let map_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y + 1,
        width: map_width as u16,
        height: map_height as u16,
    };

    f.render_widget(map_paragraph, map_area);

    // Render legend
    let legend_text = vec![
        Line::from(vec![
            Span::styled("@ ", Style::default().fg(Color::Yellow)),
            Span::raw("Player  "),
            Span::styled("! ", Style::default().fg(Color::Red)),
            Span::raw("Main Quest  "),
            Span::styled("? ", Style::default().fg(Color::Red)),
            Span::raw("Side Quest"),
        ]),
        Line::from(vec![
            Span::styled("· ", Style::default().fg(Color::Gray)),
            Span::raw("Explored  "),
            Span::styled("  ", Style::default().fg(Color::DarkGray)),
            Span::raw("Unexplored  "),
            Span::styled("$ ", Style::default().fg(Color::Red)),
            Span::raw("Treasure"),
        ]),
    ];

    let legend_paragraph = Paragraph::new(legend_text).alignment(Alignment::Left);

    let legend_area = Rect {
        x: inner_area.x + 1,
        y: inner_area.y + map_height as u16 + 2,
        width: inner_area.width.saturating_sub(2),
        height: 2,
    };

    f.render_widget(legend_paragraph, legend_area);

    // Show current position info
    let position_info = format!(
        "Player: ({}, {}, {}) | Chunk: ({}, {}, {})",
        app.core
            .game
            .get_player_position()
            .map(|p| p.x)
            .unwrap_or(0),
        app.core
            .game
            .get_player_position()
            .map(|p| p.y)
            .unwrap_or(0),
        app.core
            .game
            .get_player_position()
            .map(|p| p.z)
            .unwrap_or(0),
        player_chunk_x,
        player_chunk_y,
        player_chunk_z
    );

    let position_paragraph = Paragraph::new(position_info)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);

    let position_area = Rect {
        x: inner_area.x,
        y: inner_area.bottom().saturating_sub(1),
        width: inner_area.width,
        height: 1,
    };

    f.render_widget(position_paragraph, position_area);
}
