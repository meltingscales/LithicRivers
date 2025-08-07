"""
Test that structures are not generated on-demand when accessing tiles.
"""

import unittest
from lithicrivers.model.vector import VectorN
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestNoOnDemandStructures(OptimizedTestCase):
    """Test that structures are not generated on-demand."""
    
    def test_structures_only_generated_during_worldgen(self):
        """Test that accessing tiles doesn't trigger structure generation."""
        # Create a world with a small radius to avoid the forced structures
        world = self.get_world(seed=42)
        
        # Access a tile that should only have basic terrain
        # Use a position far from the forced structures (ship at 20,20,0 and dungeon at 50,50,-3)
        test_pos = VectorN(100, 100, 0)
        tile = world.get_tile(test_pos)
        
        # The tile should be basic terrain (dirt, tree, etc.) not a structure
        # Structures should only be generated during initial world generation
        self.assertIsNotNone(tile, "Tile should be generated")
        
        # Check that it's a basic terrain tile, not a structure tile
        basic_terrain_tiles = {"Dirt", "Tree", "Empty", "Cloud"}
        self.assertIn(tile.tileid, basic_terrain_tiles, 
                     f"Tile should be basic terrain, got {tile.tileid}")
        
        print(f"Accessed tile at {test_pos}: {tile.tileid}")
    
    def test_forced_structures_preserved(self):
        """Test that forced structures are preserved and not overwritten."""
        # Create world generator and generate a small world
        world = self.get_world(seed=42)        
        # Check that the forced ship is present
        ship_pos = VectorN(20, 20, 0)
        ship_pos_str = ship_pos.serialize()
        
        self.assertIn(ship_pos_str, world.data, "Forced ship should be present")
        ship_tile = world.data[ship_pos_str]
        
        # The ship should have iron scrap tiles (from the small_ship structure)
        # Let's check a few positions around the ship
        ship_tiles_found = 0
        for x in range(18, 23):
            for y in range(18, 23):
                for z in range(0, 2):
                    pos = VectorN(x, y, z)
                    pos_str = pos.serialize()
                    if pos_str in world.data:
                        tile = world.data[pos_str]
                        if tile.tileid == "Iron Scrap":
                            ship_tiles_found += 1
        
        self.assertGreater(ship_tiles_found, 0, "Ship should have iron scrap tiles")
        print(f"Found {ship_tiles_found} iron scrap tiles from ship")
        
        # Check that the forced dungeon is present
        dungeon_pos = VectorN(50, 50, -3)
        dungeon_tiles_found = 0
        for x in range(45, 55):
            for y in range(45, 55):
                for z in range(-5, -1):
                    pos = VectorN(x, y, z)
                    pos_str = pos.serialize()
                    if pos_str in world.data:
                        tile = world.data[pos_str]
                        if tile.tileid == "Iron Scrap":
                            dungeon_tiles_found += 1
        
        self.assertGreater(dungeon_tiles_found, 0, "Dungeon should have iron scrap tiles")
        print(f"Found {dungeon_tiles_found} iron scrap tiles from dungeon")


if __name__ == "__main__":
    unittest.main() 