# Design

## Context

Current `main.rs` contents (line numbers as of `release` at proposal time):

| Lines | Item | Destination |
|---|---|---|
| 59–209 | `SplashState`, `MenuTab` + impl (tab order, indices) | `app/app.rs` |
| 210–386 | `App` struct + `on_tick`, `apply_tick_result`, `snap_view_to_player_z`, `activate_menu` | `app/app.rs` |
| 604–726 | splash-screen branch of `ui()` | `ui/splash.rs` |
| 727–901 | rest of `ui()`: hotbar check, root layout, tab dispatch, modal overlay order | `ui/layout.rs` |
| 902–974 | `render_message_log`, `render_bottom_menu` | `ui/layout.rs` |
| 1228–1309 | `calculate_game_viewport_area` | `ui/layout.rs` |
| 975–1227 | corpse selection / loot modals | `ui/modals/corpse_looting.rs` |
| 1310–1610 | NPC selection / dialogue modals | `ui/modals/npc_interaction.rs` |
| 1611–1737 | hotbar assignment + block picker | `ui/modals/hotbar_assignment.rs` |
| 1738–1856 | multi-action + action selection | `ui/modals/action_selection.rs` |
| 1857–end | cheat console modal | `ui/modals/cheat_console.rs` |

Stays in `main.rs`: module declarations, `EmbeddedAssets`, `main()`, `run_headless_from_args`, `step`, `run_app`. `BOOT_MESSAGE_TYPEWRITER_MS` is used by `App::on_tick`, so it moves with `App`.

Existing convention: `ui/panels/<name>.rs` exposes `pub fn render_*`, re-exported from `ui/panels/mod.rs`, and takes `&mut crate::App`.

## Goals / Non-Goals

**Goals:**
- `main.rs` holds only startup and the loop.
- Each modal is findable by name under `ui/modals/`.
- Byte-identical rendering.

**Non-Goals:**
- Changing any rendering, layout or behavior, or renaming functions.
- Moving `render_repair_modal` / `render_tutorial_selection_modal` out of `ui/panels/` (they already live in `ui/`; a later tidy-up can relocate them).
- Splitting `app_state.rs` or the input handlers.

## Decisions

### Move code verbatim, one area per commit
Each move is cut-and-paste plus visibility and imports. No edits inside function bodies, so review is "same code, new file" and any snapshot diff points to an accidental change. Commit per area (app, splash, layout, each modal file) so a regression bisects to one move.

### Re-export instead of rewriting paths
`pub use app::app::{App, MenuTab, SplashState};` at the crate root keeps the 39 files that use `crate::App` / `crate::MenuTab` unchanged, and `ui::layout::ui` is re-exported where `headless.rs` and `run_app` call it. Rewriting every import would bloat the diff without improving clarity.

### Visibility
Moved items become `pub(crate)` only as their callers need. Helpers used within one new file (e.g. `render_corpse_selection_modal`, called only by `render_corpse_looting_modals`) stay private.

## Risks / Trade-offs

- [An accidental edit while moving changes a screen] → The 15 snapshots must be byte-identical (no `.snap` changes in the PR); bug scripts and tests must pass.
- [Modal draw order changes] → Order lives in `ui/layout.rs` exactly as in today's `ui()`; covered by the corpse loot, NPC dialogue and cheat console snapshots.
- [Merge conflicts with in-flight UI work] → None open now; land quickly.
