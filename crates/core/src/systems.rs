use crate::components::{BlocksMovement, Position, Sheep};
use crate::resources::Resources;
use crate::tiles::TileKind;
use hecs::World;
use tracing::info;

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some((dx, dy)) = res.player_move_intent.take() {
        if let Some(player_e) = res.player_entity {
            // 1) Read current position immutably to avoid aliasing with queries below
            if let Ok(pos) = world.get::<&Position>(player_e) {
                let (cx, cy, cz) = (pos.x, pos.y, pos.z);
                // Drop immutable borrow explicitly (not strictly necessary but clarifies intent)
                drop(pos);

                let nx = cx + dx;
                let ny = cy + dy;
                let t = res.world.get_tile(nx, ny);
                // Check tile passability and blocking entities
                let mut blocked = !t.is_passable();
                if !blocked {
                    for (_e, (epos, _bm)) in world
                        .query::<(&Position, &BlocksMovement)>()
                        .iter()
                    {
                        if epos.z == cz && epos.x == nx && epos.y == ny {
                            blocked = true;
                            break;
                        }
                    }
                }
                if blocked {
                    info!("Blocked by {:?} at ({}, {})", t, nx, ny);
                    // Set resource for last blocked tile
                    res.last_blocked_tile = Some((nx, ny));
                } else {
                    // 2) Now borrow mutably to write the new position
                    if let Ok(mut pos_mut) = world.get::<&mut Position>(player_e) {
                        pos_mut.x = nx;
                        pos_mut.y = ny;
                    }
                }
            }
        }
    }
}

/// Very simple deterministic-ish wander: each sheep picks a random 4-neighbor step.
/// Uses the global RNG for determinism relative to the initial seed and tick order.
pub fn stumbling_sheep_system(world: &mut World, res: &mut Resources) {
    use rand::Rng;
    // Collect sheep entities to avoid borrow conflicts and to impose a stable order
    let mut entities: Vec<(hecs::Entity, Position)> = Vec::new();
    for (e, (pos, _s)) in world.query::<(&Position, &Sheep)>().iter() {
        entities.push((e, *pos));
    }
    // Sort by position then entity bits for a stable iteration order
    entities.sort_by_key(|(e, p)| (p.x, p.y, p.z, e.to_bits()));
    for (e, pos) in entities {
        // 25% chance to stay, else pick one of 4 directions
        let r: u32 = res.rng.gen_range(0..5);
        let (dx, dy) = match r {
            0 => (0, 0),
            1 => (1, 0),
            2 => (-1, 0),
            3 => (0, 1),
            _ => (0, -1),
        };
        if dx == 0 && dy == 0 { continue; }
        let nx = pos.x + dx;
        let ny = pos.y + dy;
        let t = res.world.get_tile(nx, ny);
        if !t.is_passable() { continue; }
        // Avoid stepping into another blocking entity
        let mut occupied = false;
        for (_oe, (op, _bm)) in world.query::<(&Position, &BlocksMovement)>().iter() {
            if _oe != e && op.z == pos.z && op.x == nx && op.y == ny {
                occupied = true; break;
            }
        }
        if occupied { continue; }
        if let Ok(mut mypos) = world.get::<&mut Position>(e) {
            mypos.x = nx; mypos.y = ny;
        }
    }
}
