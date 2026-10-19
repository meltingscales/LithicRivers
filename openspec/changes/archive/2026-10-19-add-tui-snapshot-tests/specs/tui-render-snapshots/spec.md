# Spec Delta

## Purpose

Catches unintended changes to what the client renders or reports by comparing scripted headless runs against reviewed text snapshots during `cargo test`.

## ADDED Requirements

### Requirement: Script-based snapshot suite
`cargo test --workspace` SHALL run every script in `scripts/snapshots/` headlessly and compare its complete output against that script's stored snapshot.

#### Scenario: Unchanged UI
- **WHEN** no rendering or game behavior changed
- **THEN** every snapshot test passes

#### Scenario: New script without a snapshot
- **WHEN** a script is added to `scripts/snapshots/` with no stored snapshot
- **THEN** the test fails and writes a pending snapshot for review instead of silently passing

### Requirement: Readable failure
A snapshot mismatch SHALL fail the test and print a line-oriented diff between the stored and new output, naming the script.

#### Scenario: Panel text changed
- **WHEN** a change alters a label in the inventory panel
- **THEN** the inventory snapshot test fails and the diff shows the old and new line

### Requirement: Accepting intended changes
There SHALL be one documented command that re-records or accepts pending snapshots, so an intended change can be committed together with its updated snapshots.

#### Scenario: Accept after a deliberate UI change
- **WHEN** a developer changes a panel layout on purpose and runs the documented accept command
- **THEN** the stored snapshots are updated and `cargo test --workspace` passes

### Requirement: Baseline coverage
The initial suite SHALL include snapshots for the world view after start and for each top-level menu tab, plus the combat panel and the cheat console.

#### Scenario: Every tab covered
- **WHEN** the suite runs
- **THEN** there is at least one snapshot showing each top-level menu tab

### Requirement: Stable across machines
Snapshot output SHALL be identical on Linux, Windows, and macOS and across runs, including line endings, in both debug and release test builds. Text that depends on the build or machine (version, git branch and commit, config and log file paths) SHALL be masked or hidden.

#### Scenario: CI on another OS
- **WHEN** the PR checks run the snapshot tests on Windows and macOS after snapshots were recorded on Linux
- **THEN** they pass

#### Scenario: Release build
- **WHEN** the tests run with `cargo test --release`
- **THEN** the snapshots recorded from a debug build pass

#### Scenario: New commit
- **WHEN** the version or git commit changes and nothing else does
- **THEN** every snapshot test still passes
