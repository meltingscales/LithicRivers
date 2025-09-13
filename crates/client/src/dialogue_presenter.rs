use crate::app_state::{DialogueNode, NPCMood};
use crate::dialogue_engine::{ConversationState, DialogueEngine};
use ratatui::style::Color;

/// UI presentation layer for dialogue system - handles only formatting and display
pub struct DialoguePresenter;

impl DialoguePresenter {
    /// Format dialogue text for display in UI
    pub fn format_dialogue_text(
        engine: &DialogueEngine,
        conversation: &ConversationState,
        selected_choice: usize,
    ) -> String {
        if let Some(node) = engine.get_current_node(conversation) {
            let mood_prefix = Self::get_mood_prefix(node.mood);
            let speaker_text = format!("{}{}: \"{}\"", mood_prefix, node.speaker, node.text);

            let mut full_text = format!("{}\n\n", speaker_text);
            for (i, choice) in node.choices.iter().enumerate() {
                let prefix = if i == selected_choice { ">" } else { " " };
                full_text.push_str(&format!("{} {}. {}\n", prefix, i + 1, choice.text));
            }
            full_text
        } else {
            "No dialogue available".to_string()
        }
    }

    /// Get mood-based color for UI styling
    pub fn get_mood_color(mood: NPCMood) -> Color {
        match mood {
            NPCMood::Friendly => Color::Green,
            NPCMood::Neutral => Color::White,
            NPCMood::Hostile => Color::Red,
            NPCMood::Sad => Color::Blue,
            NPCMood::Excited => Color::Yellow,
            NPCMood::Mysterious => Color::Magenta,
        }
    }

    /// Get mood prefix for text display
    pub fn get_mood_prefix(mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "> ",
            NPCMood::Neutral => "- ",
            NPCMood::Hostile => "! ",
            NPCMood::Sad => "~ ",
            NPCMood::Excited => "* ",
            NPCMood::Mysterious => "? ",
        }
    }

    /// Generate ASCII portrait based on NPC data and current mood
    pub fn generate_portrait(npc_name: &str, mood: NPCMood) -> String {
        let eyes = Self::get_mood_eyes(mood);
        let mouth = Self::get_mood_face(mood);
        let symbol = Self::get_npc_symbol(npc_name);

        format!(
            "    ╔═══════════╗
    ║  {}  ║
    ║     {}     ║
    ║   ┌───┐   ║
    ║   │ {} │   ║
    ║   └───┘   ║
    ╚═══════════╝",
            eyes, mouth, symbol
        )
    }

    /// Get eyes based on mood
    fn get_mood_eyes(mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "◉     ◉",
            NPCMood::Neutral => "○     ○",
            NPCMood::Hostile => "▲     ▲",
            NPCMood::Sad => "◌     ◌",
            NPCMood::Excited => "★     ★",
            NPCMood::Mysterious => "◇     ◇",
        }
    }

    /// Get mouth/expression based on mood
    fn get_mood_face(mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "◡",
            NPCMood::Neutral => "─",
            NPCMood::Hostile => "▼",
            NPCMood::Sad => "︶",
            NPCMood::Excited => "◠",
            NPCMood::Mysterious => "~",
        }
    }

    /// Get symbol based on NPC type/name
    fn get_npc_symbol(npc_name: &str) -> &'static str {
        match npc_name {
            name if name.contains("Merchant") => "$",
            name if name.contains("Knight") || name.contains("Captain") => ">",
            name if name.contains("Oracle") || name.contains("Mysterious") => "?",
            name if name.contains("Innkeeper") => "@",
            name if name.contains("Bandit") => "X",
            _ => "*",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue_engine::DialogueEngine;

    #[test]
    fn test_format_dialogue_text() {
        let engine = DialogueEngine::new();
        let conversation = engine.start_conversation(0).unwrap();

        let text = DialoguePresenter::format_dialogue_text(&engine, &conversation, 0);

        assert!(text.contains("Merchant Aldric"));
        assert!(text.contains("Welcome, traveler"));
        assert!(text.contains("> 1. Show me your weapons")); // First choice selected
        assert!(text.contains("  2. I need healing supplies")); // Other choices not selected
    }

    #[test]
    fn test_mood_colors() {
        assert_eq!(
            DialoguePresenter::get_mood_color(NPCMood::Friendly),
            Color::Green
        );
        assert_eq!(
            DialoguePresenter::get_mood_color(NPCMood::Hostile),
            Color::Red
        );
        assert_eq!(
            DialoguePresenter::get_mood_color(NPCMood::Neutral),
            Color::White
        );
    }

    #[test]
    fn test_mood_prefixes() {
        assert_eq!(DialoguePresenter::get_mood_prefix(NPCMood::Friendly), "> ");
        assert_eq!(DialoguePresenter::get_mood_prefix(NPCMood::Hostile), "! ");
        assert_eq!(DialoguePresenter::get_mood_prefix(NPCMood::Excited), "* ");
    }

    #[test]
    fn test_portrait_generation() {
        let portrait = DialoguePresenter::generate_portrait("Merchant Aldric", NPCMood::Friendly);

        assert!(portrait.contains("◉     ◉")); // Friendly eyes
        assert!(portrait.contains("◡")); // Friendly mouth
        assert!(portrait.contains("$")); // Merchant symbol
    }

    #[test]
    fn test_npc_symbols() {
        assert_eq!(DialoguePresenter::get_npc_symbol("Merchant Aldric"), "$");
        assert_eq!(
            DialoguePresenter::get_npc_symbol("Knight Captain Elena"),
            ">"
        );
        assert_eq!(DialoguePresenter::get_npc_symbol("Mysterious Oracle"), "?");
        assert_eq!(DialoguePresenter::get_npc_symbol("Innkeeper Marta"), "@");
        assert_eq!(DialoguePresenter::get_npc_symbol("Unknown Person"), "*");
    }
}
