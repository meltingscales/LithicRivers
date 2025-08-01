"""
Seeded world generation for deterministic world creation.
This module provides seeded randomness for reproducible world generation.
"""

import random
from dataclasses import dataclass
from typing import Any, Dict, List, Optional

from lithicrivers.game import Tile, Tiles
from lithicrivers.model.vector import VectorN


@dataclass
class WorldSeed:
    """Represents a world seed for deterministic generation."""

    seed: int

    def __str__(self) -> str:
        return f"WorldSeed({self.seed})"

    def __repr__(self) -> str:
        return str(self)


class SeededWorldGenerator:
    """
    World generator that uses seeded randomness for deterministic generation.
    Similar to Minecraft's world generation system.
    """

    def __init__(self, seed: Optional[int] = None):
        """
        Initialize the world generator with a seed.

        Args:
            seed: The seed for deterministic generation. If None, uses a random seed.
        """
        if seed is None:
            seed = random.randint(0, 2**32 - 1)

        self.seed = WorldSeed(seed)
        self.rng = random.Random(seed)

    def get_seed(self) -> WorldSeed:
        """Get the current world seed."""
        return self.seed

    def set_seed(self, seed: int) -> None:
        """Set a new seed for the generator."""
        self.seed = WorldSeed(seed)
        self.rng = random.Random(seed)

    def seeded_weighted_choice(
        self, weights: List[float], choices: List[Any], context: str = ""
    ) -> Any:
        """
        Make a weighted choice using the seeded random number generator.

        Args:
            weights: List of weights for each choice
            choices: List of choices to select from
            context: Optional context string for additional seeding

        Returns:
            The selected choice
        """
        if len(weights) != len(choices):
            raise ValueError(
                f"Weights={weights} and choices={choices} must be the same length!"
            )

        # Normalize weights to sum to 1
        total_weight = sum(weights)
        if total_weight == 0:
            raise ValueError("Weights cannot all be zero")

        normalized_weights = [w / total_weight for w in weights]

        # Create a context-specific RNG for deterministic results
        context_seed = hash((self.seed.seed, context))
        local_rng = random.Random(context_seed)

        # Use the context-specific RNG for weighted selection
        return local_rng.choices(choices, weights=normalized_weights, k=1)[0]

    def generate_tile_for_position(self, position: VectorN) -> Tile:
        """
        Generate a tile for a specific position using seeded randomness.

        Args:
            position: The position to generate a tile for

        Returns:
            The generated tile
        """
        # Use position-based seeding for consistent generation
        # This ensures the same position always generates the same tile
        position_seed = hash((self.seed.seed, position.x, position.y, position.z))
        local_rng = random.Random(position_seed)

        # Generate based on height (z-coordinate)
        if position.z > 0:
            # Sky level - always clouds
            return Tiles.Cloud()
        elif position.z < 0:
            # Underground - weighted choice between bedrock, dirt, and rare items
            weights = [1, 0.2, 0.05]
            choices = [Tiles.Bedrock(), Tiles.Dirt(), Tiles.Gold_Ore()]
            return local_rng.choices(choices, weights=weights, k=1)[0]
        else:
            # Surface level - weighted choice between trees, dirt, and rare items
            weights = [5, 100, 1]
            choices = [Tiles.Tree(), Tiles.Dirt(), Tiles.Gold_Ore()]
            return local_rng.choices(choices, weights=weights, k=1)[0]

    def generate_world_data(self, radius: VectorN) -> Dict[str, Tile]:
        """
        Generate world data for the given radius using seeded randomness.

        Args:
            radius: The radius of the world to generate

        Returns:
            Dictionary mapping position strings to tiles
        """
        world_data = {}

        for z in range(-radius.z, radius.z):
            for y in range(-radius.y, radius.y):
                for x in range(-radius.x, radius.x):
                    pos = VectorN(x, y, z)
                    tile = self.generate_tile_for_position(pos)
                    world_data[pos.serialize()] = tile

        return world_data


def create_world_generator(seed: Optional[int] = None) -> SeededWorldGenerator:
    """
    Create a new world generator with the given seed.

    Args:
        seed: The seed for deterministic generation. If None, uses a random seed.

    Returns:
        A new SeededWorldGenerator instance
    """
    return SeededWorldGenerator(seed)


def generate_world_with_seed(
    radius: VectorN, seed: Optional[int] = None
) -> Dict[str, Tile]:
    """
    Generate world data with a specific seed.

    Args:
        radius: The radius of the world to generate
        seed: The seed for deterministic generation. If None, uses a random seed.

    Returns:
        Dictionary mapping position strings to tiles
    """
    generator = create_world_generator(seed)
    return generator.generate_world_data(radius)
