# Tasks

## 1. Harness

- [x] 1.1 Add `insta` as a client dev-dependency, add `cargo-insta` to `just install`, and mark `*.snap` as `text eol=lf` in `.gitattributes`; verify `cargo test --workspace` still passes
- [x] 1.2 Add the snapshot test that runs each `scripts/snapshots/*.txt` in sorted order through the in-process runner and asserts a named snapshot; verify a temporary script produces a pending snapshot and the test fails until accepted

## 2. Baseline snapshots

- [x] 2.1 Add scripts for the world view after start and each top-level menu tab; record and review snapshots; verify `cargo test --workspace` passes twice in a row with no changes
- [x] 2.2 Add scripts for the combat panel (using `tp`/existing spawns to reach a feral dog) and the cheat console; record and review; verify they pass
- [x] 2.3 Add scripts for the corpse-loot and NPC dialogue modals if reachable with existing cheats and starting spawns; otherwise note them as follow-ups in this file; verify recorded snapshots pass

## 3. Docs and CI

- [x] 3.1 Document in AGENTS.md how to add a snapshot script, review diffs (`cargo insta review` or `INSTA_UPDATE=always cargo test`), and that snapshot changes need a reason in the PR; verify the documented commands work
- [x] 3.2 Add a PR check workflow (`.github/workflows/pr-checks.yml`: clippy with `-D warnings` and `cargo test --workspace` on Linux, Windows, macOS), since `ci.yml` only runs on version tags; verify the snapshot tests pass on all three in the PR run
