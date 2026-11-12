# Design

## Context

- **Machines** (`machines.rs`, `furnace.rs`, `spawner.rs`): factory blocks with an inventory or slots, a `PowerConsumer` drawing from the pole network, and per-step logic in `factory::run_due_steps` (one factory step per 20 ticks). The electric furnace already shows the shape this needs: input from the back and sides, output from the front, progress kept while unpowered, speed scaled by the network's satisfaction.
- **Recipes** live in `RecipeHandler` as `Recipe { ingredients, result, quantity }`. Some items have more than one recipe (wooden shavings from acorns or sticks).
- **Item flow stats:** code that makes or uses items calls `factory_stats::record`.

## Goals / Non-Goals

**Goals:**
- One early machine that turns unwanted crafted items into some of their materials.
- Never a net gain: recovery can't beat crafting cost.
- Reproducible results.

**Non-Goals:**
- Hand uncrafting without a machine.
- Upgraded workbenches with better odds (a later tier can add one).
- Scrapping world tiles or creatures.
- Keeping a power cell's charge or a part's integrity: scrapped items lose that.

## Decisions

### A batch is the recipe's quantity
Scrapping one plank of a 3-plank recipe would have to return a third of a log. Requiring the full output quantity keeps every roll whole. Alternative: round down per item. Rejected: it quietly makes single items worthless to scrap.

### Only items with exactly one recipe
An item with two recipes has no single ingredient list to return, and picking one could turn a cheap input into a dearer one (shavings back into acorns, when they were made from sticks). Refusing them keeps the rule simple and exploit-free. The input filter is a pure function of the recipe list (`scrap_recipe(item) -> Option<&Recipe>`), shared by the window, belts and tests.

### Rolls
Each ingredient unit rolls 50% with a generator seeded from the world seed, the workbench's position and its `jobs_done` counter, saved in its state. Alternative: the world RNG. Rejected: machine rolls shouldn't shift with unrelated combat rolls.

### Machine shape
`FactoryKind::ScrapWorkbench` with a `ScrapWorkbench { input: Option<ItemStack>, output: Vec<ItemStack>, progress_steps, jobs_done }` component, 30 kW (`SCRAP_WORKBENCH_W`), 4 factory seconds per batch scaled by satisfaction. The output buffer holds up to 6 stacks; when it can't fit a batch's results, the workbench waits rather than losing items. It reuses the furnace's input-from-back-and-sides and output-to-front helpers. The machine window lists the input, the output stacks and progress, and names what the input scraps into.

### Numbers

| What | Value |
|---|---|
| Power | 30 kW |
| Time per batch | 4 factory seconds |
| Recovery | 50% per ingredient unit |
| Output buffer | 6 stacks |
| Recipe | 4 iron scrap, 4 wooden planks, 4 nails, 2 scrap electronics |

## Risks / Trade-offs

- [Scrapping becomes a sorting chore] → It fits belt lines, so a chest of junk can feed it unattended.
- [A recipe change makes an item scrappable into something odd] → The input filter reads the live recipe list, and a test asserts no scrappable item can return an ingredient that crafting it didn't use.
- [Saves] → The new component is registered in `saved_components!`, and `worlds_from_every_spawn_path` gains a workbench.
