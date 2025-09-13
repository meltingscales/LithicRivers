use crate::app_state::{DialogueChoice, DialogueNode, DialogueType, NPCData, NPCMood};

/// Pure dialogue engine - no UI dependencies, easily unit testable
pub struct DialogueEngine {
    npcs: Vec<NPCData>,
    dialogue_tree: Vec<DialogueNode>,
}

/// Represents the current state of a conversation - pure data structure
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationState {
    pub npc_id: usize,
    pub current_node_id: Option<usize>,
    pub selected_choice: usize,
    pub conversation_log: Vec<String>,
    pub npc_mood_changed: bool,
}

/// Result of processing a dialogue choice
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueResult {
    pub next_node_id: Option<usize>, // None = conversation ended
    pub mood_change: Option<NPCMood>,
    pub unlocked_quest: bool,
    pub shop_transaction: Option<String>,
    pub conversation_ended: bool,
}

impl DialogueEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            npcs: Vec::new(),
            dialogue_tree: Vec::new(),
        };
        engine.setup_dialogue_data();
        engine
    }

    /// Start a new conversation with an NPC
    pub fn start_conversation(&self, npc_id: usize) -> Option<ConversationState> {
        if npc_id >= self.npcs.len() {
            return None;
        }

        let npc = &self.npcs[npc_id];
        Some(ConversationState {
            npc_id,
            current_node_id: Some(npc.initial_dialogue),
            selected_choice: 0,
            conversation_log: Vec::new(),
            npc_mood_changed: false,
        })
    }

    /// Process a player's choice and return the result
    pub fn process_choice(
        &self,
        conversation: &ConversationState,
        choice_index: usize,
    ) -> Option<DialogueResult> {
        let current_node_id = conversation.current_node_id?;
        let dialogue_node = self.get_dialogue_by_id(current_node_id)?;

        if choice_index >= dialogue_node.choices.len() {
            return None;
        }

        let choice = &dialogue_node.choices[choice_index];

        Some(DialogueResult {
            next_node_id: choice.leads_to,
            mood_change: choice.mood_change,
            unlocked_quest: choice.unlocks_quest,
            shop_transaction: dialogue_node.shop_item.clone(),
            conversation_ended: choice.leads_to.is_none(),
        })
    }

    /// Get the current dialogue node
    pub fn get_current_node(&self, conversation: &ConversationState) -> Option<&DialogueNode> {
        let node_id = conversation.current_node_id?;
        self.get_dialogue_by_id(node_id)
    }

    /// Get NPC data by ID
    pub fn get_npc(&self, npc_id: usize) -> Option<&NPCData> {
        self.npcs.get(npc_id)
    }

    /// Get NPC data by name
    pub fn find_npc_by_name(&self, name: &str) -> Option<(usize, &NPCData)> {
        self.npcs
            .iter()
            .enumerate()
            .find(|(_, npc)| npc.name == name)
    }

    /// Update NPC mood (mutable operation)
    pub fn update_npc_mood(&mut self, npc_id: usize, new_mood: NPCMood) -> bool {
        if let Some(npc) = self.npcs.get_mut(npc_id) {
            npc.current_mood = new_mood;
            true
        } else {
            false
        }
    }

    /// Get all available NPCs
    pub fn get_all_npcs(&self) -> &[NPCData] {
        &self.npcs
    }

    /// Private helper to find dialogue by ID
    fn get_dialogue_by_id(&self, id: usize) -> Option<&DialogueNode> {
        self.dialogue_tree.iter().find(|d| d.id == id)
    }

    /// Initialize dialogue data (same as before, but separated)
    fn setup_dialogue_data(&mut self) {
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
    ║   │ > │   ║
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
            // Add more NPCs as needed...
        ];

        // Create dialogue tree (abbreviated for example)
        self.dialogue_tree = vec![
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
                        leads_to: None,
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
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: Some("Iron Sword".to_string()),
            },
            // Add more dialogue nodes as needed...
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_conversation() {
        let engine = DialogueEngine::new();
        let conversation = engine.start_conversation(0).unwrap();

        assert_eq!(conversation.npc_id, 0);
        assert_eq!(conversation.current_node_id, Some(0));
        assert_eq!(conversation.selected_choice, 0);
        assert!(conversation.conversation_log.is_empty());
    }

    #[test]
    fn test_invalid_npc_id() {
        let engine = DialogueEngine::new();
        let conversation = engine.start_conversation(999);

        assert!(conversation.is_none());
    }

    #[test]
    fn test_process_choice() {
        let engine = DialogueEngine::new();
        let conversation = engine.start_conversation(0).unwrap();

        // Test first choice (weapons)
        let result = engine.process_choice(&conversation, 0).unwrap();
        assert_eq!(result.next_node_id, Some(1));
        assert_eq!(result.mood_change, None);
        assert!(!result.unlocked_quest);

        // Test third choice (browsing - ends conversation)
        let result = engine.process_choice(&conversation, 2).unwrap();
        assert_eq!(result.next_node_id, None);
        assert!(result.conversation_ended);
    }

    #[test]
    fn test_get_current_node() {
        let engine = DialogueEngine::new();
        let conversation = engine.start_conversation(0).unwrap();

        let node = engine.get_current_node(&conversation).unwrap();
        assert_eq!(node.id, 0);
        assert_eq!(node.speaker, "Merchant Aldric");
        assert_eq!(node.choices.len(), 3);
    }

    #[test]
    fn test_find_npc_by_name() {
        let engine = DialogueEngine::new();
        let (id, npc) = engine.find_npc_by_name("Merchant Aldric").unwrap();

        assert_eq!(id, 0);
        assert_eq!(npc.name, "Merchant Aldric");
        assert_eq!(npc.dialogue_type, DialogueType::Shop);
    }

    #[test]
    fn test_update_npc_mood() {
        let mut engine = DialogueEngine::new();

        // Update mood
        assert!(engine.update_npc_mood(0, NPCMood::Excited));

        // Verify mood changed
        let npc = engine.get_npc(0).unwrap();
        assert_eq!(npc.current_mood, NPCMood::Excited);

        // Test invalid NPC ID
        assert!(!engine.update_npc_mood(999, NPCMood::Sad));
    }
}
