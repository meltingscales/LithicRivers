//! Where materials come from and how they combine: the mining drop table, raw sources,
//! placeable items, the reachability check over recipes, and the ingredient tree.

use std::collections::BTreeSet;

use strum::IntoEnumIterator;

use crate::components::{itemkind_name, ItemKind};
use crate::recipe_handler::Recipe;
use crate::spawn_utils::EnemyKind;
use crate::tiles::TileKind;

/// What mining a tile leaves behind and drops. Each item drops `min..=max` (seeded by the
/// world seed and the tile's position when the range is wider than one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiningDrop {
    pub becomes: TileKind,
    pub items: &'static [(ItemKind, u32, u32)],
}

/// The mining drop table. Tiles not listed can't be mined.
pub fn mining_drop(tile: TileKind) -> Option<MiningDrop> {
    let drop = |becomes, items| Some(MiningDrop { becomes, items });
    match tile {
        TileKind::Tree => drop(
            TileKind::Dirt,
            &[(ItemKind::Log, 2, 3), (ItemKind::Acorn, 1, 3)],
        ),
        TileKind::PlankBlock => drop(TileKind::Air, &[(ItemKind::PlankBlock, 1, 1)]),
        TileKind::Stairs => drop(TileKind::Air, &[(ItemKind::Stairs, 1, 1)]),
        TileKind::Rock => drop(TileKind::Air, &[(ItemKind::Stone, 1, 1)]),
        TileKind::IronScrap => drop(TileKind::Air, &[(ItemKind::IronScrap, 1, 2)]),
        TileKind::ScrapElectronics => drop(TileKind::Air, &[(ItemKind::ScrapElectronics, 1, 1)]),
        TileKind::PlasteelScrap => drop(TileKind::Air, &[(ItemKind::PlasteelScrap, 1, 1)]),
        _ => None,
    }
}

/// Mine the tile at `pos`: it becomes what the drop table says, and the rolled drops are
/// returned (items rolling 0 are left out). The player and machines both mine through here,
/// so the same tile always drops the same. `None` if the tile can't be mined.
pub fn mine_tile(
    res: &mut crate::resources::Resources,
    pos: crate::components::Position,
) -> Option<Vec<(ItemKind, u32)>> {
    use rand::Rng;
    let world = &mut res.world_state.world;
    let drop = mining_drop(world.get_tile_cached(pos.x, pos.y, pos.z))?;
    world.set_tile_cached(pos.x, pos.y, pos.z, drop.becomes);
    // Seeded by world seed + tile coords so drops are reproducible
    let mut rng = world.new_rng(pos.x, pos.y, pos.z);
    let gained = drop
        .items
        .iter()
        .map(|&(kind, min, max)| {
            let qty = if min == max {
                min
            } else {
                rng.gen_range(min..=max)
            };
            (kind, qty)
        })
        .filter(|&(_, qty)| qty > 0)
        .collect();
    Some(gained)
}

/// One kind of item a creature can carry: with `chance_percent`, it carries `min..=max`
/// of it, rolled when it spawns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureLoot {
    pub item: ItemKind,
    pub chance_percent: u32,
    pub min: u32,
    pub max: u32,
}

const fn always(item: ItemKind, min: u32, max: u32) -> CreatureLoot {
    CreatureLoot {
        item,
        chance_percent: 100,
        min,
        max,
    }
}

/// What a feral dog can carry.
pub const FERAL_DOG_LOOT: &[CreatureLoot] = &[
    always(ItemKind::Leather, 0, 2),
    always(ItemKind::Meat, 1, 3),
];

/// What a nuclear slime can carry: rarely, a lump of nuclear waste.
pub const NUCLEAR_SLIME_LOOT: &[CreatureLoot] = &[CreatureLoot {
    item: ItemKind::NuclearWaste,
    chance_percent: 5,
    min: 1,
    max: 1,
}];

/// Every creature's loot, with how the Item Dictionary names the creature. Each can also
/// drop an embryo of its own kind when it dies (`spawner::roll_embryo`).
pub const CREATURE_LOOT: &[(&str, EnemyKind, &[CreatureLoot])] = &[
    ("feral dogs", EnemyKind::FeralDog, FERAL_DOG_LOOT),
    (
        "nuclear slimes",
        EnemyKind::NuclearSlime,
        NUCLEAR_SLIME_LOOT,
    ),
];

/// Whether a creature can carry or drop `item`.
fn creature_gives(creature: EnemyKind, table: &[CreatureLoot], item: ItemKind) -> bool {
    table
        .iter()
        .any(|loot| loot.item == item && can_carry(loot))
        || crate::spawner::embryo_kind(item).is_some_and(|(c, _)| c == creature)
}

/// Roll a creature's loot. Entries that always drop skip the chance roll, so adding
/// chances never shifts the rolls of creatures that don't use them.
pub fn roll_loot(table: &[CreatureLoot], rng: &mut impl rand::Rng) -> Vec<(ItemKind, u32)> {
    let mut items = Vec::new();
    for loot in table {
        if loot.chance_percent < 100 && rng.gen_range(0..100) >= loot.chance_percent {
            continue;
        }
        let qty = rng.gen_range(loot.min..=loot.max);
        items.push((loot.item, qty));
    }
    items
}

fn can_carry(loot: &CreatureLoot) -> bool {
    loot.chance_percent > 0 && loot.max > 0
}

/// Items obtainable without crafting: everything the mining table drops, plus creature loot.
pub fn raw_sources() -> BTreeSet<ItemKind> {
    let mined = TileKind::all_variants()
        .iter()
        .filter_map(|&tile| mining_drop(tile))
        .flat_map(|drop| drop.items.iter())
        .filter(|(_, _, max)| *max > 0)
        .map(|(item, _, _)| *item);
    let carried = CREATURE_LOOT
        .iter()
        .flat_map(|(_, _, table)| table.iter())
        .filter(|loot| can_carry(loot))
        .map(|loot| loot.item);
    let embryos = CREATURE_LOOT.iter().flat_map(|&(_, creature, _)| {
        crate::spawner::EmbryoGrade::ALL.map(|g| crate::spawner::embryo(creature, g))
    });
    mined.chain(carried).chain(embryos).collect()
}

/// Where an item comes from without crafting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawSource {
    /// Mining or chopping this tile drops it
    Mined(TileKind),
    /// These creatures (named in the plural) can carry it
    Carried(&'static str),
}

impl RawSource {
    pub fn describe(self) -> String {
        match self {
            RawSource::Mined(tile) => format!("Mined from {}", tile.display_name()),
            RawSource::Carried(creatures) => format!("Carried by {creatures}"),
        }
    }
}

/// Every raw source of `item`, in tile variant order, then creature loot.
pub fn raw_sources_of(item: ItemKind) -> Vec<RawSource> {
    let mined = TileKind::all_variants().iter().filter(|&&tile| {
        mining_drop(tile).is_some_and(|d| d.items.iter().any(|&(i, _, max)| i == item && max > 0))
    });
    let mut sources: Vec<RawSource> = mined.map(|&tile| RawSource::Mined(tile)).collect();
    for &(creatures, creature, table) in CREATURE_LOOT {
        if creature_gives(creature, table, item) {
            sources.push(RawSource::Carried(creatures));
        }
    }
    sources
}

/// The recipes that make `item`.
pub fn made_by(recipes: &[Recipe], item: ItemKind) -> Vec<&Recipe> {
    recipes.iter().filter(|r| r.result == item).collect()
}

/// The recipes that use `item` as an ingredient.
pub fn used_in(recipes: &[Recipe], item: ItemKind) -> Vec<&Recipe> {
    recipes
        .iter()
        .filter(|r| r.ingredients.iter().any(|&(i, _)| i == item))
        .collect()
}

/// Items that can be placed in the world, as plain blocks or factory blocks.
pub fn placeable_items() -> Vec<ItemKind> {
    ItemKind::iter()
        .filter(|&item| {
            TileKind::from_item_kind(item).is_some()
                || crate::factory::FactoryKind::from_item(item).is_some()
        })
        .collect()
}

/// Items obtainable from `raw` by applying `recipes` any number of times.
pub fn obtainable(recipes: &[Recipe], raw: &BTreeSet<ItemKind>) -> BTreeSet<ItemKind> {
    let mut have = raw.clone();
    loop {
        let before = have.len();
        for recipe in recipes {
            if recipe.ingredients.iter().all(|(i, _)| have.contains(i)) {
                have.insert(recipe.result);
            }
        }
        if have.len() == before {
            return have;
        }
    }
}

/// For each item in `items` that can't be obtained, a line naming it and the first
/// ingredient missing on its first recipe (or that it has no recipe).
pub fn unobtainable(
    items: &[ItemKind],
    recipes: &[Recipe],
    raw: &BTreeSet<ItemKind>,
) -> Vec<String> {
    let have = obtainable(recipes, raw);
    items
        .iter()
        .filter(|item| !have.contains(item))
        .map(|&item| match recipes.iter().find(|r| r.result == item) {
            None => format!(
                "{} has no recipe and isn't a raw source",
                itemkind_name(item)
            ),
            Some(recipe) => {
                let missing = recipe
                    .ingredients
                    .iter()
                    .find(|(i, _)| !have.contains(i))
                    .map_or("?", |(i, _)| itemkind_name(*i));
                format!(
                    "{} can't be crafted: recipe \"{}\" needs {missing}, which can't be obtained",
                    itemkind_name(item),
                    recipe.name
                )
            }
        })
        .collect()
}

/// One line of an ingredient tree: an item at some depth, how many one craft of the top
/// recipe needs, and whether it's a raw material (a leaf).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeLine {
    pub depth: usize,
    pub item: ItemKind,
    pub needed: u32,
    pub raw: bool,
}

/// The ingredients of `crafts` crafts of `recipe`, expanded down to raw materials.
/// Intermediate items use their first recipe; an item already being expanded higher up ends
/// the branch.
pub fn ingredient_tree(
    recipes: &[Recipe],
    recipe: &Recipe,
    crafts: u32,
    raw: &BTreeSet<ItemKind>,
) -> Vec<TreeLine> {
    fn expand(
        recipes: &[Recipe],
        ingredients: &[(ItemKind, u32)],
        times: u32,
        depth: usize,
        raw: &BTreeSet<ItemKind>,
        path: &mut Vec<ItemKind>,
        out: &mut Vec<TreeLine>,
    ) {
        for &(item, qty) in ingredients {
            let needed = qty * times;
            let sub = recipes.iter().find(|r| r.result == item);
            let leaf = raw.contains(&item) || sub.is_none() || path.contains(&item);
            out.push(TreeLine {
                depth,
                item,
                needed,
                raw: leaf,
            });
            if let (false, Some(sub)) = (leaf, sub) {
                path.push(item);
                let crafts = needed.div_ceil(sub.quantity);
                expand(recipes, sub.ingredients, crafts, depth + 1, raw, path, out);
                path.pop();
            }
        }
    }
    let mut out = Vec::new();
    let mut path = vec![recipe.result];
    expand(
        recipes,
        recipe.ingredients,
        crafts,
        0,
        raw,
        &mut path,
        &mut out,
    );
    out
}

/// What it takes to craft at least `amount` of `item` with its first recipe, rounded up to
/// whole crafts: the recipe's ingredients, or with `raw`, the leaves of its ingredient tree
/// (raw materials, and items that end a branch), summed by item in tree order.
pub fn components_for(
    recipes: &[Recipe],
    item: ItemKind,
    amount: u32,
    raw: bool,
) -> Result<Vec<(ItemKind, u32)>, NoRecipe> {
    let recipe = recipes
        .iter()
        .find(|r| r.result == item)
        .ok_or(NoRecipe(item))?;
    let crafts = amount.max(1).div_ceil(recipe.quantity);
    if !raw {
        return Ok(recipe
            .ingredients
            .iter()
            .map(|&(i, q)| (i, q * crafts))
            .collect());
    }
    let mut leaves: Vec<(ItemKind, u32)> = Vec::new();
    for line in ingredient_tree(recipes, recipe, crafts, &raw_sources()) {
        if !line.raw {
            continue;
        }
        match leaves.iter_mut().find(|(i, _)| *i == line.item) {
            Some((_, q)) => *q += line.needed,
            None => leaves.push((line.item, line.needed)),
        }
    }
    Ok(leaves)
}

/// The item has no recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoRecipe(pub ItemKind);

impl std::fmt::Display for NoRecipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} has no recipe", itemkind_name(self.0))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn item_relationships_for_logs() {
        let handler = crate::recipe_handler::RecipeHandler::new();
        let recipes = handler.get_recipes();
        let uses: Vec<&str> = used_in(recipes, ItemKind::Log)
            .iter()
            .map(|r| r.name)
            .collect();
        assert!(uses.contains(&"Wooden Planks"), "{uses:?}");
        assert!(uses.contains(&"Power Pole"), "{uses:?}");
        assert!(made_by(recipes, ItemKind::Log).is_empty());
        assert_eq!(
            raw_sources_of(ItemKind::Log),
            [RawSource::Mined(TileKind::Tree)]
        );
        assert_eq!(
            raw_sources_of(ItemKind::Leather),
            [RawSource::Carried("feral dogs")]
        );
        assert_eq!(
            raw_sources_of(ItemKind::NuclearWaste),
            [RawSource::Carried("nuclear slimes")]
        );
        assert_eq!(
            made_by(recipes, ItemKind::Chest).first().map(|r| r.name),
            Some("Chest")
        );
    }

    use super::*;
    use crate::recipe_handler::RecipeHandler;

    #[test]
    fn raw_sources_are_what_the_world_drops() {
        let expected: BTreeSet<ItemKind> = [
            ItemKind::Log,
            ItemKind::Acorn,
            ItemKind::PlankBlock,
            // Generated stairs can be mined too
            ItemKind::Stairs,
            ItemKind::Stone,
            ItemKind::IronScrap,
            ItemKind::ScrapElectronics,
            ItemKind::PlasteelScrap,
            ItemKind::Leather,
            ItemKind::Meat,
            ItemKind::NuclearWaste,
            // Creatures drop embryos of their own kind
            ItemKind::FeralDogEmbryoPoor,
            ItemKind::FeralDogEmbryoFlawed,
            ItemKind::FeralDogEmbryoFlawless,
            ItemKind::NuclearSlimeEmbryoPoor,
            ItemKind::NuclearSlimeEmbryoFlawed,
            ItemKind::NuclearSlimeEmbryoFlawless,
        ]
        .into_iter()
        .collect();
        assert_eq!(raw_sources(), expected);
    }

    #[test]
    fn every_placeable_item_is_craftable_from_raw_sources() {
        let recipes = RecipeHandler::new();
        let problems = unobtainable(&placeable_items(), recipes.get_recipes(), &raw_sources());
        assert!(problems.is_empty(), "{}", problems.join("\n"));
        assert!(placeable_items().contains(&ItemKind::Chest));
    }

    #[test]
    fn every_body_part_is_craftable_from_raw_sources() {
        let recipes = RecipeHandler::new();
        let parts: Vec<ItemKind> = ItemKind::iter()
            .filter(|&item| crate::model::body::crafted_part(item).is_some())
            .collect();
        assert_eq!(parts.len(), 15);
        let problems = unobtainable(&parts, recipes.get_recipes(), &raw_sources());
        assert!(problems.is_empty(), "{}", problems.join("\n"));
    }

    #[test]
    fn an_uncraftable_placeable_item_is_named() {
        let recipes = RecipeHandler::new();
        // Pretend the diamond became placeable without a recipe, and the chest lost nails
        let mut items = placeable_items();
        items.push(ItemKind::Diamond);
        let problems = unobtainable(&items, recipes.get_recipes(), &raw_sources());
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].contains("Diamond has no recipe"),
            "{}",
            problems[0]
        );

        let mut raw = raw_sources();
        raw.remove(&ItemKind::IronScrap);
        let problems = unobtainable(&placeable_items(), recipes.get_recipes(), &raw);
        assert!(
            problems
                .iter()
                .any(|p| p.contains("Chest") && p.contains("Nail")),
            "{problems:?}"
        );
    }

    #[test]
    fn chest_tree_goes_down_to_logs_and_iron_scrap() {
        let handler = RecipeHandler::new();
        let chest = handler
            .get_recipes()
            .iter()
            .find(|r| r.result == ItemKind::Chest)
            .unwrap();
        let tree = ingredient_tree(handler.get_recipes(), chest, 1, &raw_sources());
        let line = |depth, item, needed, raw| TreeLine {
            depth,
            item,
            needed,
            raw,
        };
        assert_eq!(
            tree,
            [
                line(0, ItemKind::WoodenPlank, 8, false),
                line(1, ItemKind::Log, 3, true), // 3 planks per log, so 3 logs
                line(0, ItemKind::Nail, 4, false),
                line(1, ItemKind::IronScrap, 1, true), // 8 nails per iron scrap
            ]
        );
    }

    use crate::components::{DroppedItem, Position};
    use crate::intent::PlayerAction;
    use crate::test_support::{TestGame, TestWorld};
    use std::collections::HashMap;

    /// Mine the tile at (1, 0, 0) and return what's left there and what dropped.
    fn mine(seed: u64, tile: TileKind) -> (TileKind, Vec<(ItemKind, u32)>, TestGame) {
        let mut game = TestWorld::new(seed)
            .fill((-2, -2, 0), (2, 2, 0), TileKind::Grass)
            .tile(1, 0, 0, tile)
            .build();
        game.act(PlayerAction::MineAt { x: 1, y: 0, z: 0 });
        game.tick();
        let left = game.tile(1, 0, 0);
        let mut dropped: Vec<(ItemKind, u32)> = game
            .game
            .world
            .query::<(&Position, &DroppedItem)>()
            .iter()
            .filter(|(_, (p, _))| **p == Position { x: 1, y: 0, z: 0 })
            .map(|(_, (_, d))| (d.kind, d.qty))
            .collect();
        dropped.sort();
        (left, dropped, game)
    }

    #[test]
    fn mining_rock_iron_scrap_and_scrap_electronics() {
        assert_eq!(mine(1, TileKind::Rock).0, TileKind::Air);
        assert_eq!(mine(1, TileKind::Rock).1, [(ItemKind::Stone, 1)]);

        let (left, dropped, game) = mine(1, TileKind::IronScrap);
        assert_eq!(left, TileKind::Air);
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].0, ItemKind::IronScrap);
        assert!((1..=2).contains(&dropped[0].1));
        let log = &game.game.res.events.get_message_log().messages;
        assert!(log.last().unwrap().text.contains("You mine the iron scrap"));

        assert_eq!(
            mine(1, TileKind::ScrapElectronics).1,
            [(ItemKind::ScrapElectronics, 1)]
        );
        assert_eq!(
            mine(1, TileKind::Bedrock).0,
            TileKind::Bedrock,
            "not minable"
        );
    }

    #[test]
    fn iron_scrap_amount_is_seeded_by_world_and_position() {
        for seed in 0..20 {
            assert_eq!(
                mine(seed, TileKind::IronScrap).1,
                mine(seed, TileKind::IronScrap).1
            );
        }
        let amounts: BTreeSet<u32> = (0..20)
            .map(|s| mine(s, TileKind::IronScrap).1[0].1)
            .collect();
        assert_eq!(
            amounts,
            [1, 2].into_iter().collect(),
            "both amounts occur across seeds"
        );
    }

    fn craft(name: &str, have: &[(ItemKind, u32)]) -> HashMap<ItemKind, u32> {
        let handler = RecipeHandler::new();
        let mut inv: HashMap<ItemKind, u32> = have.iter().copied().collect();
        let index = handler.find_recipe_by_name(name).expect("recipe exists");
        assert!(handler.craft(index, &mut inv), "{name} craftable");
        inv.retain(|_, q| *q > 0);
        inv
    }

    #[test]
    fn nails_string_and_chest_recipes() {
        let nails = craft("Nails", &[(ItemKind::IronScrap, 1)]);
        assert_eq!(nails, [(ItemKind::Nail, 8)].into_iter().collect());
        let string = craft("String", &[(ItemKind::Leather, 1)]);
        assert_eq!(string, [(ItemKind::String, 4)].into_iter().collect());
        let chest = craft("Chest", &[(ItemKind::WoodenPlank, 9), (ItemKind::Nail, 4)]);
        assert_eq!(
            chest,
            [(ItemKind::Chest, 1), (ItemKind::WoodenPlank, 1)]
                .into_iter()
                .collect()
        );
    }

    #[test]
    fn a_chest_from_an_empty_inventory() {
        // 3 logs and 1 iron scrap, gathered; then planks, nails, chest
        let handler = RecipeHandler::new();
        let mut inv: HashMap<ItemKind, u32> = [(ItemKind::Log, 3), (ItemKind::IronScrap, 1)]
            .into_iter()
            .collect();
        for name in [
            "Wooden Planks",
            "Wooden Planks",
            "Wooden Planks",
            "Nails",
            "Chest",
        ] {
            let index = handler.find_recipe_by_name(name).unwrap();
            assert!(handler.craft(index, &mut inv), "{name}");
        }
        assert_eq!(inv[&ItemKind::Chest], 1);
    }

    #[test]
    fn components_for_direct_raw_and_rounding() {
        let handler = RecipeHandler::new();
        let r = handler.get_recipes();
        assert_eq!(
            components_for(r, ItemKind::Chest, 1, false),
            Ok(vec![(ItemKind::WoodenPlank, 8), (ItemKind::Nail, 4)])
        );
        assert_eq!(
            components_for(r, ItemKind::Chest, 1, true),
            Ok(vec![(ItemKind::Log, 3), (ItemKind::IronScrap, 1)])
        );
        // Planks come 3 per log: 4 planks need 2 crafts
        assert_eq!(
            components_for(r, ItemKind::WoodenPlank, 4, false),
            Ok(vec![(ItemKind::Log, 2)])
        );
        assert_eq!(
            components_for(r, ItemKind::Chest, 2, true),
            Ok(vec![(ItemKind::Log, 6), (ItemKind::IronScrap, 1)]),
            "16 planks = 6 logs; 8 nails = 1 iron scrap"
        );
        let err = components_for(r, ItemKind::Log, 1, false).unwrap_err();
        assert_eq!(err, NoRecipe(ItemKind::Log));
        assert_eq!(err.to_string(), "Log has no recipe");
    }
}
