# Tasks

## 1. Core

- [ ] 1.1 Gzipped compact JSON in `save_to_file` (with `flate2`) and header sniffing in `load_from_file`; verify `cargo test` for a gzipped round trip, a plain save loading, and the new world's save under 100 KB
- [ ] 1.2 Safe writes (temporary file, sync, rename); verify a test where a failing writer leaves the previous save intact

## 2. Client and tools

- [ ] 2.1 `world.json.gz` as the save file, loading the old `world.json` too, and removing it after a successful save; verify a headless test that upgrades a folder with only `world.json`
- [ ] 2.2 A `just util-print-save <world folder>` recipe that prints the save as readable JSON; verify it on a test save

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` (including the save-reload identity test) and PR checks pass
- [x] 3.2 User check: an existing world from before this change loads and saves in a real install
