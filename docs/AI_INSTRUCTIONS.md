I'm using cursor, which can't run a TUI. If you want me to test the game TUI, please ask me to.

Don't ever run `make run`, because you don't have a terminal that's guaranteed to have space. Instead, tell me to run it.

Keep small copyrights inside of the main game code files. Make sure they exist if they aren't declared.

Guiding principles:

- All randomly-generated choices, actions, damage, loot, worldgen, etc - should be fully deterministic and based on world seed and world tick. This means that this game should be fully TAS-able for any specific initial seed (and version of game code). Always use the seed and any permutation (when appropriate) of XYZ coordinate, biome, or other seeded randomness to generate anything "random".

- TODO.md contains a big, unedited list of my long-term goals with this game. By no means do we need to implement all of them. I will likely cut or edit parts of the TODO. I just want to focus on the section called "MVP for steam release (2026)".

- We're currently in the process of migrating from Python to Rust. Please see the migration plan for more information, called "migration-plan.md" and "TODO-RUST-MIGRATION.md".