"""
Test the debug repair items functionality.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import unittest

from lithicrivers.game.game import Game, Items
from lithicrivers.settings import DEFAULT_SEED
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestDebugRepairItems(OptimizedTestCase):
    """Test the debug repair items system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = self.get_game(seed=DEFAULT_SEED)

    def test_debug_repair_items_are_added(self):
        """Test that debug repair items are added to the player's inventory."""
        player = self.game.player
        
        # Check that the player has items in their inventory
        self.assertGreater(len(player.inventory.itemsdata), 1)  # More than just the cookie
        
        # Count the items
        item_counts = player.inventory.count_items()
        
        # Should have iron_scrap and scrap_electronics items
        self.assertIn("Iron Scrap", item_counts)
        self.assertIn("Scrap Electronics", item_counts)
        
        # Should have reasonable amounts (at least 2 of each for double the requirements)
        self.assertGreaterEqual(item_counts["Iron Scrap"], 2)
        self.assertGreaterEqual(item_counts["Scrap Electronics"], 2)

    def test_repair_requirements_calculation(self):
        """Test that repair requirements are calculated correctly."""
        from lithicrivers.model.body import BodyPartType
        
        player = self.game.player
        requirements = player.body.get_repair_requirements()
        
        # Should have requirements for damaged and missing parts
        self.assertIn(BodyPartType.RIGHT_ARM, requirements)  # Missing arm
        self.assertIn(BodyPartType.RIGHT_LEG, requirements)  # Damaged leg
        self.assertIn(BodyPartType.TORSO, requirements)      # Damaged torso
        
        # Should not have requirements for functional parts
        self.assertNotIn(BodyPartType.HEAD, requirements)    # Functional head
        self.assertNotIn(BodyPartType.LEFT_ARM, requirements)  # Functional left arm
        self.assertNotIn(BodyPartType.LEFT_LEG, requirements)  # Functional left leg

    def test_item_creation(self):
        """Test that the correct items are created."""
        # Test iron_scrap item
        iron_scrap = Items.iron_scrap()
        self.assertEqual(iron_scrap.name, "Iron Scrap")
        # Now expects multi-scale sprites from external files
        self.assertIsInstance(iron_scrap.sprite_sheet, list)
        self.assertGreater(len(iron_scrap.sprite_sheet), 0)
        
        # Test scrap_electronics item
        scrap_electronics = Items.scrap_electronics()
        self.assertEqual(scrap_electronics.name, "Scrap Electronics")
        # Now expects multi-scale sprites from external files
        self.assertIsInstance(scrap_electronics.sprite_sheet, list)
        self.assertGreater(len(scrap_electronics.sprite_sheet), 0)

    def test_tile_drops_updated(self):
        """Test that tile drops are updated to include the correct items."""
        from lithicrivers.game.game import Tiles
        
        # Test iron_scrap tile drops
        iron_scrap_tile = Tiles.iron_scrap()
        self.assertIsNotNone(iron_scrap_tile.drops)
        
        # Check that iron_scrap tile can drop iron_scrap items
        dropped_item = iron_scrap_tile.calc_drop()
        self.assertIn(dropped_item.name, ["Iron Scrap", "Gold Nugget"])
        
        # Test scrap_electronics tile drops
        scrap_electronics_tile = Tiles.scrap_electronics()
        self.assertIsNotNone(scrap_electronics_tile.drops)
        
        # Check that scrap_electronics tile can drop scrap_electronics items
        dropped_item = scrap_electronics_tile.calc_drop()
        self.assertIn(dropped_item.name, ["Scrap Electronics", "Gold Nugget"])


if __name__ == "__main__":
    unittest.main() 