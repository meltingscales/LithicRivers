# AI Instructions

You are allowed to run `just build` and `just install`, and also `just run-*`. This game uses ratatui for the UI.

If you're running NixOS, make sure to run `nix-shell` to get the right environment.

`./docs/*.md` contains lots of useful documentation.
`.ai/*.md` contains some prompts I reuse from time to time.

You should use `rustup` to run `cargo` and other tools.

Please avoid running `cargo clean`. It takes a long time to rebuild rust modules.

Also never run `just client` or any other TUI because you might be in a terminal that cannot display properly as you're an AI agent. Instead, ask me to run `just client` or use unit testing to test code.

## Demos

All code prefixed with `demo_*.rs` is not meant to be part of the main codebase. These are self-contained demos that should not be impacted by main codebase changes or refactors.

## Testing

Run tests with `make test` to validate game systems. Write simple unit tests for:
- Combat mechanics (damage, death, action queues)
- Game logic (movement, energy, cooldowns)
- Edge cases and bug fixes

Tests help ensure systems work correctly and prevent regressions during development.

## Keybind handling

Avoid hardcoding keys (i.e. `KeyCode::Char('v')`). Make sure to populate `default_config.rs` as well as use the `keybinds.matches` method i.e. `app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key)` if you want to check for a keypress. This makes our game actually key-mappable.

Also, if you need to convert a key into a printable version, please use code like `app.core.config_manager.get_printable_key_for_keybind("movement", "MOVE_NORTHWEST"),`

## Guiding principles

- All randomly-generated choices, actions, damage, loot, worldgen, etc - should be fully deterministic and based on world seed and world tick. This means that this game should be fully TAS-able for any specific initial seed (and version of game code). Always use the seed and any permutation (when appropriate) of XYZ coordinate, biome, or other seeded randomness to generate anything "random".

- See `docs/INSPIRATION.md` for inspiration.

- This game renders using ASCII text.

- docs/TODO.md contains a big, unedited list of my long-term goals with this game. By no means do we need to implement all of them. I will likely cut or edit parts of the TODO. I just want to focus on the section called "MVP for steam release (2026)".

Periodic tasks:

- Prune our various docs/ documents and make sure tasks are marked as completed, specifically `docs/STEAM-MVP.md`.

- Make sure rust warnings get handled.

- Examine code for any TODOs and make sure they are still relevant.