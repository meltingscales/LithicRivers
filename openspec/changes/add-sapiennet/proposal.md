# Proposal

## Why

Factories today run whenever they have inputs and power. There's no way to say "stop feeding when the pen has 12 sheep" or "only burn fuel when the batteries are low". The user's idea (IDEAS-INBOX.md) is SapienNet: SapienCorp's old signal network, with cables like power cables, sensors such as the animal counter, and a logic editor like Factorio's circuit network. Scripting comes later (`add-sapiennet-scripting`).

## What Changes

- **SapienNet cables:** a cheap block placed on tiles. Touching cables join one network, and blocks next to a network connect to it. Cables carry signals, not power or items. They can be walked over.
- **Signals:**
  - Each network carries named integer signals, for example `sheep = 9` or `chest_logs = 120`.
  - Every sensor on the network adds its values each factory step, and every block reads the totals.
- **Sensors:**
  - the animal counter (`add-animal-farming`), outputting a count per animal kind
  - a **storage reader** next to a chest or machine, outputting its item counts
  - a **power reader** on a pole, outputting the grid's supply, demand and battery charge (once `add-power-cells` exists)
- **Conditions:**
  - Any machine, extractor, router or generator connected to a network can have an **enable condition** set in its window, like `sheep < 12` or `chest_logs >= 64 AND power_satisfaction < 100`.
  - It runs only while the condition holds.
- **Logic blocks**, for computing new signals:
  - a **comparator** outputs a signal while a condition holds
  - a **calculator** does arithmetic (`+ - * / %`) on two signals or a constant
  - a **memory cell** holds a value until set or reset
- **The logic editor:** one window for all of this. It lists the network's signals and their live values, and edits a block's condition or formula with a small expression language, with autocomplete over signal names.
- **Look mode** shows a cable's network and its current signals.

## Capabilities

### New Capabilities
- `sapiennet`: Cables and networks, named signals, sensors, enable conditions, logic blocks and the logic editor.

### Modified Capabilities
- `factory-blocks`: connected blocks can have an enable condition.

## Impact

- **Core:**
  - `sapiennet.rs` with networks, rebuilt like power networks when cables change.
  - A per-step signal pass: sensors write, then logic blocks compute, in a fixed order.
  - Conditions (a small parsed expression type, saved).
  - Blocks check their condition before working.
- **Client:**
  - the logic editor window, with expression input and autocomplete
  - cable sprites and connections
  - look-mode signal listing
- **Showcase:** a sheep farm that stops its feeder at 12 sheep, and a generator that only burns fuel when demand is high.
- **Depends on:** `add-animal-farming` (the counter, its first sensor) and the power grid patterns.
