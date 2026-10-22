# Design

## Context

- `blueprints::Blueprint { name, width, height, blocks }` and `BlueprintLibrary { blueprints, selected }` on `Resources` (saved).
- The Blueprints window (`B` in Blueprint mode) lists blueprints and selects, renames and deletes them.
- No clipboard, base64 or compression crates are in the workspace yet.

## Goals / Non-Goals

**Goals:** strings that survive copy and paste anywhere (chat, forums, issue trackers); old strings keep working as the format grows; the library stays easy to manage when it's large.

**Non-Goals:** an online blueprint browser, sharing between game versions with different block kinds (an unknown kind refuses the import, naming it), Factorio's deconstruction and upgrade planners.

## Decisions

### String format
- `LITHICRIVERS-BLUEPRINT-` then URL-safe base64, with no padding, of deflate-compressed JSON: `{ "v": 1, "blueprint": ... }`, or `"book": ...` for a book.
  - JSON keeps it readable when decoded and lets serde defaults absorb new fields.
  - URL-safe base64 has no `+` or `/`, which some chat apps mangle.
- Import trims whitespace and checks the prefix and version. Unknown block kinds are refused by name.

### Clipboard
- Export writes OSC 52 (`ESC ] 52 ; c ; <base64> BEL`). Most modern terminals put this on the system clipboard, and it needs no dependency.
- Export always shows the string in the window and writes `<data folder>/blueprints/<name>.txt` too, because some terminals ignore OSC 52.
- Import is a paste into the window: crossterm's bracketed paste gives the whole string as one event instead of thousands of key presses. Typing it by hand works too.

### Books
- `BlueprintLibrary` becomes a root `Book`, whose entries are blueprints or books. The selection is a path of indices.
- Moving uses a "cut" key and a "put here" key, so it needs no drag and drop. The mouse can click rows (the click targets from `add-mouse-menus`).

### Removing kinds
- The edit pane groups the selected blueprint's blocks by `Piece` and counts them. Removing a kind filters the blocks out and keeps the blueprint's size. An empty result is allowed, but pasting it says it has no blocks.

## Risks / Trade-offs

- [Strings from a future version] → The version check refuses them with "made by a newer LithicRivers".
- [A huge paste floods the input] → Bracketed paste makes it one event. Strings over 1 MB are refused.
- [OSC 52 silently ignored] → The string is always on screen and in a file as well.
