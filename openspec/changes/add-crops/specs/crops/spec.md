# Spec Delta

## Purpose

Wheat, potatoes and carrots that grow, can be farmed by hand or by machines, and feed fuel and animal chains.

## ADDED Requirements

### Requirement: Growing crops
Wheat, potatoes and carrots SHALL be planted from their seeds on dirt or grass, SHALL grow through visible stages over fixed times, and SHALL survive save and reload at any stage.

#### Scenario: Wheat grows
- **WHEN** wheat seeds are planted and 3 game minutes of daylight pass
- **THEN** the wheat is ripe

### Requirement: Harvesting
Harvesting a ripe crop SHALL give its produce and 1 to 2 seeds, deterministically; harvesting an unripe crop SHALL give back its seed only.

#### Scenario: Harvest
- **WHEN** the player breaks ripe wheat
- **THEN** they get 2 wheat and 1 or 2 wheat seeds, the same every time for the same world and planting

### Requirement: Wild crops give the first seeds
Wild wheat, potato and carrot plants SHALL generate on surface grassland, and harvesting them SHALL give seeds.

#### Scenario: First seeds
- **WHEN** the player harvests a wild carrot plant
- **THEN** they get carrots to plant

### Requirement: Fermenting
A powered fermenter SHALL turn wheat or potatoes into ethanol plus straw or mash, with potatoes giving more ethanol per item than wheat.

#### Scenario: Ethanol
- **WHEN** a fermenter processes 4 potatoes
- **THEN** it outputs more ethanol than it would from 4 wheat, and 2 mash
