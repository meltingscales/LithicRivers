# Tasks

## 0. Decisions

- [ ] 0.1 User decisions: whether the player passes through plain fences, and whether spawners are craftable; record them in design.md

## 1. Core

- [ ] 1.1 The spawn rule table, the 10-second check, per-chunk cooldowns and caps (saved); verify `cargo test` for refilling, caps and determinism
- [ ] 1.2 The no-spawn rules (in view, near the player, near lights, inside fenced areas); verify tests for each
- [ ] 1.3 Fences and fence gates, blocking creature movement but not sight or light; verify tests with a dog behind a fence
- [ ] 1.4 Spawner blocks with their own cap and the spawner core item; verify tests, the reachability rule if craftable, and a showcase farm with the coverage test

## 2. Client

- [ ] 2.1 Sprites for spawners, fences and gates, and the look panel naming a spawner's creature and timer; verify the sprite gallery and a snapshot

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that areas feel alive without floods, and that a farm is fun to build
