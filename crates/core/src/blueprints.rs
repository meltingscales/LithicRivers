//! Blueprints: copy the player-built blocks in an area and paste them elsewhere, rotated,
//! paid for from the player's inventory. A paste is all-or-nothing.

use crate::components::{
    itemkind_name, itemkind_sprite_name, DroppedItem, Inventory, ItemKind, Position, SpriteRef,
};
use crate::factory::{self, Facing, FactoryBlock, FactoryKind};
use crate::resources::Resources;
use crate::tiles::TileKind;
use hecs::{Entity, World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Plain blocks only the player builds, with the item each one costs. Rock is left out:
/// placed stone can't be told apart from the rock the world generates.
const BUILT_TILES: &[(TileKind, ItemKind)] = &[(TileKind::PlankBlock, ItemKind::PlankBlock)];

fn built_tile_item(tile: TileKind) -> Option<ItemKind> {
    BUILT_TILES
        .iter()
        .find(|(t, _)| *t == tile)
        .map(|&(_, item)| item)
}

/// What a blueprint places at one tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Piece {
    Factory(FactoryKind),
    Tile(TileKind),
}

impl Piece {
    /// The item one of these costs.
    pub fn item(self) -> Option<ItemKind> {
        match self {
            Piece::Factory(kind) => kind.item(),
            Piece::Tile(tile) => built_tile_item(tile),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintBlock {
    /// Offset from the blueprint's top-left corner
    pub dx: i64,
    pub dy: i64,
    pub piece: Piece,
    pub facing: Facing,
    pub filter: Option<ItemKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blueprint {
    pub name: String,
    pub width: i64,
    pub height: i64,
    pub blocks: Vec<BlueprintBlock>,
}

impl Blueprint {
    /// The blueprint turned 90 degrees clockwise about its own area, facings included.
    pub fn rotated_cw(&self) -> Blueprint {
        Blueprint {
            name: self.name.clone(),
            width: self.height,
            height: self.width,
            blocks: self
                .blocks
                .iter()
                .map(|b| BlueprintBlock {
                    dx: self.height - 1 - b.dy,
                    dy: b.dx,
                    facing: b.facing.rotate_cw(),
                    ..*b
                })
                .collect(),
        }
    }

    /// Turned clockwise `quarter_turns` times.
    pub fn rotated(&self, quarter_turns: u8) -> Blueprint {
        (0..quarter_turns % 4).fold(self.clone(), |bp, _| bp.rotated_cw())
    }

    /// Items it takes to paste, in item order.
    pub fn cost(&self) -> BTreeMap<ItemKind, u32> {
        let mut cost = BTreeMap::new();
        for item in self.blocks.iter().filter_map(|b| b.piece.item()) {
            *cost.entry(item).or_insert(0) += 1;
        }
        cost
    }
}

/// The saved blueprints, with one selected for pasting.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintLibrary {
    pub blueprints: Vec<Blueprint>,
    pub selected: usize,
}

impl BlueprintLibrary {
    pub fn selected(&self) -> Option<&Blueprint> {
        self.blueprints.get(self.selected)
    }

    /// Add a blueprint and select it.
    pub fn add(&mut self, blueprint: Blueprint) {
        self.blueprints.push(blueprint);
        self.selected = self.blueprints.len() - 1;
    }

    /// A name no saved blueprint has yet: "Blueprint 1", "Blueprint 2", ...
    pub fn next_name(&self) -> String {
        (1..)
            .map(|n| format!("Blueprint {n}"))
            .find(|name| self.blueprints.iter().all(|b| &b.name != name))
            .expect("some number is free")
    }

    pub fn remove_selected(&mut self) -> Option<Blueprint> {
        if self.selected >= self.blueprints.len() {
            return None;
        }
        let removed = self.blueprints.remove(self.selected);
        self.selected = self.selected.min(self.blueprints.len().saturating_sub(1));
        Some(removed)
    }
}

/// Every tile of the rectangle with corners `a` and `b` (on `a`'s level), top-left first.
fn area(a: Position, b: Position) -> (Position, i64, i64) {
    let corner = Position {
        x: a.x.min(b.x),
        y: a.y.min(b.y),
        z: a.z,
    };
    (corner, (a.x - b.x).abs() + 1, (a.y - b.y).abs() + 1)
}

/// What a tile holds that the player built.
fn built_at(
    world: &World,
    res: &mut Resources,
    pos: Position,
) -> Option<(Piece, Facing, Option<ItemKind>)> {
    if let Some(&entity) = res.factory_index.get(&pos) {
        let block = *world.get::<&FactoryBlock>(entity).ok()?;
        return Some((
            Piece::Factory(block.kind),
            block.facing,
            crate::logistics::filter(world, entity),
        ));
    }
    let tile = res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z);
    built_tile_item(tile).map(|_| (Piece::Tile(tile), Facing::default(), None))
}

/// Copy the player-built blocks in the rectangle with corners `a` and `b`: their kinds,
/// facings and filters, but not what they hold. Terrain is left out.
pub fn capture(
    world: &World,
    res: &mut Resources,
    a: Position,
    b: Position,
    name: String,
) -> Blueprint {
    let (corner, width, height) = area(a, b);
    let mut blocks = Vec::new();
    for dy in 0..height {
        for dx in 0..width {
            let pos = Position {
                x: corner.x + dx,
                y: corner.y + dy,
                z: corner.z,
            };
            if let Some((piece, facing, filter)) = built_at(world, res, pos) {
                blocks.push(BlueprintBlock {
                    dx,
                    dy,
                    piece,
                    facing,
                    filter,
                });
            }
        }
    }
    Blueprint {
        name,
        width,
        height,
        blocks,
    }
}

/// What pasting would do at one tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Open terrain: built over without asking
    Free,
    /// A player-built block that would be broken into the inventory first
    Replace,
    /// Solid terrain, a creature, an item or the player: the paste can't go here
    Blocked,
}

/// Where each block of the blueprint would go with its top-left corner at `anchor`, and
/// what is there now.
pub fn targets(
    world: &World,
    res: &mut Resources,
    blueprint: &Blueprint,
    anchor: Position,
) -> Vec<(Position, Target)> {
    blueprint
        .blocks
        .iter()
        .map(|b| {
            let pos = Position {
                x: anchor.x + b.dx,
                y: anchor.y + b.dy,
                z: anchor.z,
            };
            let target = if built_at(world, res, pos).is_some() {
                Target::Replace
            } else if factory::can_place_at(world, res, pos) {
                Target::Free
            } else {
                Target::Blocked
            };
            (pos, target)
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PasteError {
    /// The blueprint has no blocks
    Empty,
    /// These tiles hold rock, trees, creatures or items
    Blocked(usize),
    /// Short of these items (with how many more are needed)
    Missing(Vec<(ItemKind, u32)>),
    /// This many player-built blocks would be replaced: paste again with `replace` to confirm
    NeedsConfirm(usize),
}

impl PasteError {
    pub fn message(&self) -> String {
        match self {
            PasteError::Empty => "That blueprint has no blocks".to_string(),
            PasteError::Blocked(n) => format!("Can't paste here: {} blocked", tiles(*n)),
            PasteError::Missing(missing) => {
                let list: Vec<String> = missing
                    .iter()
                    .map(|&(item, n)| format!("{n} more {}", plural(item, n)))
                    .collect();
                format!("Not enough to paste: need {}", list.join(", "))
            }
            PasteError::NeedsConfirm(n) => format!(
                "Pasting here replaces {} you built; paste again to confirm",
                if *n == 1 {
                    "1 block".to_string()
                } else {
                    format!("{n} blocks")
                }
            ),
        }
    }
}

fn tiles(n: usize) -> String {
    if n == 1 {
        "1 tile".to_string()
    } else {
        format!("{n} tiles")
    }
}

fn plural(item: ItemKind, n: u32) -> String {
    let name = itemkind_name(item).to_lowercase();
    if n == 1 {
        name
    } else {
        format!("{name}s")
    }
}

fn count(world: &World, entity: Entity, item: ItemKind) -> u32 {
    world
        .get::<&Inventory>(entity)
        .map(|inv| {
            inv.slots
                .iter()
                .filter(|s| s.kind == item)
                .map(|s| s.qty)
                .sum()
        })
        .unwrap_or(0)
}

fn take(world: &mut World, entity: Entity, item: ItemKind, mut qty: u32) {
    if let Ok(mut inv) = world.get::<&mut Inventory>(entity) {
        for slot in inv.slots.iter_mut().filter(|s| s.kind == item) {
            let n = slot.qty.min(qty);
            slot.qty -= n;
            qty -= n;
        }
        inv.slots.retain(|s| s.qty > 0);
    }
}

/// Put items in the player's inventory, or on the ground under the player if it is full.
fn give_or_drop(world: &mut World, player: Entity, item: ItemKind, qty: u32) {
    let fits = world.get::<&Inventory>(player).is_ok_and(|inv| {
        inv.slots.iter().any(|s| s.kind == item) || inv.slots.len() < inv.max_slots
    });
    if fits {
        if let Ok(mut inv) = world.get::<&mut Inventory>(player) {
            inv.add(item, qty);
        }
    } else if let Ok(pos) = world.get::<&Position>(player).map(|p| *p) {
        world.spawn((
            pos,
            DroppedItem { kind: item, qty },
            SpriteRef::new("items", itemkind_sprite_name(item)),
        ));
    }
}

/// Break the player-built block at `pos` into the player's inventory, contents included.
fn break_into_inventory(world: &mut World, res: &mut Resources, pos: Position, player: Entity) {
    if factory::break_block(world, res, pos, player).is_some() {
        // `break_block` drops what the block held on its tile; pick it all up
        let spilled: Vec<(Entity, DroppedItem)> = world
            .query::<(&Position, &DroppedItem)>()
            .iter()
            .filter(|(_, (p, _))| **p == pos)
            .map(|(e, (_, d))| (e, *d))
            .collect();
        for (entity, item) in spilled {
            let _ = world.despawn(entity);
            give_or_drop(world, player, item.kind, item.qty);
        }
        return;
    }
    let tile = res.world_state.world.get_tile_cached(pos.x, pos.y, pos.z);
    if let Some(item) = built_tile_item(tile) {
        res.world_state
            .world
            .set_tile_cached(pos.x, pos.y, pos.z, TileKind::Air);
        give_or_drop(world, player, item, 1);
    }
}

/// Paste `blueprint` with its top-left corner at `anchor`, paid from `player`'s inventory.
/// Places everything or nothing. Covering player-built blocks needs `replace`; they are
/// broken into the inventory first, and their items count toward the cost.
pub fn paste(
    world: &mut World,
    res: &mut Resources,
    blueprint: &Blueprint,
    anchor: Position,
    player: Entity,
    replace: bool,
) -> Result<usize, PasteError> {
    if blueprint.blocks.is_empty() {
        return Err(PasteError::Empty);
    }
    let targets = targets(world, res, blueprint, anchor);
    let blocked = targets
        .iter()
        .filter(|(_, t)| *t == Target::Blocked)
        .count();
    if blocked > 0 {
        return Err(PasteError::Blocked(blocked));
    }

    // Replaced blocks come back as items, so they help pay
    let mut refunds: BTreeMap<ItemKind, u32> = BTreeMap::new();
    for (pos, _) in targets.iter().filter(|(_, t)| *t == Target::Replace) {
        if let Some(item) = built_at(world, res, *pos).and_then(|(piece, ..)| piece.item()) {
            *refunds.entry(item).or_insert(0) += 1;
        }
    }
    let missing: Vec<(ItemKind, u32)> = blueprint
        .cost()
        .into_iter()
        .filter_map(|(item, need)| {
            let have = count(world, player, item) + refunds.get(&item).copied().unwrap_or(0);
            (have < need).then(|| (item, need - have))
        })
        .collect();
    if !missing.is_empty() {
        return Err(PasteError::Missing(missing));
    }
    let replaced = targets
        .iter()
        .filter(|(_, t)| *t == Target::Replace)
        .count();
    if replaced > 0 && !replace {
        return Err(PasteError::NeedsConfirm(replaced));
    }

    for (pos, _) in targets.iter().filter(|(_, t)| *t == Target::Replace) {
        break_into_inventory(world, res, *pos, player);
    }
    for (item, n) in blueprint.cost() {
        take(world, player, item, n);
    }
    for (block, (pos, _)) in blueprint.blocks.iter().zip(&targets) {
        match block.piece {
            Piece::Factory(kind) => {
                if let Ok(entity) = factory::place(world, res, kind, *pos, block.facing) {
                    crate::logistics::set_filter(world, entity, block.filter);
                }
            }
            Piece::Tile(tile) => res
                .world_state
                .world
                .set_tile_cached(pos.x, pos.y, pos.z, tile),
        }
    }
    Ok(blueprint.blocks.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{TestGame, TestWorld};

    fn pos(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn site(inventory: &[(ItemKind, u32)]) -> TestGame {
        TestWorld::new(3)
            .fill((-8, -8, 0), (8, 8, 0), TileKind::Grass)
            .player_at(0, 5, 0)
            .inventory(inventory)
            .build()
    }

    fn place(t: &mut TestGame, kind: FactoryKind, x: i64, y: i64, facing: Facing) -> Entity {
        factory::place(&mut t.game.world, &mut t.game.res, kind, pos(x, y), facing).unwrap()
    }

    fn block_at(t: &TestGame, x: i64, y: i64) -> Option<FactoryBlock> {
        let e = *t.game.res.factory_index.get(&pos(x, y))?;
        t.game.world.get::<&FactoryBlock>(e).ok().map(|b| *b)
    }

    fn player(t: &TestGame) -> Entity {
        t.game.get_player_entity().unwrap()
    }

    /// Three east belts at (0..3, 0) and a chest holding logs at (3, 0).
    fn belt_line(t: &mut TestGame) -> Blueprint {
        for x in 0..3 {
            place(t, FactoryKind::Belt, x, 0, Facing::East);
        }
        let chest = place(t, FactoryKind::Chest, 3, 0, Facing::North);
        t.game
            .world
            .get::<&mut Inventory>(chest)
            .unwrap()
            .add(ItemKind::Log, 5);
        let game = &mut t.game;
        capture(
            &game.world,
            &mut game.res,
            pos(3, 0),
            pos(0, 0),
            "line".into(),
        )
    }

    #[test]
    fn capture_keeps_kinds_facings_and_filters_but_not_contents() {
        let mut t = site(&[]);
        let blueprint = belt_line(&mut t);
        let router = place(&mut t, FactoryKind::BeltRouter, 0, 1, Facing::West);
        crate::logistics::set_filter(&mut t.game.world, router, Some(ItemKind::Acorn));
        assert_eq!((blueprint.width, blueprint.height), (4, 1));
        let kinds: Vec<_> = blueprint
            .blocks
            .iter()
            .map(|b| (b.dx, b.piece, b.facing))
            .collect();
        assert_eq!(
            kinds,
            vec![
                (0, Piece::Factory(FactoryKind::Belt), Facing::East),
                (1, Piece::Factory(FactoryKind::Belt), Facing::East),
                (2, Piece::Factory(FactoryKind::Belt), Facing::East),
                (3, Piece::Factory(FactoryKind::Chest), Facing::North),
            ]
        );
        assert_eq!(
            blueprint.cost().get(&ItemKind::Log),
            None,
            "contents are left out"
        );
        let game = &mut t.game;
        let with_router = capture(&game.world, &mut game.res, pos(0, 0), pos(0, 1), "r".into());
        assert_eq!(with_router.blocks[1].filter, Some(ItemKind::Acorn));
        assert_eq!(with_router.blocks[1].dy, 1);
    }

    #[test]
    fn rotating_turns_offsets_and_facings() {
        let mut t = site(&[]);
        let blueprint = belt_line(&mut t).rotated(1);
        assert_eq!((blueprint.width, blueprint.height), (1, 4));
        let rotated: Vec<_> = blueprint
            .blocks
            .iter()
            .map(|b| (b.dx, b.dy, b.facing))
            .collect();
        assert_eq!(
            rotated,
            vec![
                (0, 0, Facing::South),
                (0, 1, Facing::South),
                (0, 2, Facing::South),
                (0, 3, Facing::East),
            ]
        );
        assert_eq!(
            blueprint.rotated(3).rotated(1),
            blueprint,
            "four turns go all the way round"
        );
    }

    #[test]
    fn a_short_inventory_places_nothing_and_says_what_is_missing() {
        let mut t = site(&[(ItemKind::WoodenConveyor, 2)]);
        let blueprint = Blueprint {
            name: "belts".into(),
            width: 4,
            height: 1,
            blocks: (0..4)
                .map(|dx| BlueprintBlock {
                    dx,
                    dy: 0,
                    piece: Piece::Factory(FactoryKind::Belt),
                    facing: Facing::East,
                    filter: None,
                })
                .collect(),
        };
        let p = player(&t);
        let game = &mut t.game;
        let err = paste(
            &mut game.world,
            &mut game.res,
            &blueprint,
            pos(-4, -4),
            p,
            false,
        )
        .unwrap_err();
        assert_eq!(
            err,
            PasteError::Missing(vec![(ItemKind::WoodenConveyor, 2)])
        );
        assert_eq!(
            err.message(),
            "Not enough to paste: need 2 more wooden conveyors"
        );
        assert_eq!(block_at(&t, -4, -4), None);
        assert_eq!(t.player_has(ItemKind::WoodenConveyor), 2);
    }

    #[test]
    fn rock_blocks_a_paste() {
        let mut t = site(&[(ItemKind::WoodenConveyor, 3), (ItemKind::Chest, 1)]);
        let blueprint = belt_line(&mut t);
        t.game
            .res
            .world_state
            .world
            .set_tile_cached(-3, -4, 0, TileKind::Rock);
        let p = player(&t);
        let game = &mut t.game;
        let err = paste(
            &mut game.world,
            &mut game.res,
            &blueprint,
            pos(-4, -4),
            p,
            true,
        )
        .unwrap_err();
        assert_eq!(err, PasteError::Blocked(1));
        assert_eq!(block_at(&t, -4, -4), None, "nothing is placed");
    }

    #[test]
    fn an_exact_paste_uses_the_inventory_and_keeps_filters() {
        let mut t = site(&[
            (ItemKind::WoodenConveyor, 3),
            (ItemKind::Chest, 1),
            (ItemKind::PlankBlock, 1),
        ]);
        let mut blueprint = belt_line(&mut t);
        blueprint.blocks.push(BlueprintBlock {
            dx: 0,
            dy: 0,
            piece: Piece::Tile(TileKind::PlankBlock),
            facing: Facing::North,
            filter: None,
        });
        blueprint.blocks[0].dy = 1;
        blueprint.height = 2;
        let p = player(&t);
        let game = &mut t.game;
        assert_eq!(
            paste(
                &mut game.world,
                &mut game.res,
                &blueprint,
                pos(-4, -4),
                p,
                false
            ),
            Ok(5)
        );
        assert_eq!(block_at(&t, -4, -3).map(|b| b.facing), Some(Facing::East));
        assert_eq!(
            block_at(&t, -1, -4).map(|b| b.kind),
            Some(FactoryKind::Chest)
        );
        assert_eq!(t.tile(-4, -4, 0), TileKind::PlankBlock);
        assert_eq!(t.player_inventory(), vec![], "every item is used");
    }

    #[test]
    fn pasting_over_a_chest_needs_confirming_and_refunds_it_with_its_contents() {
        let mut t = site(&[(ItemKind::WoodenConveyor, 3)]);
        let blueprint = belt_line(&mut t);
        // The chest from the copy is the only chest: pasting over it pays for itself
        let p = player(&t);
        let game = &mut t.game;
        let err = paste(
            &mut game.world,
            &mut game.res,
            &blueprint,
            pos(0, 0),
            p,
            false,
        )
        .unwrap_err();
        assert_eq!(err, PasteError::NeedsConfirm(4));
        let game = &mut t.game;
        assert_eq!(
            paste(
                &mut game.world,
                &mut game.res,
                &blueprint,
                pos(0, 0),
                p,
                true
            ),
            Ok(4)
        );
        assert_eq!(
            t.player_has(ItemKind::Log),
            5,
            "the chest's logs come to the inventory"
        );
        assert_eq!(t.player_has(ItemKind::WoodenConveyor), 3);
        assert_eq!(t.player_has(ItemKind::Chest), 0);
        assert_eq!(block_at(&t, 3, 0).map(|b| b.kind), Some(FactoryKind::Chest));
        assert!(
            t.items_at(3, 0, 0).is_empty(),
            "nothing is left on the ground"
        );
    }

    #[test]
    fn library_names_and_selects() {
        let mut library = BlueprintLibrary::default();
        assert_eq!(library.next_name(), "Blueprint 1");
        library.add(Blueprint {
            name: library.next_name(),
            width: 1,
            height: 1,
            blocks: vec![],
        });
        library.add(Blueprint {
            name: "Farm".into(),
            width: 1,
            height: 1,
            blocks: vec![],
        });
        assert_eq!(library.selected().map(|b| b.name.as_str()), Some("Farm"));
        assert_eq!(library.next_name(), "Blueprint 2");
        library.remove_selected();
        assert_eq!(
            library.selected().map(|b| b.name.as_str()),
            Some("Blueprint 1")
        );
    }

    #[test]
    fn the_library_survives_a_save_and_load() {
        let mut t = site(&[]);
        let blueprint = belt_line(&mut t);
        t.game.res.blueprints.add(blueprint.clone());
        let json = crate::save_load::save_to_string(&t.game, &serde_json::Value::Null).unwrap();
        let mut loaded = site(&[]);
        crate::save_load::load_from_str(&mut loaded.game, &json).unwrap();
        assert_eq!(loaded.game.res.blueprints.selected(), Some(&blueprint));
    }
}
