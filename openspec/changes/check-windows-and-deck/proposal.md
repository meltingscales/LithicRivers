# Proposal

## Why

`add-game-window` replaced the terminal with the game's own window. Its Linux check passed (all three Display modes, text size with no leftover text, fullscreen). Windows and the Steam Deck can't be tested by an agent or in CI beyond building and unit tests, and the user will check them by hand later. This change holds those checks so `add-game-window` can be archived without losing them.

## What Changes

- No code changes. This is a manual test checklist for the Windows build and the Steam Deck, taken over from `add-game-window`'s user checks 4.2 and 4.3.
- A failed check becomes a bug fix of its own (a `fix-...` branch, or a new change if it's large), linked from the task here.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None. This only verifies behavior already in `game-window`, `headless-client` and `title-menu`; it sets `skip_specs`.

## Impact

None on code. When every check passes, archive this change and mark the Steam Deck item in `docs/STEAM-MVP.md`'s launch checklist.
