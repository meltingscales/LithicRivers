use crate::components::{
    BlocksMovement, Combat, DroppedItem, Energy, EntityKind, FeralDog, GameEntity, Glyph, Health,
    Inventory, ItemKind, Player, Position, Sheep, SpriteRef,
};
use crate::model::body::Body;
use crate::resources::Resources;
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

        // spawn 2 feral dogs a bit further
        let dog_positions = [(sx + 12, sy + 12, sz), (sx + 13, sy + 13, sz)];
        for (x, y, z) in dog_positions {
            // Create dog inventory with 0-2 Leather and 1-3 Meat using deterministic RNG
            let mut dog_rng = ChaCha20Rng::seed_from_u64(seed.wrapping_add((x + y + z) as u64));
            let mut dog_inventory = Inventory::default();
            dog_inventory.add(ItemKind::Leather, dog_rng.gen_range(0..=2));
            dog_inventory.add(ItemKind::Meat, dog_rng.gen_range(1..=3));

            world.spawn((
                Position { x, y, z },
                GameEntity,
                EntityKind::FeralDog,
                FeralDog,
                Health::new(80), // Feral dogs have 80 HP
                Glyph('d'),
                SpriteRef::new("entities", "feral_dog"),
                BlocksMovement,
                Combat::default(),
                dog_inventory,
            ));
        }

        //spawn 4 feral dogs a little far away, in a diagonal line to test combat
        let combatTestX = -20;
        let combatTestY = -20;
        let combatTestZ = sz;
        let dog_positions = (5..=8).map(|n| (combatTestX + n, combatTestY + n, combatTestZ));
        for (x, y, z) in dog_positions {
            // Create dog inventory with 0-2 Leather and 1-3 Meat using deterministic RNG
            let mut dog_rng = ChaCha20Rng::seed_from_u64(seed.wrapping_add((x + y + z) as u64));
            let mut dog_inventory = Inventory::default();
            dog_inventory.add(ItemKind::Leather, dog_rng.gen_range(0..=2));
            dog_inventory.add(ItemKind::Meat, dog_rng.gen_range(1..=3));

            world.spawn((
                Position { x, y, z },
                GameEntity,
                EntityKind::FeralDog,
                FeralDog,
                Health::new(80), // Feral dogs have 80 HP
                Glyph('d'),
                SpriteRef::new("entities", "feral_dog"),
                BlocksMovement,
                Combat::default(),
                dog_inventory,
            ));
        }

        // Deterministically spawn a few Logs near the player (~5 tiles away)
        // Use a local RNG derived from the seed so we don't perturb the global RNG sequence
        let mut spawn_rng = ChaCha20Rng::seed_from_u64(seed.wrapping_add(0x5eed_cafe_f00d_dead));
        let dir8: &[(i32, i32)] = &[
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
            let r: i32 = 5 + spawn_rng.gen_range(-1..=1);
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
        Self {
            world,
            res,
            scheduler: SystemScheduler::new(),
        }
    }

    pub fn queue_player_move(&mut self, dx: i32, dy: i32) {
        // Calculate move cost using player's Body modifiers
        let mult: f32 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f32 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;

        // Set movement intent with cost
        self.res.player_state.intent = crate::intent::PlayerIntent::movement(dx, dy, 0, cost);
    }

    /// Advance the game state by one tick using the system scheduler.
    pub fn tick(&mut self) -> GameTickResult {
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
        let mult: f32 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f32 = 300.0; // slightly slower than a normal move
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;

        // Set mining intent with cost
        self.res.player_state.intent = crate::intent::PlayerIntent::mine(cost);
    }

    pub fn queue_mine_at(&mut self, x: i32, y: i32, z: i32) {
        // Set an action cost similar to moving; could use Body modifiers later
        let mult: f32 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f32 = 300.0; // slightly slower than a normal move
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;

        // Set mining intent with cost at specific coordinates
        self.res.player_state.intent = crate::intent::PlayerIntent::mine_at(x, y, z, cost);
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
    pub fn queue_player_move_z(&mut self, dz: i32) {
        // Use same base cost as lateral movement for now
        let mult: f32 = self
            .get_player_component::<Body>()
            .map(|body| body.walk_speed_modifier())
            .unwrap_or(1.0);
        let base: f32 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;

        // Set vertical movement intent
        self.res.player_state.intent = crate::intent::PlayerIntent::movement(0, 0, dz, cost);
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
}
