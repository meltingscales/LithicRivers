# Proposal

## Why

SapienNet's conditions and logic blocks (`add-sapiennet`) cover simple rules. Bigger automation (a sorting system, a state machine for a reactor, a smart farm) gets unreadable as a wall of comparators. The user's idea (IDEAS-INBOX.md), inspired by ComputerCraft: a lightweight Lua IDE for SapienNet logic. Scripts live as plain files in the world's folder, so any real editor can work on them, with a guide and examples for different machines.

## What Changes

- **A SapienNet computer:** a block that runs one Lua script. Through a small API, the script reads the network's signals, writes its own output signals, and sets the enable state of connected blocks.
- **Scripts are files:**
  - Each computer's script is `scripts/<computer name>.lua` in the world's folder, next to the gzipped save.
  - Edit it in the game or in any editor; the game reloads it when the file changes.
  - Saves keep only the computer's name and state, not the script text.
- **The in-game IDE:** a window with the script, line numbers, syntax colors, the network's live signals, the last error with its line, and a log of the script's `print` output.
- **Safe and deterministic:**
  - Scripts run once per factory step with a fixed instruction budget.
  - They can't touch files, the network beyond SapienNet, the clock or random numbers, except a seeded `random()` the API provides.
  - A script that errors or runs out of budget stops and shows why. The factory keeps running.
- **A guide and examples:** `docs/SAPIENNET-SCRIPTING.md` with the API, and example scripts for a farm, a sorter, a power manager and a furnace line. Examples are also found in the world, on dead SapienCorp robots, which ties in the old idea of salvaged programs like `TreeCutter.lua` from `docs/TODO.md`.

## Capabilities

### New Capabilities
- `sapiennet-scripting`: The SapienNet computer, Lua scripts as world files, the sandboxed per-step API, the in-game IDE and the guide.

### Modified Capabilities
- `sapiennet`: computers are network members that read signals and write outputs and enable states.

## Impact

- **Core:**
  - `FactoryKind::Computer`.
  - An embedded Lua (the `mlua` crate with Luau, chosen for its sandboxing and instruction limits).
  - The API (`signals()`, `emit(name, value)`, `enable(block, on)`, `print`, `random()`).
  - Per-step execution with a budget.
  - Script state kept in the save.
- **Client:**
  - the IDE window (an editor widget with syntax colors and line numbers, and the error and log panes)
  - watching the world's `scripts/` folder for changes
- **Saves:** scripts are separate files in the world's folder. Moving a world folder keeps them together.
- **Depends on:** `add-sapiennet`. `compress-saves` keeps the JSON save separate, as the user asked.
