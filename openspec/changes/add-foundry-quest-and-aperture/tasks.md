# Tasks

## 1. The foundry and quest 5

- [ ] 1.1 The `sapiencorp-foundry` plan (entrance yard with guards, casting hall with a five-tile lava pit and stone heaps, pattern vault with a `TreasureQuest4` chest), queued at `FOUNDRY_POS` with the other quest structures and for old saves; verify `just util-structure-preview sapiencorp-foundry` and `cargo test` that every new world has it, its chest holds the casting patterns, and the vault is reachable only across the pit
- [ ] 1.2 `QuestType::Foundry`, its four ordered steps, the offer and hand-in dialogue, the casting patterns item, the reveal on hand-in, the foundry marker, and `tp_quest foundry`; verify `cargo test` for not offered before quest 4, offered after, the steps in order (including early), bridging the pit completing step 2, and the hand-in revealing the Aperture

## 2. The Aperture

- [ ] 2.1 `Resources::revealed` and `Reveal::Aperture` (saved) gating the Crafting tab, Item Dictionary, picker and placement, with the `reveal aperture` cheat; verify `cargo test` and client tests that nothing shows before, everything after, and across a save and reload
- [ ] 2.2 The core, frame and port blocks, items and recipes, and the 11-by-11 completeness check naming the first missing position; verify `cargo test` for complete, missing a frame, a wrong port, and breaking a block
- [ ] 2.3 Per-port input tracking (each port takes only its item) and the charge model in milli-points; verify `cargo test` for full rates opening after 120 minutes, a 60% input adding 0.6, a three-minute shortage draining a point, the cap at zero, and determinism
- [ ] 2.4 The flow window (rates, colors, charge, time left, or the missing position) through Interact; verify snapshots of an incomplete Aperture and of green, yellow and red inputs

## 3. The ending

- [ ] 3.1 The closing sequence and credits on opening, stepping on keys and timed from `app.clock`, then back to the world with the open Aperture saved, and "The far side is not ready for you yet." on Interact; verify a `just client-blind` snapshot driving it with a cheated charge, and a save/reload test of the opened state

## 4. Art, bench and docs

- [ ] 4.1 Sprites for the core, frame, ports (closed and open), the casting patterns and the foundry's tiles, and the closing pages' art; verify the sprite gallery and font tests
- [ ] 4.2 `scripts/bench/aperture-farm.txt`; verify `just util-bench` stays inside the budget
- [ ] 4.3 `docs/QUESTS.md` (quest 5 and the ending), `AGENTS.md` (`tp_quest foundry`, `reveal aperture`, the Aperture), `docs/STEAM-MVP.md`, and remove `openspec/changes/add-endgame-teleporter`; verify by reading the diff

## 5. Checks

- [ ] 5.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 5.2 User check: the foundry and the reveal land, the Aperture's rates make building a big factory feel worth it, and the ending feels earned

## Workflow follow-up

- When merged with only 5.2 left, add a playtest queue entry for it.
- The far side past the Aperture is a separate change, to propose once its direction is chosen.
