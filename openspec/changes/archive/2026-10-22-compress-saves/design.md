# Design

## Context

- `save_load::save_to_file` writes pretty JSON straight to the target path through a `BufWriter`, so a crash mid-write can truncate the only save.
- `load_from_file` reads JSON and checks `version`.
- The client keeps each world in `saves/<world>/`, with `world.json` and settings (`worlds::SAVE_FILE`).
- `save_to_string` (in-memory, used by `save-reload`) is compact JSON already.
- `save_game_msgpack` exists but nothing uses it.

## Goals / Non-Goals

**Goals:**
- Much smaller saves with no change to what a save holds.
- Old saves load.
- No half-written saves.
- Still debuggable.

**Non-Goals:**
- A new save version or structure.
- Tile run-length or palette encoding.
- Cloud sync, snapshots.
- Removing the unused MessagePack path (a separate cleanup).

## Decisions

### gzip, not zstd
gzip -9 gave 19 KB and zstd -19 gave 20 KB on the measured save. gzip is readable with standard tools everywhere, and `flate2` is a small, mature crate. Use the default compression level, not 9, for speed. Pin the size in a test so a regression shows.

### Telling formats apart
Loading reads the first two bytes: `1f 8b` means gzip, anything else is plain JSON. A renamed or hand-extracted file still loads.

### Safe writes
1. Write to `world.json.gz.tmp` in the same folder.
2. Flush and sync it.
3. `rename` it over `world.json.gz`. A rename within a folder replaces the file in one step on Linux, Windows and macOS.
4. Remove a leftover `world.json` only after the rename succeeds.

### Where the format is decided
`save_load` owns the bytes. The client only chooses the path, so headless runs and MCP sessions get the same format.

## Risks / Trade-offs

- [Saves aren't readable in a text editor] → `gunzip -c world.json.gz` or the new `just util-print-save` recipe shows them.
- [Rollback] → An older build can't read `world.json.gz`. Loading plain JSON stays supported forever, so un-gzipping the file restores compatibility.
