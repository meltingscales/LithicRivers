use crate::components::NPCMood;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub leads_to: Option<usize>, // Index of next dialogue node, None = end conversation
    pub requires_item: Option<String>,
    pub mood_change: Option<NPCMood>,
    pub unlocks_quest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: usize,
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

    pub fn get_node(&self, id: usize) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn add_node(&mut self, node: DialogueNode) {
        self.nodes.push(node);
    }

    // Create a simple test dialogue tree for QuestTesty NPCs
    pub fn create_quest_testy_tree() -> Self {
        let mut tree = Self::new();

        // Initial greeting
        tree.add_node(DialogueNode {
            id: 0,
            speaker: "QuestTesty".to_string(),
            text: "Greetings, traveler! I am QuestTesty, here to test our dialogue system. How are you feeling today?".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "I'm doing great!".to_string(),
                    leads_to: Some(1),
                    requires_item: None,
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "I've been better...".to_string(),
                    leads_to: Some(2),
                    requires_item: None,
                    mood_change: Some(NPCMood::Sad),
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Something strange is happening...".to_string(),
                    leads_to: Some(3),
                    requires_item: None,
                    mood_change: Some(NPCMood::Weird),
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Just passing through.".to_string(),
                    leads_to: Some(4),
                    requires_item: None,
                    mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Happy response
        tree.add_node(DialogueNode {
            id: 1,
            speaker: "QuestTesty".to_string(),
            text: "Wonderful! Your positive energy brightens my day. I have a simple task if you're interested!".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "What kind of task?".to_string(),
                    leads_to: Some(5),
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "Maybe another time.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Sad response
        tree.add_node(DialogueNode {
            id: 2,
            speaker: "QuestTesty".to_string(),
            text: "Oh dear... Life can be challenging. Perhaps I can help lift your spirits with a small quest?".to_string(),
            mood: NPCMood::Sad,
            choices: vec![
                DialogueChoice {
                    text: "That's very kind of you.".to_string(),
                    leads_to: Some(5),
                    requires_item: None,
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "I'd rather be alone right now.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Weird response
        tree.add_node(DialogueNode {
            id: 3,
            speaker: "QuestTesty".to_string(),
            text: "Strange indeed... I sense unusual energies around you. This calls for investigation!".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "Can you help me figure it out?".to_string(),
                    leads_to: Some(6),
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "It's probably nothing.".to_string(),
                    leads_to: Some(4),
                    requires_item: None,
                    mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Neutral response
        tree.add_node(DialogueNode {
            id: 4,
            speaker: "QuestTesty".to_string(),
            text: "I understand. Safe travels, adventurer. May your journey be peaceful."
                .to_string(),
            mood: NPCMood::Neutral,
            choices: vec![DialogueChoice {
                text: "Thank you.".to_string(),
                leads_to: None,
                requires_item: None,
                mood_change: None,
                unlocks_quest: false,
            }],
            auto_continue: false,
            shop_item: None,
        });

        // Quest offer
        tree.add_node(DialogueNode {
            id: 5,
            speaker: "QuestTesty".to_string(),
            text: "I need someone to collect 3 stones from around town. It's simple work, but it would help me greatly!".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "I'll do it!".to_string(),
                    leads_to: Some(7),
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "Sounds too easy. What's the catch?".to_string(),
                    leads_to: Some(8),
                    requires_item: None,
                    mood_change: Some(NPCMood::Weird),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Weird investigation
        tree.add_node(DialogueNode {
            id: 6,
            speaker: "QuestTesty".to_string(),
            text: "The energies... they're connected to the ancient stones scattered around town. Gather them, and we'll uncover the truth!".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "I'm ready for this mystery!".to_string(),
                    leads_to: Some(7),
                    requires_item: None,
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Quest accepted
        tree.add_node(DialogueNode {
            id: 7,
            speaker: "QuestTesty".to_string(),
            text: "Excellent! Return to me when you have collected 3 stones. I'll be right here waiting. Oh, and I also trade acorns for wood if you need some!".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "I'll be back soon!".to_string(),
                    leads_to: Some(9), // Go to trading menu instead of ending
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Wait, you trade acorns for wood?".to_string(),
                    leads_to: Some(10),
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Suspicious response
        tree.add_node(DialogueNode {
            id: 8,
            speaker: "QuestTesty".to_string(),
            text: "Hah! You're perceptive. The truth is... these aren't just any stones. They're test data for our dialogue system!".to_string(),
            mood: NPCMood::Weird,
            choices: vec![
                DialogueChoice {
                    text: "I should have known!".to_string(),
                    leads_to: Some(7),
                    requires_item: None,
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
                DialogueChoice {
                    text: "That's... actually kind of cool.".to_string(),
                    leads_to: Some(7),
                    requires_item: None,
                    mood_change: Some(NPCMood::Happy),
                    unlocks_quest: true,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Trading menu
        tree.add_node(DialogueNode {
            id: 9,
            speaker: "QuestTesty".to_string(),
            text: "Need anything else? I can trade 1 acorn for 1 wood, up to 5 times per conversation. Very fair deal!".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "Trade 1 acorn for 1 wood".to_string(),
                    leads_to: Some(11), // Trading transaction
                    requires_item: Some("Acorn".to_string()),
                    mood_change: None,
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "No thanks, goodbye!".to_string(),
                    leads_to: None,
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Trading explanation
        tree.add_node(DialogueNode {
            id: 10,
            speaker: "QuestTesty".to_string(),
            text: "Indeed! I have an abundance of wood but I'm always short on acorns. Fair trade: 1 acorn gets you 1 wood. I can do this up to 5 times!".to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "That sounds great!".to_string(),
                    leads_to: Some(9), // Go to trading menu
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "Maybe later.".to_string(),
                    leads_to: None,
                    requires_item: None,
                    mood_change: Some(NPCMood::Neutral),
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: None,
        });

        // Trading transaction success
        tree.add_node(DialogueNode {
            id: 11,
            speaker: "QuestTesty".to_string(),
            text:
                "Excellent trade! Here's your wood. I still have more wood if you have more acorns!"
                    .to_string(),
            mood: NPCMood::Happy,
            choices: vec![
                DialogueChoice {
                    text: "Trade another acorn for wood".to_string(),
                    leads_to: Some(11), // Loop back for more trading
                    requires_item: Some("Acorn".to_string()),
                    mood_change: None,
                    unlocks_quest: false,
                },
                DialogueChoice {
                    text: "That's all for now, thanks!".to_string(),
                    leads_to: None,
                    requires_item: None,
                    mood_change: None,
                    unlocks_quest: false,
                },
            ],
            auto_continue: false,
            shop_item: Some("Log".to_string()), // QuestTesty gives wood (Log item)
        });

        tree
    }
}

impl Default for DialogueTree {
    fn default() -> Self {
        Self::new()
    }
}
