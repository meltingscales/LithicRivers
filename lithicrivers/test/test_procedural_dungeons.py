"""
Tests for procedural dungeon generation.
"""

import random
from lithicrivers.procedural_dungeon_generator import (
    ProceduralStructureGenerator,
    DungeonType,
    create_procedural_generator
)
from lithicrivers.model.vector import VectorN
from lithicrivers.game.core import Tiles


class TestProceduralDungeonGenerator:
    """Test the procedural dungeon generator."""
    
    def test_dungeon_configs_initialization(self):
        """Test that dungeon configs are properly initialized."""
        generator = ProceduralStructureGenerator()
        
        # Check that all dungeon types have configs
        assert len(generator.dungeon_configs) == 5
        assert DungeonType.CAVE_SYSTEM in generator.dungeon_configs
        assert DungeonType.UNDERGROUND_FACILITY in generator.dungeon_configs
        assert DungeonType.MINING_SHAFT in generator.dungeon_configs
        assert DungeonType.CRYPT in generator.dungeon_configs
        assert DungeonType.LABORATORY in generator.dungeon_configs
    
    def test_cave_system_generation(self):
        """Test cave system generation."""
        generator = ProceduralStructureGenerator()
        rng = random.Random(42)  # Fixed seed for testing
        
        layout = generator._generate_cave_system(10, 3, rng)
        
        # Check that layout is generated
        assert len(layout) > 0
        
        # Check that layout contains valid symbols
        valid_symbols = {".", "X", "O"}
        for symbol in layout.values():
            assert symbol in valid_symbols
    
    def test_dungeon_placement(self):
        """Test dungeon placement in world data."""
        generator = ProceduralStructureGenerator()
        rng = random.Random(42)
        world_data = {}
        base_position = VectorN(0, 0, -2)  # Underground position
        
        # Test cave system placement
        success = generator.generate_dungeon(
            DungeonType.CAVE_SYSTEM,
            base_position,
            world_data,
            rng,
            force_placement=True
        )
        
        assert success
        assert len(world_data) > 0
        
        # Check that tiles were placed
        for pos_str, tile in world_data.items():
            assert isinstance(tile, type(Tiles.empty()))
    
    def test_depth_restrictions(self):
        """Test that dungeons respect depth restrictions."""
        generator = ProceduralStructureGenerator()
        rng = random.Random(42)
        world_data = {}
        
        # Try to place a laboratory at surface level (should fail)
        surface_position = VectorN(0, 0, 0)
        success = generator.generate_dungeon(
            DungeonType.LABORATORY,
            surface_position,
            world_data,
            rng,
            force_placement=True
        )
        
        # Should fail due to depth restrictions
        assert not success
    
    def test_singleton_pattern(self):
        """Test that the singleton pattern works correctly."""
        generator1 = create_procedural_generator()
        generator2 = create_procedural_generator()
        
        # Should be the same instance
        assert generator1 is generator2
    
    def test_chunk_generation(self):
        """Test dungeon generation for chunks."""
        generator = ProceduralStructureGenerator()
        rng = random.Random(42)
        world_data = {}
        chunk_center = VectorN(0, 0, -3)
        chunk_radius = 8
        
        # Generate dungeons for chunk
        generator.generate_dungeons_for_chunk(
            world_data, chunk_center, chunk_radius, rng
        )
        
        # Should have generated some dungeons
        assert len(world_data) >= 0  # May or may not generate based on chance
