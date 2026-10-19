# Design

## Context

- `add-headless-client-driver` provides deterministic headless runs with framed `screen` and `state` dumps on stdout.
- The client is a binary crate; its `#[cfg(test)]` modules run under `cargo test`.
- The release workflow (`.github/workflows/ci.yml`) runs `just test-release-*` on Windows, macOS and Linux, but only on `v*` tags. A new `pr-checks.yml` runs clippy and `cargo test --workspace` on all three for every PR into `release`, so snapshot drift is caught before merge.

## Goals / Non-Goals

**Goals:**
- One snapshot mechanism, reusing scripts, so agents write a script, not Rust, to add coverage.

**Non-Goals:**
- Per-widget unit snapshots of individual render functions (scripts cover the real composition; add widget-level tests later only if a panel needs isolation).
- Color/style snapshots.

## Decisions

### Snapshot the script runner's output, not render functions directly
Each `scripts/snapshots/<name>.txt` produces one snapshot of its stdout. This tests the composed screen the player sees, needs no per-panel harness, and makes adding coverage a text-file change.
Alternative: call `render_*_panel` into a `TestBackend` per panel. More granular but needs fabricated `App` state per panel and Rust code per test.

### `insta` for storage and review
`insta` (dev-dependency only) gives stored `.snap` files, readable diffs, pending `.snap.new` files for new/changed output, and `cargo insta review`/`accept`. Hand-rolling this is ~150 lines of diff/IO code we'd then maintain. Without `cargo-insta` installed, `INSTA_UPDATE=always cargo test` also re-records; document both, and add `cargo-insta` to `just install`.

### Test discovery
One test function iterates `scripts/snapshots/*.txt` in sorted order and calls `insta::assert_snapshot!(name, output)` for each, using `insta`'s glob support or a manual loop with named snapshots. Runs in-process via the runner API, not by spawning the binary.

### Small, focused screens
Snapshot scripts use `size 100x30` unless a panel needs more, and start with `skip-splash`. Smaller screens mean smaller diffs and less churn from unrelated map changes.

### Line endings
Output always uses `\n`; `.gitattributes` marks `*.snap` as `text eol=lf`.

## Risks / Trade-offs

- [Worldgen or spawn changes churn many snapshots] → Prefer `state` dumps and panels over the map view where the map isn't the subject; use `tp` to a fixed location; review churn in the same PR as the cause.
- [Snapshot fatigue: people accept without reading] → Keep snapshots small; AGENTS.md asks agents to state in the PR why each snapshot changed.
- [Hidden nondeterminism] → The driver's repeat-run test runs first; a flaky snapshot points to a determinism bug to fix, not to re-record.
