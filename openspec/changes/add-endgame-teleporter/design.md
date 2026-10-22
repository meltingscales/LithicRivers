# Design

## Context

- **Factory blocks** run wherever they are, on elapsed game time (`factory::run_due_steps`).
- **`add-factory-stats`** counts items produced and consumed per game minute.
- **Multiblocks** exist in a small form in other proposals (the distillation column).
- **Story:** the lore principle in AGENTS.md says the goal is revealed slowly.

## Goals / Non-Goals

**Goals:**
- An ending that tests sustained production.
- Clear feedback on which inputs keep up.
- Hidden until earned.
- Factory steps fast enough at endgame scale.

**Non-Goals:**
- The ending's cinematic (a later change).
- The quest line that reveals the teleporter (a quest change).
- Multiplayer.

## Decisions

### The multiblock
- An 11-by-11 square: a core block at the center, frame blocks around the border, and input port blocks at set border positions (one per input kind).
- It's checked when a block of it is placed or broken. An incomplete frame shows which positions are missing.

### Rates and charge
- **Tracking:** each input port records how many items arrived per game minute, over the last 10 minutes.
- **Rates:** each input's required rate is a constant (tuned so the full factory is large but buildable).
- **Charge, per game minute:**
  - every input at or above its rate: +1 point
  - any input under: + (the lowest input's fraction) points
  - any input under 50% for 3 minutes straight: -1 point

  120 points opens it.
- **Colors:**

  | Color | Recent rate (vs. required) |
  |---|---|
  | Green | at least 110%, and not falling |
  | Yellow | 95% to 110% |
  | Red | below 95%, or falling over the last 3 minutes |

### Hidden until earned
`Resources::revealed: BTreeSet<Reveal>` (saved). `Reveal::Teleporter` gates its recipes in the crafting tab, its Item Dictionary entries, and its block placement. A cheat can set it for testing.

### Performance
A benchmark test builds a 2,000-block factory (belts, machines, power) and asserts a factory step stays under a budget in release builds. It runs in CI's release job, not in every `cargo test`.

## Risks / Trade-offs

- [Two hours is too long or too short] → The total and rates are constants, tuned in playtests.
- [Spoiling the secret] → No recipe, dictionary entry or tooltip names it before the reveal, and a test checks the crafting list and the dictionary for it before and after.
