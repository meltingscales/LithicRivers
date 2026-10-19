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
Snapshot output SHALL be identical on Linux, Windows, and macOS CI and across runs, including line endings, in both debug and release test builds.

#### Scenario: CI on another OS
- **WHEN** the snapshot tests run on Windows or macOS in release mode after snapshots were recorded on Linux in debug mode
- **THEN** they pass
