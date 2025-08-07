"""
Test that underground facilities use iron scrap walls.
"""

import random
from lithicrivers.procedural_dungeon_generator import (
    ProceduralStructureGenerator,
    DungeonType
)
from lithicrivers.model.vector import VectorN
from lithicrivers.test.test_fixtures import OptimizedTestCase

class TestIronScrapWalls(OptimizedTestCase):
    """Test that underground facilities use iron scrap walls."""
    
    def test_underground_facility_uses_iron_scrap_walls(self):
        """Test that underground facilities generate iron scrap walls."""
        generator = ProceduralStructureGenerator()
        rng = random.Random(42)  # Fixed seed for testing
        
        # Generate an underground facility
        layout = generator._generate_underground_facility(15, 5, rng)
        
        # Count wall tiles (should be "W" symbol)
        wall_count = 0
        for pos, symbol in layout.items():
            if symbol == "W":
                wall_count += 1
        
        # Should have some wall tiles
        self.assertGreater(wall_count, 0, "Underground facility should have wall tiles")
        
        # Most tiles should be walls (since we initialize with walls)
        total_tiles = len(layout)
        wall_percentage = wall_count / total_tiles
        self.assertGreater(wall_percentage, 0.5, 
                          f"Most tiles should be walls, got {wall_percentage:.1%}")
        
        print(f"Underground facility has {wall_count}/{total_tiles} wall tiles ({wall_percentage:.1%})")
    
    def test_iron_scrap_walls_placed_in_world(self):
        """Test that iron scrap walls are actually placed in the world."""
        generator = ProceduralStructureGenerator()
        rng = random.Random(42)
        world_data = {}
        base_position = VectorN(0, 0, -3)
        
        # Generate dungeon
        success = generator.generate_dungeon(
            DungeonType.UNDERGROUND_FACILITY,
            base_position,
            world_data,
            rng,
            force_placement=True
        )
        
        self.assertTrue(success, "Dungeon should generate successfully")
        
        # Count iron scrap tiles in the world
        iron_scrap_count = 0
        for pos_str, tile in world_data.items():
            if tile.tileid == "Iron Scrap":
                iron_scrap_count += 1
        
        # Should have some iron scrap tiles
        self.assertGreater(iron_scrap_count, 0, 
                          "Underground facility should have iron scrap walls")
        
        print(f"Found {iron_scrap_count} iron scrap tiles in underground facility")