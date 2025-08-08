"""
Core game engine module that handles game state and logic independently of the UI.
This module is designed to be easily testable and manipulatable programmatically.
"""

from typing import TYPE_CHECKING, Optional, Protocol

import msgspec

from lithicrivers.model.model import Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_VIEWPORT

if TYPE_CHECKING:
    from lithicrivers.game.core import Tile, Tiles
    from lithicrivers.game.entities import Item


class GameState(msgspec.Struct, frozen=False):
    """Immutable game state that can be easily serialized and tested."""

    player_position: VectorN
    player_health: int = 100
    player_stamina: int = 100
    viewport: Viewport = msgspec.field(default_factory=lambda: DEFAULT_VIEWPORT)
    world_data: dict[str, "Tile"] = msgspec.field(default_factory=dict)
    entities: list["Entity"] = msgspec.field(default_factory=list)
    inventory: "Inventory" = msgspec.field(default_factory=lambda: Inventory())
    gametick: int = 0
    tick_rate: int = 200  # Higher tick rate for more granular timing

    def copy(self) -> "GameState":
        """Create a deep copy of the game state."""
        return GameState(
            player_position=self.player_position,
            player_health=self.player_health,
            player_stamina=self.player_stamina,
            viewport=self.viewport,
            world_data=self.world_data.copy(),
            entities=[entity.copy() for entity in self.entities],
            inventory=self.inventory.copy(),
            gametick=self.gametick,
            tick_rate=self.tick_rate,
        )

    def __eq__(self, other: object) -> bool:
        """Compare two game states for equality."""
        if not isinstance(other, GameState):
            return False
        return (
            self.player_position == other.player_position
            and self.player_health == other.player_health
            and self.player_stamina == other.player_stamina
            and self.viewport == other.viewport
            and self.world_data == other.world_data
            and self.entities == other.entities
            and self.inventory == other.inventory
            and self.gametick == other.gametick
            and self.tick_rate == other.tick_rate
        )


class GameAction(Protocol):
    """Protocol for game actions that can be applied to game state."""

    def apply(self, state: GameState) -> GameState:
        """Apply this action to the given game state and return the new state."""
        ...


class MovePlayerAction(msgspec.Struct, frozen=False):
    """Action to move the player in a specific direction."""

    direction: VectorN

    def apply(self, state: GameState) -> GameState:
        new_position = state.player_position + self.direction
        new_state = state.copy()
        new_state.player_position = new_position
        return new_state


class MineAction(msgspec.Struct, frozen=False):
    """Action to mine a tile at the player's position."""

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        tile_key = f"{state.player_position.x},{state.player_position.y},{state.player_position.z}"

        if tile_key in state.world_data:
            tile = state.world_data[tile_key]
            if hasattr(tile, "drops") and tile.drops:
                # Add dropped items to inventory
                dropped_item = tile.calc_drop()
                if dropped_item:
                    new_state.inventory.add_item(dropped_item)

            # Replace with empty tile

            new_state.world_data[tile_key] = Tiles.empty()

        return new_state


class SetTileAction(msgspec.Struct, frozen=False):
    """Action to set a tile at a specific position."""

    position: VectorN
    tile: "Tile"

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        tile_key = f"{self.position.x},{self.position.y},{self.position.z}"
        new_state.world_data[tile_key] = self.tile
        return new_state


class IncrementTickAction(msgspec.Struct, frozen=False):
    """Action to increment the game tick counter."""

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        new_state.gametick += 1
        return new_state


class SetTickRateAction(msgspec.Struct, frozen=False):
    """Action to set the game tick rate."""

    tick_rate: int

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        new_state.tick_rate = max(1, self.tick_rate)  # Ensure minimum tick rate of 1
        return new_state


class Inventory(msgspec.Struct, frozen=False):
    """Inventory system for the game."""

    @classmethod
    def create(cls, items: Optional[list["Item"]] = None):
        return cls(items=items)

    def add_item(self, item: "Item") -> None:
        """Add an item to the inventory."""
        self.items.append(item)

    def copy(self) -> "Inventory":
        """Create a copy of this inventory."""
        return Inventory(items=self.items.copy())

    def count_items(self) -> dict[str, int]:
        """Count items by name."""
        counts: dict[str, int] = {}
        for item in self.items:
            counts[item.name] = counts.get(item.name, 0) + 1
        return counts

    def __eq__(self, other: object) -> bool:
        """Compare two inventories for equality."""
        if not isinstance(other, Inventory):
            return False
        return self.items == other.items


class Entity(msgspec.Struct, frozen=False):
    """Base entity class."""

    @classmethod
    def create(cls, name: str, position: VectorN):
        instance = cls(name=name, position=position)
        instance.health = 100
        instance.stamina = 100
        return instance

    def copy(self) -> "Entity":
        """Create a copy of this entity."""
        copied = Entity.create(self.name, self.position)
        copied.health = self.health
        copied.stamina = self.stamina
        return copied

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, Entity):
            return False
        return (
            self.name == other.name
            and self.position == other.position
            and self.health == other.health
            and self.stamina == other.stamina
        )


# Import these here to avoid circular imports
