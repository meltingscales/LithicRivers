# Proposal

## Why

Blueprints live in one world's save, so a good build can't be reused in another world or shared with other players. Factorio's blueprint strings fixed this for its community: a build becomes one line of text to paste anywhere. Factorio's blueprint books and its filter for removing kinds of blocks from a blueprint also keep a big library manageable.

## What Changes

- **Blueprint strings:** a blueprint exports as one line, `LITHICRIVERS-BLUEPRINT-<payload>`. The payload is base64 of the compressed blueprint, with a format version.
  - **Export:** the Blueprints window shows the string, copies it to the system clipboard where the terminal allows it, and saves it to a text file in the game's data folder.
  - **Import:** pasting a string into the Blueprints window adds the blueprint to the library. A bad or newer-format string is refused with a message, and nothing is added.
- **Blueprint books:** folders of blueprints, nested if the player wants, shown as a tree in the Blueprints window. Blueprints and books can be moved between books, renamed and deleted. A whole book exports and imports as one string too.
- **Removing kinds of blocks:** an edit pane lists the kinds of blocks in the selected blueprint with their counts (for example, "Wooden Conveyor x12"). Removing a kind deletes every block of that kind from the blueprint.

## Capabilities

### New Capabilities
- `blueprint-sharing`: Blueprint strings, export and import, blueprint books, and removing kinds of blocks from a blueprint.

### Modified Capabilities
- `blueprints`: The library becomes a tree of books and blueprints, and the Blueprints window gains export, import, moving and the edit pane.

## Impact

- **Core:**
  - `blueprints::to_string` / `from_string` (JSON, deflate, URL-safe base64, with `v: 1`), using the `flate2` and `base64` crates.
  - The library becomes a tree (`Book { name, entries }`). Old saves load their flat list as the top-level book (serde default).
- **Client:**
  - The Blueprints window's tree view, with move, rename, delete, export, import and the edit pane.
  - Import reads a terminal paste (crossterm bracketed paste), so a long string arrives at once.
  - Export uses the OSC 52 clipboard escape (no clipboard dependency) and also writes a file.
- **Headless:** a `paste "<text>"` script step to drive imports in tests.
