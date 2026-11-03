## ADDED Requirements

### Requirement: Scrap lanterns
The scrap lantern SHALL be a factory block crafted from 1 torch and 1 iron scrap. It SHALL be placed and broken like other factory blocks, SHALL need no fuel or power, and SHALL not block light.

#### Scenario: Craft and place a lantern
- **WHEN** the player crafts a scrap lantern from a torch and an iron scrap and places it east
- **THEN** a lantern stands on the tile east of the player and the inventory has no lantern left

#### Scenario: Break a lantern
- **WHEN** the player breaks a placed lantern
- **THEN** the lantern item returns to the player's inventory and the tile is no longer lit

### Requirement: Lantern light
A lantern SHALL light every tile on its own layer within 6 tiles by circular distance to which it has line of sight, using the same line-of-sight rule as the player's light. Lit tiles SHALL count as illuminated for fog of war, and SHALL be remembered as seen, whenever the player is on the lantern's layer within 48 tiles of it. Lantern light SHALL not reach other layers.

#### Scenario: A lantern lights a cave room
- **WHEN** a lantern stands underground 10 tiles from the player, on the player's layer
- **THEN** a tile 4 tiles from the lantern with a clear line to it is illuminated, and a tile 7 tiles from it is not lit by the lantern

#### Scenario: Walls cast shadows
- **WHEN** a wall stands between a lantern and a tile 3 tiles away
- **THEN** that tile is not lit by the lantern

#### Scenario: Lit tiles are remembered
- **WHEN** the player walks past a lit lantern and then more than 48 tiles away
- **THEN** the tiles the lantern lit are remembered as seen

#### Scenario: Night on the surface
- **WHEN** a lantern stands on the surface at night
- **THEN** the tiles it reaches are illuminated, and tiles out of its reach and the player's are not

#### Scenario: Other layers stay dark
- **WHEN** a lantern stands one layer above the player
- **THEN** no tile on the player's layer is lit by it
