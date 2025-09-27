use crate::components::NPCMood;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub leads_to: Option<String>, // ID of next dialogue node, None = end conversation
    pub requires_item: Option<String>,
    pub npc_mood_change: Option<NPCMood>, //Does this choice change NPC mood?
    pub player_mood_change: Option<NPCMood>, //Does this choice change player mood?
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
}

impl Default for DialogueTree {
    fn default() -> Self {
        Self::new()
    }
}
