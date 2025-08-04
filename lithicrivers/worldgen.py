"""
Seeded world generation for deterministic world creation.
This module provides seeded randomness for reproducible world generation.
"""

import random
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
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


class ChunkCache:
    """Thread-safe cache for pre-generated chunks."""

    def __init__(self, max_chunks: int = 100):
        self.max_chunks = max_chunks
        self.cache: dict[tuple[int, int, int], dict[str, Tile]] = {}
        self.lock = threading.RLock()

    def get_chunk_key(self, pos: VectorN) -> tuple[int, int, int]:
        """Get chunk coordinates for a position."""
        chunk_size = 16
        chunk_x = pos.x // chunk_size
        chunk_y = pos.y // chunk_size
        chunk_z = pos.z // chunk_size
        return (chunk_x, chunk_y, chunk_z)

    def get_tile(self, pos: VectorN) -> Optional[Tile]:
        """Get a tile from cache if available."""
        with self.lock:
            chunk_key = self.get_chunk_key(pos)
            if chunk_key in self.cache:
                pos_str = pos.serialize()
                return self.cache[chunk_key].get(pos_str)
        return None

    def set_tile(self, pos: VectorN, tile: Tile) -> None:
        """Set a tile in cache."""
        with self.lock:
            chunk_key = self.get_chunk_key(pos)
            if chunk_key not in self.cache:
                self.cache[chunk_key] = {}
            self.cache[chunk_key][pos.serialize()] = tile

            # Evict oldest chunks if cache is full
            if len(self.cache) > self.max_chunks:
                oldest_key = next(iter(self.cache))
                del self.cache[oldest_key]

    def pre_generate_chunk(
        self,
        chunk_x: int,
        chunk_y: int,
        chunk_z: int,
        generator: "SeededWorldGenerator",
    ) -> None:
        """Pre-generate a chunk in background."""
        chunk_size = 16
        start_x = chunk_x * chunk_size
        start_y = chunk_y * chunk_size
        start_z = chunk_z * chunk_size

        chunk_data = {}
        for x in range(start_x, start_x + chunk_size):
            for y in range(start_y, start_y + chunk_size):
                for z in range(start_z, start_z + chunk_size):
                    pos = VectorN(x, y, z)
                    tile = generator.generate_tile_for_position(pos)
                    chunk_data[pos.serialize()] = tile

        with self.lock:
            self.cache[(chunk_x, chunk_y, chunk_z)] = chunk_data


class PerlinNoise:
    """
    A simple seeded perlin noise implementation for terrain generation.
    This provides smooth, continuous noise that's deterministic based on seed.
    """

    def __init__(self, seed: int):
        """Initialize perlin noise with a seed."""
        self.seed = seed
        self.rng = random.Random(seed)
        # Generate a permutation table for noise
        self.permutation = list(range(256))
        self.rng.shuffle(self.permutation)
        self.permutation *= 2  # Duplicate for wrapping

    def _fade(self, t: float) -> float:
        """Fade function for smooth interpolation."""
        return t * t * t * (t * (t * 6 - 15) + 10)

    def _lerp(self, t: float, a: float, b: float) -> float:
        """Linear interpolation."""
        return a + t * (b - a)

    def _grad_1d(self, hash_val: int, x: float) -> float:
        """1D gradient function."""
        return (hash_val & 1) * x

    def noise_1d(self, x: float) -> float:
        """Generate 1D perlin noise."""
        # Find the unit grid cell containing the point
        xi = int(x) & 255
        xf = x - int(x)

        # Compute fade curves for each of x
        u = self._fade(xf)

        # Hash coordinates of the 2 square corners
        A = self.permutation[xi]
        AA = self.permutation[A]

        # Add blended results from 2 corners of 1D cube
        return self._lerp(u, self._grad_1d(AA, xf), self._grad_1d(AA, xf - 1))

    def noise_2d(self, x: float, y: float) -> float:
        """Generate 2D perlin noise."""
        # Find the unit grid cell containing the point
        xi = int(x) & 255
        yi = int(y) & 255
        xf = x - int(x)
        yf = y - int(y)

        # Compute fade curves for each of x, y
        u = self._fade(xf)
        v = self._fade(yf)

        # Hash coordinates of the 4 square corners
        A = self.permutation[xi] + yi
        AA = self.permutation[A]
        AB = self.permutation[A + 1]
        B = self.permutation[xi + 1] + yi
        BA = self.permutation[B]
        BB = self.permutation[B + 1]

        # Add blended results from 4 corners of 2D cube
        return self._lerp(
            v,
            self._lerp(u, self._grad_2d(AA, xf, yf), self._grad_2d(BA, xf - 1, yf)),
            self._lerp(
                u, self._grad_2d(AB, xf, yf - 1), self._grad_2d(BB, xf - 1, yf - 1)
            ),
        )

    def _grad_2d(self, hash_val: int, x: float, y: float) -> float:
        """2D gradient function."""
        # Convert low 4 bits of hash code into 12 simple gradient directions
        h = hash_val & 15
        u = x if h < 8 else y
        v = y if h < 4 else (x if h == 12 or h == 14 else 0)
        return (u if (h & 1) == 0 else -u) + (v if (h & 2) == 0 else -v)

    def octave_noise_2d(
        self,
        x: float,
        y: float,
        octaves: int = 4,
        persistence: float = 0.5,
        scale: float = 1.0,
    ) -> float:
        """
        Generate octave noise (fractal noise) for more natural terrain.

        Args:
            x, y: Coordinates
            octaves: Number of noise layers to combine
            persistence: How much each octave contributes (0.5 = half amplitude each octave)
            scale: Overall scale of the noise
        """
        total = 0
        frequency = scale
        amplitude = 1.0
        max_value = 0

        for _ in range(octaves):
            total += self.noise_2d(x * frequency, y * frequency) * amplitude
            max_value += amplitude
            amplitude *= persistence
            frequency *= 2

        return total / max_value


class SeededWorldGenerator:
    """
    World generator that uses seeded randomness for deterministic generation.
    Similar to Minecraft's world generation system.
    """

    def __init__(self, seed: int):
        """
        Initialize the world generator with a seed.

        Args:
            seed: The seed for deterministic generation.
        """

        self.seed = WorldSeed(seed)
        self.rng = random.Random(seed)
        self.perlin = PerlinNoise(seed)
        self.structure_manager = create_structure_manager()
        self.chunk_cache = ChunkCache()

    def get_seed(self) -> WorldSeed:
        """Get the current world seed."""
        return self.seed

    def set_seed(self, seed: int) -> None:
        """Set a new seed for the generator."""
        self.seed = WorldSeed(seed)
        self.rng = random.Random(seed)
        self.perlin = PerlinNoise(seed)

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
        Generate a tile for a specific position using perlin noise and seeded randomness.

        Args:
            position: The position to generate a tile for

        Returns:
            The generated tile
        """
        # Check cache first
        cached_tile = self.chunk_cache.get_tile(position)
        if cached_tile is not None:
            return cached_tile

        # Generate tile if not in cache
        tile = self._generate_tile_at_position(position)

        # Cache the tile
        self.chunk_cache.set_tile(position, tile)

        return tile

    def _generate_tile_at_position(self, position: VectorN) -> Tile:
        """Internal method to generate a tile at a specific position."""
        # Use position-based context for consistent generation

        # Generate terrain using perlin noise
        if position.z > 0:
            # Sky level - always clouds
            return Tiles.cloud()
        elif position.z < 0:
            # Underground - use perlin noise for cave systems and ore distribution
            # Scale noise to create larger cave systems
            # Include Z coordinate in noise generation for depth variation
            cave_noise = self.perlin.octave_noise_2d(
                position.x * 0.1, position.y * 0.1, octaves=3, scale=1.0
            )
            ore_noise = self.perlin.octave_noise_2d(
                position.x * 0.05, position.y * 0.05, octaves=2, scale=0.5
            )
            # Add depth-based variation using Z coordinate
            # Create a separate noise function for depth to avoid vertical striping
            depth_perlin = PerlinNoise(
                self.seed.seed + abs(position.z) * 1000
            )  # Different seed for each Z level
            depth_noise = depth_perlin.octave_noise_2d(
                position.x * 0.05, position.y * 0.05, octaves=3, scale=1.0
            )

            # Create cave systems with depth variation
            # Combine cave noise with depth noise for more varied underground terrain
            # Use a more balanced approach to reduce striping
            combined_cave_noise = cave_noise * 0.7 + depth_noise * 0.3

            # Add direct Z-based variation for more dramatic depth differences
            # Deeper levels have more caves and less ore, but with reasonable limits
            depth_factor = min(
                abs(position.z) * 0.02, 0.3
            )  # Cap the depth factor to prevent massive voids
            adjusted_cave_threshold = (
                0.15 - depth_factor
            )  # Lower threshold for deeper levels, but less aggressive
            adjusted_ore_threshold = (
                0.2 + depth_factor
            )  # Higher threshold for deeper levels

            if combined_cave_noise > adjusted_cave_threshold:
                return Tiles.empty()  # Cave
            elif ore_noise > adjusted_ore_threshold:
                return Tiles.gold_ore()  # Ore vein
            else:
                return Tiles.bedrock()  # Solid rock
        else:
            # Surface level - use perlin noise for terrain features
            # Scale noise to create larger terrain features
            terrain_noise = self.perlin.octave_noise_2d(
                position.x * 0.02, position.y * 0.02, octaves=4, scale=0.1
            )
            tree_noise = self.perlin.octave_noise_2d(
                position.x * 0.1, position.y * 0.1, octaves=2, scale=1.0
            )

            # Create varied terrain
            if terrain_noise > 0.1:
                # Higher elevation - more trees
                if tree_noise > 0.0:
                    return Tiles.tree()
                else:
                    return Tiles.dirt()
            elif terrain_noise < -0.05:
                # Lower elevation - sparse vegetation
                if tree_noise > 0.1:
                    return Tiles.tree()
                else:
                    return Tiles.dirt()
            else:
                # Medium elevation - balanced
                if tree_noise > 0.0:
                    return Tiles.tree()
                else:
                    return Tiles.dirt()

    def pre_generate_chunks_around(self, center_pos: VectorN, radius: int = 2) -> None:
        """
        Pre-generate chunks around a center position in background threads.

        Args:
            center_pos: The center position to generate chunks around
            radius: Number of chunks to generate in each direction
        """
        chunk_size = 16
        center_chunk_x = center_pos.x // chunk_size
        center_chunk_y = center_pos.y // chunk_size
        center_chunk_z = center_pos.z // chunk_size

        # Create thread pool for background generation
        from lithicrivers.settings import MAX_CPU_THREADS

        with ThreadPoolExecutor(max_workers=MAX_CPU_THREADS) as executor:
            futures = []

            # Generate chunks in a cube around the center
            for chunk_x in range(center_chunk_x - radius, center_chunk_x + radius + 1):
                for chunk_y in range(
                    center_chunk_y - radius, center_chunk_y + radius + 1
                ):
                    for chunk_z in range(
                        center_chunk_z - radius, center_chunk_z + radius + 1
                    ):
                        future = executor.submit(
                            self.chunk_cache.pre_generate_chunk,
                            chunk_x,
                            chunk_y,
                            chunk_z,
                            self,
                        )
                        futures.append(future)

            # Wait for all chunks to be generated
            for future in as_completed(futures):
                try:
                    future.result()
                except Exception as e:
                    logger.warning(f"Failed to pre-generate chunk: {e}")

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

    def _generate_structures(
        self, world_data: dict[str, Tile], radius: VectorN
    ) -> None:
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
        self.structure_manager.place_structure(
            "small_ship", world_data, forced_ship_pos, self.rng
        )

        # Generate structures in chunks for better distribution
        chunk_size = 16  # 16x16 chunks

        for chunk_z in range(-radius.z // chunk_size, radius.z // chunk_size + 1):
            for chunk_y in range(-radius.y // chunk_size, radius.y // chunk_size + 1):
                for chunk_x in range(
                    -radius.x // chunk_size, radius.x // chunk_size + 1
                ):
                    chunk_center = VectorN(
                        chunk_x * chunk_size, chunk_y * chunk_size, chunk_z * chunk_size
                    )

                    # Use chunk-specific seeding for deterministic structure placement
                    chunk_seed = hash((self.seed.seed, chunk_x, chunk_y, chunk_z))
                    chunk_rng = random.Random(chunk_seed)

                    self.structure_manager.generate_structures_for_chunk(
                        world_data, chunk_center, chunk_size // 2, chunk_rng
                    )


def create_world_generator(seed: int) -> SeededWorldGenerator:
    """
    Create a new world generator with the given seed.

    Args:
        seed: The seed for deterministic generation.

    Returns:
        A new SeededWorldGenerator instance
    """
    return SeededWorldGenerator(seed)


def generate_world_with_seed(radius: VectorN, seed: int) -> dict[str, Tile]:
    """
    Generate world data with a specific seed.

    Args:
        radius: The radius of the world to generate
        seed: The seed for deterministic generation.

    Returns:
        Dictionary mapping position strings to tiles
    """
    generator = create_world_generator(seed)
    return generator.generate_world_data(radius)
