# Tasks

## 1. App types

- [x] 1.1 Move `SplashState`, `MenuTab`, `BOOT_MESSAGE_TYPEWRITER_MS`, the `App` struct and its `impl` from `main.rs` to `app/client_app.rs` (not `app/app.rs`: clippy `module_inception`), re-exported at the crate root; verify `cargo test --workspace` passes with no `.snap` changes

## 2. Frame rendering

- [x] 2.1 Move the splash-screen branch of `ui()` into `ui/splash.rs` (`render_splash`, returning whether it drew), called from `ui()`; verify `cargo test --workspace` passes with no `.snap` changes and `just client-blind scripts/agent/smoke.txt` exits 0
- [x] 2.2 Move `ui()`, `render_message_log`, `render_bottom_menu` and `calculate_game_viewport_area` into `ui/layout.rs`, keeping modal draw order unchanged; verify `cargo test --workspace` passes with no `.snap` changes

## 3. Modals

- [x] 3.1 Create `ui/modals/` and move the corpse looting modals into `corpse_looting.rs`; verify the `modal-corpse-loot` snapshot is unchanged
- [x] 3.2 Move the NPC selection and dialogue modals into `npc_interaction.rs`; verify the `modal-npc-dialogue` snapshot is unchanged
- [x] 3.3 Move the hotbar assignment and block picker modals into `hotbar_assignment.rs`, and the multi-action / action selection modals into `action_selection.rs`; verify `cargo test --workspace` passes with no `.snap` changes
- [x] 3.4 Move the cheat console modal into `cheat_console.rs`; verify the `cheat-console` snapshot is unchanged

## 4. Wrap-up

- [x] 4.1 Confirm `main.rs` contains only module declarations, `EmbeddedAssets`, `main`, `run_headless_from_args`, `step` and `run_app` (about 300 lines); verify `just code-clippy` passes, `git diff --stat crates/client/src/snapshots` is empty, and all bug scripts pass via `just client-blind`
- [ ] 4.2 User check: the interactive client looks and behaves the same (splash, tabs, a modal or two)
