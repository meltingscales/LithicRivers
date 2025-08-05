"""
Test that the forced underground facility spawns correctly.
"""

import unittest
from lithicrivers.worldgen import create_world_generator
from lithicrivers.model.vector import VectorN


class TestForcedDungeon(unittest.TestCase):
    """Test that the forced dungeon spawns correctly."""
    
    def test_forced_dungeon_spawns(self):
        """Test that a forced underground facility spawns at the expected location."""
        # Create world generator with fixed seed
        generator = create_world_generator(42)
        
        # Generate a world large enough to include the forced dungeon
        # The forced dungeon is at (50, 50, -3), so we need radius at least 55
        radius = VectorN(55, 55, 5)  # Large enough to include the dungeon
        world_data = generator.generate_world_data(radius)
        
        # Check that the forced dungeon position has been modified
        # The forced dungeon should be at (50, 50, -3)
        forced_pos = VectorN(50, 50, -3)
        forced_pos_str = forced_pos.serialize()
        

        
        # The dungeon should have placed tiles around this position
        # Let's check a few positions around the forced dungeon location
        dungeon_tiles_found = 0
        
        # Check a small area around the forced dungeon position
        for x in range(45, 55):
            for y in range(45, 55):
                for z in range(-5, -1):
                    pos = VectorN(x, y, z)
                    pos_str = pos.serialize()
                    if pos_str in world_data:
                        tile = world_data[pos_str]
                        # Count non-bedrock tiles (dungeon tiles)
                        if tile.tileid != "Bedrock":
                            dungeon_tiles_found += 1
        
        # We should find some dungeon tiles (empty spaces, treasure, etc.)
        self.assertGreater(dungeon_tiles_found, 0, 
                          "No dungeon tiles found around forced dungeon position")
        

    
    def test_forced_dungeon_deterministic(self):
        """Test that the forced dungeon spawns deterministically with the same seed."""
        # Generate world with same seed twice
        generator1 = create_world_generator(12345)
        generator2 = create_world_generator(12345)
        
        radius = VectorN(55, 55, 5)
        world_data1 = generator1.generate_world_data(radius)
        world_data2 = generator2.generate_world_data(radius)
        
        # The worlds should be identical
        self.assertEqual(world_data1, world_data2, 
                        "Worlds generated with same seed should be identical")
        
        # Count dungeon tiles in both worlds
        dungeon_tiles1 = 0
        dungeon_tiles2 = 0
        
        for x in range(45, 55):
            for y in range(45, 55):
                for z in range(-5, -1):
                    pos = VectorN(x, y, z)
                    pos_str = pos.serialize()
                    
                    if pos_str in world_data1:
                        tile1 = world_data1[pos_str]
                        if tile1.tileid != "Bedrock":
                            dungeon_tiles1 += 1
                    
                    if pos_str in world_data2:
                        tile2 = world_data2[pos_str]
                        if tile2.tileid != "Bedrock":
                            dungeon_tiles2 += 1
        
        self.assertEqual(dungeon_tiles1, dungeon_tiles2,
                        "Dungeon tile counts should be identical with same seed")


if __name__ == "__main__":
    unittest.main() 