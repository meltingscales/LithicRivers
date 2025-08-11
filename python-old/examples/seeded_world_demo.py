#!/usr/bin/env python3
"""
Demonstration of seeded world generation in LithicRivers.
This script shows how to create deterministic worlds using seeds.
"""

from lithicrivers.worldgen import SeededWorldGenerator, generate_world_with_seed
from lithicrivers.model.vector import VectorN
from lithicrivers.game.game import World, Game


def demo_seeded_world_generation():
    """Demonstrate seeded world generation."""
    print("=== Seeded World Generation Demo ===\n")

    # Create a small world radius for demonstration
    radius = VectorN(3, 3, 1)

    print("1. Creating worlds with the same seed (should be identical):")
    world1 = generate_world_with_seed(radius, seed=42)
    world2 = generate_world_with_seed(radius, seed=42)

    print(f"   World 1 has {len(world1)} tiles")
    print(f"   World 2 has {len(world2)} tiles")
    print(f"   Worlds are identical: {world1 == world2}")
    print()

    print("2. Creating worlds with different seeds (should be different):")
    world3 = generate_world_with_seed(radius, seed=12345)
    print(f"   World 3 has {len(world3)} tiles")
    print(f"   World 3 is different from World 1: {world1 != world3}")
    print()

    print("3. Using the World class with seeds:")
    world_a = World(seed=42)
    world_b = World(seed=42)
    world_c = World(seed=12345)

    print(
        f"   World A and B are identical: {world_a.data.tile_data == world_b.data.tile_data}"
    )
    print(
        f"   World A and C are different: {world_a.data.tile_data != world_c.data.tile_data}"
    )
    print()

    print("4. Using the Game class with seeds:")
    game1 = Game(seed=42)
    game2 = Game(seed=42)
    game3 = Game(seed=12345)

    print(
        f"   Game 1 and 2 have identical worlds: {game1.world.data.tile_data == game2.world.data.tile_data}"
    )
    print(
        f"   Game 1 and 3 have different worlds: {game1.world.data.tile_data != game3.world.data.tile_data}"
    )
    print()

    print("5. Using the SeededWorldGenerator class:")
    generator = SeededWorldGenerator(seed=42)
    print(f"   Generator seed: {generator.get_seed()}")

    # Generate a tile for a specific position
    pos = VectorN(10, 20, 0)
    tile = generator.generate_tile_for_position(pos)
    print(f"   Tile at position {pos}: {tile.tileid}")

    # Generate the same tile again (should be identical)
    tile2 = generator.generate_tile_for_position(pos)
    print(f"   Same tile generated again: {tile == tile2}")
    print()

    print("=== Demo Complete ===")


def demo_world_comparison():
    """Demonstrate comparing worlds with different seeds."""
    print("\n=== World Comparison Demo ===\n")

    radius = VectorN(2, 2, 1)

    # Generate several worlds with different seeds
    seeds = [42, 123, 456, 789, 999]
    worlds = {}

    for seed in seeds:
        worlds[seed] = generate_world_with_seed(radius, seed=seed)
        print(f"World with seed {seed}: {len(worlds[seed])} tiles")

    print("\nComparing worlds:")
    for i, seed1 in enumerate(seeds):
        for seed2 in seeds[i + 1 :]:
            are_identical = worlds[seed1] == worlds[seed2]
            print(
                f"  Seed {seed1} vs Seed {seed2}: {'Identical' if are_identical else 'Different'}"
            )


if __name__ == "__main__":
    demo_seeded_world_generation()
    demo_world_comparison()
