# Proposal

## Why

A new world's save is 5.87 MB, and it grows with every chunk explored. Measured on seed 12345 with 59 generated chunks:

| Format | Size |
|---|---|
| Pretty JSON (what `save_to_file` writes today) | 5.87 MB |
| Compact JSON | 1.73 MB |
| Compact JSON, gzip -9 | 19 KB |

Almost all of a save (99%) is chunk tiles stored as strings, and 241,664 tiles are only 10,274 runs of the same tile, so compression shrinks it about 300 times. Smaller saves load and save faster, and make Steam Cloud saves and `add-world-snapshots` (a snapshot every few minutes) affordable. A TOON or binary format was considered: TOON saves only about a quarter, and both are harder to read and debug than JSON (see IDEAS-INBOX.md).

## What Changes

- **Saves are compact JSON, gzipped,** written to `world.json.gz` in the world's folder. The JSON inside is unchanged (same version 5 structure), so a save can still be read with `gunzip`.
- **Loading accepts both:**
  - gzipped or plain JSON, told apart by the gzip header, not the file name
  - `world.json.gz` first, then the old `world.json`

  Old saves keep loading, and the next save writes the gzipped file.
- **Saving is safe against crashes:** the save is written to a temporary file in the same folder, then renamed over the old one, so a crash mid-save never leaves a half-written world.
- **The old plain file is removed** after a gzipped save succeeds, so a world folder holds one save.
- **Headless and debugging:**
  - `save-reload` and the MCP tools go through the same code.
  - A `just util-*` recipe prints a world's save as readable JSON, for debugging.
- **Not in this change:** storing tiles as runs or palette indexes (a later step if saves grow again), Steam Cloud and snapshots themselves.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `save-data`: saves are gzipped compact JSON, and both gzipped and plain saves load.
- `world-saves`: a world's save file is `world.json.gz`, written safely, with the old `world.json` loaded and replaced.

## Impact

- **Core:**
  - `save_load::save_to_file` writes compact JSON through `flate2::write::GzEncoder` to a temporary file, then renames it into place.
  - `load_from_file` sniffs the gzip header (`1f 8b`).
  - Adds the `flate2` crate.
- **Client:** `worlds::SAVE_FILE` becomes `world.json.gz`, and a world's load tries it first, then `world.json`.
- **Tests:**
  - a round trip through the gzipped file
  - loading a plain `world.json` from before this change
  - a crash mid-save (simulated by a failing writer) leaving the old save intact
  - the plain file removed after saving
  - the save-reload identity test unchanged
