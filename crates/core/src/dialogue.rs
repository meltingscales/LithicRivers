use crate::components::NPCMood;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum QuestType {
    RepairBrokenAndroid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestState {
    NotStarted,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveQuest {
    pub quest_type: QuestType,
    pub state: QuestState,
    pub name: String,
    pub description: String,
    pub objectives: Vec<QuestObjective>,
    pub rewards: Vec<String>, // Item names for now
    #[serde(skip)]
    pub npc_entity: Option<hecs::Entity>, // Entity that gave the quest
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestObjective {
    pub description: String,
    pub completed: bool,
    pub objective_type: QuestObjectiveType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjectiveType {
    FetchItem { item_name: String, quantity: u32 },
    TalkToNPC { npc_name: String },
    GoToLocation { x: i64, y: i64, z: i64 },
}

impl ActiveQuest {
    pub fn new(
        quest_type: QuestType,
        name: String,
        description: String,
        objectives: Vec<QuestObjective>,
        npc_entity: Option<hecs::Entity>,
    ) -> Self {
        Self {
            quest_type,
            state: QuestState::Active,
            name,
            description,
            objectives,
            rewards: Vec::new(),
            npc_entity,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.objectives.iter().all(|obj| obj.completed)
    }

    pub fn complete_objective(&mut self, index: usize) {
        if let Some(objective) = self.objectives.get_mut(index) {
            objective.completed = true;
        }

        // Auto-complete quest if all objectives are done
        if self.is_completed() && self.state == QuestState::Active {
            self.state = QuestState::Completed;
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DialogueNodeID {
    Start,
    AreYouAlright,
    WhatHappened,
    CanIHelp,
    WhatDoYouNeed,
    WhereAreWe,
    OverrideModelNumber,
    CompleteQuest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextEffect {
    Static,
    Glitch,
    Corrupt,
    Fade,
    Buzz,
    Crackle,
    PopHiss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueChoice {
    pub text: String,
    pub leads_to: Option<DialogueNodeID>, // ID of next dialogue node, None = end conversation
    pub requires_item: Option<String>,
    pub npc_mood_change: Option<NPCMood>, //Does this choice change NPC mood?
    pub player_mood_change: Option<NPCMood>, //Does this choice change player mood?
    pub unlocks_quest: Option<QuestType>, // Quest to unlock, None = no quest
    pub requires_quest_active: Option<QuestType>, // Only show if this quest is active
    pub requires_quest_complete: Option<QuestType>, // Only show if this quest is completed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueNode {
    pub id: DialogueNodeID,
    pub speaker: String,
    pub text: String,
    pub text_effects: Vec<TextEffect>,
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

    pub fn get_node(&self, id: &DialogueNodeID) -> Option<&DialogueNode> {
        self.nodes.iter().find(|n| n.id == *id)
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

impl DialogueNode {
    pub fn parse_text_with_effects(text: &str, effects: &[TextEffect]) -> String {
        let mut result = text.to_string();

        for (i, _effect) in effects.iter().enumerate() {
            let pattern = format!("<{}>", i + 1);
            let close_pattern = format!("</{}>", i + 1);
            result = result.replace(&pattern, "").replace(&close_pattern, "");
        }

        result
    }

    pub fn extract_effect_ranges(text: &str) -> Vec<(usize, usize, usize)> {
        let mut ranges = Vec::new();
        let mut effect_num = 1;

        while let Some(start_pos) = text.find(&format!("<{}>", effect_num)) {
            if let Some(end_pos) = text.find(&format!("</{}>", effect_num)) {
                let content_start = start_pos + format!("<{}>", effect_num).len();
                ranges.push((effect_num - 1, content_start, end_pos));
                effect_num += 1;
            } else {
                break;
            }
        }

        ranges
    }
}
