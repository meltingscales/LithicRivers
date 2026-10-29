# Spec Delta

## MODIFIED Requirements

### Requirement: Smelting table
Smelting SHALL follow one table shared by every furnace: log → 1 biochar, 2 wooden planks → 1 biochar, 4 sticks → 1 biochar, 4 acorns → 1 biochar, 8 wood shavings → 1 biochar, 1 copper concentrate → 1 copper ingot, 1 iron concentrate → 1 iron ingot, 1 clay → 1 fired brick, each with its own smelting time. Items not in the table SHALL NOT be accepted as smelting input.

#### Scenario: Smelt a log
- **WHEN** a fueled crude furnace has a log in its input and its smelting time passes
- **THEN** the log is gone and 1 biochar is in the output

#### Scenario: Batch input
- **WHEN** a furnace has 3 wooden planks in its input and enough time passes for one smelt
- **THEN** 2 planks are consumed, 1 biochar is produced, and 1 plank waits for another plank

#### Scenario: Not smeltable
- **WHEN** the player tries to put stone into a furnace's input
- **THEN** it is refused with a message that stone can't be smelted

#### Scenario: Smelt copper concentrate
- **WHEN** a powered electric furnace has copper concentrate in its input and its smelting time passes
- **THEN** it holds 1 copper ingot
