"""
Core game engine module that handles game state and logic independently of the UI.
This module is designed to be easily testable and manipulatable programmatically.
"""

from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Optional, Protocol

from lithicrivers.model.model import RenderedData, Viewport
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
        )

    def __eq__(self, other):
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


class GameEngine:
    """
    Core game engine that manages game state and applies actions.
    This class is designed to be easily testable and manipulatable.
    """

    def __init__(
        self, initial_state: Optional[GameState] = None, seed: Optional[int] = None
    ):
        self.state = initial_state or GameState(player_position=DEFAULT_PLAYER_POSITION)
        self.seed = seed
        self.action_history: list[GameAction] = []

    def reset_to_initial_state(self) -> None:
        """Reset the game to its initial state."""
        self.state = GameState(player_position=DEFAULT_PLAYER_POSITION)
        self.action_history.clear()

    def apply_action(self, action: GameAction) -> GameState:
        """Apply an action to the current game state."""
        self.state = action.apply(self.state)
        self.action_history.append(action)
        return self.state

    def get_tile_at_position(self, position: VectorN) -> Optional["Tile"]:
        """Get the tile at a specific position."""
        tile_key = f"{position.x},{position.y},{position.z}"
        return self.state.world_data.get(tile_key)

    def get_tile_at_player_feet(self) -> Optional["Tile"]:
        """Get the tile at the player's current position."""
        return self.get_tile_at_position(self.state.player_position)

    def move_player(self, direction: VectorN) -> GameState:
        """Move the player in the specified direction."""
        action = MovePlayerAction(direction)
        return self.apply_action(action)

    def mine_at_player_position(self) -> GameState:
        """Mine the tile at the player's current position."""
        action = MineAction()
        return self.apply_action(action)

    def set_tile(self, position: VectorN, tile: "Tile") -> GameState:
        """Set a tile at the specified position."""
        action = SetTileAction(position, tile)
        return self.apply_action(action)

    def save_state(self, filepath: Path) -> Path:
        """Save the current game state to a file."""
        import pickle

        with open(filepath, "wb") as f:
            pickle.dump(self.state, f)
        return filepath

    def load_state(self, filepath: Path) -> None:
        """Load a game state from a file."""
        import pickle

        with open(filepath, "rb") as f:
            self.state = pickle.load(f)


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
        counts = {}
        for item in self.items:
            counts[item.name] = counts.get(item.name, 0) + 1
        return counts

    def __eq__(self, other):
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
