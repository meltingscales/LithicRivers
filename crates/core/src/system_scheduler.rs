use crate::resources::Resources;
use hecs::World;
use std::collections::{HashMap, VecDeque};

/// System execution phases for logical grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SystemPhase {
    /// Player input processing (movement, mining)
    PlayerInput,
    /// Entity AI and movement
    EntityAI,
    /// Interaction systems (pickup, collision)
    Interactions,
    /// Combat preparation and timer updates
    CombatPrep,
    /// Combat execution
    Combat,
}

/// Unique identifier for each system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemId {
    MiningSystem,
    MovePlayerSystem,
    PickupSystem,
    FeralDogSystem,
    StumblingSheepSystem,
    BattleDelayTimerSystem,
    CombatTriggerSystem,
    ActionQueueSystem,
    EnemyCombatAiSystem,
}

/// System metadata and dependencies
#[derive(Debug)]
pub struct SystemDesc {
    pub id: SystemId,
    pub phase: SystemPhase,
    pub depends_on: Vec<SystemId>,
    pub name: &'static str,
}

/// Result data that systems can return
#[derive(Debug, Default)]
pub struct SystemResults {
    pub mining_success: bool,
    pub combat_triggered: bool,
}

/// System function signature
pub type SystemFn = fn(&mut World, &mut Resources) -> Option<SystemResults>;

/// System registry and scheduler
pub struct SystemScheduler {
    systems: HashMap<SystemId, (SystemDesc, SystemFn)>,
    execution_order: Vec<SystemId>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        let mut scheduler = Self {
            systems: HashMap::new(),
            execution_order: Vec::new(),
        };
        scheduler.register_default_systems();
        scheduler.compute_execution_order();
        scheduler
    }

    /// Register a system with its metadata and function
    pub fn register_system(&mut self, desc: SystemDesc, system_fn: SystemFn) {
        self.systems.insert(desc.id, (desc, system_fn));
    }

    /// Register all default game systems
    fn register_default_systems(&mut self) {
        // Player input systems
        self.register_system(
            SystemDesc {
                id: SystemId::MiningSystem,
                phase: SystemPhase::PlayerInput,
                depends_on: vec![],
                name: "mining_system",
            },
            mining_system_wrapper,
        );

        self.register_system(
            SystemDesc {
                id: SystemId::MovePlayerSystem,
                phase: SystemPhase::PlayerInput,
                depends_on: vec![],
                name: "move_player_system",
            },
            move_player_system_wrapper,
        );

        // Entity AI systems
        self.register_system(
            SystemDesc {
                id: SystemId::FeralDogSystem,
                phase: SystemPhase::EntityAI,
                depends_on: vec![SystemId::MovePlayerSystem], // Dogs react to player position
                name: "feral_dog_system",
            },
            feral_dog_system_wrapper,
        );

        self.register_system(
            SystemDesc {
                id: SystemId::StumblingSheepSystem,
                phase: SystemPhase::EntityAI,
                depends_on: vec![],
                name: "stumbling_sheep_system",
            },
            stumbling_sheep_system_wrapper,
        );

        // Interaction systems
        self.register_system(
            SystemDesc {
                id: SystemId::PickupSystem,
                phase: SystemPhase::Interactions,
                depends_on: vec![SystemId::MovePlayerSystem], // Needs final player position
                name: "pickup_system",
            },
            pickup_system_wrapper,
        );

        // Combat preparation
        self.register_system(
            SystemDesc {
                id: SystemId::BattleDelayTimerSystem,
                phase: SystemPhase::CombatPrep,
                depends_on: vec![],
                name: "battle_delay_timer_system",
            },
            battle_delay_timer_system_wrapper,
        );

        self.register_system(
            SystemDesc {
                id: SystemId::CombatTriggerSystem,
                phase: SystemPhase::CombatPrep,
                depends_on: vec![
                    SystemId::MovePlayerSystem,       // Position-based combat detection
                    SystemId::FeralDogSystem,         // AI positions finalized
                    SystemId::BattleDelayTimerSystem, // Timers updated first
                ],
                name: "combat_trigger_system",
            },
            combat_trigger_system_wrapper,
        );

        // Combat execution
        self.register_system(
            SystemDesc {
                id: SystemId::ActionQueueSystem,
                phase: SystemPhase::Combat,
                depends_on: vec![SystemId::CombatTriggerSystem],
                name: "action_queue_system",
            },
            action_queue_system_wrapper,
        );

        self.register_system(
            SystemDesc {
                id: SystemId::EnemyCombatAiSystem,
                phase: SystemPhase::Combat,
                depends_on: vec![SystemId::CombatTriggerSystem],
                name: "enemy_combat_ai_system",
            },
            enemy_combat_ai_system_wrapper,
        );
    }

    /// Compute topological execution order based on dependencies
    fn compute_execution_order(&mut self) {
        let mut in_degree: HashMap<SystemId, usize> = HashMap::new();
        let mut adj_list: HashMap<SystemId, Vec<SystemId>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for &system_id in self.systems.keys() {
            in_degree.insert(system_id, 0);
            adj_list.insert(system_id, Vec::new());
        }

        // Build dependency graph
        for (system_id, (desc, _)) in &self.systems {
            for &dep in &desc.depends_on {
                if let Some(deps) = adj_list.get_mut(&dep) {
                    deps.push(*system_id);
                }
                if let Some(degree) = in_degree.get_mut(system_id) {
                    *degree += 1;
                }
            }
        }

        // Topological sort with phase-based secondary ordering
        let mut queue: VecDeque<SystemId> = VecDeque::new();
        let mut execution_order = Vec::new();

        // Start with systems that have no dependencies, grouped by phase
        let mut phase_groups: HashMap<SystemPhase, Vec<SystemId>> = HashMap::new();
        for (&system_id, &degree) in &in_degree {
            if degree == 0 {
                let (desc, _) = &self.systems[&system_id];
                phase_groups.entry(desc.phase).or_default().push(system_id);
            }
        }

        // Process phases in order
        let mut phases: Vec<SystemPhase> = phase_groups.keys().cloned().collect();
        phases.sort();

        for phase in phases {
            if let Some(mut systems) = phase_groups.remove(&phase) {
                systems.sort_by_key(|&id| self.systems[&id].0.name);
                for system_id in systems {
                    queue.push_back(system_id);
                }
            }
        }

        // Topological sort
        while let Some(current) = queue.pop_front() {
            execution_order.push(current);

            // Update dependencies
            if let Some(dependents) = adj_list.get(&current) {
                for &dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(&dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        self.execution_order = execution_order;
    }

    /// Execute all systems in dependency order
    pub fn run_all_systems(&self, world: &mut World, res: &mut Resources) -> SystemResults {
        let mut combined_results = SystemResults::default();

        for &system_id in &self.execution_order {
            if let Some((desc, system_fn)) = self.systems.get(&system_id) {
                tracing::trace!(target: "systems", "Running system: {}", desc.name);

                if let Some(results) = system_fn(world, res) {
                    // Combine results from different systems
                    combined_results.mining_success |= results.mining_success;
                    combined_results.combat_triggered |= results.combat_triggered;
                }
            }
        }

        combined_results
    }

    /// Get execution order for debugging
    pub fn get_execution_order(&self) -> &[SystemId] {
        &self.execution_order
    }
}

// System wrapper functions that adapt the existing system signatures
fn mining_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    let mining_success = crate::systems::mining_system(world, res);
    Some(SystemResults {
        mining_success,
        ..Default::default()
    })
}

fn move_player_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    crate::systems::move_player_system(world, res);
    None
}

fn pickup_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    crate::systems::pickup_system(world, res);
    None
}

fn feral_dog_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    crate::systems::feral_dog_system(world, res);
    None
}

fn stumbling_sheep_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    crate::systems::stumbling_sheep_system(world, res);
    None
}

fn battle_delay_timer_system_wrapper(
    world: &mut World,
    res: &mut Resources,
) -> Option<SystemResults> {
    crate::systems::battle_delay_timer_system(world, res);
    None
}

fn combat_trigger_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    let combat_state = crate::systems::combat_trigger_system(world, res);
    let combat_triggered = matches!(combat_state, crate::systems::CombatState::CombatStarted);
    Some(SystemResults {
        combat_triggered,
        ..Default::default()
    })
}

fn action_queue_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    crate::systems::action_queue_system(world, res);
    None
}

fn enemy_combat_ai_system_wrapper(world: &mut World, res: &mut Resources) -> Option<SystemResults> {
    crate::systems::enemy_combat_ai_system(world, res);
    None
}
