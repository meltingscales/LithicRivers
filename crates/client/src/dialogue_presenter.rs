use crate::app_state::{DialogueNode, NPCMood};
use crate::dialogue_engine::{ConversationState, DialogueEngine};
use crate::sprite_loader::SpriteLoader;
use lithicrivers_core::components::{NPCMood as CoreNPCMood, SpriteRef};
use ratatui::style::Color;

/// UI presentation layer for dialogue system - handles only formatting and display
pub struct DialoguePresenter;

impl DialoguePresenter {
    /// Convert dialogue mood to core NPC mood for sprite loader
    fn dialogue_mood_to_core_mood(mood: NPCMood) -> CoreNPCMood {
        match mood {
            NPCMood::Friendly => CoreNPCMood::Happy,
            NPCMood::Neutral => CoreNPCMood::Neutral,
            NPCMood::Hostile => CoreNPCMood::Weird, // Hostile maps to "weird" expression
            NPCMood::Sad => CoreNPCMood::Sad,
            NPCMood::Excited => CoreNPCMood::Happy, // Excited is also happy
            NPCMood::Mysterious => CoreNPCMood::Weird, // Mysterious maps to "weird" expression
        }
    }

    /// Get character portrait for NPC in current mood
    pub fn get_character_portrait(
        sprite_loader: &mut SpriteLoader,
        npc_name: &str,
        mood: NPCMood,
    ) -> Option<String> {
        // Map NPC names to sprite references - this would normally come from NPC data
        let sprite_ref = match npc_name {
            "Merchant Aldric" => SpriteRef {
                category: "npcs".to_string(),
                name: "merchant".to_string(),
            },
            "Knight Captain Elena" => SpriteRef {
                category: "npcs".to_string(),
                name: "knight".to_string(),
            },
            "Mysterious Oracle" => SpriteRef {
                category: "npcs".to_string(),
                name: "oracle".to_string(),
            },
            "Innkeeper Marta" => SpriteRef {
                category: "npcs".to_string(),
                name: "innkeeper".to_string(),
            },
            "Bandit Leader Raven" => SpriteRef {
                category: "npcs".to_string(),
                name: "bandit".to_string(),
            },
            "QuestTesty" => SpriteRef {
                category: "entities".to_string(),
                name: "quest_testy".to_string(),
            },
            _ => return None,
        };

        let core_mood = Self::dialogue_mood_to_core_mood(mood);
        sprite_loader.get_mood_portrait(&sprite_ref, core_mood)
    }

    /// Format dialogue with Summon Night-style layout including both player and NPC portraits
    pub fn format_dialogue_with_portraits(
        sprite_loader: &mut SpriteLoader,
        engine: &DialogueEngine,
        conversation: &ConversationState,
        selected_choice: usize,
    ) -> (String, Option<String>, Option<String>) {
        if let Some(node) = engine.get_current_node(conversation) {
            let npc_portrait =
                Self::get_character_portrait(sprite_loader, &node.speaker, node.mood);
            let player_portrait = Self::get_player_portrait(sprite_loader, selected_choice);
            let dialogue_text = Self::format_dialogue_text(engine, conversation, selected_choice);
            (dialogue_text, npc_portrait, player_portrait)
        } else {
            ("No dialogue available".to_string(), None, None)
        }
    }

    /// Get player portrait based on current dialogue context (player mood/response)
    pub fn get_player_portrait(
        sprite_loader: &mut SpriteLoader,
        selected_choice: usize,
    ) -> Option<String> {
        let sprite_ref = SpriteRef {
            category: "entities".to_string(),
            name: "player".to_string(),
        };

        // Map player choice index to mood - this simulates player emotional response
        let player_mood = match selected_choice % 4 {
            0 => CoreNPCMood::Happy,   // First choice - confident/positive
            1 => CoreNPCMood::Neutral, // Second choice - neutral/thoughtful
            2 => CoreNPCMood::Sad,     // Third choice - cautious/worried
            3 => CoreNPCMood::Weird,   // Fourth choice - suspicious/confused
            _ => CoreNPCMood::Neutral,
        };

        sprite_loader.get_mood_portrait(&sprite_ref, player_mood)
    }
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

        assert!(text.contains("QuestTesty"));
        assert!(text.contains("Greetings, brave traveler"));
        assert!(text.contains("> 1. Tell me about these mysteries")); // First choice selected
        assert!(text.contains("  2. I'm looking for adventure")); // Other choices not selected
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
