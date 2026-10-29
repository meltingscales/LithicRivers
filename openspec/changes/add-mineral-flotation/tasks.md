# Tasks

## 0. Before starting

- [ ] 0.1 Check `add-ore-spouts` is archived and its machine fluid input supports a block with two fluid inputs; if not, update this design first; verify by reading the `fluids` spec

## 1. Materials and simple machines

- [ ] 1.1 Items and fluids (gravel, sand, mineral dust, copper and iron concentrate, gem grit, sapphire, alexandrite, clay, fired brick, copper ingot, copper wire, tanned leather; water, slurry, tannin, tailings), the copper wire recipe, and the smelting table rows; verify `cargo test` for each smelt and the reachability test
- [ ] 1.2 The crusher (or its stone and gravel rows, if `add-advanced-processing` landed first) and the ball mill; verify tests for stone to dust, unpowered machines and belt input
- [ ] 1.3 The water well, the slurry mixer and the tannery (tannin and tanned leather); verify tests for each requirement, including the mixer with no water

## 2. Flotation

- [ ] 2.1 The flotation tank: two fluid inputs, the seeded roll table, the saved batch count, and tailings out the front; verify tests for a batch, no frother, identical replays and the 1000-batch shares
- [ ] 2.2 The sieve and the settling pond; verify tests for sieving shares and replays, and for 1000 mB of tailings returning 1 clay and 800 mB of water
- [ ] 2.3 Recipes for all seven blocks (the tank needs tanned leather), and registration of every new component in `saved_components!`; verify the reachability test, `every_component_is_registered` and `worlds_from_every_spawn_path`

## 3. Client and showcase

- [ ] 3.1 Machine windows for each block (inputs, fluids, progress, and what's missing), and sprites for every new item and block following `docs/ART-DIRECTION.md`; verify snapshots of the mixer, tank and pond windows, and the sprite gallery
- [ ] 3.2 The flotation line in the Factory showcase, from a stone chest to ingots; verify `every_factory_block_kind_has_an_example` and a test that the line delivers ingots within 10 game minutes
- [ ] 3.3 Update `docs/TECH-TREE.md` (a Mineral processing tier, no copper spouts) and AGENTS.md's factory notes; verify `just code-clippy`, `cargo test --workspace` and PR checks pass

## 4. Check

- [ ] 4.1 User check: building a flotation line is fun, and copper and gems feel worth the effort
