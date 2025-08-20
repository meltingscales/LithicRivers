"""
Simplified TUI testing framework for LithicRivers.
This module provides basic testing for TUI components with minimal mocking.
"""

import os

# Set TESTING environment BEFORE importing any game modules
os.environ["TESTING"] = "1"

import unittest
from unittest.mock import Mock

from lithicrivers.constants import (
    VEC_DOWN,
    VEC_EAST,
    VEC_NORTH,
    VEC_NORTHEAST,
    VEC_NORTHWEST,
    VEC_SOUTH,
    VEC_SOUTHEAST,
    VEC_SOUTHWEST,
    VEC_UP,
    VEC_WEST,
)
from lithicrivers.game.core import Tiles
from lithicrivers.model.vector import VectorN
from lithicrivers.test.test_fixtures import OptimizedTestCase
from lithicrivers.ui import KEYMAP, InputHandler


class SimpleTUITestCase(OptimizedTestCase):
    """Simple TUI test case with minimal setup."""

    def setUp(self):
        """Set up common test fixtures."""
        self.game = self.get_game()

    def create_keyboard_event(self, key_code: int):
        """Create a mock keyboard event."""
        event = Mock()
        event.key_code = key_code
        return event


class TestInputHandlerSimple(SimpleTUITestCase):
    """Test the InputHandler with simple mocking."""

    def test_movement_input_handling(self):
        """Test that movement inputs are handled correctly."""
        # Test each movement direction with numpad keys
        movement_tests = [
            (ord("8"), VEC_NORTH),  # Numpad 8
            (ord("2"), VEC_SOUTH),  # Numpad 2
            (ord("4"), VEC_WEST),  # Numpad 4
            (ord("6"), VEC_EAST),  # Numpad 6
            (ord("7"), VEC_NORTHWEST),  # Numpad 7
            (ord("9"), VEC_NORTHEAST),  # Numpad 9
            (ord("1"), VEC_SOUTHWEST),  # Numpad 1
            (ord("3"), VEC_SOUTHEAST),  # Numpad 3
        ]

        for key_code, expected_direction in movement_tests:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = InputHandler.handle_movement(event)
                self.assertEqual(result, expected_direction)

    def test_character_movement_input_handling(self):
        """Test that character-based movement inputs still work for up/down."""
        # Test character-based movement for up/down (Q/E)
        character_movement_tests = [
            (ord("q"), VEC_UP),  # Q for up
            (ord("e"), VEC_DOWN),  # E for down
        ]

        for key_code, expected_direction in character_movement_tests:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = InputHandler.handle_movement(event)
                self.assertEqual(result, expected_direction)

    def test_invalid_movement_input(self):
        """Test that invalid movement inputs return None."""
        invalid_keys = [
            ord("x"),
            ord("y"),
            ord("z"),
            ord("0"),
        ]  # 0 is not used

        for key_code in invalid_keys:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = InputHandler.handle_movement(event)
                self.assertIsNone(result)

    def test_mining_input_handling(self):
        """Test that mining inputs are handled correctly."""
        # Set up a mineable tile at player's position
        player_pos = self.game.player.position
        self.game.world.set_tile(player_pos, Tiles.gold_ore())

        # Create mining event (use 'u' key which is mapped to MINE)
        event = self.create_keyboard_event(ord("u"))
        world_map = Mock()
        InputHandler.handle_mining(event, self.game, world_map)

        # Check that tile was replaced with dirt
        tile = self.game.get_tile_at_player_feet()
        self.assertEqual(tile.tileid, Tiles.dirt().tileid)

    def test_mining_unmineable_tiles(self):
        """Test that mining unmineable tiles doesn't change them."""
        # Set up an unmineable tile at player's position
        player_pos = self.game.player.position
        self.game.world.get_tile(player_pos)
        self.game.world.set_tile(player_pos, Tiles.dirt())

        # Try to mine the tile
        event = self.create_keyboard_event(ord("u"))
        world_map = Mock()
        InputHandler.handle_mining(event, self.game, world_map)

        # Check that tile is still dirt (unmineable)
        tile = self.game.get_tile_at_player_feet()
        self.assertEqual(tile.tileid, Tiles.dirt().tileid)

    def test_mining_trees_implemented(self):
        """Test that mining trees works correctly."""
        # Set up a tree at player's position
        player_pos = self.game.player.position
        self.game.world.set_tile(player_pos, Tiles.tree())

        # Mine the tree
        event = self.create_keyboard_event(ord("u"))
        world_map = Mock()
        InputHandler.handle_mining(event, self.game, world_map)

        # Check that tree was replaced with dirt
        tile = self.game.get_tile_at_player_feet()
        self.assertEqual(tile.tileid, Tiles.dirt().tileid)

    def test_viewport_input_handling(self):
        """Test that viewport inputs are handled correctly."""
        # Test viewport movement keys
        viewport_tests = [
            (ord("["), "SLIDE_VIEWPORT_WEST"),
            (ord("]"), "SLIDE_VIEWPORT_EAST"),
        ]

        for key_code, expected_action in viewport_tests:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                # Test that the key is recognized as a viewport key
                self.assertTrue(KEYMAP.matches(expected_action, event))

    def test_scale_input_handling(self):
        """Test that scale inputs are handled correctly."""
        # Test scale change keys
        scale_tests = [
            (ord("="), "SCALE_UP"),
            (ord("-"), "SCALE_DOWN"),
        ]

        for key_code, expected_action in scale_tests:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                # Test that the key is recognized as a scale key
                self.assertTrue(KEYMAP.matches(expected_action, event))


class TestGameIntegrationSimple(SimpleTUITestCase):
    """Test game integration with simple mocking."""

    def test_player_movement_integration(self):
        """Test that player movement works correctly."""
        initial_pos = self.game.player.position

        # Move player east
        event = self.create_keyboard_event(ord("6"))  # Numpad 6 for east
        move_vec = InputHandler.handle_movement(event)
        self.game.move_player(move_vec)

        # Check that player moved
        self.assertEqual(self.game.player.position, initial_pos + VEC_EAST)

    def test_mining_integration(self):
        """Test that mining works correctly."""
        # Set up a mineable tile
        player_pos = self.game.player.position
        self.game.world.set_tile(player_pos, Tiles.gold_ore())

        # Mine the tile
        event = self.create_keyboard_event(ord("u"))
        world_map = Mock()
        InputHandler.handle_mining(event, self.game, world_map)

        # Check that tile was replaced
        tile = self.game.get_tile_at_player_feet()
        self.assertEqual(tile.tileid, Tiles.dirt().tileid)

    def test_viewport_integration(self):
        """Test that viewport changes work correctly."""
        initial_viewport_pos = self.game.viewport.top_left

        # Move viewport west
        event = self.create_keyboard_event(ord("["))
        InputHandler.handle_viewport(event, self.game)

        # Check that viewport moved
        self.assertNotEqual(initial_viewport_pos, self.game.viewport.top_left)

    def test_scale_integration(self):
        """Test that scale changes work correctly."""
        initial_scale = self.game.viewport.scale

        # Try to decrease scale first to ensure we can increase it
        event = self.create_keyboard_event(ord("-"))
        InputHandler.handle_scale(event, self.game)

        # Now try to increase scale
        event = self.create_keyboard_event(ord("="))
        InputHandler.handle_scale(event, self.game)

        # Check that scale changed from the initial value
        # If initial scale was 3, we can only decrease, so check that it changed
        final_scale = self.game.viewport.scale
        self.assertNotEqual(initial_scale, final_scale)

        # Verify scale is within valid range (1-3)
        self.assertGreaterEqual(final_scale, 1)
        self.assertLessEqual(final_scale, 3)


class TestKeymapSimple(SimpleTUITestCase):
    """Test the keymap functionality."""

    def test_keymap_matches(self):
        """Test that keymap matching works correctly."""
        # Test character-based movement keys (Q/E for up/down)
        character_movement_keys = [
            (ord("q"), "MOVE_UP"),
            (ord("e"), "MOVE_DOWN"),
        ]

        for key_code, key_name in character_movement_keys:
            with self.subTest(f"key_name={key_name}"):
                event = self.create_keyboard_event(key_code)
                self.assertTrue(KEYMAP.matches(key_name, event))

    def test_keymap_numpad_movement(self):
        """Test that numpad movement keys work correctly."""
        # Test numpad movement keys
        numpad_movement_keys = [
            (ord("8"), "MOVE_NORTH"),
            (ord("2"), "MOVE_SOUTH"),
            (ord("4"), "MOVE_WEST"),
            (ord("6"), "MOVE_EAST"),
        ]

        for key_code, key_name in numpad_movement_keys:
            with self.subTest(f"key_name={key_name}"):
                event = self.create_keyboard_event(key_code)
                # Numpad keys are handled differently, so we test the movement handler directly
                result = InputHandler.handle_movement(event)
                self.assertIsNotNone(result)

    def test_keymap_mine_key(self):
        """Test that mining key is mapped correctly."""
        event = self.create_keyboard_event(ord("u"))
        self.assertTrue(KEYMAP.matches("MINE", event))

    def test_keymap_viewport_keys(self):
        """Test that viewport keys are mapped correctly."""
        viewport_keys = [
            (ord("["), "SLIDE_VIEWPORT_WEST"),
            (ord("]"), "SLIDE_VIEWPORT_EAST"),
        ]

        for key_code, key_name in viewport_keys:
            with self.subTest(f"key_name={key_name}"):
                event = self.create_keyboard_event(key_code)
                self.assertTrue(KEYMAP.matches(key_name, event))

    def test_keymap_scale_keys(self):
        """Test that scale keys are mapped correctly."""
        scale_keys = [
            (ord("="), "SCALE_UP"),
            (ord("-"), "SCALE_DOWN"),
        ]

        for key_code, key_name in scale_keys:
            with self.subTest(f"key_name={key_name}"):
                event = self.create_keyboard_event(key_code)
                self.assertTrue(KEYMAP.matches(key_name, event))


class TestGameStateSimple(SimpleTUITestCase):
    """Test basic game state functionality."""

    def test_game_initialization(self):
        """Test that game initializes correctly."""
        self.assertIsNotNone(self.game)
        self.assertIsNotNone(self.game.player)
        self.assertIsNotNone(self.game.world)

    def test_player_position(self):
        """Test that player has a valid position."""
        player_pos = self.game.player.position
        self.assertIsInstance(player_pos, VectorN)
        self.assertIsInstance(player_pos.x, int)
        self.assertIsInstance(player_pos.y, int)
        self.assertIsInstance(player_pos.z, int)

    def test_world_generation(self):
        """Test that world was generated correctly."""
        # Test that we can get tiles from the world
        player_pos = self.game.player.position
        tile = self.game.world.get_tile(player_pos)
        self.assertIsNotNone(tile)

    def test_viewport_initialization(self):
        """Test that viewport was initialized correctly."""
        viewport = self.game.viewport
        self.assertIsNotNone(viewport)
        self.assertIsInstance(viewport.scale, int)
        self.assertGreaterEqual(viewport.scale, 1)
        self.assertLessEqual(viewport.scale, 3)


if __name__ == "__main__":
    unittest.main()
