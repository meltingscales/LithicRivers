from typing import TYPE_CHECKING, Optional

from lithicrivers.game.tiles import Tile
from lithicrivers.model.vector import VectorN

if TYPE_CHECKING:
    from lithicrivers.game.entities import Fluid


class FluidManager:
    """
    Manages fluid physics and spreading across the world.
    All fluid calculations are deterministic based on world seed and tick.
    """

    def __init__(self, world: "World"):
        self.world = world
        self.fluids: dict[str, Fluid] = {}  # position_key -> Fluid
        self.flow_directions = [
            VectorN.from_args(0, 0, 1),  # Down (gravity) - deeper into earth
            VectorN.from_args(-1, 0, 0),  # Left
            VectorN.from_args(1, 0, 0),  # Right
            VectorN.from_args(0, -1, 0),  # North
            VectorN.from_args(0, 1, 0),  # South
        ]

    def add_fluid(self, fluid: "Fluid") -> None:
        """Add a fluid to the manager."""
        pos_key = self._get_position_key(fluid.position)
        if pos_key in self.fluids:
            # Merge with existing fluid
            existing = self.fluids[pos_key]
            if existing.fluid_type == fluid.fluid_type:
                total_amount = existing.amount + fluid.amount

                # Disturb settled fluid when new fluid is added
                if existing.settled and fluid.amount > 0:
                    existing.settled = False
                    existing.stability_counter = 0
                    # Also disturb neighboring fluids that might be affected
                    self._disturb_neighboring_fluids(existing.position)

                if total_amount <= existing.max_amount:
                    existing.amount = total_amount
                else:
                    # Overflow - create new fluid entities
                    existing.amount = existing.max_amount
                    overflow = total_amount - existing.max_amount
                    if overflow > 0:
                        # Create overflow fluid that will spread
                        from lithicrivers.game.entities import Fluid

                        overflow_fluid = Fluid(
                            fluid.fluid_type, fluid.position, overflow, fluid.viscosity
                        )
                        self.fluids[pos_key] = overflow_fluid
        else:
            self.fluids[pos_key] = fluid

    def remove_fluid(self, position: VectorN) -> None:
        """Remove fluid from a position."""
        pos_key = self._get_position_key(position)
        if pos_key in self.fluids:
            del self.fluids[pos_key]

    def get_fluid(self, position: VectorN) -> Optional["Fluid"]:
        """Get fluid at a position."""
        pos_key = self._get_position_key(position)
        return self.fluids.get(pos_key)

    def process_fluids(self, gametick: int) -> None:
        """Process all fluid physics for a given tick."""
        # Use deterministic randomness based on world seed and tick
        import random

        random.seed(f"fluids_{self.world.seed}_{gametick}")

        # Create a copy of fluids to avoid modifying during iteration
        fluids_to_process = list(self.fluids.items())

        for pos_key, fluid in fluids_to_process:
            if fluid.amount <= 0:
                # Remove empty fluids
                del self.fluids[pos_key]
                continue

            # Update settlement status
            self._update_fluid_settlement(fluid, gametick)

            # Only process unsettled fluids that should spread
            if not fluid.settled and fluid.amount >= fluid.spread_threshold:
                self._spread_fluid(fluid, gametick)

    def _spread_fluid(self, fluid: "Fluid", gametick: int) -> None:
        """Spread fluid to adjacent tiles based on physics. Fully deterministic and not random at all."""

        # Early exit if already settled (redundant check for safety)
        if fluid.settled:
            return

        # Calculate how much to spread
        spread_amount = fluid.amount - fluid.spread_threshold
        original_amount = fluid.amount
        fluid.amount = fluid.spread_threshold

        # Try to spread to adjacent positions
        valid_targets = []

        for direction in self.flow_directions:
            target_pos = fluid.position + direction
            target_tile = self.world.get_tile(target_pos)

            # Check if target position can hold fluid
            if self._can_hold_fluid(target_pos, target_tile):
                valid_targets.append(target_pos)

        if not valid_targets:
            return

        # Distribute fluid among valid targets using integer division
        amount_per_target = spread_amount // len(valid_targets)
        remainder = spread_amount % len(valid_targets)

        for i, target_pos in enumerate(valid_targets):
            # Distribute remainder to first few targets to ensure exact distribution
            target_amount = amount_per_target + (1 if i < remainder else 0)
            if (
                target_amount > 0
            ):  # Only create fluid if there's actually amount to spread
                from lithicrivers.game.entities import Fluid

                new_fluid = Fluid(
                    fluid.fluid_type, target_pos, target_amount, fluid.viscosity
                )
                self.add_fluid(new_fluid)

        # Track spreading activity for settlement optimization
        if len(valid_targets) > 0:
            # Fluid actually spread - reset settlement tracking
            fluid.last_spread_tick = gametick
            fluid.stability_counter = 0
            fluid.settled = False
        else:
            # Fluid wanted to spread but couldn't - this counts as stability
            if fluid.last_spread_tick != gametick:
                fluid.stability_counter += 1

    def _update_fluid_settlement(self, fluid: "Fluid", gametick: int) -> None:
        """Update the settlement status of a fluid based on its stability."""
        # Skip if already settled
        if fluid.settled:
            return

        # Check if fluid is below spread threshold (naturally stable)
        if fluid.amount < fluid.spread_threshold:
            fluid.stability_counter += 1

        # Mark as settled if stable for enough ticks
        if fluid.stability_counter >= fluid.settlement_threshold:
            fluid.settled = True
            # Optional: Log settlement for debugging
            # print(f"Fluid {fluid.fluid_type} at {fluid.position} settled after {fluid.stability_counter} stable ticks")

    def _disturb_neighboring_fluids(self, position: VectorN) -> None:
        """Disturb neighboring fluids when a fluid at the given position changes."""
        for direction in self.flow_directions:
            neighbor_pos = position + direction
            neighbor_key = self._get_position_key(neighbor_pos)
            if neighbor_key in self.fluids:
                neighbor_fluid = self.fluids[neighbor_key]
                if neighbor_fluid.settled:
                    neighbor_fluid.settled = False
                    neighbor_fluid.stability_counter = 0

    def _can_hold_fluid(self, position: VectorN, tile: Optional[Tile]) -> bool:
        """Check if a position can hold fluid."""
        if tile is None:
            return True  # Empty space can hold fluid

        # Check if tile is solid (can't hold fluid)
        solid_tiles = ["bedrock", "door"]
        if tile.tileid.lower() in solid_tiles:
            return False

        # Check if there's already too much fluid at this position
        existing_fluid = self.get_fluid(position)
        if existing_fluid and existing_fluid.amount >= existing_fluid.max_amount:
            return False

        return True

    def _get_position_key(self, pos: VectorN) -> str:
        """Get a string key for a position."""
        return pos.serialize()

    def get_all_fluids(self) -> list["Fluid"]:
        """Get all fluids in the manager."""
        return list(self.fluids.values())

    def clear(self) -> None:
        """Clear all fluids."""
        self.fluids.clear()
