You are allowed to run `just build` and `just install`, and also `just run-*`. This game was going to use a GUI (bevy), but I decided that `ratatui` would be faster, so we're back to using a TUI.

You should use `rustup` to run `cargo` and other tools.

Please avoid running `cargo clean`. It takes a long time to rebuild rust modules.

Keep small copyrights inside of the main game code files. Make sure they exist if they aren't declared.

Guiding principles:

- All randomly-generated choices, actions, damage, loot, worldgen, etc - should be fully deterministic and based on world seed and world tick. This means that this game should be fully TAS-able for any specific initial seed (and version of game code). Always use the seed and any permutation (when appropriate) of XYZ coordinate, biome, or other seeded randomness to generate anything "random".

- This game renders using ASCII text.

- TODO.md contains a big, unedited list of my long-term goals with this game. By no means do we need to implement all of them. I will likely cut or edit parts of the TODO. I just want to focus on the section called "MVP for steam release (2026)".

- We're currently in the process of migrating from Python to Rust. Please see the migration plan for more information, called "migration-plan.md" and "TODO-RUST-MIGRATION.md".