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
    #[allow(dead_code)]
    pub fn get_npc(&self, npc_id: usize) -> Option<&NPCData> {
        self.npcs.get(npc_id)
    }

    /// Get NPC data by name
    #[allow(dead_code)]
    pub fn find_npc_by_name(&self, name: &str) -> Option<(usize, &NPCData)> {
        self.npcs
            .iter()
            .enumerate()
            .find(|(_, npc)| npc.name == name)
    }

    /// Update NPC mood (mutable operation)
    #[allow(dead_code)]
    pub fn update_npc_mood(&mut self, npc_id: usize, new_mood: NPCMood) -> bool {
        if let Some(npc) = self.npcs.get_mut(npc_id) {
            npc.current_mood = new_mood;
            true
        } else {
            false
        }
    }

    /// Get all available NPCs
    #[allow(dead_code)]
    pub fn get_all_npcs(&self) -> &[NPCData] {
        &self.npcs
    }

    /// Private helper to find dialogue by ID
    fn get_dialogue_by_id(&self, id: usize) -> Option<&DialogueNode> {
        self.dialogue_tree.iter().find(|d| d.id == id)
    }

    /// Initialize dialogue data (same as before, but separated)
    fn setup_dialogue_data(&mut self) {
        // Create NPCs - QuestTesty only for testing
        self.npcs = vec![NPCData {
            name: "QuestTesty".to_string(),
            portrait: "".to_string(), // Will use sprite loader
            dialogue_type: DialogueType::Quest,
            current_mood: NPCMood::Friendly,
            initial_dialogue: 20,
            met_before: false,
            has_quest: true,
            shop_inventory: Vec::new(),
        }];
        // Create dialogue tree - QuestTesty only
        self.dialogue_tree = vec![
            // === QuestTesty Dialogue Tree (Node 20+) ===
            DialogueNode {
                id: 20,
                speaker: "QuestTesty".to_string(),
                text: "Greetings, brave traveler! I am QuestTesty, guardian of ancient mysteries. I sense great potential in you...".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Tell me about these mysteries".to_string(),
                        leads_to: Some(21),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "I'm looking for adventure!".to_string(),
                        leads_to: Some(22),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "You look suspicious...".to_string(),
                        leads_to: Some(23),
                        requires_item: None,
                        mood_change: Some(NPCMood::Sad),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Not interested, goodbye".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 21 - Mysterious/Neutral
            DialogueNode {
                id: 21,
                speaker: "QuestTesty".to_string(),
                text: "Ah, a seeker of knowledge... The ancient ruins hold secrets that could change the very fabric of reality. But knowledge comes with a price...".to_string(),
                mood: NPCMood::Mysterious,
                choices: vec![
                    DialogueChoice {
                        text: "What kind of price?".to_string(),
                        leads_to: Some(24),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "I'm ready for anything!".to_string(),
                        leads_to: Some(25),
                        requires_item: None,
                        mood_change: Some(NPCMood::Excited),
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "This sounds too dangerous".to_string(),
                        leads_to: Some(26),
                        requires_item: None,
                        mood_change: Some(NPCMood::Sad),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 22 - Excited
            DialogueNode {
                id: 22,
                speaker: "QuestTesty".to_string(),
                text: "YES! That's the spirit I was hoping to find! Adventure awaits in the depths of the Crystal Caverns! Monsters, treasures, glory!".to_string(),
                mood: NPCMood::Excited,
                choices: vec![
                    DialogueChoice {
                        text: "Where are these caverns?".to_string(),
                        leads_to: Some(27),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "What kind of monsters?".to_string(),
                        leads_to: Some(28),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Maybe I should prepare first...".to_string(),
                        leads_to: Some(29),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 23 - Sad
            DialogueNode {
                id: 23,
                speaker: "QuestTesty".to_string(),
                text: "*sighs deeply* I suppose I can't blame you for your suspicion. Many have called me strange, eccentric... but I only want to help...".to_string(),
                mood: NPCMood::Sad,
                choices: vec![
                    DialogueChoice {
                        text: "I'm sorry, I didn't mean to offend".to_string(),
                        leads_to: Some(30),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Prove you're trustworthy".to_string(),
                        leads_to: Some(31),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "My instincts are usually right".to_string(),
                        leads_to: Some(32),
                        requires_item: None,
                        mood_change: Some(NPCMood::Hostile),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 24 - Neutral/Cautious
            DialogueNode {
                id: 24,
                speaker: "QuestTesty".to_string(),
                text: "Wisdom before valor. The price is simple: you must be willing to face your deepest fears. The ruins test not just strength, but character.".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "I accept the challenge".to_string(),
                        leads_to: Some(25),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "What exactly will I face?".to_string(),
                        leads_to: Some(33),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 25 - Happy/Proud
            DialogueNode {
                id: 25,
                speaker: "QuestTesty".to_string(),
                text: "Excellent! I knew I sensed true courage in you! Take this map - it will guide you to the Sunken Temple. May fortune favor your quest!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Thank you for trusting me".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "Any advice for the journey?".to_string(),
                        leads_to: Some(34),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: true,
                    },
                ],
                auto_continue: false,
                shop_item: Some("Ancient Map".to_string()),
            },
            // Node 26 - Understanding but disappointed
            DialogueNode {
                id: 26,
                speaker: "QuestTesty".to_string(),
                text: "I understand your caution. Perhaps when you're ready, you'll return. The mysteries will wait... they always do...".to_string(),
                mood: NPCMood::Sad,
                choices: vec![
                    DialogueChoice {
                        text: "Maybe someday I'll be ready".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Actually, tell me more first".to_string(),
                        leads_to: Some(21),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 30 - Forgiveness/Happy
            DialogueNode {
                id: 30,
                speaker: "QuestTesty".to_string(),
                text: "Your apology shows wisdom and kindness. Few are brave enough to admit when they've misjudged. Perhaps you ARE the hero I've been waiting for!".to_string(),
                mood: NPCMood::Friendly,
                choices: vec![
                    DialogueChoice {
                        text: "Tell me about this quest".to_string(),
                        leads_to: Some(21),
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "I'd like to make it up to you".to_string(),
                        leads_to: Some(25),
                        requires_item: None,
                        mood_change: Some(NPCMood::Excited),
                        unlocks_quest: true,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 32 - Hostile/Angry
            DialogueNode {
                id: 32,
                speaker: "QuestTesty".to_string(),
                text: "So be it! Your 'instincts' have cost you the adventure of a lifetime. Others will claim the glory that could have been yours!".to_string(),
                mood: NPCMood::Hostile,
                choices: vec![
                    DialogueChoice {
                        text: "Wait, I changed my mind!".to_string(),
                        leads_to: Some(35),
                        requires_item: None,
                        mood_change: Some(NPCMood::Neutral),
                        unlocks_quest: false,
                    },
                    DialogueChoice {
                        text: "Good riddance".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: None,
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
            // Node 35 - Reluctant second chance
            DialogueNode {
                id: 35,
                speaker: "QuestTesty".to_string(),
                text: "Hmph. Perhaps you're not entirely hopeless after all. But trust must be earned back. Are you truly committed this time?".to_string(),
                mood: NPCMood::Neutral,
                choices: vec![
                    DialogueChoice {
                        text: "Yes, I swear it!".to_string(),
                        leads_to: Some(25),
                        requires_item: None,
                        mood_change: Some(NPCMood::Friendly),
                        unlocks_quest: true,
                    },
                    DialogueChoice {
                        text: "Never mind, I'll leave".to_string(),
                        leads_to: None,
                        requires_item: None,
                        mood_change: Some(NPCMood::Hostile),
                        unlocks_quest: false,
                    },
                ],
                auto_continue: false,
                shop_item: None,
            },
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
        assert_eq!(conversation.current_node_id, Some(20));
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

        // Test first choice (mysteries)
        let result = engine.process_choice(&conversation, 0).unwrap();
        assert_eq!(result.next_node_id, Some(21));
        assert_eq!(result.mood_change, None);
        assert!(!result.unlocked_quest);

        // Test fourth choice (not interested - ends conversation)
        let result = engine.process_choice(&conversation, 3).unwrap();
        assert_eq!(result.next_node_id, None);
        assert!(result.conversation_ended);
    }

    #[test]
    fn test_get_current_node() {
        let engine = DialogueEngine::new();
        let conversation = engine.start_conversation(0).unwrap();

        let node = engine.get_current_node(&conversation).unwrap();
        assert_eq!(node.id, 20);
        assert_eq!(node.speaker, "QuestTesty");
        assert_eq!(node.choices.len(), 4);
    }

    #[test]
    fn test_find_npc_by_name() {
        let engine = DialogueEngine::new();
        let (id, npc) = engine.find_npc_by_name("QuestTesty").unwrap();

        assert_eq!(id, 0);
        assert_eq!(npc.name, "QuestTesty");
        assert_eq!(npc.dialogue_type, DialogueType::Quest);
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
