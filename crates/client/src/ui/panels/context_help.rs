//! Context help: the keys that matter right now, shown below the inventory on the World page.

use crate::app::input::world_action::nearby_interactions;
use crate::app_state::{BuildMode, InteractionType};
use crate::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Nearby interactions listed at most.
const MAX_INTERACTIONS: usize = 3;

fn key(app: &App, category: &str, action: &str) -> &'static str {
    app.core
        .config_manager
        .get_printable_key_for_keybind(category, action)
}

/// The keys of a 3x3 direction set (NW..SE, center in the middle) as three rows.
fn grid(app: &App, category: &str, actions: [&str; 9]) -> Vec<String> {
    actions
        .chunks(3)
        .map(|row| {
            row.iter()
                .map(|a| key(app, category, a))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .map(|row| format!("  {row}"))
        .collect()
}

const MOVE_KEYS: [&str; 9] = [
    "MOVE_NORTHWEST",
    "MOVE_NORTH",
    "MOVE_NORTHEAST",
    "MOVE_WEST",
    "WAIT",
    "MOVE_EAST",
    "MOVE_SOUTHWEST",
    "MOVE_SOUTH",
    "MOVE_SOUTHEAST",
];

fn directions(prefix: &str) -> [String; 9] {
    [
        "NORTHWEST",
        "NORTH",
        "NORTHEAST",
        "WEST",
        "CENTER",
        "EAST",
        "SOUTHWEST",
        "SOUTH",
        "SOUTHEAST",
    ]
    .map(|d| format!("{prefix}_{d}"))
}

fn interaction_label(action: &InteractionType) -> String {
    match action {
        InteractionType::PickupItem { item_name, .. } => format!("pick up {item_name}"),
        InteractionType::LootCorpse { .. } => "loot corpse".to_string(),
        InteractionType::ClearCorpse { wreck: true, .. } => "clear wreck".to_string(),
        InteractionType::ClearCorpse { wreck: false, .. } => "clear corpse".to_string(),
        InteractionType::TalkToNPC { npc_name, .. } => format!("talk to {npc_name}"),
        InteractionType::OpenChest { .. } => "open chest".to_string(),
        InteractionType::OpenGenerator { .. } => "fuel generator".to_string(),
        InteractionType::OpenMachine { kind, .. } => format!("open {}", kind.name()),
        InteractionType::OpenFurnace { .. } => "open furnace".to_string(),
        InteractionType::SetFilter { .. } => "set filter".to_string(),
        InteractionType::InspectPole { .. } => "inspect pole".to_string(),
        InteractionType::LoadBelt { item, .. } => format!(
            "{} on belt",
            lithicrivers_core::components::itemkind_name(*item)
        ),
        InteractionType::OpenCloseDoor { is_open: true, .. } => "close door".to_string(),
        InteractionType::OpenCloseDoor { is_open: false, .. } => "open door".to_string(),
    }
}

/// The panel's lines for the current situation, most specific first: look mode, then
/// break or place mode, then walking with what interact would do nearby.
pub fn context_help_lines(app: &App) -> Vec<String> {
    let mut lines = Vec::new();
    if app.panels.look.mode {
        lines.push("Look mode".to_string());
        lines.push("Move cursor:".to_string());
        lines.extend(grid(app, "movement", MOVE_KEYS));
        lines.push(format!(
            "{}: stop looking",
            key(app, "action", "LOOK_TOGGLE")
        ));
        return lines;
    }
    let mode_key = key(app, "build", "TOGGLE_BREAK_PLACE_MODE");
    match app.panels.build.mode {
        BuildMode::Break | BuildMode::Place => {
            let (name, prefix) = if app.panels.build.mode == BuildMode::Break {
                ("Break", "BREAK")
            } else {
                ("Place", "PLACE")
            };
            lines.push(format!("{name} mode, direction:"));
            let actions = directions(prefix);
            lines.extend(grid(app, "build", actions.each_ref().map(String::as_str)));
            if app.panels.build.mode == BuildMode::Place {
                lines.push(format!(
                    "{}-{}: hotbar",
                    key(app, "hotbar", "HOTBAR_SLOT_1"),
                    key(app, "hotbar", "HOTBAR_SLOT_12")
                ));
                lines.push(format!(
                    "{}: turn ({})",
                    key(app, "build", "ROTATE_PLACEMENT"),
                    app.panels.build.placement_facing.arrow()
                ));
            }
            lines.push(format!("{mode_key}: next mode"));
        }
        BuildMode::Rotate => {
            lines.push("Rotate block, direction:".to_string());
            let actions = directions("PLACE");
            lines.extend(grid(app, "build", actions.each_ref().map(String::as_str)));
            lines.push(format!("{mode_key}: next mode"));
        }
        BuildMode::Blueprint => {
            let center = key(app, "build", "PLACE_CENTER");
            if app.panels.build.blueprint.copying {
                lines.push("Copy, cursor:".to_string());
                let actions = directions("PLACE");
                lines.extend(grid(app, "build", actions.each_ref().map(String::as_str)));
                lines.push(format!("{center}: mark corner"));
            } else {
                lines.push("Paste, cursor:".to_string());
                let actions = directions("PLACE");
                lines.extend(grid(app, "build", actions.each_ref().map(String::as_str)));
                let (replace, blocked) = app.panels.build.blueprint.preview;
                if blocked > 0 {
                    lines.push(format!("{blocked} blocked here"));
                } else if replace > 0 {
                    lines.push(format!("Replaces {replace} built"));
                }
                lines.push(format!("{center}: paste"));
                lines.push(format!("{}: turn", key(app, "build", "ROTATE_PLACEMENT")));
                lines.push(format!("{}: copy new", key(app, "build", "NEW_BLUEPRINT")));
            }
            lines.push(format!(
                "{}: blueprints",
                key(app, "build", "BLUEPRINT_LIBRARY")
            ));
            lines.push(format!("{}: leave", key(app, "build", "BLUEPRINT_MODE")));
        }
        BuildMode::Movement => {
            lines.push("Move:".to_string());
            lines.extend(grid(app, "movement", MOVE_KEYS));
            lines.push(format!("{}: mine", key(app, "action", "MINE")));
            lines.push(format!("{}: look", key(app, "action", "LOOK_TOGGLE")));
            lines.push(format!("{mode_key}: build mode"));
            let interact = key(app, "action", "INTERACT");
            let nearby = nearby_interactions(app)
                .map(|n| n.actions)
                .unwrap_or_default();
            if nearby.is_empty() {
                lines.push(format!("{interact}: interact"));
            }
            for action in nearby.iter().take(MAX_INTERACTIONS) {
                lines.push(format!("{interact}: {}", interaction_label(action)));
            }
        }
    }
    lines
}

/// Draw the panel, or (when hidden) the one-line hint naming the toggle key.
pub fn render_context_help(f: &mut Frame, app: &App, area: Rect) {
    let toggle = key(app, "ui", "TOGGLE_CONTEXT_HELP");
    let hint_style = Style::default().fg(Color::DarkGray);
    if !app.ui.show_context_help {
        let hint = Paragraph::new(Line::from(Span::styled(
            format!("{toggle}: help"),
            hint_style,
        )));
        f.render_widget(hint, area);
        return;
    }
    let lines: Vec<Line> = context_help_lines(app)
        .into_iter()
        .map(Line::from)
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(format!("Help ({toggle})")));
    f.render_widget(Paragraph::new(lines).block(block), area);
}

#[cfg(test)]
mod tests {
    use super::context_help_lines;
    use crate::app_state::BuildMode;
    use crate::headless::chain_commands;

    fn has(lines: &[String], text: &str) -> bool {
        lines.iter().any(|l| l.contains(text))
    }

    #[test]
    fn walking_lists_move_mine_look_build_and_interact() {
        let lines = context_help_lines(&chain_commands(&["skip-splash", "cheat tp 0 0 0"]).app);
        assert_eq!(lines[0], "Move:");
        assert_eq!(lines[1], "  7 8 9");
        for text in ["q: mine", "v: look", "`: build mode", "f: interact"] {
            assert!(has(&lines, text), "{text}: {lines:?}");
        }
    }

    #[test]
    fn place_mode_lists_directions_and_hotbar() {
        let mut runner = chain_commands(&["skip-splash"]);
        runner.app.panels.build.mode = BuildMode::Place;
        let lines = context_help_lines(&runner.app);
        assert_eq!(lines[0], "Place mode, direction:");
        assert_eq!(lines[1], "  q w e");
        assert!(has(&lines, "F1-F12: hotbar"), "{lines:?}");
        assert!(!has(&lines, "mine"));

        runner.app.panels.build.mode = BuildMode::Break;
        let lines = context_help_lines(&runner.app);
        assert_eq!(lines[0], "Break mode, direction:");
        assert!(!has(&lines, "hotbar"));
    }

    #[test]
    fn look_mode_lists_cursor_keys_and_exit() {
        let mut runner = chain_commands(&["skip-splash"]);
        runner.app.panels.look.mode = true;
        let lines = context_help_lines(&runner.app);
        assert_eq!(lines[0], "Look mode");
        assert!(has(&lines, "v: stop looking"), "{lines:?}");
    }

    #[test]
    fn nearby_corpse_shows_loot_until_it_is_empty_then_clear() {
        // Next to the corpse Game::new leaves at (18, 21, 0)
        let mut runner = chain_commands(&["skip-splash", "cheat tp 17 21 0"]);
        assert!(has(&context_help_lines(&runner.app), "f: loot corpse"));

        runner.chain(&[
            "key ui.MENU_PREV",
            "key action.INTERACT",
            "key ui.MENU_ACTIVATE 3",
            "key ui.CLOSE_HELP_MENU",
        ]);
        let lines = context_help_lines(&runner.app);
        assert!(!has(&lines, "loot corpse"), "{lines:?}");
        assert!(has(&lines, "f: clear corpse"), "{lines:?}");
    }

    #[test]
    fn remapped_interact_key_is_shown() {
        let mut runner = chain_commands(&["skip-splash", "cheat tp 17 21 0"]);
        runner.app.core.config_manager.data.keybinds["action"]["INTERACT"] =
            serde_json::json!(["j"]);
        let lines = context_help_lines(&runner.app);
        assert!(has(&lines, "j: loot corpse"), "{lines:?}");
        assert!(!has(&lines, "f: "), "{lines:?}");
    }

    #[test]
    fn toggle_flips_the_setting_without_writing_the_config_file() {
        let mut runner = chain_commands(&["skip-splash", "key ui.MENU_PREV"]);
        let path = runner.app.core.config_manager.file_path.clone();
        let before = std::fs::read_to_string(&path).ok();
        assert!(runner.app.ui.show_context_help);

        runner.chain(&["key ui.TOGGLE_CONTEXT_HELP"]);
        assert!(!runner.app.ui.show_context_help);
        let setting = runner
            .app
            .core
            .config_manager
            .get_setting("ui", "SHOW_CONTEXT_HELP");
        assert_eq!(setting, Some(&serde_json::json!(false)));
        assert!(runner.screen_text().contains("h: help"));

        runner.chain(&["key ui.TOGGLE_CONTEXT_HELP"]);
        assert!(runner.app.ui.show_context_help);
        assert!(runner.screen_text().contains("Help (h)"));
        assert_eq!(
            std::fs::read_to_string(&path).ok(),
            before,
            "headless runs never save config"
        );
    }
}
