use crate::component_access::{ComponentAccess, ComponentUpdate};
use crate::components::{BlocksMovement, DroppedItem, Inventory, Position, SpriteRef};
use crate::intent::PlayerAction;
use crate::resources::Resources;
use crate::systems::ecs_utils::get_player_entity;
use hecs::World;
use tracing::info;

pub fn move_player_system(world: &mut World, res: &mut Resources) {
    if let Some(action) = res.player_state.intent.take() {
        if let Some(player_e) = get_player_entity(world) {
            match action {
                PlayerAction::Move { dx, dy, dz } => {
                    // Read current position and check for movement blocking
                    let (cx, cy, cz) = if let Ok(pos) = world.get::<&Position>(player_e) {
                        (pos.x, pos.y, pos.z)
                    } else {
                        return; // No position component
                    };

                    let nx = cx + dx;
                    let ny = cy + dy;
                    let nz = cz + dz;

                    // Check movement blocking (horizontal and vertical)
                    let mut blocked = false;

                    // Skip collision checks if noclip is enabled
                    if !res.player_state.noclip_enabled {
                        if dx != 0 || dy != 0 {
                            // Horizontal movement collision
                            let t = res.world_state.world.get_tile_cached(nx, ny, cz);
                            blocked = !t.is_passable();
                            if !blocked {
                                for (_, (_, epos)) in
                                    world.query::<(&BlocksMovement, &Position)>().iter()
                                {
                                    if epos.x == nx && epos.y == ny && epos.z == cz {
                                        blocked = true;
                                        break;
                                    }
                                }
                            }
                            if blocked {
                                info!("Blocked by {:?} at ({}, {})", t, nx, ny);
                                res.player_state.last_blocked_tile = Some((nx, ny));
                            }
                        } else if dz != 0 {
                            // Vertical movement - requires stairs at current position AND destination
                            let current_tile = res.world_state.world.get_tile_cached(cx, cy, cz);
                            use crate::tiles::TileKind;
                            if current_tile != TileKind::Stairs {
                                blocked = true;
                                info!(
                                    "Vertical movement blocked - not on stairs at ({}, {}, {})",
                                    cx, cy, cz
                                );
                                res.events.interaction_event(
                                    "You need to be on stairs to move up or down.".to_string(),
                                    res.time.tick,
                                );
                            } else {
                                // Also check destination has stairs to prevent stranding
                                let destination_tile =
                                    res.world_state.world.get_tile_cached(nx, ny, nz);
                                if destination_tile != TileKind::Stairs {
                                    blocked = true;
                                    info!(
                                        "Vertical movement blocked - no stairs at destination ({}, {}, {})",
                                        nx, ny, nz
                                    );
                                    res.events.interaction_event(
                                        "Cannot move there - no stairs at the destination level."
                                            .to_string(),
                                        res.time.tick,
                                    );
                                }
                            }
                        }
                    } else {
                        // Noclip enabled - bypassing all collision checks
                        info!(
                            "Noclip enabled - allowing movement from ({},{},{}) to ({},{},{})",
                            cx, cy, cz, nx, ny, nz
                        );
                    }

                    // Apply movement if not blocked
                    if !blocked {
                        let new_position = Position {
                            x: nx,
                            y: ny,
                            z: nz,
                        };
                        let mut comp_access = ComponentAccess::new(world, res);
                        if let Err(e) = comp_access.atomic_update(
                            player_e,
                            ComponentUpdate::Move {
                                new_position,
                                remove_combat_delay: false,
                            },
                        ) {
                            tracing::warn!(target: "systems", "Failed to update player position: {}", e);
                        }
                    }
                }
                PlayerAction::Interact => {
                    // Handle interaction with nearby entities
                    // Note: interaction_system function is not included as it's not directly depended upon
                    // by the move_player_system logic flow
                }
                PlayerAction::InteractWith { target_id: _ } => {
                    // Handle interaction with specific target
                    // Note: interaction_with_target_system function is not included as it's not directly depended upon
                    // by the move_player_system logic flow
                }
                _ => {
                    // Other actions handled by different systems
                }
            }
        }
    }
}

/// Process mining intent: if the player requested mining, act on current tile.
/// Returns true if mining was successful (e.g. chopped a tree), false otherwise.
pub fn mining_system(world: &mut World, res: &mut Resources) -> bool {
    // Check if there's a mining action in the intent and get coordinates
    let (x, y, z) = match &res.player_state.intent.action {
        Some(PlayerAction::Mine) => {
            // Mine at player's current position
            let Some(player_e) = get_player_entity(world) else {
                return false;
            };
            let Ok(pos) = world.get::<&Position>(player_e) else {
                return false;
            };
            let coords = (pos.x, pos.y, pos.z);
            // End immutable borrow before mutating the world
            drop(pos);
            coords
        }
        Some(PlayerAction::MineAt { x, y, z }) => {
            // Mine at specified coordinates
            (*x, *y, *z)
        }
        _ => {
            return false;
        }
    };
    let tile_to_mine = res.world_state.world.get_tile_cached(x, y, z);
    use crate::tiles::TileKind;
    match tile_to_mine {
        TileKind::Sapling => {
            let Some(player) = get_player_entity(world) else {
                return false;
            };
            let pulled = crate::plants::break_sapling(world, res, Position { x, y, z }, player);
            if pulled {
                res.events.interaction_event(
                    "You pull up the sapling. (+1 Acorn)".to_string(),
                    res.time.tick,
                );
            }
            pulled
        }
        TileKind::FactoryBlock => {
            let Some(player) = get_player_entity(world) else {
                return false;
            };
            match crate::factory::break_block(world, res, Position { x, y, z }, player) {
                Some(kind) => {
                    res.events.interaction_event(
                        format!("You break the {}.", kind.name()),
                        res.time.tick,
                    );
                    true
                }
                None => false,
            }
        }
        tile => {
            let Some(gained) = crate::crafting::mine_tile(res, Position { x, y, z }) else {
                res.events.interaction_event(
                    format!("You can't mine the {:?} tile.", tile_to_mine),
                    res.time.tick,
                );
                return false;
            };
            for &(kind, qty) in &gained {
                world.spawn((
                    Position { x, y, z },
                    DroppedItem { kind, qty },
                    SpriteRef::new("items", crate::components::itemkind_sprite_name(kind)),
                ));
            }
            let message = if tile == TileKind::Tree {
                format!(
                    "You chop the tree. (+{} Wood, +{} Acorn)",
                    gained[0].1, gained[1].1
                )
            } else {
                let items: Vec<String> = gained
                    .iter()
                    .map(|(kind, qty)| {
                        format!("+{qty} {}", crate::components::itemkind_name(*kind))
                    })
                    .collect();
                format!(
                    "You mine the {}. ({})",
                    tile.display_name(),
                    items.join(", ")
                )
            };
            res.events.interaction_event(message, res.time.tick);
            true
        }
    }
}
/// When the player is on the same tile as any DroppedItem, pick it up into Inventory.
pub fn pickup_system(world: &mut World, res: &mut Resources) {
    let Some(player_e) = get_player_entity(world) else {
        return;
    };

    // If Item Auto Pickup is toggled OFF, skip
    if let Ok(inv) = world.get::<&Inventory>(player_e) {
        if !inv.auto_pickup {
            // info!("ItemAutoPickup is toggled off, skipping pickup system");
            return;
        }
    }

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

    // Process inventory updates using safe component access
    let mut comp_access = ComponentAccess::new(world, res);
    if let Ok(()) = comp_access.modify_inventory(player_e, |inv| {
        let mut total = 0u32;
        for di in &items {
            inv.add(di.kind, di.qty);
            total += di.qty;
        }
        if total > 0 {
            // This will be handled outside the closure
        }
    }) {
        let total: u32 = items.iter().map(|di| di.qty).sum();
        if total > 0 {
            res.events
                .interaction_event(format!("Picked up {} items", total), res.time.tick);
        }
    }

    // Remove the picked up items from the world
    for e in pickups {
        let _ = world.despawn(e);
    }
}

#[cfg(test)]
mod tests {
    use crate::components::ItemKind;
    use crate::intent::PlayerAction;
    use crate::test_support::TestWorld;
    use crate::tiles::TileKind;

    fn chop_tree_drops(seed: u64) -> Vec<(ItemKind, u32)> {
        let mut game = TestWorld::new(seed).tile(1, 0, 0, TileKind::Tree).build();
        game.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        game.tick();
        game.items_at(1, 0, 0)
    }

    #[test]
    fn test_tree_chop_drops_are_deterministic() {
        for seed in [0, 1, 12345, u64::MAX] {
            let first = chop_tree_drops(seed);
            assert_eq!(
                first,
                chop_tree_drops(seed),
                "seed {seed} not deterministic"
            );
            assert_eq!(first.len(), 2, "expected logs and acorns: {first:?}");
            for (kind, qty) in first {
                match kind {
                    ItemKind::Log => assert!((2..=3).contains(&qty)),
                    ItemKind::Acorn => assert!((1..=3).contains(&qty)),
                    other => panic!("unexpected drop {other:?}"),
                }
            }
        }
    }

    fn walk_onto_logs(auto_pickup: bool) -> crate::test_support::TestGame {
        let mut world = TestWorld::new(3)
            .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
            .inventory(&[])
            .auto_pickup(auto_pickup);
        world.item(1, 0, 0, ItemKind::Log, 4);
        let mut game = world.build();
        game.act(PlayerAction::Move {
            dx: 1,
            dy: 0,
            dz: 0,
        });
        game.tick();
        assert_eq!(game.player_pos().x, 1);
        game
    }

    #[test]
    fn walking_onto_an_item_picks_it_up() {
        let game = walk_onto_logs(true);
        assert_eq!(game.player_inventory(), vec![(ItemKind::Log, 4)]);
        assert!(game.items_at(1, 0, 0).is_empty());
    }

    #[test]
    fn auto_pickup_off_leaves_the_item() {
        let game = walk_onto_logs(false);
        assert!(game.player_inventory().is_empty());
        assert_eq!(game.items_at(1, 0, 0), vec![(ItemKind::Log, 4)]);
    }
}
