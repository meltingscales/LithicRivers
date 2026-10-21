# Tasks

## 1. Playback

- [ ] 1.1 The demo player: parse a scenario script, run one step per beat on `app.clock`, start each scenario in a throwaway app state, cut and loop; verify headless tests for cuts, looping, determinism, and that no world folder is created
- [ ] 1.2 `SplashState::Demo`, the Watch Demo title choice, and Escape back to the title; verify headless tests for starting, ignored keys and leaving

## 2. Banner and scenarios

- [ ] 2.1 The scrolling red DEMO banner with the scenario name; verify a snapshot and a test that it shifts one column per 100 ms
- [ ] 2.2 Explore, combat, build and factory scenario scripts; verify each runs as a snapshot test and fails loudly if it breaks

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that the demo reads well in the interactive client
