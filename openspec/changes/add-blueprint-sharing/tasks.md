# Tasks

## 1. Core

- [ ] 1.1 `to_string` / `from_string` with the prefix, version, deflate and URL-safe base64; verify `cargo test` for a round trip, a damaged payload, a missing prefix, a newer version and an unknown block kind
- [ ] 1.2 The library as a tree of books, loading old flat libraries; verify tests for moving, renaming, deleting, and loading a save from before books
- [ ] 1.3 Removing kinds of blocks; verify a test that removes chests and keeps belts

## 2. Client

- [ ] 2.1 The Blueprints window's tree, with move, rename, delete and new book; verify a headless test and a snapshot
- [ ] 2.2 Export: string on screen, OSC 52 and the file; verify a headless test that reads the file back
- [ ] 2.3 Import from a paste (bracketed paste, and a `paste` script step); verify a headless test that imports an exported string into a fresh world
- [ ] 2.4 The edit pane for removing kinds; verify a headless test and a snapshot

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that exporting and importing between two worlds works in a real terminal, including the clipboard
