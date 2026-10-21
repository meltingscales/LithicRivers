# Tasks

## 1. Fluids and spouts (core)

- [ ] 1.1 Add `FluidKind`, `FluidTank` and `transfer`; register them for saving; verify unit tests for same-kind, capacity and empty cases, and a save/reload test
- [ ] 1.2 Add spout worldgen (seeded `SpoutSite` markers turned into entities when the chunk is ensured; surface chunks at half the underground chance) and the spout fill system; verify a same-seed placement test, a count test showing about half as many surface spouts per chunk as underground, a machine-generated vs walked-to chunk equality test with a spout, and fill/cap/refill tests using elapsed time
- [ ] 1.3 Make spouts unbreakable; verify a mining test

## 2. Pump, basin, crucible

- [ ] 2.1 Add the pump (powered, faces away from source) and its factory system; verify tests for pumping from a spout, no power, partial power, and across a chunk edge
- [ ] 2.2 Add pipes and pipe networks (rebuilt on place and break, cached and rebuilt on load) and `FluidInput`; verify tests for a pump through a pipe line into a basin, a network across a chunk edge, fixed delivery order to two machines, throughput limit, wrong fluid refused, breaking a pipe splitting a network, and save/reload of network contents
- [ ] 2.3 Add the casting basin and iron ingots; verify tests for casting into a chest, keeping ingots with nothing in front, and refusing a different fluid
- [ ] 2.4 Add the crucible with scoop and pour interactions; verify headless tests for scooping and pouring and that both appear in Choose Interaction and context help
- [ ] 2.5 Add recipes for the crucible, pump, pipe and basin; verify the crafting reachability test passes

## 3. Client and docs

- [ ] 3.1 Sprites for the spout, pump, pipe (joined to neighbors), basin and items; fill levels in Look mode; verify snapshot scripts for a spout with a pump, a pipe line and a basin, and Look mode on a spout
- [ ] 3.2 Update Help, AGENTS.md (scripting spouts with cheats) and `docs/TECH-TREE.md`; verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check: finding a spout, scooping, pumping through pipes and casting feel right
