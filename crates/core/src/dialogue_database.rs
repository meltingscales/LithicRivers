use crate::components::NPCMood;
use crate::dialogue::{
    DialogueChoice, DialogueNode, DialogueNodeID, DialogueTree, QuestType, TextEffect,
};
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
            id: DialogueNodeID::Start,
            speaker: "Broken Android".to_string(),
            text: "<1>Hello</1>... <2>...user detected</2>. I am... <3>...SapienCorp maintenance unit</3>. My work task <4>is ... welding ... today</4>. How <2>can I assist you</2>... ... ... <5>today</5>? \n\n (You gaze at the rusty, flickering lump of metal on the floor. You wonder how it's still running. These construction and maintenance models were released only a few months ago. How is this one so messed up?)".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle, TextEffect::PopHiss, TextEffect::Corrupt],
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Are you alright?".to_string(),
                    leads_to: Some(DialogueNodeID::AreYouAlright),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: Some(NPCMood::Neutral), // Player shows concern
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "What happened to you?".to_string(),
                    leads_to: Some(DialogueNodeID::WhatHappened),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: Some(NPCMood::Weird), // Player is curious
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "Can I help?".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy), // Player feels helpful/compassionate
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "Where are we right now? Why is it so dark?".to_string(),
                    leads_to: Some(DialogueNodeID::WhereAreWe),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Neutral), // Player is confused/seeking info
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "Override Alpha-7: Emergency diagnostic mode. State model number and diagnostic info.".to_string(),
                    leads_to: Some(DialogueNodeID::OverrideModelNumber),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Weird),
                    player_mood_change: Some(NPCMood::Weird), // Player is being technical/commanding
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "I have the diamond and scrap electronics you need.".to_string(),
                    leads_to: Some(DialogueNodeID::CompleteQuest),
                    requires_item: Some("lab-grown diamond".to_string()),
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "Are you alright?"
        tree.add_node(DialogueNode {
            id: DialogueNodeID::AreYouAlright,
            speaker: "Broken Android".to_string(),
            text: "<1>Systems... failing</1>. Memory core... <2>...damaged</2>. Cannot return to... <3>...base</3>.".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle],
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "Maybe I can help repair you.".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "I'm sorry, I have to go.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Sad),
                    player_mood_change: Some(NPCMood::Sad),
                    unlocks_quest: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "What happened to you?"
        tree.add_node(DialogueNode {
            id: DialogueNodeID::WhatHappened,
            speaker: "Broken Android".to_string(),
            text: "<1>Facility... explosion</1>. Lost contact with... <2>...SapienCorp</2>. Been here... days? Weeks? <3>*transmission degrades*</3>".to_string(),
            text_effects: vec![TextEffect::Buzz, TextEffect::Static, TextEffect::Crackle],
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "I'll help you get back online.".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest offer
        tree.add_node(DialogueNode {
            id: DialogueNodeID::CanIHelp,
            speaker: "Broken Android".to_string(),
            text: "<1>You would... help?</1> Need spare parts... <2>...to repair primary systems</2>. Basic components scattered around facility.".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz],
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "What do you need?".to_string(),
                    leads_to: Some(DialogueNodeID::WhatDoYouNeed),
                    requires_item: None,
                    npc_mood_change: None,
                    player_mood_change: None,
                    unlocks_quest: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest details
        tree.add_node(DialogueNode {
            id: DialogueNodeID::WhatDoYouNeed,
            speaker: "Broken Android".to_string(),
            text: "<1>Simple components</1>... <2>...a lab-grown diamond for my fusion reactor...and scrap electronics...</2>. Will... try to... <3>...stay online</3>.".to_string(),
            text_effects: vec![TextEffect::Crackle, TextEffect::Buzz, TextEffect::Static],
            mood: NPCMood::Neutral,
            choices: vec![
                DialogueChoice {
                    text: "I'll find a diamond and scrap electronics for you.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: Some(QuestType::RepairBrokenAndroid),
                },
                DialogueChoice {
                    text: "I can't do that right now.".to_string(),
                    leads_to: Some(DialogueNodeID::Start),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Sad),
                    player_mood_change: Some(NPCMood::Sad),
                    unlocks_quest:None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to "Where are we?"
        tree.add_node(DialogueNode {
            id: DialogueNodeID::WhereAreWe,
            speaker: "Broken Android".to_string(),
            text: "<1>Location... SapienCorp Factory</1>... <2>...welding section</2>. Dark because... <3>...power grid failure</3>... I think? <4>Sensors... malfunctioning</4>... cannot confirm... <5>...been here so long</5>... <6>999 days</6>? Or... was it... 9 days?".to_string(),
            text_effects: vec![TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt, TextEffect::Corrupt],
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "... [say nothing] (This poor thing has clearly got a few screws loose...)".to_string(),
                    leads_to: Some(DialogueNodeID::Start),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Response to override command
        tree.add_node(DialogueNode {
            id: DialogueNodeID::OverrideModelNumber,
            speaker: "Broken Android".to_string(),
            text: "<1>Override accepted</1>... <2>...Model SC-M4X7</2>... Maintenance unit... <3>...Primary systems: 15% operational</3>... Secondary systems: offline... <4>...Critical errors in navigation, visual processing, memory core</4>...".to_string(),
            text_effects: vec![TextEffect::Buzz, TextEffect::Static, TextEffect::Crackle, TextEffect::Buzz],
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Understood. Can you be repaired?".to_string(),
                    leads_to: Some(DialogueNodeID::CanIHelp),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "Thank you for the diagnostic. Return to standard mode.".to_string(),
                    leads_to: Some(DialogueNodeID::Start),
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest completion dialogue
        tree.add_node(DialogueNode {
            id: DialogueNodeID::CompleteQuest,
            speaker: "Broken Android".to_string(),
            text: "<1>*scanning items*</1>... Lab-grown diamond detected... Scrap electronics detected... <2>*systems activating*</2>... Thank you, user! <3>Initiating repair protocol</3>... Memory core stabilizing... <4>*mechanical whirring sounds*</4>... <5>Repair successful!</5> I am now operational. SapienCorp maintenance unit SC-M4X7 reporting for duty. How may I assist the facility today?".to_string(),
            text_effects: vec![TextEffect::Static, TextEffect::Buzz, TextEffect::Crackle, TextEffect::PopHiss, TextEffect::Fade],
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "You're welcome! Glad I could help.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Happy),
                    player_mood_change: Some(NPCMood::Happy),
                    unlocks_quest: None,
                },
                DialogueChoice {
                    text: "What will you do now?".to_string(),
                    leads_to: None,
                    requires_item: None,
                    npc_mood_change: Some(NPCMood::Neutral),
                    player_mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: None,
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
