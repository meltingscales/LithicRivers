# Tasks

## 1. Cues

- [x] 1.1 Add `sound::SoundCue`, `Resources::sounds` (not saved, capped) and `Resources::play`; queue cues for player mining (Chop or Mine), breaking blocks, pickup, combat damage (Hit or Hurt) and teleporting; verify core tests for mining, chopping, hurt, a harvester felling a tree silently, and a save with no sounds

## 2. Playback and sounds

- [x] 2.1 Add `scripts/generate_sfx.py` and the seven WAVs it writes; register all eight effects; play and clear queued cues in `step`; queue Place and Craft in the client; skip effects when the sink is backed up; verify a client test that every cue's asset is embedded, a client test that placing a block queues its cue, and `cargo test --workspace`
- [x] 2.2 Update `docs/STEAM-MVP.md`, `docs/SPEC-GAPS.md` and AGENTS.md; verify `just code-clippy`
- [ ] 2.3 User check: each sound fits its action, nothing is too loud or grating, and fast mining doesn't lag behind
