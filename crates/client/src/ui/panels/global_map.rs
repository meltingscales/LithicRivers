use crate::App;
use lithicrivers_core::resources::QuestMarkerType;
use lithicrivers_core::world::{CHUNK_SIZE, CHUNK_SIZE_Z};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the global map panel
pub fn render_global_map_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Split the area horizontally: 70% for map, 30% for markers panel
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Render the map panel
    render_map_panel(f, app, main_chunks[0]);

    // Render the markers panel
    render_markers_panel(f, app, main_chunks[1]);
}

/// Render the map portion of the global map
fn render_map_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(Line::from(" Global Map "))
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

    // Get selected marker for arrow drawing
    // Index 0 is the "unselect" option, so actual markers start at index 1
    let selected_marker = if app.panels.global_map.selected_marker_index > 0
        && app.core.game.res.quest_markers.len() >= app.panels.global_map.selected_marker_index
    {
        let marker_index = app.panels.global_map.selected_marker_index - 1; // Convert to 0-based marker index
        app.core.game.res.quest_markers.get(marker_index)
    } else {
        None
    };

    let selected_marker_chunk = selected_marker.map(|marker| {
        (
            marker.x.div_euclid(CHUNK_SIZE),
            marker.y.div_euclid(CHUNK_SIZE),
        )
    });

    // Build the map display
    let mut map_lines = Vec::new();

    for chunk_y in min_chunk_y..=max_chunk_y {
        let mut line_spans = Vec::new();

        for chunk_x in min_chunk_x..=max_chunk_x {
            let char_to_display;
            let style;

            // Check if this is part of the arrow path to selected marker
            let is_arrow_path = selected_marker_chunk.map_or(false, |(target_x, target_y)| {
                is_on_arrow_path(
                    player_chunk_x,
                    player_chunk_y,
                    target_x,
                    target_y,
                    chunk_x,
                    chunk_y,
                )
            });

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
                    QuestMarkerType::FetchQuest => 'F',
                    QuestMarkerType::Location => 'L',
                    QuestMarkerType::Treasure => '$',
                };
                style = Style::default().fg(Color::Red);
            }
            // Check if this is part of an arrow path
            else if is_arrow_path {
                if let Some((target_x, target_y)) = selected_marker_chunk {
                    char_to_display = get_arrow_char(
                        player_chunk_x,
                        player_chunk_y,
                        target_x,
                        target_y,
                        chunk_x,
                        chunk_y,
                    );
                    style = Style::default().fg(Color::Cyan);
                } else {
                    char_to_display = ' ';
                    style = Style::default().fg(Color::DarkGray);
                }
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
            Span::styled("[@] ", Style::default().fg(Color::Yellow)),
            Span::raw("Player  "),
            Span::styled("[!] ", Style::default().fg(Color::Red)),
            Span::raw("Main Quest  "),
            Span::styled("[?] ", Style::default().fg(Color::Red)),
            Span::raw("Side Quest"),
        ]),
        Line::from(vec![
            Span::styled("[·] ", Style::default().fg(Color::Gray)),
            Span::raw("Explored  "),
            Span::styled("[ ] ", Style::default().fg(Color::DarkGray)),
            Span::raw("Unexplored  "),
            Span::styled("[$] ", Style::default().fg(Color::Red)),
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

/// Render the markers panel on the right side
fn render_markers_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(Line::from(" Quest Markers "))
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Magenta));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Get player position for distance calculations
    let player_pos = app.core.game.get_player_position();

    if let Some(player_position) = player_pos {
        // Get all quest markers and calculate distances
        let mut marker_distances: Vec<_> = app
            .core
            .game
            .res
            .quest_markers
            .iter()
            .map(|marker| {
                let dx = marker.x - player_position.x;
                let dy = marker.y - player_position.y;
                let dz = marker.z - player_position.z;
                let distance = ((dx * dx + dy * dy + dz * dz) as f64).sqrt();
                (marker, distance)
            })
            .collect();

        // Sort by distance (closest first)
        marker_distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Create list items - start with "unselect" option
        let mut marker_items: Vec<ListItem> = Vec::new();

        // Add "unselect quest marker" option at index 0
        let is_unselect_selected = app.panels.global_map.selected_marker_index == 0;
        let unselect_style = if is_unselect_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };
        let unselect_indicator = if is_unselect_selected { "► " } else { "  " };

        let unselect_content = vec![
            Line::from(vec![
                Span::styled(unselect_indicator, Style::default().fg(Color::Cyan)),
                Span::styled("✗ ", Style::default().fg(Color::Gray)),
                Span::styled("(unselect quest marker)", unselect_style),
            ]),
            Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    "Hide arrow",
                    if is_unselect_selected {
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]),
        ];
        marker_items.push(ListItem::new(unselect_content));

        // Add actual quest markers (their indices will be offset by 1)
        let quest_marker_items: Vec<ListItem> = marker_distances
            .iter()
            .enumerate()
            .map(|(marker_index, (marker, distance))| {
                let list_index = marker_index + 1; // Offset by 1 due to unselect option
                let is_selected = list_index == app.panels.global_map.selected_marker_index;
                let marker_symbol = match marker.marker_type {
                    QuestMarkerType::MainQuest => "!",
                    QuestMarkerType::SideQuest => "?",
                    QuestMarkerType::FetchQuest => "F",
                    QuestMarkerType::Location => "L",
                    QuestMarkerType::Treasure => "$",
                };

                let marker_color = match marker.marker_type {
                    QuestMarkerType::MainQuest => Color::Red,
                    QuestMarkerType::SideQuest => Color::Yellow,
                    QuestMarkerType::FetchQuest => Color::Cyan,
                    QuestMarkerType::Location => Color::Blue,
                    QuestMarkerType::Treasure => Color::Green,
                };

                let distance_text = if *distance < 1000.0 {
                    format!("{:.0} blocks", distance)
                } else {
                    format!("{:.1}k blocks", distance / 1000.0)
                };

                let (name_style, distance_style, coords_style) = if is_selected {
                    (
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::DarkGray),
                        Style::default()
                            .fg(Color::Gray)
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::DarkGray),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                            .bg(Color::DarkGray),
                    )
                } else {
                    (
                        Style::default().fg(Color::White),
                        Style::default().fg(Color::Gray),
                        Style::default().fg(Color::DarkGray),
                    )
                };

                let selection_indicator = if is_selected { "► " } else { "  " };

                let content = vec![
                    Line::from(vec![
                        Span::styled(selection_indicator, Style::default().fg(Color::Cyan)),
                        Span::styled(
                            format!("{} ", marker_symbol),
                            Style::default()
                                .fg(marker_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(&marker.name, name_style),
                    ]),
                    Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled(distance_text, distance_style),
                    ]),
                    Line::from(vec![
                        Span::styled("   ", Style::default()),
                        Span::styled(
                            format!("({}, {}, {})", marker.x, marker.y, marker.z),
                            coords_style,
                        ),
                    ]),
                ];

                ListItem::new(content)
            })
            .collect();

        marker_items.extend(quest_marker_items);

        if !marker_items.is_empty() {
            let markers_list = List::new(marker_items)
                .block(Block::default())
                .style(Style::default().fg(Color::White));

            f.render_widget(markers_list, inner_area);
        } else {
            // No markers found
            let no_markers_text = Paragraph::new("No quest markers found.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center);

            f.render_widget(no_markers_text, inner_area);
        }
    } else {
        // Player position not found
        let error_text = Paragraph::new("Player position unknown.")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);

        f.render_widget(error_text, inner_area);
    }
}

/// Generate all points on a line using Bresenham's algorithm
fn get_line_points(start_x: i64, start_y: i64, end_x: i64, end_y: i64) -> Vec<(i64, i64)> {
    let mut points = Vec::new();

    let dx = (end_x - start_x).abs();
    let dy = (end_y - start_y).abs();

    let sx = if start_x < end_x { 1 } else { -1 };
    let sy = if start_y < end_y { 1 } else { -1 };

    let mut err = dx - dy;
    let mut x = start_x;
    let mut y = start_y;

    loop {
        points.push((x, y));

        if x == end_x && y == end_y {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x += sx;
        }

        if e2 < dx {
            err += dx;
            y += sy;
        }
    }

    points
}

/// Check if a point (chunk_x, chunk_y) is on the arrow path from start to end
fn is_on_arrow_path(
    start_x: i64,
    start_y: i64,
    end_x: i64,
    end_y: i64,
    chunk_x: i64,
    chunk_y: i64,
) -> bool {
    // Don't draw arrows for the start and end points (they have their own symbols)
    if (chunk_x == start_x && chunk_y == start_y) || (chunk_x == end_x && chunk_y == end_y) {
        return false;
    }

    if start_x == end_x && start_y == end_y {
        return false; // Start and end are the same
    }

    // Generate the line points using Bresenham's algorithm
    let line_points = get_line_points(start_x, start_y, end_x, end_y);

    // Check if our point is in the line (excluding start and end)
    line_points
        .iter()
        .skip(1) // Skip start point
        .take(line_points.len().saturating_sub(2)) // Skip end point
        .any(|&(px, py)| px == chunk_x && py == chunk_y)
}

/// Get the appropriate arrow character for a position on the line
fn get_arrow_char(
    start_x: i64,
    start_y: i64,
    end_x: i64,
    end_y: i64,
    chunk_x: i64,
    chunk_y: i64,
) -> char {
    // Generate the line points to find our position and get local direction
    let line_points = get_line_points(start_x, start_y, end_x, end_y);

    // Find our current position in the line
    let current_pos = line_points
        .iter()
        .position(|&(x, y)| x == chunk_x && y == chunk_y);

    if let Some(pos) = current_pos {
        // Check if we're at the last point (arrowhead)
        if pos == line_points.len() - 1 {
            // This is the end point - draw arrowhead based on overall direction
            let dx = end_x - start_x;
            let dy = end_y - start_y;

            if dx == 0 {
                if dy > 0 {
                    '▼'
                } else {
                    '▲'
                }
            } else if dy == 0 {
                if dx > 0 {
                    '►'
                } else {
                    '◄'
                }
            } else {
                // Diagonal arrowhead
                if dx > 0 && dy > 0 {
                    '◣' // Down-right
                } else if dx < 0 && dy > 0 {
                    '◤' // Down-left
                } else if dx > 0 && dy < 0 {
                    '◥' // Up-right
                } else {
                    '◢' // Up-left
                }
            }
        } else if pos > 0 && pos < line_points.len() - 1 {
            // Middle of line - determine character based on local direction
            let prev_point = line_points[pos - 1];
            let next_point = line_points[pos + 1];

            let local_dx = next_point.0 - prev_point.0;
            let local_dy = next_point.1 - prev_point.1;

            if local_dx == 0 {
                '│' // Vertical
            } else if local_dy == 0 {
                '─' // Horizontal
            } else if (local_dx > 0 && local_dy > 0) || (local_dx < 0 && local_dy < 0) {
                '╲' // Backslash-like diagonal
            } else {
                '╱' // Slash-like diagonal
            }
        } else {
            '·' // Fallback
        }
    } else {
        '·' // Point not on line
    }
}
