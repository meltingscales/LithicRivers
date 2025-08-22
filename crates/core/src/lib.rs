pub mod components;
pub mod config;
pub mod default_config;
pub mod model;
pub mod palettekey;
pub mod resources;
pub mod save_load;
mod structure;
pub mod systems;
pub mod tiles;
pub mod view;

use components::*;
use model::body::Body;
use resources::*;
use systems::*;
use view::*;

use hecs::World;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub struct Game {
    pub world: World,
    pub res: Resources,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut world = World::new();
        let mut res = Resources::new(seed);
        // Determine starting position from config (production environment)
        let [sx, sy, sz] =
            res.config
                .get_vector_setting("world", "DEFAULT_PLAYER_POSITION", "production");
        // Initialize viewport center to player's starting position
        res.view_x = sx;
        res.view_y = sy;
        res.view_z = sz;
        res.world.set_generation_z(sz);
        // Read auto-pickup default from config
        let auto_pickup_default = res
            .config
            .get_setting("inventory", "TOGGLE_ITEM_AUTO_PICKUP_DEFAULT_ENABLED")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Spawn a player entity with a Position
        let player = world.spawn((
            Position {
                x: sx,
                y: sy,
                z: sz,
            },
            GameEntity,
            EntityKind::Player,
            Player,
            Body::default(),
            Glyph('@'),
            SpriteRef::new("entities", "player"),
            BlocksMovement,
            Inventory {
                auto_pickup: auto_pickup_default,
                ..Default::default()
            },
        ));
        res.player_entity = Some(player);
        // Spawn several StumblingSheep near the player for visibility
        let sheep_positions = [
            (sx + 2, sy + 2, sz),
            (sx + 3, sy, sz),
            (sx, sy + 3, sz),
        ];
        for (x, y, z) in sheep_positions {
            world.spawn((
                Position { x, y, z },
                GameEntity,
                EntityKind::Sheep,
                Sheep,
                Glyph('s'),
                SpriteRef::new("entities", "sheep"),
                BlocksMovement,
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
                        let t = res.world.get_tile_cached(px, py, tz);
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
                    Position { x: tx, y: ty, z: tz },
                    DroppedItem {
                        kind: ItemKind::Wood,
                        qty: 1,
                    },
                    SpriteRef::new("items", "log"),
                ));
            }
        }
        Self { world, res }
    }

    pub fn queue_player_move(&mut self, dx: i32, dy: i32) {
        self.res.player_move_intent = Some((dx, dy));
        // Set the move cost so the next tick advances by this many ticks,
        // using the player's Body.walk_speed_mult if available.
        let mult: f32 = if let Some(e) = self.res.player_entity {
            if let Ok(body) = self.world.get::<&Body>(e) {
                body.walk_speed_modifier()
            } else {
                1.0
            }
        } else {
            1.0
        };
        let base: f32 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;
        self.res.pending_tick_increase = Some(cost);
    }

    /// Advance the game state by one tick.
    /// Returns true if mining was successful during this tick, false otherwise.
    pub fn tick(&mut self) -> bool {
        // In the future, run an ordered system schedule.
        // For now, just increment tick and maybe move the player slowly.
        let inc = self.res.pending_tick_increase.take().unwrap_or(1);
        self.res.gametick = self.res.gametick.saturating_add(inc);

        // Process systems
        let mining_success = mining_system(&mut self.world, &mut self.res);
        move_player_system(&mut self.world, &mut self.res);
        pickup_system(&mut self.world, &mut self.res);
        stumbling_sheep_system(&mut self.world, &mut self.res);

        // Fluids removed

        mining_success
    }

    pub fn build_view(&self) -> RenderView {
        build_render_view(&self.world, &self.res)
    }

    pub fn queue_mine(&mut self) {
        self.res.mining_intent = true;
        // Set an action cost similar to moving; could use Body modifiers later
        let mult: f32 = if let Some(e) = self.res.player_entity {
            if let Ok(body) = self.world.get::<&Body>(e) {
                body.walk_speed_modifier()
            } else {
                1.0
            }
        } else {
            1.0
        };
        let base: f32 = 300.0; // slightly slower than a normal move
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;
        self.res.pending_tick_increase = Some(cost);
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

    /// Queue a vertical movement for the player by dz levels.
    pub fn queue_player_move_z(&mut self, dz: i32) {
        self.res.player_move_intent_z = Some(dz);
        // Use same base cost as lateral movement for now
        let mult: f32 = if let Some(e) = self.res.player_entity {
            if let Ok(body) = self.world.get::<&Body>(e) {
                body.walk_speed_modifier()
            } else {
                1.0
            }
        } else {
            1.0
        };
        let base: f32 = 200.0;
        let cost = (base / mult.max(0.01)).round().max(1.0) as u64;
        self.res.pending_tick_increase = Some(cost);
    }
}
