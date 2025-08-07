"""
Core game engine module that handles game state and logic independently of the UI.
This module is designed to be easily testable and manipulatable programmatically.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Optional, Protocol

from lithicrivers.model.model import Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_PLAYER_POSITION, DEFAULT_VIEWPORT

if TYPE_CHECKING:
    from lithicrivers.game import Item, Tile


@dataclass
class GameState:
    """Immutable game state that can be easily serialized and tested."""

    player_position: VectorN
    player_health: int = 100
    player_stamina: int = 100
    viewport: Viewport = field(default_factory=lambda: DEFAULT_VIEWPORT)
    world_data: dict[str, "Tile"] = field(default_factory=dict)
    entities: list["Entity"] = field(default_factory=list)
    inventory: "Inventory" = field(default_factory=lambda: Inventory())
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


@dataclass
class MovePlayerAction:
    """Action to move the player in a specific direction."""

    direction: VectorN

    def apply(self, state: GameState) -> GameState:
        new_position = state.player_position + self.direction
        new_state = state.copy()
        new_state.player_position = new_position
        return new_state


@dataclass
class MineAction:
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
            from lithicrivers.game import Tiles

            new_state.world_data[tile_key] = Tiles.empty()

        return new_state


@dataclass
class SetTileAction:
    """Action to set a tile at a specific position."""

    position: VectorN
    tile: "Tile"

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        tile_key = f"{self.position.x},{self.position.y},{self.position.z}"
        new_state.world_data[tile_key] = self.tile
        return new_state


@dataclass
class IncrementTickAction:
    """Action to increment the game tick counter."""

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        new_state.gametick += 1
        return new_state


@dataclass
class SetTickRateAction:
    """Action to set the game tick rate."""

    tick_rate: int

    def apply(self, state: GameState) -> GameState:
        new_state = state.copy()
        new_state.tick_rate = max(1, self.tick_rate)  # Ensure minimum tick rate of 1
        return new_state

class Inventory:
    """Inventory system for the game."""

    def __init__(self, items: Optional[list["Item"]] = None):
        self.items = items or []

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


class Entity:
    """Base entity class."""

    def __init__(self, name: str, position: VectorN):
        self.name = name
        self.position = position
        self.health = 100
        self.stamina = 100

    def copy(self) -> "Entity":
        """Create a copy of this entity."""
        copied = Entity(self.name, self.position)
        copied.health = self.health
        copied.stamina = self.stamina
        return copied


# Import these here to avoid circular imports
