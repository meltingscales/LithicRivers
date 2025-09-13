use crate::app_state::{DialogueChoice, DialogueNode, DialogueType, NPCData, NPCMood};
use ratatui::style::Color;

/// Complete dialogue system with all NPCs and conversation trees from the demo
pub struct DialogueSystem {
    pub npcs: Vec<NPCData>,
    pub dialogue_tree: Vec<DialogueNode>,
    pub message_log: Vec<String>,
}

impl DialogueSystem {
    pub fn new() -> Self {
        let mut system = Self {
            npcs: Vec::new(),
            dialogue_tree: Vec::new(),
            message_log: Vec::new(),
        };

        system.setup_npcs_and_dialogue();
        system
    }

    fn setup_npcs_and_dialogue(&mut self) {
        // Create NPCs with portraits inspired by classic RPGs
        self.npcs = vec![
            NPCData {
                name: "Merchant Aldric".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ◉     ◉  ║
    ║     ◡     ║
    ║   ┌───┐   ║
    ║   │ $ │   ║
    ║   └───┘   ║
    ╚═══════════╝
"
                .to_string(),
                dialogue_type: DialogueType::Shop,
                current_mood: NPCMood::Friendly,
                initial_dialogue: 0,
                met_before: false,
                has_quest: false,
                shop_inventory: vec![
                    "Iron Sword".to_string(),
                    "Magic Scroll".to_string(),
                    "Elixir".to_string(),
                ],
            },
            NPCData {
                name: "Knight Captain Elena".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ◉     ◉  ║
    ║     ─     ║
    ║   ┌───┐   ║
    ║   │ ⚔ │   ║
    ║   └───┘   ║
    ╚═══════════╝
"
                .to_string(),
                dialogue_type: DialogueType::Quest,
                current_mood: NPCMood::Neutral,
                initial_dialogue: 10,
                met_before: false,
                has_quest: true,
                shop_inventory: Vec::new(),
            },
            NPCData {
                name: "Mysterious Oracle".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ◌     ◌  ║
    ║     ~     ║
    ║   ┌───┐   ║
    ║   │ ?  │   ║
    ║   └───┘   ║
    ╚═══════════╝
"
                .to_string(),
                dialogue_type: DialogueType::Branching,
                current_mood: NPCMood::Mysterious,
                initial_dialogue: 20,
                met_before: false,
                has_quest: false,
                shop_inventory: Vec::new(),
            },
            NPCData {
                name: "Innkeeper Marta".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ♥     ♥  ║
    ║     ⌣     ║
    ║   ┌───┐   ║
    ║   │ @  │   ║
    ║   └───┘   ║
    ╚═══════════╝
"
                .to_string(),
                dialogue_type: DialogueType::Linear,
                current_mood: NPCMood::Friendly,
                initial_dialogue: 30,
                met_before: false,
                has_quest: false,
                shop_inventory: Vec::new(),
            },
            NPCData {
                name: "Bandit Leader Raven".to_string(),
                portrait: "
    ╔═══════════╗
    ║  ▲     ▲  ║
    ║    \\▼/    ║
    ║   ┌───┐   ║
    ║   │ X  │   ║
    ║   └───┘   ║
    ╚═══════════╝
"
                .to_string(),
                dialogue_type: DialogueType::Battle,
                current_mood: NPCMood::Hostile,
                initial_dialogue: 40,
                met_before: false,
                has_quest: false,
                shop_inventory: Vec::new(),
            },
        ];

        // Create comprehensive dialogue tree (truncated version for now)
        self.dialogue_tree = vec![
            // Merchant Aldric (0-9)
            DialogueNode {
                id: 0,
                speaker: "Merchant Aldric".to_string(),
                text: "Welcome, traveler! I have the finest wares in all the land. What catches your eye?".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Show me your weapons".to_string(),
                        leads_to: Some(1),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "I need healing supplies".to_string(),
                        leads_to: Some(2),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Just browsing, thanks".to_string(),
                        leads_to: Some(3),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            DialogueNode {
                id: 1,
                speaker: "Merchant Aldric".to_string(),
                text: "Ah, a warrior! This Iron Sword is newly forged and sharp as winter wind. Only 30 gold!".to_string(),
                mood: NPCMood::Excited,
                choices: vec![
                    DialogueChoice {
                        text: "I'll take it!".to_string(),
                        leads_to: Some(4),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Too expensive for me".to_string(),
                        leads_to: Some(5),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: Some("Iron Sword".to_string()),
            },
            // Add more dialogue nodes as needed...
        ];
    }

    pub fn get_mood_color(&self, mood: NPCMood) -> Color {
        match mood {
            NPCMood::Friendly => Color::Green,
            NPCMood::Neutral => Color::White,
            NPCMood::Hostile => Color::Red,
            NPCMood::Sad => Color::Blue,
            NPCMood::Excited => Color::Yellow,
            NPCMood::Mysterious => Color::Magenta,
        }
    }

    pub fn get_mood_prefix(&self, mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "> ",
            NPCMood::Neutral => "- ",
            NPCMood::Hostile => "! ",
            NPCMood::Sad => "~ ",
            NPCMood::Excited => "* ",
            NPCMood::Mysterious => "? ",
        }
    }

    pub fn get_mood_face(&self, mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "◡",
            NPCMood::Neutral => "─",
            NPCMood::Hostile => "▼",
            NPCMood::Sad => "︶",
            NPCMood::Excited => "◠",
            NPCMood::Mysterious => "~",
        }
    }

    pub fn get_mood_eyes(&self, mood: NPCMood) -> &'static str {
        match mood {
            NPCMood::Friendly => "◉     ◉",
            NPCMood::Neutral => "○     ○",
            NPCMood::Hostile => "▲     ▲",
            NPCMood::Sad => "◌     ◌",
            NPCMood::Excited => "★     ★",
            NPCMood::Mysterious => "◇     ◇",
        }
    }

    pub fn get_portrait_with_mood(&self, npc: &NPCData) -> String {
        let eyes = self.get_mood_eyes(npc.current_mood);
        let mouth = self.get_mood_face(npc.current_mood);
        let symbol = match npc.dialogue_type {
            DialogueType::Shop => "$",
            DialogueType::Quest => ">",     // Using > instead of unicode
            DialogueType::Branching => "?", // Using ? instead of unicode
            DialogueType::Linear => "@",    // Using @ instead of unicode
            DialogueType::Battle => "X",    // Using X instead of unicode
        };

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

    pub fn get_dialogue_by_id(&self, id: usize) -> Option<&DialogueNode> {
        self.dialogue_tree.iter().find(|d| d.id == id)
    }
}
