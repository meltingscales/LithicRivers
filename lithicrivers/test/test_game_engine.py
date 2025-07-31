"""
Comprehensive unit tests for the modular game engine.
These tests demonstrate how to programmatically manipulate game state for testing.
"""

import unittest
from pathlib import Path
from tempfile import NamedTemporaryFile

from lithicrivers.game_engine import (
    GameEngine, GameState, MovePlayerAction, MineAction, SetTileAction,
    Inventory, Entity
)
from lithicrivers.game import Tiles, Items
from lithicrivers.model.vector import VectorN
from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST


class TestGameState(unittest.TestCase):
    """Test the GameState dataclass."""
    
    def test_game_state_creation(self):
        """Test creating a game state with default values."""
        state = GameState(player_position=VectorN(0, 0, 0))
        
        self.assertEqual(state.player_position, VectorN(0, 0, 0))
        self.assertEqual(state.player_health, 100)
        self.assertEqual(state.player_stamina, 100)
        self.assertEqual(len(state.world_data), 0)
        self.assertEqual(len(state.entities), 0)
    
    def test_game_state_copy(self):
        """Test that game state copying works correctly."""
        original_state = GameState(
            player_position=VectorN(1, 2, 3),
            player_health=50,
            player_stamina=75
        )
        
        copied_state = original_state.copy()
        
        # Test that the copy is equal but not the same object
        self.assertEqual(original_state, copied_state)
        self.assertIsNot(original_state, copied_state)
        
        # Test that modifying the copy doesn't affect the original
        copied_state.player_health = 25
        self.assertEqual(original_state.player_health, 50)
        self.assertEqual(copied_state.player_health, 25)


class TestGameActions(unittest.TestCase):
    """Test the various game actions."""
    
    def setUp(self):
        """Set up a test game state."""
        self.initial_state = GameState(
            player_position=VectorN(5, 5, 0),
            world_data={
                "5,5,0": Tiles.Dirt(),
                "6,5,0": Tiles.Tree(),
                "5,6,0": Tiles.Bedrock()
            }
        )
    
    def test_move_player_action(self):
        """Test moving the player in different directions."""
        # Test moving north
        action = MovePlayerAction(VEC_NORTH)
        new_state = action.apply(self.initial_state)
        
        self.assertEqual(new_state.player_position, VectorN(5, 4, 0))
        self.assertEqual(new_state.player_health, 100)  # Health unchanged
        
        # Test moving east
        action = MovePlayerAction(VEC_EAST)
        new_state = action.apply(self.initial_state)
        
        self.assertEqual(new_state.player_position, VectorN(6, 5, 0))
    
    def test_mine_action(self):
        """Test mining tiles at the player's position."""
        # Set up a state with a mineable tile at player position
        state = GameState(
            player_position=VectorN(5, 5, 0),
            world_data={
                "5,5,0": Tiles.Tree(),  # Tree should be mineable
                "6,5,0": Tiles.Dirt()
            }
        )
        
        action = MineAction()
        new_state = action.apply(state)
        
        # Check that the tile was replaced with empty
        self.assertEqual(new_state.world_data["5,5,0"], Tiles.Empty())
        
        # Check that other tiles are unchanged
        self.assertEqual(new_state.world_data["6,5,0"], Tiles.Dirt())
    
    def test_set_tile_action(self):
        """Test setting tiles at specific positions."""
        action = SetTileAction(VectorN(10, 10, 0), Tiles.Tree())
        new_state = action.apply(self.initial_state)
        
        # Check that the tile was set
        self.assertEqual(new_state.world_data["10,10,0"], Tiles.Tree())
        
        # Check that existing tiles are unchanged
        self.assertEqual(new_state.world_data["5,5,0"], Tiles.Dirt())
    
    def test_action_immutability(self):
        """Test that actions don't modify the original state."""
        action = MovePlayerAction(VEC_NORTH)
        new_state = action.apply(self.initial_state)
        
        # Original state should be unchanged
        self.assertEqual(self.initial_state.player_position, VectorN(5, 5, 0))
        
        # New state should be different
        self.assertEqual(new_state.player_position, VectorN(5, 4, 0))


class TestGameEngine(unittest.TestCase):
    """Test the main GameEngine class."""
    
    def setUp(self):
        """Set up a test game engine."""
        self.engine = GameEngine()
    
    def test_game_engine_initialization(self):
        """Test that the game engine initializes correctly."""
        self.assertIsNotNone(self.engine.state)
        self.assertEqual(len(self.engine.action_history), 0)
    
    def test_move_player(self):
        """Test moving the player through the engine."""
        initial_position = self.engine.state.player_position
        
        # Move north
        new_state = self.engine.move_player(VEC_NORTH)
        
        self.assertEqual(new_state.player_position, initial_position + VEC_NORTH)
        self.assertEqual(len(self.engine.action_history), 1)
        
        # Move east
        new_state = self.engine.move_player(VEC_EAST)
        
        self.assertEqual(new_state.player_position, initial_position + VEC_NORTH + VEC_EAST)
        self.assertEqual(len(self.engine.action_history), 2)
    
    def test_mine_at_player_position(self):
        """Test mining at the player's position."""
        # Set up a tile at player position
        player_pos = self.engine.state.player_position
        tile_key = f"{player_pos.x},{player_pos.y},{player_pos.z}"
        
        self.engine.state.world_data[tile_key] = Tiles.Tree()
        
        # Mine the tile
        new_state = self.engine.mine_at_player_position()
        
        # Check that the tile was replaced
        self.assertEqual(new_state.world_data[tile_key], Tiles.Empty())
    
    def test_set_tile(self):
        """Test setting tiles through the engine."""
        position = VectorN(10, 10, 0)
        tile = Tiles.Tree()
        
        new_state = self.engine.set_tile(position, tile)
        
        # Check that the tile was set
        tile_key = f"{position.x},{position.y},{position.z}"
        self.assertEqual(new_state.world_data[tile_key], tile)
    
    def test_get_tile_at_position(self):
        """Test getting tiles at specific positions."""
        position = VectorN(5, 5, 0)
        tile = Tiles.Tree()
        
        # Set a tile
        self.engine.set_tile(position, tile)
        
        # Get the tile
        retrieved_tile = self.engine.get_tile_at_position(position)
        self.assertEqual(retrieved_tile, tile)
        
        # Test getting non-existent tile
        non_existent_tile = self.engine.get_tile_at_position(VectorN(999, 999, 999))
        self.assertIsNone(non_existent_tile)
    
    def test_save_and_load_state(self):
        """Test saving and loading game state."""
        # Set up some game state
        self.engine.move_player(VEC_NORTH)
        self.engine.set_tile(VectorN(1, 1, 0), Tiles.Tree())
        
        # Save the state
        with NamedTemporaryFile(delete=False) as tmp_file:
            save_path = Path(tmp_file.name)
        
        try:
            self.engine.save_state(save_path)
            
            # Create a new engine and load the state
            new_engine = GameEngine()
            new_engine.load_state(save_path)
            
            # Check that the states are the same
            self.assertEqual(self.engine.state.player_position, new_engine.state.player_position)
            self.assertEqual(self.engine.state.world_data, new_engine.state.world_data)
            
        finally:
            # Clean up
            if save_path.exists():
                save_path.unlink()
    
    def test_reset_to_initial_state(self):
        """Test resetting the game to initial state."""
        # Make some changes
        self.engine.move_player(VEC_NORTH)
        self.engine.set_tile(VectorN(1, 1, 0), Tiles.Tree())
        
        # Reset
        self.engine.reset_to_initial_state()
        
        # Check that we're back to initial state
        self.assertEqual(len(self.engine.action_history), 0)
        self.assertEqual(len(self.engine.state.world_data), 0)


class TestInventory(unittest.TestCase):
    """Test the inventory system."""
    
    def test_inventory_creation(self):
        """Test creating an inventory."""
        inventory = Inventory()
        self.assertEqual(len(inventory.items), 0)
    
    def test_add_item(self):
        """Test adding items to inventory."""
        inventory = Inventory()
        rock = Items.Rock()
        
        inventory.add_item(rock)
        
        self.assertEqual(len(inventory.items), 1)
        self.assertEqual(inventory.items[0], rock)
    
    def test_count_items(self):
        """Test counting items in inventory."""
        inventory = Inventory()
        
        # Add some items
        inventory.add_item(Items.Rock())
        inventory.add_item(Items.Rock())
        inventory.add_item(Items.Stick())
        
        counts = inventory.count_items()
        
        self.assertEqual(counts["Rock"], 2)
        self.assertEqual(counts["Stick"], 1)
    
    def test_inventory_copy(self):
        """Test copying an inventory."""
        original = Inventory()
        original.add_item(Items.Rock())
        
        copied = original.copy()
        
        # Test that they're equal but not the same object
        self.assertEqual(len(original.items), len(copied.items))
        self.assertIsNot(original.items, copied.items)


class TestEntity(unittest.TestCase):
    """Test the entity system."""
    
    def test_entity_creation(self):
        """Test creating entities."""
        entity = Entity("Test Entity", VectorN(1, 2, 3))
        
        self.assertEqual(entity.name, "Test Entity")
        self.assertEqual(entity.position, VectorN(1, 2, 3))
        self.assertEqual(entity.health, 100)
        self.assertEqual(entity.stamina, 100)
    
    def test_entity_copy(self):
        """Test copying entities."""
        original = Entity("Test", VectorN(1, 1, 1))
        original.health = 50
        
        copied = original.copy()
        
        # Test that they're equal but not the same object
        self.assertEqual(original.name, copied.name)
        self.assertEqual(original.position, copied.position)
        self.assertEqual(original.health, copied.health)
        self.assertIsNot(original, copied)


class TestGameEngineIntegration(unittest.TestCase):
    """Integration tests for the game engine."""
    
    def test_complex_game_scenario(self):
        """Test a complex game scenario with multiple actions."""
        engine = GameEngine()
        
        # Start with player at origin for this test
        engine.state.player_position = VectorN(0, 0, 0)
        
        # Set up a small world
        engine.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
        engine.set_tile(VectorN(1, 0, 0), Tiles.Tree())
        engine.set_tile(VectorN(0, 1, 0), Tiles.Bedrock())
        
        # Move player to tree
        engine.move_player(VEC_EAST)
        
        # Mine the tree
        engine.mine_at_player_position()
        
        # Check final state
        self.assertEqual(engine.state.player_position, VectorN(1, 0, 0))
        self.assertEqual(engine.state.world_data["1,0,0"], Tiles.Empty())
        # We expect 3 actions: set_tile (3 times) + move_player + mine_at_player_position
        self.assertEqual(len(engine.action_history), 5)
    
    def test_action_history(self):
        """Test that action history is maintained correctly."""
        engine = GameEngine()
        
        # Perform some actions
        engine.move_player(VEC_NORTH)
        engine.move_player(VEC_EAST)
        engine.set_tile(VectorN(1, 1, 0), Tiles.Tree())
        
        # Check action history
        self.assertEqual(len(engine.action_history), 3)
        self.assertIsInstance(engine.action_history[0], MovePlayerAction)
        self.assertIsInstance(engine.action_history[1], MovePlayerAction)
        self.assertIsInstance(engine.action_history[2], SetTileAction)


if __name__ == '__main__':
    unittest.main() 