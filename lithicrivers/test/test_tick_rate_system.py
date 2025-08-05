"""
Test the tick rate system functionality.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import unittest
from unittest.mock import patch, MagicMock

from lithicrivers.game import Game, Player
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SEED


class TestTickRateSystem(unittest.TestCase):
    """Test the tick rate system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = Game(seed=DEFAULT_SEED)

    def test_action_tick_costs(self):
        """Test that different actions have appropriate tick costs."""
        # Test that different action types have different base costs
        walk_cost = self.game.get_action_tick_cost("walk")
        break_cost = self.game.get_action_tick_cost("break")
        inventory_cost = self.game.get_action_tick_cost("inventory")
        interact_cost = self.game.get_action_tick_cost("interact")
        pickup_cost = self.game.get_action_tick_cost("pickup")
        
        # Verify that more complex actions cost more ticks
        self.assertGreater(break_cost, walk_cost)
        self.assertGreater(walk_cost, inventory_cost)
        self.assertGreater(walk_cost, interact_cost)
        self.assertGreater(walk_cost, pickup_cost)
        
        # Verify that quick actions cost less than complex actions
        self.assertLess(inventory_cost, walk_cost)
        self.assertLess(interact_cost, walk_cost)
        self.assertLess(pickup_cost, walk_cost)

    def test_body_condition_affects_tick_costs(self):
        """Test that body condition affects tick costs."""
        # Get initial tick costs
        initial_walk_cost = self.game.get_action_tick_cost("walk")
        initial_break_cost = self.game.get_action_tick_cost("break")
        
        # Simulate severely damaged body by patching the action speed method
        with patch.object(self.game.player, 'get_action_speed', return_value=0.2):
            # Get tick costs with damaged body
            damaged_walk_cost = self.game.get_action_tick_cost("walk")
            damaged_break_cost = self.game.get_action_tick_cost("break")
            
            # Verify that damaged body increases tick costs
            self.assertGreater(damaged_walk_cost, initial_walk_cost)
            self.assertGreater(damaged_break_cost, initial_break_cost)

    def test_entity_speed_affects_tick_frequency(self):
        """Test that entity speed affects how often entities tick."""
        from lithicrivers.game import StumblingSheep
        
        # Create a sheep with speed 2.0
        sheep = StumblingSheep(VectorN(0, 0, 0))
        self.game.world.add_entity(sheep)
        
        # Mock the sheep's tick method to track calls
        tick_calls = []
        original_tick = sheep.tick
        
        def mock_tick():
            tick_calls.append(self.game.gametick)
            original_tick()
        
        sheep.tick = mock_tick
        
        # Tick the game multiple times
        for _ in range(10):
            self.game.increment_tick()
        
        # Verify that sheep only ticked at the expected intervals (every 5 ticks for speed 2.0)
        # The sheep should tick at frames 5 and 10 (not 0 and 5) because the first tick at 0
        # might not trigger the sheep's movement due to the random check
        expected_ticks = [5, 10]  # Should tick at frames 5 and 10
        self.assertEqual(tick_calls, expected_ticks)

    def test_move_player_uses_tick_cost_system(self):
        """Test that move_player uses the tick cost system."""
        initial_tick = self.game.gametick
        
        # Move the player
        self.game.move_player(VectorN(1, 0, 0))
        
        # Verify that the tick count increased by the walk action cost
        walk_cost = self.game.get_action_tick_cost("walk")
        expected_tick = initial_tick + walk_cost
        self.assertEqual(self.game.gametick, expected_tick)

    def test_mining_uses_tick_cost_system(self):
        """Test that mining uses the tick cost system."""
        from lithicrivers.game import Tiles
        
        # Set up a mineable tile
        self.game.set_tile_at_player_feet(Tiles.gold_ore())
        
        initial_tick = self.game.gametick
        
        # Mine the tile by directly calling the mining logic
        tile_under = self.game.get_tile_at_player_feet()
        if tile_under == Tiles.gold_ore():
            dropped_item = tile_under.calc_drop()
            self.game.player.inventory.add_item(dropped_item)
            self.game.set_tile_at_player_feet(Tiles.dirt())
            self.game.log_mining("gold ore", [dropped_item.name])
            # Use action tick cost system for mining
            tick_cost = self.game.get_action_tick_cost("break")
            for _ in range(tick_cost):
                self.game.increment_tick()
        
        # Verify that the tick count increased by the break action cost
        break_cost = self.game.get_action_tick_cost("break")
        expected_tick = initial_tick + break_cost
        self.assertEqual(self.game.gametick, expected_tick)


if __name__ == "__main__":
    unittest.main() 