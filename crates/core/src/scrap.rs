//! The scrap workbench: a powered early-tier machine that uncrafts items. It takes a full
//! batch of an item (as many as its one recipe makes) and returns that recipe's
//! ingredients, each unit with a 50% chance, rolled from the world seed, the workbench's
//! position and how many jobs it has done. Items to scrap wait in its own input; what it
//! recovers goes into its inventory (where the window, extractors and the player take
//! from) and is pushed out of its front, one item a step.

use crate::components::{Inventory, ItemKind, ItemStack, Position};
use crate::factory::{FactoryBlock, FactoryKind, FACTORY_SECOND_TICKS, FACTORY_STEP_TICKS};
use crate::power::PowerConsumer;
use crate::recipe_handler::{Recipe, RecipeHandler};
use crate::resources::Resources;
use hecs::{Entity, World};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// What a scrap workbench draws while it works, in watts.
pub const SCRAP_WORKBENCH_W: u64 = 30_000;
/// Factory seconds per batch at full power.
pub const SCRAP_SECONDS: u64 = 4;
/// Percent chance to recover each ingredient unit.
pub const RECOVERY_PERCENT: u32 = 50;
/// Items its input holds.
pub const INPUT_CAP: u32 = 32;
/// Stacks of recovered items it holds.
pub const OUTPUT_SLOTS: usize = 6;

/// A scrap workbench's input and progress. Recovered items live in its `Inventory`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrapWorkbench {
    pub input: Option<ItemStack>,
    /// Progress on the current batch, in tick-permille of full power
    pub progress: u64,
    /// Batches finished, for the seeded rolls
    pub jobs_done: u64,
}

fn recipes() -> &'static [Recipe] {
    static RECIPES: OnceLock<Vec<Recipe>> = OnceLock::new();
    RECIPES.get_or_init(|| RecipeHandler::new().get_recipes().to_vec())
}

/// The one recipe that makes `item`, if exactly one does: raw materials and items with
/// several recipes can't be scrapped.
pub fn scrap_recipe(item: ItemKind) -> Option<&'static Recipe> {
    let mut makes = recipes().iter().filter(|r| r.result == item);
    let recipe = makes.next()?;
    makes.next().is_none().then_some(recipe)
}

/// Whether a scrap workbench takes `item` at all.
pub fn takes(item: ItemKind) -> bool {
    scrap_recipe(item).is_some()
}

/// Whether `bench` takes `item` now: scrappable, and its input is empty or holds the same
/// item with room.
pub fn accepts(world: &World, bench: Entity, item: ItemKind) -> bool {
    takes(item)
        && world
            .get::<&ScrapWorkbench>(bench)
            .is_ok_and(|b| match b.input {
                None => true,
                Some(s) => s.kind == item && s.qty < INPUT_CAP,
            })
}

/// Put one `item` into the workbench's input (the caller checked `accepts`).
pub fn insert(bench: &mut ScrapWorkbench, item: ItemKind) {
    match &mut bench.input {
        Some(s) => s.qty += 1,
        None => bench.input = Some(ItemStack::new(item, 1)),
    }
}

/// Whether the workbench holds a full batch and room for everything it could recover.
fn ready(bench: &ScrapWorkbench, inv: &Inventory) -> bool {
    let Some(input) = bench.input else {
        return false;
    };
    let Some(recipe) = scrap_recipe(input.kind) else {
        return false;
    };
    let new_kinds = recipe
        .ingredients
        .iter()
        .filter(|(k, _)| !inv.slots.iter().any(|s| s.kind == *k))
        .count();
    input.qty >= recipe.quantity && inv.slots.len() + new_kinds <= inv.max_slots
}

fn benches(world: &World) -> Vec<(Entity, Position, FactoryBlock)> {
    let mut found: Vec<_> = world
        .query::<(&FactoryBlock, &Position)>()
        .with::<&ScrapWorkbench>()
        .iter()
        .map(|(e, (b, p))| (e, *p, *b))
        .collect();
    found.sort_by_key(|(e, p, _)| (p.z, p.y, p.x, e.to_bits()));
    found
}

/// Before power is shared: a workbench wants power only with a batch it can finish.
pub fn update_demand(world: &mut World) {
    for (_, (bench, inv, consumer)) in
        world.query_mut::<(&ScrapWorkbench, &Inventory, &mut PowerConsumer)>()
    {
        consumer.active = ready(bench, inv);
    }
}

/// How far the current batch is, in percent.
pub fn progress_percent(world: &World, bench: Entity) -> u64 {
    world.get::<&ScrapWorkbench>(bench).map_or(0, |b| {
        b.progress * 100 / (SCRAP_SECONDS * FACTORY_SECOND_TICKS * 1000)
    })
}

/// What a batch of `item` scraps into, for the window: "up to 3 Iron Scrap, 4 Nail", with
/// the batch size first when it is more than one ("4 give up to 1 Stick, 1 String").
pub fn describe(item: ItemKind) -> Option<String> {
    let recipe = scrap_recipe(item)?;
    let parts: Vec<String> = recipe
        .ingredients
        .iter()
        .map(|(k, n)| format!("{n} {}", crate::components::itemkind_name(*k)))
        .collect();
    let parts = parts.join(", ");
    Some(if recipe.quantity > 1 {
        format!("{} give up to {parts}", recipe.quantity)
    } else {
        format!("up to {parts}")
    })
}

/// One factory step, after power is shared: work the batch at the network's power, scrap
/// it when done, and push one recovered item out of the front.
pub fn step(world: &mut World, res: &mut Resources) {
    for (bench, pos, block) in benches(world) {
        let satisfaction = world.get::<&PowerConsumer>(bench).map_or(0, |c| {
            if c.active {
                c.satisfaction_permille as u64
            } else {
                0
            }
        });
        let done = {
            let (Ok(mut b), Ok(inv)) = (
                world.get::<&mut ScrapWorkbench>(bench),
                world.get::<&Inventory>(bench),
            ) else {
                continue;
            };
            if ready(&b, &inv) {
                b.progress += satisfaction * FACTORY_STEP_TICKS;
            }
            b.progress >= SCRAP_SECONDS * FACTORY_SECOND_TICKS * 1000
        };
        if done {
            scrap_batch(world, res, bench, pos);
        }
        crate::machines::push_output(world, res, bench, pos, block.facing);
    }
}

/// Use up a batch from the input and roll each ingredient unit.
fn scrap_batch(world: &mut World, res: &mut Resources, bench: Entity, pos: Position) {
    let (item, recipe, job) = {
        let Ok(mut b) = world.get::<&mut ScrapWorkbench>(bench) else {
            return;
        };
        let Some(input) = b.input else {
            return;
        };
        let Some(recipe) = scrap_recipe(input.kind) else {
            return;
        };
        b.progress = 0;
        b.jobs_done += 1;
        let left = input.qty - recipe.quantity;
        b.input = (left > 0).then(|| ItemStack::new(input.kind, left));
        (input.kind, recipe, b.jobs_done)
    };
    let source = crate::factory_stats::Source::Machine(FactoryKind::ScrapWorkbench);
    crate::factory_stats::record(res, item, recipe.quantity, source, false);
    let seed =
        res.world_state.world.mix_coords(pos.x, pos.y, pos.z) ^ job.wrapping_mul(0x5C2A_9B11);
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
    for &(kind, n) in recipe.ingredients {
        let got = (0..n)
            .filter(|_| rng.gen_range(0..100) < RECOVERY_PERCENT)
            .count() as u32;
        if got == 0 {
            continue;
        }
        crate::factory_stats::record(res, kind, got, source, true);
        if let Ok(mut inv) = world.get::<&mut Inventory>(bench) {
            inv.add(kind, got);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::Facing;
    use crate::game::Game;
    use crate::tiles::TileKind;

    fn at(x: i64, y: i64) -> Position {
        Position { x, y, z: 0 }
    }

    fn game() -> Game {
        crate::test_support::TestWorld::new(9)
            .fill((-10, -10, 0), (12, 10, 0), TileKind::Grass)
            .player_at(0, 8, 0)
            .build()
            .game
    }

    fn put(g: &mut Game, kind: FactoryKind, pos: Position, facing: Facing) -> Entity {
        crate::factory::place(&mut g.world, &mut g.res, kind, pos, facing).unwrap()
    }

    /// A workbench at the origin facing east, with a fueled burner and a pole when
    /// `powered`.
    fn bench(g: &mut Game, powered: bool) -> Entity {
        let b = put(g, FactoryKind::ScrapWorkbench, at(0, 0), Facing::East);
        let generator = put(g, FactoryKind::BurnerGenerator, at(-3, 2), Facing::North);
        put(g, FactoryKind::PowerPole, at(-1, 2), Facing::North);
        if powered {
            g.world
                .get::<&mut Inventory>(generator)
                .unwrap()
                .add(ItemKind::Biochar, 50);
        }
        b
    }

    fn feed(g: &mut Game, b: Entity, item: ItemKind, n: u32) {
        for _ in 0..n {
            assert!(
                crate::factory::insert(&mut g.world, b, item),
                "{item:?} refused"
            );
        }
    }

    fn steps(g: &mut Game, n: u64) {
        for _ in 0..n {
            g.res.time.tick += FACTORY_STEP_TICKS;
            crate::factory::run_due_steps(&mut g.world, &mut g.res);
        }
    }

    fn held(g: &Game, b: Entity, item: ItemKind) -> u32 {
        g.world.get::<&Inventory>(b).unwrap().count(item)
    }

    const BATCH_STEPS: u64 = SCRAP_SECONDS * crate::factory::STEPS_PER_FACTORY_SECOND;

    #[test]
    fn only_items_with_one_recipe_can_be_scrapped() {
        assert!(scrap_recipe(ItemKind::Log).is_none(), "raw");
        assert!(
            scrap_recipe(ItemKind::WoodenShavings).is_none(),
            "two recipes"
        );
        let pipe = scrap_recipe(ItemKind::PipeGun).expect("one recipe");
        assert!(pipe.ingredients.contains(&(ItemKind::IronScrap, 3)));
    }

    #[test]
    fn refused_items_stay_out_from_the_window_and_belts() {
        let mut g = game();
        let b = bench(&mut g, true);
        assert!(!crate::factory::insert(&mut g.world, b, ItemKind::Log));
        assert!(!crate::factory::accepts(
            &g.world,
            b,
            ItemKind::WoodenShavings
        ));
        assert!(crate::factory::takes(
            FactoryKind::ScrapWorkbench,
            ItemKind::PipeGun
        ));
    }

    #[test]
    fn it_recovers_about_half_and_never_more_than_the_cost() {
        let mut g = game();
        let b = bench(&mut g, true);
        let mut iron = 0;
        for _ in 0..60 {
            feed(&mut g, b, ItemKind::PipeGun, 1);
            let before = held(&g, b, ItemKind::IronScrap);
            steps(&mut g, BATCH_STEPS + 1);
            let got = held(&g, b, ItemKind::IronScrap) - before;
            assert!(got <= 3, "at most what one pipe gun cost");
            iron += got;
            // Empty the output so it never fills
            g.world.get::<&mut Inventory>(b).unwrap().slots.clear();
        }
        let cost = 60 * 3;
        assert!(
            (cost * 35 / 100..=cost * 65 / 100).contains(&iron),
            "{iron} of {cost} iron scrap"
        );
        assert_eq!(g.world.get::<&ScrapWorkbench>(b).unwrap().jobs_done, 60);
    }

    #[test]
    fn it_waits_for_a_full_batch() {
        let mut g = game();
        let b = bench(&mut g, true);
        // Torches come four to a recipe
        assert_eq!(scrap_recipe(ItemKind::Torch).unwrap().quantity, 4);
        feed(&mut g, b, ItemKind::Torch, 3);
        steps(&mut g, 3 * BATCH_STEPS);
        assert_eq!(g.world.get::<&ScrapWorkbench>(b).unwrap().jobs_done, 0);
        feed(&mut g, b, ItemKind::Torch, 1);
        steps(&mut g, BATCH_STEPS + 1);
        let state = *g.world.get::<&ScrapWorkbench>(b).unwrap();
        assert_eq!(state.jobs_done, 1);
        assert_eq!(state.input, None, "all four went as one batch");
    }

    #[test]
    fn without_power_it_keeps_its_batch_and_finishes_later() {
        let mut g = game();
        let b = bench(&mut g, false);
        feed(&mut g, b, ItemKind::PipeGun, 1);
        steps(&mut g, 3 * BATCH_STEPS);
        assert_eq!(g.world.get::<&ScrapWorkbench>(b).unwrap().jobs_done, 0);
        let generator = g.res.factory_index[&at(-3, 2)];
        g.world
            .get::<&mut Inventory>(generator)
            .unwrap()
            .add(ItemKind::Biochar, 50);
        steps(&mut g, BATCH_STEPS + 2);
        assert_eq!(g.world.get::<&ScrapWorkbench>(b).unwrap().jobs_done, 1);
    }

    #[test]
    fn a_belt_line_fills_the_chest_in_front() {
        let mut g = game();
        let b = bench(&mut g, true);
        let chest = put(&mut g, FactoryKind::Chest, at(1, 0), Facing::North);
        feed(&mut g, b, ItemKind::PipeGun, 4);
        steps(&mut g, 4 * (BATCH_STEPS + 1) + 30);
        let inv = g.world.get::<&Inventory>(chest).unwrap();
        let total: u32 = inv.slots.iter().map(|s| s.qty).sum();
        assert!(total > 0, "recovered items reached the chest");
        assert!(inv.slots.iter().all(|s| [
            ItemKind::IronScrap,
            ItemKind::WoodenPlank,
            ItemKind::Nail
        ]
        .contains(&s.kind)));
    }

    #[test]
    fn the_same_world_scraps_the_same() {
        let run = || {
            let mut g = game();
            let b = bench(&mut g, true);
            feed(&mut g, b, ItemKind::PipeGun, 5);
            steps(&mut g, 5 * (BATCH_STEPS + 1));
            let slots = g.world.get::<&Inventory>(b).unwrap().slots.clone();
            slots
        };
        assert_eq!(run(), run());
    }
}
