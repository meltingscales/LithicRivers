use crate::components::NPCMood;
use crate::dialogue::{DialogueChoice, DialogueNode, DialogueTree};
use std::collections::HashMap;

/// Central database for all dialogue trees in the game
/// Provides efficient access to dialogue content by NPC/quest ID
pub struct DialogueDatabase {
    trees: HashMap<String, DialogueTree>,
}

// TODO: ask ai - let's also add a TextEffect system where <Bracketed text> will have effects put into it, like random corruption. How hard would that be to do? I want to do that instead of adding random asterisk texts to our dialogue trees.

impl DialogueDatabase {
    /// Create a new dialogue database and load all dialogue trees
    pub fn new() -> Self {
        let mut database = Self {
            trees: HashMap::new(),
        };

        // Load all dialogue trees
        database.load_all_trees();
        database
    }

    /// Get a dialogue tree by ID
    pub fn get_tree(&self, tree_id: &str) -> Option<&DialogueTree> {
        self.trees.get(tree_id)
    }

    /// Get all available dialogue tree IDs
    pub fn get_tree_ids(&self) -> Vec<&String> {
        self.trees.keys().collect()
    }

    /// Load all dialogue trees into the database
    fn load_all_trees(&mut self) {
        // Tutorial quest NPCs
        self.trees.insert(
            "broken_android_tutorial".to_string(),
            Self::create_quest_tutorial_broken_android_tree(),
        );

        // Future dialogue trees will be added here:
        // self.trees.insert("merchant_basic".to_string(), Self::create_merchant_basic_tree());
        // self.trees.insert("guard_hostile".to_string(), Self::create_guard_hostile_tree());
        // self.trees.insert("scientist_friendly".to_string(), Self::create_scientist_friendly_tree());
    }

    /// Create dialogue tree for the broken SapienCorp android tutorial
    fn create_quest_tutorial_broken_android_tree() -> DialogueTree {
        let mut tree = DialogueTree::new();

        // Initial greeting
        tree.add_node(DialogueNode {
            id: "start".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*static* Hello... *bzzt* ...user detected. I am... *crackle* ...SapienCorp maintenance unit. My work task *pop-hiss* is ... welding ... today. How *bzzt* can I assist you... ... ... today? \n\n (You gaze at the rusty, flickering lump of metal on the floor. You wonder how it's still running. These construction and maintenance models were released only a few months ago. How is this one so messed up?)".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Are you alright?".to_string(),
                    leads_to: Some("are-you-alright".to_string()),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: Some(NPCMood::Neutral), // Player shows concern
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "What happened to you?".to_string(),
                    leads_to: Some("what-happened".to_string()),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: Some(NPCMood::Weird), // Player is curious
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Can I help?".to_string(),
                    leads_to: Some("can-i-help".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy), // Player feels helpful/compassionate
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "Where are we right now? Why is it so dark?".to_string(),
                    leads_to: Some("where-are-we".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Neutral), // Player is confused/seeking info
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Override Alpha-7: Emergency diagnostic mode. State model number and diagnostic info.".to_string(),
                    leads_to: Some("override-model-number".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Weird),
                    player_mood_change: Some(NPCMood::Weird), // Player is being technical/commanding
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "Are you alright?"
        tree.add_node(DialogueNode {
            id: "are-you-alright".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*static* Systems... failing. Memory core... *bzzt* ...damaged. Cannot return to... *crackle* ...base.".to_string(),
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "Maybe I can help repair you.".to_string(),
                    leads_to: Some("can-i-help".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "I'm sorry, I have to go.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Sad),
                    player_mood_change: Some(NPCMood::Sad),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "What happened to you?"
        tree.add_node(DialogueNode {
            id: "what-happened".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*bzzt* Facility... explosion. Lost contact with... *static* ...SapienCorp. Been here... days? Weeks? *crackle*".to_string(),
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "I'll help you get back online.".to_string(),
                    leads_to: Some("can-i-help".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest offer
        tree.add_node(DialogueNode {
            id: "can-i-help".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*static* You would... help? Need spare parts... *bzzt* ...to repair primary systems. Basic components scattered around facility.".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "What do you need?".to_string(),
                    leads_to: Some("what-do-you-need".to_string()),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: None,
                    unlocks_quest: true,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest details
        tree.add_node(DialogueNode {
            id: "what-do-you-need".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*crackle* Simple components... *bzzt* ...wood for structural repair. Find some... wood pieces. Will... try to... *static* ...stay online.".to_string(),
            mood: NPCMood::Neutral,
            choices: vec![
                DialogueChoice {
                    text: "I'll find some wood for you.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "Where are we?"
        tree.add_node(DialogueNode {
            id: "where-are-we".to_string(),
            speaker: "Broken Android".to_string(),
            text: "(Obviously wrong) We're in the SapienCorp Factory floor in the welding section (notes visual sensor failure) (notes network connectivity failure). (notes GPS sensor unable to connect to satellite.) (notes greatly degraded fusion core output.) I've been in need of repairs for 999 days.".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "... [say nothing] (This poor thing has clearly got a few screws loose...)".to_string(),
                    leads_to: Some("start".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to override command
        tree.add_node(DialogueNode {
            id: "override-model-number".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*bzzt* Override accepted... *static* ...Model SC-M4X7... Maintenance unit... *crackle* ...Primary systems: 15% operational... Secondary systems: offline... *pop* ...Critical errors in navigation, visual processing, memory core... *bzzt*".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Understood. Can you be repaired?".to_string(),
                    leads_to: Some("can-i-help".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "Thank you for the diagnostic. Return to standard mode.".to_string(),
                    leads_to: Some("start".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to override command
        tree.add_node(DialogueNode {
            id: "override-model-number".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*bzzt* Override accepted... *static* ...Model SC-M4X7... Maintenance unit... *crackle* ...Primary systems: 15% operational... Secondary systems: offline... *pop* ...Critical errors in navigation, visual processing, memory core... *bzzt*".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Understood. Can you be repaired?".to_string(),
                    leads_to: Some("can-i-help".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "Thank you for the diagnostic. Return to standard mode.".to_string(),
                    leads_to: Some("start".to_string()),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        tree
    }
}

impl Default for DialogueDatabase {
    fn default() -> Self {
        Self::new()
    }
}
