# sprite-art Specification

## Purpose
Every sprite has real art, and stays that way.

## Requirements

### Requirement: No placeholder art
Every sprite's 12x8 art SHALL be real art; a test SHALL fail and name any sprite whose art is a placeholder (mostly `x` characters).

#### Scenario: A new sprite without art
- **WHEN** a developer adds a sprite whose art12x8 is the placeholder box
- **THEN** `cargo test` fails and names the sprite

### Requirement: Sprite gallery snapshot
A headless snapshot SHALL show every sprite with its name, its 1x, 2x and 3x sprites and its 12x8 art, so art changes appear in review.

#### Scenario: Art changes
- **WHEN** a sprite's art changes
- **THEN** the gallery snapshot diff shows the old and new art

### Requirement: Small sprites are distinct
Sprites that appear side by side in normal play SHALL be distinguishable at 1x by glyph or color.

#### Scenario: Harvester next to a belt
- **WHEN** an auto-harvester faces a belt at the default zoom
- **THEN** the two read as different blocks
