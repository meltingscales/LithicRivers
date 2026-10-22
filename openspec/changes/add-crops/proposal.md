# Proposal

## Why

Trees are the only thing that grows. A factory game wants farms of several kinds feeding several chains. The user's idea (IDEAS-INBOX.md): wheat and potatoes for biofuel or ethanol, with different byproduct ratios, and carrots to feed and breed animals (`add-animal-farming`).

## What Changes

- **Three crops:**

  | Crop | Main use |
  |---|---|
  | Wheat | Ferments into ethanol; the straw left over burns as weak fuel |
  | Potatoes | Ferment into more ethanol per item than wheat, but leave mash instead of straw |
  | Carrots | Feed and breed animals (`add-animal-farming`); eaten raw by nothing yet |

- **Growing:**
  - Seeds (or seed potatoes, or carrots) are planted on dirt or grass, like acorns.
  - Each crop grows through four stages over a few game minutes, shown by its glyph.
  - Crops grow only on the surface and only by day once `add-day-night-cycle` lands. Underground, they need a grow lamp (later).
  - A ripe crop harvests into its produce plus 1 to 2 seeds, seeded by position.
- **Finding seeds:** wild wheat, potato and carrot plants generate in surface grasslands, and harvesting them gives the first seeds.
- **Automation:**
  - The auto-planter plants any seed in its inventory, not just acorns.
  - The auto-harvester harvests ripe crops as well as grown trees.
- **Processing:** a **fermenter** (a powered machine) turns wheat or potatoes into ethanol (a fluid, through `add-ore-spouts`' fluids) plus straw or mash.
  - Ethanol burns in the gas generator (`add-advanced-processing`), or fills fuel canisters for power cells (`add-power-cells`).
  - Straw is weak furnace fuel, and mash feeds animals.

## Capabilities

### New Capabilities
- `crops`: Planting, growing and harvesting wheat, potatoes and carrots; wild crops; and fermenting into ethanol.

### Modified Capabilities
- `tree-automation`: the auto-planter plants any seed, and the auto-harvester harvests ripe crops.

## Impact

- **Core:**
  - `plants.rs` gains crop kinds with growth stages (like the sapling's growth timer), and harvest drops through a seeded table.
  - Wild crop tiles are placed in surface worldgen. This changes the surface, so it goes through a separate surface feature step, and the surface hash test is updated deliberately.
  - Seed items, produce and the fermenter.
- **Client:** crop glyphs per stage, sprites, and the fermenter's window.
- **Showcase:** an automated wheat farm into a fermenter.
- **Depends on:** `add-ore-spouts` (fluids) for ethanol. Without it, the fermenter outputs bottled ethanol items instead, and switches to fluid when pipes exist.
