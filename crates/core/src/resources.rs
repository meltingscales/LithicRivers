use crate::dialogue::{ActiveQuest, QuestType};
use crate::game_config::GameConfig;
use crate::game_events::GameEvents;
use crate::game_time::GameTime;
use crate::player_state::PlayerState;
use crate::target_tracker::TargetTracker;
use crate::world_state::WorldState;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum StructureGenerationState {
    NotGenerated,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ChunkGenerationState {
    NotGenerated,
    Generated,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingStructure {
    pub name: String,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub bury_structure: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestMarker {
    pub name: String,
    pub description: String,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub marker_type: QuestMarkerType,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QuestMarkerType {
    MainQuest,
    SideQuest,
    FetchQuest,
    Location,
    Treasure,
}

pub struct Resources {
    pub time: GameTime,
    pub world_state: WorldState,
    pub config: GameConfig,
    pub player_state: PlayerState,
    pub events: GameEvents,
    pub target_tracker: TargetTracker,
    pub pending_structures: Vec<PendingStructure>,
    pub chunk_generation_states: HashMap<(i64, i64, i64), ChunkGenerationState>,
    pub structure_generation_states: HashMap<String, StructureGenerationState>,
    pub quest_markers: Vec<QuestMarker>,
    pub active_quests: Vec<ActiveQuest>,
    pub explored_chunks: HashMap<(i64, i64, i64), bool>,
}

impl Resources {
    pub fn new(seed: u64) -> Self {
        Self {
            time: GameTime::new(),
            world_state: WorldState::new(seed),
            config: GameConfig::new(),
            player_state: PlayerState::new(),
            events: GameEvents::new(),
            target_tracker: TargetTracker::new(),
            pending_structures: Vec::new(),
            chunk_generation_states: HashMap::new(),
            structure_generation_states: HashMap::new(),
            quest_markers: Vec::new(),
            active_quests: Vec::new(),
            explored_chunks: HashMap::new(),
        }
    }

    /// Log a game event (backwards compatibility)
    pub fn log<S: Into<String>>(&mut self, msg: S) {
        self.events.game_event(msg, self.time.tick);
    }

    /// Log a game event with color
    pub fn log_colored<S: Into<String>>(
        &mut self,
        msg: S,
        color: crate::message_log::MessageColor,
    ) {
        self.events.game_event_colored(msg, self.time.tick, color);
    }

    /// Log a red message (for warnings and errors)
    pub fn log_red<S: Into<String>>(&mut self, msg: S) {
        self.log_colored(msg, crate::message_log::MessageColor::Red);
    }

    /// Log a yellow message (for warnings)
    pub fn log_yellow<S: Into<String>>(&mut self, msg: S) {
        self.log_colored(msg, crate::message_log::MessageColor::Yellow);
    }

    /// Log a green message (for success/positive events)
    pub fn log_green<S: Into<String>>(&mut self, msg: S) {
        self.log_colored(msg, crate::message_log::MessageColor::Green);
    }

    /// Add a quest marker to the world
    pub fn add_quest_marker(&mut self, marker: QuestMarker) {
        self.quest_markers.push(marker);
    }

    /// Start a new quest
    pub fn start_quest(&mut self, quest: ActiveQuest) {
        // Check if quest is already active to prevent duplicates
        if !self
            .active_quests
            .iter()
            .any(|q| q.quest_type == quest.quest_type)
        {
            self.active_quests.push(quest);
            self.log(format!(
                "Quest started: {}",
                self.active_quests.last().unwrap().name
            ));
        }
    }

    /// Start a new quest and check existing inventory for completion
    pub fn start_quest_with_inventory_check(
        &mut self,
        quest: ActiveQuest,
        player_inventory: &crate::components::Inventory,
    ) {
        // Check if quest is already active to prevent duplicates
        if !self
            .active_quests
            .iter()
            .any(|q| q.quest_type == quest.quest_type)
        {
            self.active_quests.push(quest);
            self.log(format!(
                "Quest started: {}",
                self.active_quests.last().unwrap().name
            ));

            // Check existing inventory for quest objective completion
            self.update_quest_objectives_from_inventory(player_inventory);
        }
    }

    /// Get active quest by type
    pub fn get_active_quest(&self, quest_type: QuestType) -> Option<&ActiveQuest> {
        self.active_quests
            .iter()
            .find(|q| q.quest_type == quest_type)
    }

    /// Get mutable active quest by type
    pub fn get_active_quest_mut(&mut self, quest_type: QuestType) -> Option<&mut ActiveQuest> {
        self.active_quests
            .iter_mut()
            .find(|q| q.quest_type == quest_type)
    }

    /// Complete a quest and remove its marker
    pub fn complete_quest(&mut self, quest_type: QuestType) {
        let quest_name = if let Some(quest) = self.get_active_quest_mut(quest_type) {
            quest.state = crate::dialogue::QuestState::Completed;
            quest.name.clone()
        } else {
            return;
        };

        self.log(format!("Quest completed: {}", quest_name));

        // Remove associated quest markers
        self.quest_markers.retain(|marker| {
            // Remove markers associated with this quest type
            // For now, we'll remove FetchQuest markers when RepairBrokenAndroid completes
            match quest_type {
                QuestType::RepairBrokenAndroid => marker.marker_type != QuestMarkerType::FetchQuest,
            }
        });
    }

    /// Check if a quest is active
    pub fn is_quest_active(&self, quest_type: QuestType) -> bool {
        self.active_quests
            .iter()
            .any(|q| q.quest_type == quest_type && q.state == crate::dialogue::QuestState::Active)
    }

    /// Check if a quest is completed
    pub fn is_quest_completed(&self, quest_type: QuestType) -> bool {
        self.active_quests.iter().any(|q| {
            q.quest_type == quest_type && q.state == crate::dialogue::QuestState::Completed
        })
    }

    /// Mark a chunk as explored
    pub fn mark_chunk_explored(&mut self, chunk_x: i64, chunk_y: i64, chunk_z: i64) {
        self.explored_chunks
            .insert((chunk_x, chunk_y, chunk_z), true);
    }

    /// Check if a chunk has been explored
    pub fn is_chunk_explored(&self, chunk_x: i64, chunk_y: i64, chunk_z: i64) -> bool {
        self.explored_chunks
            .get(&(chunk_x, chunk_y, chunk_z))
            .unwrap_or(&false)
            .clone()
    }

    /// Update quest objectives based on current player inventory
    pub fn update_quest_objectives_from_inventory(
        &mut self,
        player_inventory: &crate::components::Inventory,
    ) {
        let mut completed_objectives = Vec::new();
        let mut completed_quests = Vec::new();

        for quest in self.active_quests.iter_mut() {
            if quest.state != crate::dialogue::QuestState::Active {
                continue;
            }

            for objective in quest.objectives.iter_mut() {
                if !objective.completed {
                    if let crate::dialogue::QuestObjectiveType::FetchItem {
                        item_name: required_item,
                        quantity: required_qty,
                        consumed: _,
                    } = &objective.objective_type
                    {
                        // Check if player has the required item
                        let has_item = player_inventory.slots.iter().any(|stack| {
                            crate::components::itemkind_name(stack.kind) == *required_item
                                && stack.qty >= *required_qty
                        });

                        if has_item {
                            objective.completed = true;
                            completed_objectives.push(objective.description.clone());
                        }
                    }
                }
            }

            // Check if all objectives are completed
            if quest.is_completed() && quest.state == crate::dialogue::QuestState::Active {
                quest.state = crate::dialogue::QuestState::Completed;
                completed_quests.push(quest.name.clone());
            }
        }

        // Log messages for completed objectives and quests
        for objective_desc in completed_objectives {
            self.log_green(format!("Quest objective completed: {}", objective_desc));
        }
        for quest_name in completed_quests {
            self.log_green(format!("Quest completed: {}", quest_name));
        }
    }

    /// Get all consumable items required for a quest
    pub fn get_quest_consumable_items(&self, quest_type: QuestType) -> Vec<(String, u32)> {
        let mut consumable_items = Vec::new();

        // Find the active quest
        for quest in &self.active_quests {
            if quest.quest_type == quest_type {
                // Get all FetchItem objectives that should be consumed
                for objective in &quest.objectives {
                    if let crate::dialogue::QuestObjectiveType::FetchItem {
                        item_name,
                        quantity,
                        consumed: true,
                    } = &objective.objective_type
                    {
                        consumable_items.push((item_name.clone(), *quantity));
                    }
                }
                break;
            }
        }

        consumable_items
    }
}
