#!/usr/bin/env python3
"""
Demo script for procedural dungeon generation.
Shows how different dungeon types are generated and placed in the world.
"""

from lithicrivers.game.rng import SimpleRNG
import sys
from pathlib import Path

# Add the project root to the path
sys.path.insert(0, str(Path(__file__).parent.parent))

from lithicrivers.model.vector import VectorN
from lithicrivers.procedural_dungeon_generator import (
    DungeonType,
    ProceduralStructureGenerator,
)


def demo_dungeon_generation():
    """Demonstrate procedural dungeon generation."""
    print("🏰 Procedural Dungeon Generation Demo")
    print("=" * 50)

    # Create generator
    generator = ProceduralStructureGenerator()

    # Show available dungeon types
    print("\n📋 Available Dungeon Types:")
    for dungeon_type in DungeonType:
        config = generator.dungeon_configs[dungeon_type]
        print(f"  • {dungeon_type.value}")
        print(f"    - Size: {config.min_size}-{config.max_size}")
        print(f"    - Depth: {config.depth_range[0]} to {config.depth_range[1]}")
        print(f"    - Chance: {config.generation_chance:.1%}")
        print(f"    - Rooms: {config.room_count_range[0]}-{config.room_count_range[1]}")
        print()

    # Demo each dungeon type
    print("\n🎲 Generating Sample Dungeons:")
    print("-" * 50)

    rng = SimpleRNG.create(42)  # Fixed seed for demo
    world_data = {}

    for dungeon_type in DungeonType:
        print(f"\n🏛️  {dungeon_type.value.upper()}:")

        # Generate at appropriate depth
        config = generator.dungeon_configs[dungeon_type]
        depth = (config.depth_range[0] + config.depth_range[1]) // 2
        base_position = VectorN.create(0, 0, depth)

        # Generate dungeon
        success = generator.generate_dungeon(
            dungeon_type, base_position, world_data, rng, force_placement=True
        )

        if success:
            # Count different tile types
            tile_counts = {}
            for pos_str, tile in world_data.items():
                tile_type = tile.tileid
                tile_counts[tile_type] = tile_counts.get(tile_type, 0) + 1

            print("  ✅ Generated successfully")
            print(f"  📍 Position: {base_position}")
            print(f"  🧱 Tiles placed: {len(world_data)}")
            print("  📊 Tile breakdown:")
            for tile_type, count in tile_counts.items():
                print(f"    - {tile_type}: {count}")
        else:
            print("  ❌ Failed to generate")

    print(f"\n🎯 Total tiles placed: {len(world_data)}")

    # Demo chunk generation
    print("\n🗺️  Chunk Generation Demo:")
    print("-" * 30)

    chunk_world_data = {}
    chunk_center = VectorN.create(100, 100, -3)
    chunk_radius = 8

    generator.generate_dungeons_for_chunk(
        chunk_world_data, chunk_center, chunk_radius, rng
    )

    print(f"  📍 Chunk center: {chunk_center}")
    print(f"  📏 Chunk radius: {chunk_radius}")
    print(f"  🧱 Tiles generated: {len(chunk_world_data)}")

    # Show some sample tiles
    if chunk_world_data:
        print("  📋 Sample tiles:")
        sample_count = 0
        for pos_str, tile in chunk_world_data.items():
            if sample_count < 5:
                print(f"    - {pos_str}: {tile.tileid}")
                sample_count += 1
            else:
                print(f"    ... and {len(chunk_world_data) - 5} more")
                break


def demo_deterministic_generation():
    """Demonstrate that generation is deterministic."""
    print("\n🎲 Deterministic Generation Demo:")
    print("=" * 40)

    generator = ProceduralStructureGenerator()
    dungeon_type = DungeonType.CAVE_SYSTEM
    base_position = VectorN.create(0, 0, -2)

    # Generate with same seed multiple times
    for i in range(3):
        rng = SimpleRNG.create(12345)  # Same seed
        world_data = {}

        success = generator.generate_dungeon(
            dungeon_type, base_position, world_data, rng, force_placement=True
        )

        print(f"  Run {i + 1}: {len(world_data)} tiles generated")

        # Show first few tiles to verify consistency
        if world_data:
            sample_tiles = list(world_data.items())[:3]
            for pos_str, tile in sample_tiles:
                print(f"    {pos_str}: {tile.tileid}")
        print()


if __name__ == "__main__":
    demo_dungeon_generation()
    demo_deterministic_generation()

    print("\n✅ Demo completed!")
    print("\n💡 Key Features:")
    print("  • 5 different dungeon types with unique characteristics")
    print("  • Deterministic generation based on world seed")
    print("  • Depth-based placement restrictions")
    print("  • Configurable generation chances and parameters")
    print("  • Integration with existing structure system")
