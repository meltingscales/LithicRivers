Make sure to always use uv to run scripts.

I'm using cursor, which can't run a TUI. If you want me to test the game TUI, please ask me to.


Guiding principles:

- All randomly-generated choices, actions, damage, loot, worldgen, etc - should be fully deterministic and based on world seed and world tick. This means that this game should be fully TAS-able for any specific initial seed (and version of game code). Always use the seed and any permutation (when appropriate) of XYZ coordinate, biome, or other seeded randomness to generate anything "random".