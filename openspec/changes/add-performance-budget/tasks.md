# Tasks

## 1. Measure

- [x] 1.1 Add the four scenario scripts in `scripts/bench/` (`size 80x24`, setup before the first `beat`) and a test that runs each one headless; verify `cargo test --workspace` and `just client-blind scripts/bench/<name>.txt` for each
- [x] 1.2 Add the `--bench <script>...` mode: time update and draw per frame after the first `beat`, print the report table (median, 95th percentile, worst, budget, CPU and build profile), exit 1 when over budget; verify a unit test for the report and exit status with fake timings, and that snapshot tests still pass unchanged
- [x] 1.3 Add `just util-bench` (release build over `scripts/bench/*.txt`); verify it prints the table, and record the baseline and the machine in `docs/STEAM-MVP.md`

## 2. Profile

- [ ] 2.1 Add `[profile.profiling]`, `just util-profile <script>` (perf with dwarf call graphs, `profile.linux-perf.txt` for speedscope, `flamegraph.svg` via inferno) with a clear stop when perf is missing or not allowed, and `just util-profile-setup`; remove `profile-client-linux`, `profile-client-windows` and `profile-setup` and update docs that mention them; verify `just --list` and a run on one scenario that writes both files
- [ ] 2.2 Profile every scenario and name the two biggest hotspots, with their share of the time, in design.md; verify both output files exist for each scenario

## 3. Fix

- [ ] 3.1 Fix the first hotspot and add its work-counting guard test; verify the guard fails with the fix undone, `cargo test --workspace` passes with no snapshot changes, and `just util-bench` shows the improvement
- [ ] 3.2 Fix the second hotspot the same way; verify the same checks
- [ ] 3.3 If any scenario is still over budget, profile it again and record what remains (and whether it needs its own change) in `docs/STEAM-MVP.md`; verify `just util-bench`

## 4. Checks

- [ ] 4.1 Update `docs/STEAM-MVP.md` (results, tick the performance items), and add the bench and profile recipes to AGENTS.md; verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.2 User check: the game feels responsive in the interactive client on your machine (and on a Steam Deck, if one is available)
