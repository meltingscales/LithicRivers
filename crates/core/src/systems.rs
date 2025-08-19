use crate::components::{BlocksMovement, DroppedItem, Inventory, ItemKind, Position, Sheep};
use crate::resources::Resources;
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
                let nz = cz; // Keep same Z-level for now
                let t = res.world.get_tile_cached(nx, ny);
                // Check tile passability and blocking entities
                let mut blocked = !t.is_passable();
                if !blocked {
                    for (_, (_, epos)) in world.query::<(&BlocksMovement, &Position)>().iter() {
                        if epos.x == nx && epos.y == ny && epos.z == nz {
                            blocked = true;
                            break;
                        }
                    }
                }
                if blocked {
                    info!("Blocked by {:?} at ({}, {})", t, nx, ny);
                    // Set resource for last blocked tile (only x,y for now)
                    res.last_blocked_tile = Some((nx, ny));
                } else {
                    // 2) Now borrow mutably to write the new position
                    if let Ok(mut pos_mut) = world.get::<&mut Position>(player_e) {
                        pos_mut.x = nx;
                        pos_mut.y = ny;
                        pos_mut.z = nz;
                    }
                }
            }
        }
    }
}

/// Process mining intent: if the player requested mining, act on current tile.
/// Returns true if mining was successful (e.g. chopped a tree), false otherwise.
pub fn mining_system(world: &mut World, res: &mut Resources) -> bool {
    if !res.mining_intent {
        return false;
    }
    res.mining_intent = false;
    let Some(player_e) = res.player_entity else {
        return false;
    };
    let Ok(pos) = world.get::<&Position>(player_e) else {
        return false;
    };
    let (x, y, z) = (pos.x, pos.y, pos.z);
    // End immutable borrow before mutating the world
    drop(pos);
    let t = res.world.get_tile_cached(x, y);
    use crate::tiles::TileKind;
    match t {
        TileKind::Tree => {
            // Chop tree: convert to Dirt and drop Wood
            res.world.set_tile_cached(x, y, TileKind::Dirt);
            // Spawn a DroppedItem entity at player's tile
            let _ = world.spawn((
                Position { x, y, z },
                DroppedItem {
                    kind: ItemKind::Wood,
                    qty: 1,
                },
            ));
            res.log("You chop the tree. (+1 Wood)");
            true
        }
        _ => {
            // No-op for other tiles for now
            false
        }
    }
}

/// When the player is on the same tile as any DroppedItem, pick it up into Inventory.
pub fn pickup_system(world: &mut World, res: &mut Resources) {
    let Some(player_e) = res.player_entity else {
        return;
    };

    // Get player position and immediately drop the borrow
    let (px, py, pz) = {
        let Ok(ppos) = world.get::<&Position>(player_e) else {
            return;
        };
        (ppos.x, ppos.y, ppos.z)
    };

    // Collect item entities to pick up
    let mut pickups: Vec<hecs::Entity> = Vec::new();
    let mut items: Vec<DroppedItem> = Vec::new();

    // First pass: collect items and their entities
    for (e, (ipos, di)) in world.query::<(&Position, &DroppedItem)>().iter() {
        if ipos.x == px && ipos.y == py && ipos.z == pz {
            pickups.push(e);
            items.push(*di);
        }
    }

    // Early return if nothing to pick up
    if pickups.is_empty() {
        return;
    }

    // Process inventory updates
    if let Ok(mut inv) = world.get::<&mut Inventory>(player_e) {
        let mut total = 0u32;
        for di in &items {
            inv.add(di.kind, di.qty);
            total += di.qty;
        }
        if total > 0 {
            res.log(format!("Picked up {} Wood", total));
        }
    }

    // Remove the picked up items from the world
    for e in pickups {
        let _ = world.despawn(e);
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
        if dx == 0 && dy == 0 {
            continue;
        }
        let nx = pos.x + dx;
        let ny = pos.y + dy;
        let nz = pos.z; // Sheep stay on the same Z-level

        // Check if the target tile is passable
        let t = res.world.get_tile(nx, ny);
        if !t.is_passable() {
            continue;
        }

        // Avoid stepping into another blocking entity
        let mut occupied = false;
        for (other_e, (other_pos, _)) in world.query::<(&Position, &BlocksMovement)>().iter() {
            if other_e != e && other_pos.x == nx && other_pos.y == ny && other_pos.z == nz {
                occupied = true;
                break;
            }
        }
        if occupied {
            continue;
        }
        if let Ok(mut mypos) = world.get::<&mut Position>(e) {
            mypos.x = nx;
            mypos.y = ny;
        }
    }
}
