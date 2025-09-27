use crate::components::NPCMood;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub leads_to: Option<String>, // ID of next dialogue node, None = end conversation
    pub requires_item: Option<String>,
    pub mood_change: Option<NPCMood>,
    pub unlocks_quest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: String,
    pub speaker: String,
    pub text: String,
    pub mood: NPCMood,
    pub choices: Vec<DialogueChoice>,
    pub auto_continue: bool, // If true, automatically continues without player input
    pub shop_item: Option<String>, // If set, this node offers to sell/trade this item
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueTree {
    pub nodes: Vec<DialogueNode>,
}

impl DialogueTree {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn get_node(&self, id: &str) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.push(node);
    }

    // Create dialogue tree for the broken SapienCorp android tutorial
    pub fn create_quest_tutorial_broken_android_tree() -> Self {
        let mut tree = Self::new();

        // Initial greeting
        tree.add_node(DialogueNode {
            id: "start".to_string(),
            speaker: "Broken Android".to_string(),
            text: "*static* Hello... *bzzt* ...user detected. I am... *crackle* ...SapienCorp maintenance unit.".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Are you alright?".to_string(),
                    leads_to: Some("are-you-alright".to_string()),
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "What happened to you?".to_string(),
                    leads_to: Some("what-happened".to_string()),
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Can I help?".to_string(),
                    leads_to: Some("can-i-help".to_string()),
                    requires_item: None,
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
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
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "I'm sorry, I have to go.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    mood_change: Some(NPCMood::Sad),
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
                    mood_change: Some(NPCMood::Happy),
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
                    mood_change: None,
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
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        tree
    }
}

impl Default for DialogueTree {
    fn default() -> Self {
        Self::new()
    }
}
