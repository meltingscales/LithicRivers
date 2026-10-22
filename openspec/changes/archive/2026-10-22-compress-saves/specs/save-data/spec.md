# Spec Delta

## ADDED Requirements

### Requirement: Saves are compressed
A save file SHALL hold the same JSON as before, compact and gzip-compressed. Loading SHALL accept both gzipped and plain JSON saves, telling them apart by the gzip header.

#### Scenario: Smaller saves
- **WHEN** a new world for seed 12345 is saved
- **THEN** the save file is under 100 KB, and decompressing it gives the save's JSON

#### Scenario: Plain saves still load
- **WHEN** a plain JSON save from before this change is loaded
- **THEN** the game loads exactly as it did before

### Requirement: Saving never leaves a broken save
Saving SHALL write a complete new file before replacing the old one, so a failure while saving leaves the previous save intact.

#### Scenario: Failure mid-save
- **WHEN** writing a save fails partway through
- **THEN** the previous save file is unchanged and still loads
