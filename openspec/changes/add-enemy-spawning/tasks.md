# Tasks

## 0. Decisions

- [x] 0.1 User decision, recorded in design.md: the player walks through gates only, with nothing to open, and not through fences
- [x] 0.2 User decision, recorded in design.md: spawners are craftable, and a placed spawner needs power, meat slurry and an embryo (poor, flawed or flawless)

## 1. Core

- [x] 1.1 The spawn rule table, the 10-second check, per-chunk cooldowns and caps (saved); verify `cargo test` for refilling, caps and determinism
- [x] 1.2 The no-spawn rules (in view, near the player, near lights, inside fenced areas); verify tests for each
- [x] 1.3 Fences (blocking creatures and the player) and gates (blocking creatures only), neither blocking sight or light; verify tests with a dog behind a fence and the player walking through a gate
- [ ] 1.4 Wild spawner blocks with their own cap, breaking into the spawner item; verify tests for the timer, the cap and the drop
- [ ] 1.5 Meat slurry (recipe), embryo items per creature kind and grade, and embryo drops in creature loot; verify the reachability test and seeded drop tests
- [ ] 1.6 Placed spawners: the spawner recipe, power use, slurry and embryo slots fed by extractors and belts, embryo charges by grade; verify tests for running fed, pausing without power, stopping without slurry or embryo, the creature following the embryo, and a self-feeding showcase farm with the coverage test

## 2. Client

- [ ] 2.1 Sprites for spawners, fences, gates, meat slurry and embryos, and the look panel naming a spawner's creature and timer; verify the sprite gallery and a snapshot
- [ ] 2.2 The placed spawner's window (slurry, embryo, charges left, power); verify a snapshot script in `scripts/snapshots/`

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that areas feel alive without floods, and that a farm is fun to build
