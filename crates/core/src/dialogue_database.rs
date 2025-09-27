use crate::components::NPCMood;
use crate::dialogue::{DialogueChoice, DialogueNode, DialogueTree, TextEffect};
use std::collections::HashMap;

/// Central database for all dialogue trees in the game
/// Provides efficient access to dialogue content by NPC/quest ID
pub struct DialogueDatabase {
    trees: HashMap<String, DialogueTree>,
}

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
            text: "<1>Hello</1>... <2>...user detected</2>. I am... <3>...SapienCorp maintenance unit</3>. My work task <4>is ... welding ... today</4>. How <2>can I assist you</2>... ... ... today? \n\n (You gaze at the rusty, flickering lump of metal on the floor. You wonder how it's still running. These construction and maintenance models were released only a few months ago. How is this one so messed up?)".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle, TextEffect::PopHiss],
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
            text: "<1>Systems... failing</1>. Memory core... <2>...damaged</2>. Cannot return to... <3>...base</3>.".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle],
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
            text: "<1>Facility... explosion</1>. Lost contact with... <2>...SapienCorp</2>. Been here... days? Weeks? <3>*transmission degrades*</3>".to_string(),
            text_effects: vec![TextEffect::Buzz, TextEffect::Static, TextEffect::Crackle],
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
            text: "<1>You would... help?</1> Need spare parts... <2>...to repair primary systems</2>. Basic components scattered around facility.".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz],
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
            text: "<1>Simple components</1>... <2>...wood for structural repair</2>. Find some... wood pieces. Will... try to... <3>...stay online</3>.".to_string(),
            text_effects: vec![TextEffect::Crackle, TextEffect::Buzz, TextEffect::Static],
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
            text: "(Obviously wrong) We're in the <1>SapienCorp Factory floor in the welding section</1> <2>(notes visual sensor failure)</2> <3>(notes network connectivity failure)</3>. <4>(notes GPS sensor unable to connect to satellite.)</4> <5>(notes greatly degraded fusion core output.)</5> I've been in need of repairs for <6>999 days</6>.".to_string(),
            text_effects: vec![TextEffect::Glitch, TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle, TextEffect::PopHiss, TextEffect::Corrupt],
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
            text: "<1>Override accepted</1>... <2>...Model SC-M4X7</2>... Maintenance unit... <3>...Primary systems: 15% operational</3>... Secondary systems: offline... <4>...Critical errors in navigation, visual processing, memory core</4>...".to_string(),
            text_effects: vec![TextEffect::Buzz, TextEffect::Static, TextEffect::Crackle, TextEffect::Buzz],
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
