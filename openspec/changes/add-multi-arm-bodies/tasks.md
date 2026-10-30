# Tasks

## 1. Core

- [ ] 1.1 `BodyPartType::ExtraArm(n)`, the torso kind (serde default Standard) and `Body::arms()`; verify `cargo test` that old saves load with two arms and a quad body has four
- [ ] 1.2 Damage targeting, repair and install over every arm; verify tests with a six-armed body and a deterministic replay
- [ ] 1.2a Multi-arm torso items (quad, hexa, octa) that only the advanced resurrection vat accepts, with extra arm slots in its window, so the robot wakes with that many arms (`add-backup-bodies` shipped the advanced vat with two arms); verify a test waking from an advanced vat with a hexa torso
- [ ] 1.3 Mining and melee from the two best working arms (shared with `add-body-damage-effects`); verify a four-armed robot mines like a two-armed one at full health

## 2. Client

- [ ] 2.1 The Body tab listing every arm slot, and body art for 4, 6 and 8 arms; verify snapshots of each layout

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that multi-arm bodies read clearly in the Body tab and the art
