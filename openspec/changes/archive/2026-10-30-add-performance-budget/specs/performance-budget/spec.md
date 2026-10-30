# Spec Delta

## Purpose

Sets how fast the game must run (a tick budget and a frame budget), how that is measured with repeatable benchmark scenarios, and how a developer profiles a slow scenario.

## ADDED Requirements

### Requirement: Tick and frame budget
The game SHALL finish each tick in at most 16 ms and each frame in at most 16 ms at an 80x24 screen, measured on a release build. A tick is one call that advances the simulation for a player action, including a 1000-tick Pass Time. A frame is one client step plus drawing the screen. The budget applies to the worst tick and worst frame of each benchmark scenario, after the scenario's setup steps.

#### Scenario: A scenario within budget
- **WHEN** the benchmark runs a scenario and its worst tick and worst frame are each 16 ms or less
- **THEN** the scenario is reported as within budget

#### Scenario: A scenario over budget
- **WHEN** any tick or frame in a scenario takes more than 16 ms
- **THEN** the scenario is reported as over budget, naming whether ticks, frames or both went over

### Requirement: Benchmark scenarios
The benchmark SHALL run a fixed set of scenarios written as headless scripts, so every run does the same work for the same seed and build. The set SHALL cover at least walking on the surface, Pass Time at night, a busy factory, and the deep underground.

#### Scenario: Same work every run
- **WHEN** the benchmark runs the same scenario twice on the same build
- **THEN** both runs perform the same steps and print the same screens and state as running the script with the headless client

#### Scenario: Every scenario is a valid script
- **WHEN** the test suite runs
- **THEN** every benchmark scenario script runs to the end with the headless client without a script error

### Requirement: Benchmark report
The benchmark SHALL be started with a single recipe. It SHALL build in release mode and print, for each scenario, the tick count and frame count with their median, 95th percentile and worst times in milliseconds next to the budget. It SHALL exit with a non-zero status when any scenario is over budget.

#### Scenario: Running the benchmark
- **WHEN** a developer runs the benchmark recipe
- **THEN** a table lists every scenario with its tick and frame times and whether it is within budget

#### Scenario: Over budget fails the run
- **WHEN** at least one scenario is over budget
- **THEN** the recipe exits with a non-zero status after printing the full table

### Requirement: Timing never changes the game
Measuring time SHALL NOT change what the game does or prints. Game logic and rendering SHALL keep reading time only from the client's clock, and headless runs without the benchmark SHALL print identical output whether or not the benchmark code is built in.

#### Scenario: Scripts unchanged
- **WHEN** the snapshot scripts run after the benchmark is added
- **THEN** their output matches the stored snapshots

### Requirement: Profiling a scenario
The project SHALL provide a recipe that profiles one headless script on an optimized build with debug symbols, without cleaning the build and without the interactive client. It SHALL write a flamegraph and a profile file that speedscope.app can open, and print where both were written.

#### Scenario: Profiling a slow scenario
- **WHEN** a developer runs the profiling recipe with a benchmark scenario script
- **THEN** the script runs headless under the profiler and the recipe prints the paths of the flamegraph and the speedscope file

#### Scenario: Profiler missing
- **WHEN** the profiler isn't installed or the system doesn't allow profiling
- **THEN** the recipe stops with a message saying what to install or which setting to change

### Requirement: Hotspot fixes are guarded
Each hotspot fixed under this budget SHALL have a test that fails if the fix is undone. The test SHALL measure the work done (for example how many times something is computed per tick), not time, so it gives the same answer on every machine.

#### Scenario: A fix is undone
- **WHEN** a later change brings back the redundant work a hotspot fix removed
- **THEN** that hotspot's test fails in `cargo test`
