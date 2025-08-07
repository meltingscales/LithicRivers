"""Tile system for LithicRivers.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import random
from typing import TYPE_CHECKING, Optional

from lithicrivers.game.interfaces import SpriteRenderable
from lithicrivers.model.generictype import T

if TYPE_CHECKING:
    from lithicrivers.game.entities import Item


def weighted_choice(weights: list[float], choices: list[T]) -> T:
    if len(weights) != len(choices):
        import logging

        ve = ValueError(
            f"Weights={weights} and choices={choices} for {weighted_choice.__name__}() must be the same length!"
        )
        logging.error(ve)
        raise ve

    # Normalize weights to sum to 1
    total_weight = sum(weights)
    if total_weight == 0:
        raise ValueError("Weights cannot all be zero")

    normalized_weights = [w / total_weight for w in weights]

    # Use random.choices for weighted selection
    return random.choices(choices, weights=normalized_weights, k=1)[0]


def weighted_choice_dict(dict_weight: dict[float, T]) -> T:
    weights = []
    choices = []
    for k, v in dict_weight.items():
        weights.append(k)
        choices.append(v)
    return weighted_choice(weights, choices)


class Tile(SpriteRenderable):
    def __init__(
        self,
        tileid: str,
        desc: Optional[str] = None,
        sprite_sheet: Optional[list[str]] = None,
        drops: Optional[dict[float, "Item"]] = None,
    ):
        # Load sprites from external data if not provided
        if sprite_sheet is None:
            from lithicrivers.sprite_loader import get_sprite_loader

            sprite_loader = get_sprite_loader()
            sprite_data = sprite_loader.load_sprite(
                tileid.lower().replace(" ", "_"), "tiles"
            )
            sprite_sheet = sprite_data.sprites

        SpriteRenderable.__init__(self, sprite_sheet)
        self.tileid = tileid
        self.description = desc
        self.drops = drops

    def __str__(self) -> str:
        return f"<Tile '{self.tileid}': [{self.render_sprite(1)}]>"

    def __eq__(self, other: object) -> bool:
        if other is None:
            return False
        return self.tileid == other.tileid

    def __hash__(self) -> int:
        return hash(self.tileid)

    def calc_drop(self) -> "Item":
        return weighted_choice_dict(self.drops)

    def calc_tree_drops(self) -> list["Item"]:
        """
        Special drop calculation for trees that guarantees 1-3 acorns.
        90% chance of exactly 1 acorn, 10% chance of 2-3 acorns.
        """
        # Determine number of acorns (1-3, with 90% chance of 1)
        num_acorns = 1 if random.random() < 0.9 else random.randint(2, 3)

        # Create list of items to return
        items = []

        # Add guaranteed acorns
        from lithicrivers.game.entities import Items

        for _ in range(num_acorns):
            items.append(Items.acorn())

        # Add other possible drops (stick, log) with original probabilities
        if random.random() < 0.5:
            items.append(Items.stick())
        if random.random() < 0.3:
            items.append(Items.log())

        return items


class Tiles:
    """
    A bunch of default tiles.
    """

    @staticmethod
    def dirt() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Dirt",
            drops={0.99: Items.rock(), 0.01: Items.gold_nugget()},
        )

    @staticmethod
    def tree() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Tree",
            drops={0.50: Items.stick(), 0.30: Items.log(), 0.20: Items.acorn()},
        )

    @staticmethod
    def gold_ore() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Gold Ore",
            drops={0.9: Items.gold_nugget(), 0.1: Items.diamond()},
        )

    @staticmethod
    def cloud() -> "Tile":
        return Tile("Cloud")

    @staticmethod
    def bedrock() -> "Tile":
        return Tile("Bedrock")

    @staticmethod
    def empty() -> "Tile":
        return Tile("Empty")

    @staticmethod
    def iron_scrap() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Iron Scrap",
            drops={0.8: Items.iron_scrap(), 0.2: Items.gold_nugget()},
        )

    @staticmethod
    def bone_block() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Bone Block",
            drops={0.7: Items.rock(), 0.3: Items.gold_nugget()},
        )

    @staticmethod
    def door() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Door",
            drops={0.5: Items.rock()},
        )

    @staticmethod
    def scrap_electronics() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "Scrap Electronics",
            drops={0.6: Items.scrap_electronics(), 0.4: Items.gold_nugget()},
        )

    @staticmethod
    def treasure() -> "Tile":
        from lithicrivers.game.entities import Items

        return Tile(
            "buried_treasure",
            drops={0.3: Items.gold_nugget(), 0.7: Items.diamond()},
        )


class TilePalette:
    """
    Efficient storage for tile types using integer IDs.
    Similar to Minecraft's block palette system.
    """

    def __init__(self) -> None:
        self.tile_to_id = {}  # tileid -> int
        self.id_to_tile = {}  # int -> Tile
        self.next_id = 0
        self._empty_tile = None

    def get_empty_tile(self) -> Tile:
        """Get the empty tile (ID 0) - used for ungenerated areas."""
        if self._empty_tile is None:
            self._empty_tile = Tiles.empty()
        return self._empty_tile

    def get_id(self, tile: Tile) -> int:
        """Get the integer ID for a tile, creating it if needed."""
        if tile is None:
            return 0  # Empty tile is always ID 0

        # Use tileid as the key instead of the Tile object to avoid hash issues
        tile_key = tile.tileid if hasattr(tile, "tileid") else str(tile)

        if tile_key not in self.tile_to_id:
            self.tile_to_id[tile_key] = self.next_id
            self.id_to_tile[self.next_id] = tile
            self.next_id += 1
        return self.tile_to_id[tile_key]

    def get_tile(self, tile_id: int) -> Tile:
        """Get the tile for a given integer ID."""
        if tile_id == 0:
            return self.get_empty_tile()
        return self.id_to_tile.get(tile_id, self.get_empty_tile())

    def __len__(self) -> int:
        """Number of unique tile types in the palette."""
        return len(self.tile_to_id)
