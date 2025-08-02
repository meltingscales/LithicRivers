"""
Seeded world generation for deterministic world creation.
This module provides seeded randomness for reproducible world generation.
"""

import random
from dataclasses import dataclass
from typing import Any, Optional

from lithicrivers.logging_config import get_logger

logger = get_logger(__name__)
from lithicrivers.game import Tile, Tiles
from lithicrivers.model.vector import VectorN
from lithicrivers.structure_generator import create_structure_manager


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
        self.structure_manager = create_structure_manager()

    def get_seed(self) -> WorldSeed:
        """Get the current world seed."""
        return self.seed

    def set_seed(self, seed: int) -> None:
        """Set a new seed for the generator."""
        self.seed = WorldSeed(seed)
        self.rng = random.Random(seed)

    def seeded_weighted_choice(
        self, weights: list[float], choices: list[Any], context: str = ""
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
            return Tiles.cloud()
        elif position.z < 0:
            # Underground - weighted choice between bedrock, dirt, and rare items
            weights = [1, 0.2, 0.05]
            choices = [Tiles.bedrock(), Tiles.dirt(), Tiles.gold_ore()]
            return local_rng.choices(choices, weights=weights, k=1)[0]
        else:
            # Surface level - weighted choice between trees, dirt, and rare items
            weights = [5, 100, 1]
            choices = [Tiles.tree(), Tiles.dirt(), Tiles.gold_ore()]
            return local_rng.choices(choices, weights=weights, k=1)[0]

    def generate_world_data(self, radius: VectorN) -> dict[str, Tile]:
        """
        Generate world data for the given radius using seeded randomness.

        Args:
            radius: The radius of the world to generate

        Returns:
            Dictionary mapping position strings to tiles
        """
        world_data = {}

        # First, generate basic terrain
        for z in range(-radius.z, radius.z):
            for y in range(-radius.y, radius.y):
                for x in range(-radius.x, radius.x):
                    pos = VectorN(x, y, z)
                    tile = self.generate_tile_for_position(pos)
                    world_data[pos.serialize()] = tile

        # Then, generate structures
        self._generate_structures(world_data, radius)

        return world_data

    def _generate_structures(self, world_data: dict[str, Tile], radius: VectorN) -> None:
        """
        Generate structures in the world using seeded randomness.
        
        Args:
            world_data: World data dictionary to modify
            radius: The radius of the world
        """
        # Force a ship to spawn very close to the player's actual position (25,25,0)
        # Place ship at 20,20,0 which should be clearly visible from position (25,25,0)
        forced_ship_pos = VectorN(20, 20, 0)
        logger.info(f"FORCING SHIP TO SPAWN AT {forced_ship_pos}")  # Debug output
        self.structure_manager.place_structure("small_ship", world_data, forced_ship_pos, self.rng)
        
        # Generate structures in chunks for better distribution
        chunk_size = 16  # 16x16 chunks
        
        for chunk_z in range(-radius.z // chunk_size, radius.z // chunk_size + 1):
            for chunk_y in range(-radius.y // chunk_size, radius.y // chunk_size + 1):
                for chunk_x in range(-radius.x // chunk_size, radius.x // chunk_size + 1):
                    chunk_center = VectorN(
                        chunk_x * chunk_size,
                        chunk_y * chunk_size,
                        chunk_z * chunk_size
                    )
                    
                    # Use chunk-specific seeding for deterministic structure placement
                    chunk_seed = hash((self.seed.seed, chunk_x, chunk_y, chunk_z))
                    chunk_rng = random.Random(chunk_seed)
                    
                    self.structure_manager.generate_structures_for_chunk(
                        world_data, chunk_center, chunk_size // 2, chunk_rng
                    )


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
) -> dict[str, Tile]:
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
