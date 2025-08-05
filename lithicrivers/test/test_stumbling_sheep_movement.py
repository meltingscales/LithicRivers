"""
Test the StumblingSheep movement functionality.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import unittest
from unittest.mock import patch, MagicMock

from lithicrivers.game import Game, StumblingSheep, World
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SEED


class TestStumblingSheepMovement(unittest.TestCase):
    """Test the StumblingSheep movement system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = Game(seed=DEFAULT_SEED)
        self.world = self.game.world

    def test_stumbling_sheep_creation(self):
        """Test that StumblingSheep is created correctly."""
        sheep = StumblingSheep(VectorN(0, 0, 0))
        self.assertEqual(sheep.name, "Stumbling Sheep")
        self.assertEqual(sheep.position, VectorN(0, 0, 0))
        self.assertIsNone(sheep.world)  # Should be None initially

    def test_stumbling_sheep_world_assignment(self):
        """Test that StumblingSheep gets world reference during tick."""
        sheep = StumblingSheep(VectorN(0, 0, 0))
        self.world.add_entity(sheep)
        
        # Process ticks to assign world reference
        self.game.process_entity_ticks()
        
        # Check that the sheep has a world reference
        self.assertEqual(sheep.world, self.world)

    def test_stumbling_sheep_movement_with_world(self):
        """Test that StumblingSheep moves correctly using world's move_entity."""
        sheep = StumblingSheep(VectorN(0, 0, 0))
        self.world.add_entity(sheep)
        
        # Verify sheep is at initial position
        entities_at_start = self.world.get_entities(VectorN(0, 0, 0))
        self.assertIn(sheep, entities_at_start)
        
        # Mock random to always return True (so sheep always moves)
        with patch('random.random', return_value=0.1):  # Always < 0.5
            with patch('random.choice', return_value=VectorN(1, 0, 0)):  # Move east
                # Process tick
                sheep.world = self.world  # Set world reference
                sheep.tick()
        
        # Verify sheep moved to new position
        entities_at_new = self.world.get_entities(VectorN(1, 0, 0))
        self.assertIn(sheep, entities_at_new)
        
        # Verify sheep is no longer at old position
        entities_at_old = self.world.get_entities(VectorN(0, 0, 0))
        self.assertNotIn(sheep, entities_at_old)

    def test_stumbling_sheep_movement_without_world(self):
        """Test that StumblingSheep falls back to direct movement when no world."""
        sheep = StumblingSheep(VectorN(0, 0, 0))
        sheep.world = None  # No world reference
        
        # Mock random to always return True (so sheep always moves)
        with patch('random.random', return_value=0.1):  # Always < 0.5
            with patch('random.choice', return_value=VectorN(1, 0, 0)):  # Move east
                # Process tick
                sheep.tick()
        
        # Verify sheep moved using direct movement
        self.assertEqual(sheep.position, VectorN(1, 0, 0))

    def test_stumbling_sheep_in_game_tick_system(self):
        """Test that StumblingSheep moves during game tick processing."""
        # Create sheep and add to world
        sheep = StumblingSheep(VectorN(0, 0, 0))
        self.world.add_entity(sheep)
        
        # Verify sheep is at initial position
        entities_at_start = self.world.get_entities(VectorN(0, 0, 0))
        self.assertIn(sheep, entities_at_start)
        
        # Mock random to always return True (so sheep always moves)
        with patch('random.random', return_value=0.1):  # Always < 0.5
            with patch('random.choice', return_value=VectorN(1, 0, 0)):  # Move east
                # Process game tick (which processes entity ticks)
                self.game.increment_tick()
        
        # Verify sheep moved to new position
        entities_at_new = self.world.get_entities(VectorN(1, 0, 0))
        self.assertIn(sheep, entities_at_new)
        
        # Verify sheep is no longer at old position
        entities_at_old = self.world.get_entities(VectorN(0, 0, 0))
        self.assertNotIn(sheep, entities_at_old)


if __name__ == "__main__":
    unittest.main() 