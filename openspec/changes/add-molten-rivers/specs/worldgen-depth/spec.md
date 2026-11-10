# Spec Delta

## ADDED Requirements

### Requirement: Lava rivers with fords
The cores of the deep band's river channels SHALL be lava, except at fords of walkable lava crust. Fords SHALL be frequent enough that, from any point along a river, one is within 96 tiles. Lava SHALL NOT generate in the sky, on the surface or in the shallows.

#### Scenario: Lava in the deep
- **WHEN** a deep layer is surveyed for a fixed seed
- **THEN** lava makes up the pinned share of tiles, and lava crust appears too

#### Scenario: No lava above the deep
- **WHEN** the surface and the shallows are surveyed for a fixed seed
- **THEN** no tile is lava or lava crust

#### Scenario: Rivers can be crossed
- **WHEN** a river centerline in a deep layer is followed for a fixed seed
- **THEN** a ford appears within the pinned distance from any point along it

### Requirement: Riverglass deposits
Riverglass SHALL form only in the rock along the rims of the deep band's rivers, and SHALL be solid and mineable with a riverglass drop.

#### Scenario: Rare and deep only
- **WHEN** the deep, the shallows and the surface are surveyed for a fixed seed
- **THEN** riverglass makes up the pinned share of deep tiles and none of the others

#### Scenario: Mining riverglass
- **WHEN** the player mines a riverglass tile
- **THEN** they get one riverglass and the tile becomes air
