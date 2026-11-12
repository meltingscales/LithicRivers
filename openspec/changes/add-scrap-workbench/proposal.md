# Proposal

## Why

Crafted items that are no longer needed (an old pipe gun, spare conveyors, a crude part replaced by a standard one) are dead weight, and mistakes in crafting cost the whole recipe. A way to break items back down recovers some of that value without making crafting free. It also suits LithicRivers, a game about salvaging the remains of an older world, and it gives the early factory another powered machine to wire up.

## What Changes

- **Scrap workbench:** a new powered factory block in the early tier.
  - It runs on grid power, so a burner generator and a pole are enough.
  - It has an input slot and an output buffer, opened with Interact like the furnaces, and it fits into belt lines the same way: fed from the back and sides, with recovered items leaving through the front.
- **Uncrafting:** the workbench breaks a full batch of an item (as many as its recipe makes) back into that recipe's ingredients.
  - Each ingredient unit is recovered with a 50% chance, rolled from the world seed, the workbench's position and how many jobs it has done, so it never gives back more than crafting cost and the result is reproducible.
  - It takes 4 factory seconds per batch while powered, and pauses without power, keeping its progress.
  - Items it can't scrap: raw materials (no recipe), and items made by more than one recipe (no single ingredient list to return). They stay out of its input, from the window and from belts.
- **Recipe:** 4 iron scrap, 4 wooden planks, 4 nails and 2 scrap electronics.
- **Stats:** what the workbench uses and recovers shows on the Production tab.

## Capabilities

### New Capabilities
- `scrap-workbench`: what can be scrapped, the recovery chance, batches, power and timing, and how the workbench fits into belt lines.

### Modified Capabilities
None.

## Impact

- **Core:** a new `FactoryKind::ScrapWorkbench` with its own state (input, output buffer, progress, jobs done) on the machine framework the furnaces and spawners use; uncrafting looks up recipes in `RecipeHandler`; new item and recipe; `factory_stats::record` for what is used and recovered; save registration.
- **Client:** the machine window (input, output, progress, and a line naming what the selected item scraps into), the item and block sprites, and Look text.
- **Depends on:** the power grid and machine windows (merged).
