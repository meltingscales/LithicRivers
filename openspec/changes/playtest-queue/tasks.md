# Tasks

Play the real game (`just client-linux`, or the release build). Turn on Allow cheats when creating the world; the cheat console takes the commands below. For each entry: tick it here and in the named change when it passes, then ask an agent to archive that change in its own PR. When a check fails, write a short note under it (what you did, what went wrong) and ask for a fix; leave the entry open until the fix passes.

## 1. Waiting changes

- [ ] 1.1 `add-backup-bodies` (task 3.2): preparing a vat feels worth it and the flyover looks good. Setup: `giveitem crude_vat`, `giveitem crude_torso`, `giveitem crude_fusion_core`; place the vat, fill it through Interact, walk away, then `kill_player` and choose the vat on the Resurrect screen. Try a `standard_vat` on a powered pole too (`giveitem standard_vat`, `burner_generator`, `power_pole`)
- [ ] 1.2 `add-map-waypoints` (task 3.2): managing waypoints on the Global Map feels clear. Setup: open the Global Map tab, add a waypoint (`w`), walk somewhere and move it to you (`m`), rename it (the Global Map lists the rename key), delete it (`x`, then `y`), and try deleting a quest marker (it should refuse)
- [x] 1.3 `add-enemy-spawning` (task 3.2, once PR #193 is merged): areas feel alive without floods, and a farm is fun to build. Setup: walk the surface and a cave for a few game minutes and watch creatures come back out of sight; then build a fenced spawner pen (`giveitem spawner`, `fence`, `gate`, `meat_slurry`, `feral_dog_embryo_poor`, plus power) or look at the Factory showcase's last example
  - 2026-11-03: passed ("hard but fair"). Natural spawn rates are being cut and limited in a follow-up change.
- [ ] 1.4 `check-windows-and-deck`: work through that change's own checklist (Windows launch, input, the Display panel, Steam Deck)
- [ ] 1.5 `polish-android`: work through that change's own checklist (section 1, playing on a phone)

## 2. Keeping the queue

- [ ] 2.1 When the last entry above is ticked and its change archived, archive this change too; the next change that ships with only user checks left starts a new `playtest-queue`
