# Spec Delta

## Purpose

A signal network for factories: cables, sensors, conditions on machines, and logic blocks, edited in one logic editor.

## ADDED Requirements

### Requirement: Networks and signals
Touching SapienNet cables SHALL form a network, blocks next to it SHALL connect, and the network SHALL carry named integer signals summed from every sensor on it each factory step.

#### Scenario: Two counters
- **WHEN** two animal counters on one network each see 4 sheep
- **THEN** the network's `sheep` signal is 8

### Requirement: Enable conditions
A connected machine, extractor, router or generator SHALL be able to have an enable condition, SHALL work only while it holds, and SHALL show why it's waiting when it doesn't.

#### Scenario: A full pen
- **WHEN** a feeder's condition is `sheep < 12` and the network reads 14 sheep
- **THEN** the feeder doesn't feed, and its window shows the condition is false with sheep = 14

### Requirement: Logic blocks
Comparators, calculators and memory cells SHALL compute signals from other signals, each reading the previous factory step's values, so the result doesn't depend on placement order.

#### Scenario: A calculator
- **WHEN** a calculator is set to `logs_out = chest_logs - 64` and the chest holds 100 logs
- **THEN** the network carries `logs_out = 36` from the next factory step

### Requirement: The logic editor
One window SHALL list a network's signals with their live values and edit conditions and formulas, rejecting invalid expressions with a message and keeping the last valid one.

#### Scenario: A typo
- **WHEN** the player enters `sheep <` as a condition
- **THEN** the editor shows a parse error and the block keeps its previous condition
