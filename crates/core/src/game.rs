use crate::components::{
    BlocksMovement, Combat, Dialogue, DroppedItem, Energy, EntityKind, FeralDog, GameEntity, Glyph,
    Health, Inventory, ItemKind, NPCMood, Player, Position, QuestTesty, Sheep, SpriteRef,
};
use crate::model::body::Body;
use crate::resources::Resources;
use crate::spawn_utils;
use crate::system_scheduler::SystemScheduler;
use crate::view::{build_render_view, RenderView};

use hecs::World;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

use bitflags::bitflags;

pub struct Game {
    pub world: World,
    pub res: Resources,
    pub scheduler: SystemScheduler,
}

bitflags! {
    // What happened during a tick?
    pub struct GameTickResult: u32 {
        const MiningSuccess = 1 << 0;
        const CombatTriggered = 1 << 1;
        const CombatEnded = 1 << 2;
        const NoAction = 1 << 3;
    }
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut world = World::new();
        let mut res = Resources::new(seed);

        // Determine starting position from cached config
        let [sx, sy, sz] = res.config.default_player_position;

        // Note: viewport is now managed by client UI, not core game
        // Core game no longer sets generation Z - let client manage viewport
        // Read auto-pickup default from cached config
        let auto_pickup_default = res.config.auto_pickup_enabled;

        // Build starting inventory
        let mut starting_inv = Inventory {
            auto_pickup: auto_pickup_default,
            ..Default::default()
        };
        // Starting items
        starting_inv.add(ItemKind::Nail, 10);
        starting_inv.add(ItemKind::Acorn, 1);
        starting_inv.add(ItemKind::Log, 3);
        starting_inv.add(ItemKind::Stick, 2);
        starting_inv.add(ItemKind::String, 2);
        starting_inv.add(ItemKind::ScrapElectronics, 3);

        // Spawn a player entity with a Position and starting inventory
        let _player = world.spawn((
            Position {
                x: sx,
                y: sy,
                z: sz,
            },
            GameEntity,
            EntityKind::Player,
            Player,
            Body::default(),
            Energy::new(100),
            Glyph('@'),
            SpriteRef::new("entities", "player"),
            BlocksMovement,
            starting_inv,
        ));
        // Note: player_entity field will be removed - use ECS queries instead
        // Spawn several StumblingSheep near the player for visibility
        let sheep_positions = [(sx + 2, sy + 2, sz), (sx + 3, sy, sz), (sx, sy + 3, sz)];
        for (x, y, z) in sheep_positions {
            world.spawn((
                Position { x, y, z },
                GameEntity,
                EntityKind::Sheep,
                Sheep,
                Health::new(50), // Sheep have 50 HP
                Glyph('s'),
                SpriteRef::new("entities", "sheep"),
                BlocksMovement,
            ));
        }

        //spawn 1 feral dog and immediately kill it next to an npc for testing
        let dead_dog_position = (sx - 7, sy - 4, sz);
        let dead_dog = world.spawn((
            Position {
                x: dead_dog_position.0,
                y: dead_dog_position.1,
                z: dead_dog_position.2,
            },
            GameEntity,
            EntityKind::FeralDog,
            FeralDog,
            Health::new(80), // Create with full health first
            Glyph('d'),
            SpriteRef::new("entities", "feral_dog"),
            BlocksMovement,
            Combat::default(),
        ));
        // Add inventory before killing
        let mut dead_dog_inventory = Inventory::default();
        dead_dog_inventory.add(ItemKind::Leather, 1);
        dead_dog_inventory.add(ItemKind::Meat, 2);
        world.insert_one(dead_dog, dead_dog_inventory).unwrap();

        // Now properly kill it and convert to corpse
        crate::systems::handle_entity_death(&mut world, &mut res, dead_dog);

        // spawn 2 feral dogs a bit further
        let dog_positions = [(sx + 12, sy + 12, sz), (sx + 13, sy + 13, sz)];
        for (x, y, z) in dog_positions {
            spawn_utils::world_spawn_feraldog(&mut world, x, y, z, &res.world_state.world);
        }

        //spawn 4 feral dogs a little far away, in a diagonal line to test combat
        let combat_test_x = -20;
        let combat_test_y = -20;
        let combat_test_z = sz;
        let dog_positions = (5..=8).map(|n| (combat_test_x + n, combat_test_y + n, combat_test_z));
        for (x, y, z) in dog_positions {
            spawn_utils::world_spawn_feraldog(&mut world, x, y, z, &res.world_state.world);
        }

        // Spawn single QuestTesty NPC for dialogue testing
        world.spawn((
            Position {
                x: sx - 8,
                y: sy - 3,
                z: sz,
            },
            GameEntity,
            EntityKind::QuestTesty,
            QuestTesty,
            Health::new(100),
            Glyph('Q'),
            SpriteRef::new("entities", "quest_testy"),
            BlocksMovement,
            Dialogue {
                current_mood: NPCMood::Happy,
                met_before: false,
                current_dialogue_id: Some(20), // Start with QuestTesty's dialogue tree
                name: "QuestTesty".to_string(),
            },
        ));

        // Deterministically spawn a few Logs near the player (~5 tiles away)
        // Use a local RNG derived from the seed so we don't perturb the global RNG sequence
        let mut spawn_rng = ChaCha20Rng::seed_from_u64(seed.wrapping_add(0x5eed_cafe_f00d_dead));
        let dir8: &[(i64, i64)] = &[
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, 1),
            (-1, -1),
            (1, -1),
        ];
        let num_logs = 4usize;
        for _ in 0..num_logs {
            // Choose direction and radius ~5 +/- 1
            let (dx, dy) = dir8[spawn_rng.gen_range(0..dir8.len())];
            let r: i64 = 5 + spawn_rng.gen_range(-1..=1);
            let mut tx = sx + dx * r;
            let mut ty = sy + dy * r;
            let tz = sz;

            // If blocked or impassable, search a small neighborhood for a valid tile
            let mut placed = false;
            'search: for rad in 0..=2 {
                for ox in -rad..=rad {
                    for oy in -rad..=rad {
                        let px = tx + ox;
                        let py = ty + oy;
                        let t = res.world_state.world.get_tile_cached(px, py, tz);
                        if !t.is_passable() {
                            continue;
                        }
                        // Avoid spawning on an occupied blocking entity
                        let mut occupied = false;
                        for (_, (_, epos)) in world.query::<(&BlocksMovement, &Position)>().iter() {
                            if epos.x == px && epos.y == py && epos.z == tz {
                                occupied = true;
                                break;
                            }
                        }
                        if occupied {
                            continue;
                        }
                        tx = px;
                        ty = py;
                        placed = true;
                        break 'search;
                    }
                }
            }

            if placed {
                let _ = world.spawn((
                    Position {
                        x: tx,
                        y: ty,
                        z: tz,
                    },
                    DroppedItem {
                        kind: ItemKind::Log,
                        qty: 1,
                    },
                    SpriteRef::new("items", "log"),
                ));
            }
        }

        let mut new_game = Self {
            world,
            res,
            scheduler: SystemScheduler::new(),
        };

        // Queue the starting dungeon for lazy loading
        new_game
            .res
            .pending_structures
            .push(crate::resources::PendingStructure {
                name: "first-quest-sapiencorp.lrstructure".to_string(),
                x: 10,
                y: 16093,
                z: 1,
                bury_structure: true,
            });

        // Force-load the chunk that the starting dungeon is in so the quest marker appears immediately
        new_game.ensure_chunk_with_pending_structures_worldcoords(10, 16093, 1);

        // Run one tick to generate structures around the player spawn
        new_game.tick();

        return new_game;
    }

    pub fn ensure_chunk_with_pending_structures_worldcoords(
        &mut self,
        player_x: i64,
        player_y: i64,
        player_z: i64,
    ) {
        let chunk_x = player_x.div_euclid(crate::world::CHUNK_SIZE);
        let chunk_y = player_y.div_euclid(crate::world::CHUNK_SIZE);
        let chunk_z = player_z.div_euclid(crate::world::CHUNK_SIZE_Z);
        self.ensure_chunk_with_pending_structures(chunk_x, chunk_y, chunk_z);
    }

    /// Ensure a chunk exists and check for any pending structures in that chunk
    pub fn ensure_chunk_with_pending_structures(
        &mut self,
        chunk_x: i64,
        chunk_y: i64,
        chunk_z: i64,
    ) {
        tracing::info!(target: "game", "ensure_chunk_with_pending_structures called for ({}, {}, {})", chunk_x, chunk_y, chunk_z);
        // First check and load any pending structures for this chunk
        self.load_pending_structures_for_chunk(chunk_x, chunk_y, chunk_z);

        // Then ensure the chunk exists - but only if it doesn't already exist
        let chunk_was_new = !self
            .res
            .world_state
            .world
            .has_chunk(chunk_x, chunk_y, chunk_z);

        if chunk_was_new {
            tracing::info!(target: "game", "Calling world.ensure_chunk for ({}, {}, {})", chunk_x, chunk_y, chunk_z);
            self.res
                .world_state
                .world
                .ensure_chunk(chunk_x, chunk_y, chunk_z);

            // Place structures for the newly generated chunk
            self.place_structures_for_new_chunk(chunk_x, chunk_y, chunk_z);
        } else {
            tracing::info!(target: "game", "Chunk ({}, {}, {}) already exists, skipping ensure_chunk", chunk_x, chunk_y, chunk_z);
        }
        tracing::info!(target: "game", "ensure_chunk_with_pending_structures completed for ({}, {}, {})", chunk_x, chunk_y, chunk_z);
    }

    /// Place structures for a newly generated chunk, avoiding borrow checker issues
    fn place_structures_for_new_chunk(&mut self, chunk_x: i64, chunk_y: i64, chunk_z: i64) {
        // Split the borrows to avoid the borrow checker issue
        let world = &mut self.res.world_state.world;
        let ecs_world = &mut self.world;
        world.place_structures_for_chunk(ecs_world, chunk_x, chunk_y, chunk_z);
    }

    /// Load a quest structure at a specific world position
    /// This is used for placing story-specific structures like the SapienCorp factory
    pub fn load_quest_structure(
        &mut self,
        structure_name: &str,
        world_x: i64,
        world_y: i64,
        world_z: i64,
        bury_structure: bool,
    ) {
        use crate::structure::StructureDefinition;
        use tracing::info;

        let structure = StructureDefinition::load_from_embedded(structure_name);
        info!(target: "game", "Loading quest structure '{}' at ({}, {}, {})", structure_name, world_x, world_y, world_z);

        // Convert world coordinates to chunk coordinates and local offset
        let chunk_x = world_x.div_euclid(crate::world::CHUNK_SIZE);
        let chunk_y = world_y.div_euclid(crate::world::CHUNK_SIZE);
        let chunk_z = world_z.div_euclid(crate::world::CHUNK_SIZE_Z);

        let local_x = world_x.rem_euclid(crate::world::CHUNK_SIZE);
        let local_y = world_y.rem_euclid(crate::world::CHUNK_SIZE);

        // Ensure the chunk exists
        self.res
            .world_state
            .world
            .ensure_chunk(chunk_x, chunk_y, chunk_z);

        // Apply the structure
        let world = &mut self.res.world_state.world;
        let ecs_world = &mut self.world;
        world.apply_structure_world(
            ecs_world,
            chunk_x,
            chunk_y,
            chunk_z,
            &structure,
            local_x,
            local_y,
            world_z,
            bury_structure,
        );
        info!(target: "game", "Quest structure '{}' loaded successfully", structure_name);
    }

    pub fn queue_player_move(&mut self, dx: i64, dy: i64) {
        // Calculate move cost using player's Body modifiers
        let mult: f64 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f64 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;

        // Set movement intent with cost
        self.res.player_state.intent = crate::intent::PlayerIntent::movement(dx, dy, 0, cost);
    }

    /// Advance the game state by one tick using the system scheduler.
    pub fn tick(&mut self) -> GameTickResult {
        // Check for pending structures around the player before running systems
        if let Some(player_pos) = self.get_player_position() {
            let chunk_x = player_pos.x.div_euclid(crate::world::CHUNK_SIZE);
            let chunk_y = player_pos.y.div_euclid(crate::world::CHUNK_SIZE);
            let chunk_z = player_pos.z.div_euclid(crate::world::CHUNK_SIZE_Z);
            tracing::info!(target: "game", "Player at world pos ({}, {}, {}) -> chunk ({}, {}, {})",
                player_pos.x, player_pos.y, player_pos.z, chunk_x, chunk_y, chunk_z);

            // Mark the player's current chunk as explored
            self.res.mark_chunk_explored(chunk_x, chunk_y, chunk_z);

            // Check a small radius around the player's chunk
            for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        let target_chunk_x = chunk_x + dx;
                        let target_chunk_y = chunk_y + dy;
                        let target_chunk_z = chunk_z + dz;
                        tracing::info!(target: "game", "Processing chunk ({}, {}, {}) in 3x3x3 around player",
                            target_chunk_x, target_chunk_y, target_chunk_z);
                        self.ensure_chunk_with_pending_structures(
                            target_chunk_x,
                            target_chunk_y,
                            target_chunk_z,
                        );

                        // Mark nearby chunks as explored if they're close enough
                        if dx.abs() <= 1 && dy.abs() <= 1 && dz.abs() <= 1 {
                            self.res.mark_chunk_explored(
                                target_chunk_x,
                                target_chunk_y,
                                target_chunk_z,
                            );
                        }
                    }
                }
            }
        }

        // Increment tick based on pending action cost
        let inc = self.res.player_state.intent.cost().max(1);
        self.res.time.tick = self.res.time.tick.saturating_add(inc);

        // Run all systems through the scheduler
        let system_results = self
            .scheduler
            .run_all_systems(&mut self.world, &mut self.res);

        let mut result = GameTickResult::NoAction;
        if system_results.mining_success {
            result |= GameTickResult::MiningSuccess;
        }
        if system_results.combat_triggered {
            result |= GameTickResult::CombatTriggered;
        }
        if self.res.player_state.combat_ended_this_tick {
            result |= GameTickResult::CombatEnded;
        }

        result
    }

    pub fn build_view(&self) -> RenderView {
        build_render_view(&self.world, &self.res)
    }

    pub fn queue_mine(&mut self) {
        // Set an action cost similar to moving; could use Body modifiers later
        let mult: f64 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f64 = 300.0; // slightly slower than a normal move
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;

        // Set mining intent with cost
        self.res.player_state.intent = crate::intent::PlayerIntent::mine(cost);
    }

    pub fn queue_mine_at(&mut self, x: i64, y: i64, z: i64) {
        // Set an action cost similar to moving; could use Body modifiers later
        let mult: f64 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f64 = 300.0; // slightly slower than a normal move
        let cost = (base / mult.max(0.01)).round().max(1.0);

        // Set mining intent with cost at specific coordinates
        self.res.player_state.intent = crate::intent::PlayerIntent::mine_at(x, y, z, cost as u64);
    }

    // Convenience save/load wrappers
    pub fn save_json<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        crate::save_load::save_game_json(self, path)
    }
    pub fn load_json<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        crate::save_load::load_game_json(self, path)
    }
    pub fn save_msgpack<P: AsRef<std::path::Path>>(&self, path: P) -> anyhow::Result<()> {
        crate::save_load::save_game_msgpack(self, path)
    }
    pub fn load_msgpack<P: AsRef<std::path::Path>>(&mut self, path: P) -> anyhow::Result<()> {
        crate::save_load::load_game_msgpack(self, path)
    }

    // Viewport-aware save/load methods
    pub fn save_json_with_viewport<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        viewport: crate::save_load::ViewportSave,
    ) -> anyhow::Result<crate::save_load::ViewportSave> {
        crate::save_load::save_game_json_with_viewport(self, path, viewport)
    }

    pub fn load_json_with_viewport<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> anyhow::Result<crate::save_load::ViewportSave> {
        crate::save_load::load_game_json_with_viewport(self, path)
    }

    /// Queue a vertical movement for the player by dz levels.
    pub fn queue_player_move_z(&mut self, dz: i64) {
        // Use same base cost as lateral movement for now
        let mult: f64 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f64 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0);

        // Set vertical movement intent
        self.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, dz, cost as u64);
    }

    // ECS Helper Functions - Replace direct player_entity access

    /// Get the player entity using proper ECS query
    pub fn get_player_entity(&self) -> Option<hecs::Entity> {
        self.world.query::<&Player>().iter().next().map(|(e, _)| e)
    }

    /// Get player position using ECS query
    pub fn get_player_position(&self) -> Option<Position> {
        self.world
            .query::<(&Player, &Position)>()
            .iter()
            .next()
            .map(|(_, (_, pos))| *pos)
    }

    /// Check if given entity is the player
    pub fn is_player_entity(&self, entity: hecs::Entity) -> bool {
        self.world.get::<&Player>(entity).is_ok()
    }

    /// Get player component of specified type
    pub fn get_player_component<T: hecs::Component>(&self) -> Option<T>
    where
        T: Clone,
    {
        self.world
            .query::<(&Player, &T)>()
            .iter()
            .next()
            .map(|(_, (_, component))| component.clone())
    }

    /// Get mutable reference to player component
    pub fn get_player_component_mut<T: hecs::Component>(&mut self) -> Option<hecs::RefMut<'_, T>> {
        if let Some(player_entity) = self.get_player_entity() {
            self.world.get::<&mut T>(player_entity).ok()
        } else {
            None
        }
    }

    /// Apply function to player entity if it exists
    pub fn with_player_entity<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(hecs::Entity) -> R,
    {
        self.get_player_entity().map(f)
    }

    /// End combat for all enemies near the player (used when manually exiting combat)
    pub fn end_combat_around_player(&mut self) {
        if let Some(player_pos) = self.get_player_position() {
            crate::systems::end_combat_for_nearby_enemies(&mut self.world, player_pos);
            self.res.player_state.combat_active = false;
            self.res
                .log("Ended combat for nearby enemies (manual exit)".to_string());
        }
    }

    // === Generation State Management ===

    /// Check if a chunk has been processed for structure generation
    pub fn is_chunk_generated(&self, chunk_x: i64, chunk_y: i64, chunk_z: i64) -> bool {
        use crate::resources::ChunkGenerationState;
        let is_generated = matches!(
            self.res
                .chunk_generation_states
                .get(&(chunk_x, chunk_y, chunk_z)),
            Some(ChunkGenerationState::Generated)
        );
        tracing::info!(target: "game", "Checking chunk generation state for ({}, {}, {}): {}",
            chunk_x, chunk_y, chunk_z, if is_generated { "Generated" } else { "Not Generated" });
        is_generated
    }

    /// Mark a chunk as processed for structure generation
    pub fn mark_chunk_generated(&mut self, chunk_x: i64, chunk_y: i64, chunk_z: i64) {
        use crate::resources::ChunkGenerationState;
        tracing::info!(target: "game", "Marking chunk ({}, {}, {}) as generated", chunk_x, chunk_y, chunk_z);
        self.res
            .chunk_generation_states
            .insert((chunk_x, chunk_y, chunk_z), ChunkGenerationState::Generated);
    }

    /// Check if a specific structure has been generated
    pub fn is_structure_generated(&self, structure_name: &str, x: i64, y: i64, z: i64) -> bool {
        use crate::resources::StructureGenerationState;
        let structure_key = format!("{}@{},{},{}", structure_name, x, y, z);
        let is_generated = matches!(
            self.res.structure_generation_states.get(&structure_key),
            Some(StructureGenerationState::Generated)
        );
        tracing::info!(target: "game", "Checking structure generation state for '{}' at ({}, {}, {}): {}",
            structure_name, x, y, z, if is_generated { "Generated" } else { "Not Generated" });
        is_generated
    }

    /// Mark a specific structure as generated
    pub fn mark_structure_generated(&mut self, structure_name: &str, x: i64, y: i64, z: i64) {
        use crate::resources::StructureGenerationState;
        let structure_key = format!("{}@{},{},{}", structure_name, x, y, z);
        tracing::info!(target: "game", "Marking structure '{}' at ({}, {}, {}) as generated", structure_name, x, y, z);
        self.res
            .structure_generation_states
            .insert(structure_key, StructureGenerationState::Generated);
    }

    /// Mark all Z chunks that a structure spans as generated to prevent infinite loops
    pub fn mark_all_structure_chunks_generated(
        &mut self,
        structure_name: &str,
        x: i64,
        y: i64,
        z: i64,
    ) {
        use crate::structure::StructureDefinition;
        use crate::world::{CHUNK_SIZE, CHUNK_SIZE_Z};

        // Load the structure to get its dimensions
        let structure = StructureDefinition::load_from_embedded(structure_name);

        // Calculate base chunk coordinates
        let chunk_x = x.div_euclid(CHUNK_SIZE) as i64;
        let chunk_y = y.div_euclid(CHUNK_SIZE) as i64;
        let base_chunk_z = z.div_euclid(CHUNK_SIZE_Z) as i64;

        // Calculate the range of Z chunks this structure spans
        let structure_height = structure.layers.len() as i64;
        let max_z = z + structure_height - 1;
        let max_chunk_z = max_z.div_euclid(CHUNK_SIZE_Z);

        tracing::info!(target: "game", "Structure '{}' spans {} layers, marking chunks ({}, {}, {}) to ({}, {}, {})",
            structure_name, structure_height, chunk_x, chunk_y, base_chunk_z, chunk_x, chunk_y, max_chunk_z);

        // Mark all Z chunks in the range as generated
        for cz in base_chunk_z..=max_chunk_z {
            self.mark_chunk_generated(chunk_x, chunk_y, cz);
        }
    }

    // === Bounds Checking Utilities ===

    /// Convert world coordinates to chunk coordinates
    pub fn world_to_chunk_coords(world_x: i64, world_y: i64, world_z: i64) -> (i64, i64, i64) {
        use crate::world::{CHUNK_SIZE, CHUNK_SIZE_Z};
        let chunk_x = world_x.div_euclid(CHUNK_SIZE);
        let chunk_y = world_y.div_euclid(CHUNK_SIZE);
        let chunk_z = world_z.div_euclid(CHUNK_SIZE_Z);
        (chunk_x, chunk_y, chunk_z)
    }

    /// Get world bounds for a chunk
    pub fn chunk_world_bounds(
        chunk_x: i64,
        chunk_y: i64,
        chunk_z: i64,
    ) -> ((i64, i64, i64), (i64, i64, i64)) {
        use crate::world::{CHUNK_SIZE, CHUNK_SIZE_Z};
        let min_x = chunk_x * CHUNK_SIZE as i64;
        let max_x = min_x + CHUNK_SIZE as i64;
        let min_y = chunk_y * CHUNK_SIZE as i64;
        let max_y = min_y + CHUNK_SIZE as i64;
        let min_z = chunk_z * CHUNK_SIZE_Z as i64;
        let max_z = min_z + CHUNK_SIZE_Z as i64;
        ((min_x, min_y, min_z), (max_x, max_y, max_z))
    }

    /// Check if a point is within a chunk's bounds
    pub fn is_point_in_chunk(
        point_x: i64,
        point_y: i64,
        point_z: i64,
        chunk_x: i64,
        chunk_y: i64,
        chunk_z: i64,
    ) -> bool {
        let ((min_x, min_y, min_z), (max_x, max_y, max_z)) =
            Self::chunk_world_bounds(chunk_x, chunk_y, chunk_z);
        (point_x as i64) >= min_x
            && (point_x as i64) < max_x
            && (point_y as i64) >= min_y
            && (point_y as i64) < max_y
            && (point_z as i64) >= min_z
            && (point_z as i64) < max_z
    }

    /// Check if any pending structures should be loaded for a given chunk
    /// Called during chunk generation to place quest structures
    pub fn load_pending_structures_for_chunk(&mut self, chunk_x: i64, chunk_y: i64, chunk_z: i64) {
        // Check if we've already processed this chunk for structure loading
        if self.is_chunk_generated(chunk_x, chunk_y, chunk_z) {
            tracing::info!(target: "game", "Chunk ({}, {}, {}) already processed for structures", chunk_x, chunk_y, chunk_z);
            return; // Already processed this chunk
        }

        tracing::info!(target: "game", "Processing chunk ({}, {}, {}) for structures", chunk_x, chunk_y, chunk_z);
        // Mark this chunk as processed
        self.mark_chunk_generated(chunk_x, chunk_y, chunk_z);

        // No need to calculate bounds manually anymore - we have abstraction methods

        let mut structures_to_load = Vec::new();
        let mut remaining_structures = Vec::new();

        // Collect all pending structures first to avoid borrow checker issues
        let pending_structures: Vec<_> = self.res.pending_structures.drain(..).collect();
        tracing::info!(target: "game", "Found {} pending structures to evaluate for chunk ({}, {}, {})",
            pending_structures.len(), chunk_x, chunk_y, chunk_z);

        for structure in pending_structures {
            // Check if this specific structure has already been generated
            if self.is_structure_generated(&structure.name, structure.x, structure.y, structure.z) {
                tracing::info!(target: "game", "Skipping already generated structure '{}' at ({}, {}, {})",
                    structure.name, structure.x, structure.y, structure.z);
                continue; // Skip already generated structures
            }

            // Check if structure is within or overlaps this chunk
            let structure_in_chunk = Self::is_point_in_chunk(
                structure.x,
                structure.y,
                structure.z,
                chunk_x,
                chunk_y,
                chunk_z,
            );

            if structure_in_chunk {
                tracing::info!(target: "game", "Structure '{}' at ({}, {}, {}) belongs to chunk ({}, {}, {}) - will load",
                    structure.name, structure.x, structure.y, structure.z, chunk_x, chunk_y, chunk_z);
                structures_to_load.push(structure);
            } else {
                tracing::info!(target: "game", "Structure '{}' at ({}, {}, {}) does not belong to chunk ({}, {}, {}) - keeping pending",
                    structure.name, structure.x, structure.y, structure.z, chunk_x, chunk_y, chunk_z);
                remaining_structures.push(structure);
            }
        }

        // Put back structures that don't belong in this chunk
        self.res.pending_structures = remaining_structures;
        tracing::info!(target: "game", "Kept {} structures pending, loading {} structures for chunk ({}, {}, {})",
            self.res.pending_structures.len(), structures_to_load.len(), chunk_x, chunk_y, chunk_z);

        // Load structures that belong in this chunk
        let num_structures_to_load = structures_to_load.len();
        for structure in structures_to_load {
            self.res.log(format!(
                "Loading structure '{}' at ({}, {}, {}) for chunk ({}, {}, {})",
                structure.name, structure.x, structure.y, structure.z, chunk_x, chunk_y, chunk_z
            ));

            self.load_quest_structure(
                &structure.name,
                structure.x,
                structure.y,
                structure.z,
                structure.bury_structure,
            );

            // Add quest marker for this structure
            if structure
                .name
                .contains("first-quest-sapiencorp.lrstructure")
            {
                self.res.add_quest_marker(crate::resources::QuestMarker {
                    name: "SapienCorp Factory".to_string(),
                    description: "Mysterious corporate facility buried underground".to_string(),
                    x: structure.x,
                    y: structure.y,
                    z: structure.z,
                    marker_type: crate::resources::QuestMarkerType::MainQuest,
                });
                self.res.log_green("Quest marker added: SapienCorp Factory");
            }

            // Mark this structure as generated
            self.mark_structure_generated(&structure.name, structure.x, structure.y, structure.z);

            // Also mark all Z chunks that this structure spans as generated
            self.mark_all_structure_chunks_generated(
                &structure.name,
                structure.x,
                structure.y,
                structure.z,
            );
        }

        tracing::info!(target: "game", "Completed structure processing for chunk ({}, {}, {}) - loaded {} structures",
            chunk_x, chunk_y, chunk_z, num_structures_to_load);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_structure_loading() {
        // Create a new game with a test seed
        let mut game = Game::new(12345);

        // With the initial tick during game creation, structures near the player spawn
        // are now loaded immediately. The structure should be processed and removed from queue.
        assert_eq!(game.res.pending_structures.len(), 0);

        // The chunk should exist due to the initial tick processing
        let chunk_x = 30_i64.div_euclid(crate::world::CHUNK_SIZE as i64);
        let chunk_y = 30_i64.div_euclid(crate::world::CHUNK_SIZE as i64);
        let chunk_z = 0_i64.div_euclid(crate::world::CHUNK_SIZE_Z as i64);

        // Verify the chunk now exists (this should work without infinite loops)
        game.res
            .world_state
            .world
            .ensure_chunk(chunk_x, chunk_y, chunk_z);

        // Verify the chunk exists
        assert!(game
            .res
            .world_state
            .world
            .has_chunk(chunk_x, chunk_y, chunk_z));
    }

    #[test]
    fn test_structure_not_loaded_for_different_chunk() {
        let mut game = Game::new(12345);

        // With the initial tick, the structure near player spawn is already loaded
        assert_eq!(game.res.pending_structures.len(), 0);

        // Try loading structures for a chunk far away - this should not cause issues
        let far_chunk_x = 100_i64;
        let far_chunk_y = 100_i64;
        let far_chunk_z = 10_i64;

        game.load_pending_structures_for_chunk(far_chunk_x, far_chunk_y, far_chunk_z);

        // The queue should still be empty since no structures are queued for that chunk
        assert_eq!(game.res.pending_structures.len(), 0);
    }

    #[test]
    fn test_ensure_chunk_with_pending_structures() {
        let mut game = Game::new(12345);

        // Get the chunk coordinates for our structure location
        let chunk_x = 30_i64.div_euclid(crate::world::CHUNK_SIZE as i64);
        let chunk_y = 30_i64.div_euclid(crate::world::CHUNK_SIZE as i64);
        let chunk_z = 0_i64.div_euclid(crate::world::CHUNK_SIZE_Z as i64);

        // With the initial tick, structure is already loaded and queue is empty
        assert_eq!(game.res.pending_structures.len(), 0);

        // Use the combined method - this should work without issues
        game.ensure_chunk_with_pending_structures(chunk_x, chunk_y, chunk_z);

        // The queue should remain empty and chunk should exist
        assert_eq!(game.res.pending_structures.len(), 0);
        assert!(game
            .res
            .world_state
            .world
            .has_chunk(chunk_x, chunk_y, chunk_z));
    }

    #[test]
    fn test_z_coordinate_movement_no_infinite_loop() {
        let mut game = Game::new(12345);

        // Test Z coordinate chunk generation without structures

        // Get player entity and position them for testing
        let player_entity = game.get_player_entity().expect("Player should exist");
        if let Ok(mut pos) = game.world.get::<&mut Position>(player_entity) {
            pos.x = 30;
            pos.y = 30;
            pos.z = 0;
        }

        // Force some chunk generation by running ticks
        for _ in 0..3 {
            let _result = game.tick();
        }

        // Add some pending structures to trigger the real issue scenario
        // Using existing structure files to avoid panics
        use crate::structure;
        let existing_structures = structure::structures_list();
        for (i, structure_name) in existing_structures.iter().enumerate() {
            game.res
                .pending_structures
                .push(crate::resources::PendingStructure {
                    name: structure_name.to_string(),
                    x: 25 + (i as i64),
                    y: 25 + (i as i64),
                    z: -1 + (i as i64), // Include negative Z structures
                    bury_structure: true,
                });
        }

        let initial_chunk_count = game.res.chunk_generation_states.len();

        // Now test moving the player in a 3x3x3 cube, especially testing z = -1
        // This should not cause infinite loops in chunk processing even with pending structures
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    // Move player to this position
                    if let Ok(mut pos) = game.world.get::<&mut Position>(player_entity) {
                        pos.x = 30 + dx;
                        pos.y = 30 + dy;
                        pos.z = 0 + dz; // This is the critical test - going to z = -1
                    }

                    // Track chunk states before tick
                    let pre_tick_chunks = game.res.chunk_generation_states.len();

                    // Run a few ticks - this should not infinite loop
                    for _tick in 0..3 {
                        let _result = game.tick();

                        // Check that we're not generating excessive chunks
                        let current_chunks = game.res.chunk_generation_states.len();
                        assert!(
                            current_chunks <= pre_tick_chunks + 10,
                            "Excessive chunk generation at position ({}, {}, {}): {} -> {}",
                            30 + dx,
                            30 + dy,
                            0 + dz,
                            pre_tick_chunks,
                            current_chunks
                        );
                    }
                }
            }
        }

        // Verify final chunk count is reasonable
        let final_chunk_count = game.res.chunk_generation_states.len();
        assert!(
            final_chunk_count <= initial_chunk_count + 50,
            "Too many total chunks generated: {} -> {}",
            initial_chunk_count,
            final_chunk_count
        );
    }
}
