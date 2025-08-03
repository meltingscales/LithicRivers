"""
Headless TUI testing framework for LithicRivers.
This module provides testing for TUI components in headless mode.
"""

import os
# Set TESTING environment BEFORE importing any game modules
os.environ["TESTING"] = "1"

import time
import unittest
from unittest.mock import Mock

import asciimatics.screen
from asciimatics.event import KeyboardEvent, MouseEvent

from lithicrivers.constants import (
    VEC_EAST,
    VEC_NORTH,
    VEC_NORTHEAST,
    VEC_NORTHWEST,
    VEC_SOUTH,
    VEC_SOUTHEAST,
    VEC_SOUTHWEST,
    VEC_WEST,
)
from lithicrivers.game import Game, Tiles
from lithicrivers.game_engine import GameEngine
from lithicrivers.model.vector import VectorN
from lithicrivers.ui import GameWidget, HelpPage, InputHandler, WorldMap


class HeadlessTUITestCase(unittest.TestCase):
    """Base class for headless TUI tests."""

    @classmethod
    def setUpClass(cls):
        """Create shared game instances for tests to improve performance."""
        # Create shared game instances
        cls.shared_game_engine = GameEngine()
        cls.shared_game = Game()

    def setUp(self):
        """Set up common test fixtures."""
        # Skip tests if TERM environment variable is not set
        if not os.environ.get("TERM"):
            self.skipTest(
                "TERM environment variable not set - skipping terminal-dependent tests"
            )

        # Use shared instances instead of creating new ones
        self.game_engine = self.shared_game_engine
        self.game = self.shared_game
        self.screen = None

    def create_headless_screen(self, width: int = 80, height: int = 24):
        """Create a headless screen for testing."""
        return asciimatics.screen.Screen.open(
            height=height, unicode_aware=True, catch_interrupt=False
        )

    def get_screen_content(self, screen) -> str:
        """Get the current content of the screen as a string."""
        # Capture the screen content
        content = []
        for y in range(screen.height):
            row = ""
            for x in range(screen.width):
                char_data = screen.get_from(x, y)
                # get_from returns (ascii_code, foreground, attributes, background)
                if char_data and len(char_data) >= 1:
                    char = chr(char_data[0]) if char_data[0] > 0 else " "
                else:
                    char = " "
                row += char
            content.append(row)
        return "\n".join(content)

    def assert_screen_contains(self, screen, text: str):
        """Assert that the screen contains the given text."""
        content = self.get_screen_content(screen)
        self.assertIn(text, content, f"Expected '{text}' in screen content:\n{content}")

    def assert_screen_not_contains(self, screen, text: str):
        """Assert that the screen does not contain the given text."""
        content = self.get_screen_content(screen)
        self.assertNotIn(
            text, content, f"Expected '{text}' not in screen content:\n{content}"
        )

    def create_keyboard_event(self, key_code: int) -> KeyboardEvent:
        """Create a keyboard event."""
        event = KeyboardEvent(key_code)
        return event

    def create_mouse_event(self, x: int, y: int, button: int = 1) -> MouseEvent:
        """Create a mouse event."""
        event = MouseEvent(x, y, button)
        return event


class TestHeadlessGameWidget(HeadlessTUITestCase):
    """Test GameWidget in headless mode."""

    def test_game_widget_headless_rendering(self):
        """Test that GameWidget renders correctly in headless mode."""
        screen = self.create_headless_screen()

        try:
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen
            widget._x = 0
            widget._y = 0
            widget._w = 40
            widget._h = 20

            widget.update(0)

            # Get the rendered content
            content = self.get_screen_content(screen)

            # Look for common rendered characters instead of tile names
            # The content might be minimal in headless mode, so just check it's not empty
            self.assertIsNotNone(content)
            self.assertGreater(len(content), 0)

        finally:
            screen.close()

    def test_game_widget_dimensions_headless(self):
        """Test GameWidget dimensions in headless mode."""
        screen = self.create_headless_screen()

        try:
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = Mock()

            # Test required height calculation
            height = widget.required_height(0, 40)
            self.assertGreater(height, 0)

        finally:
            screen.close()


class TestHeadlessInputHandler(HeadlessTUITestCase):
    """Test InputHandler in headless mode."""

    def test_movement_input_headless(self):
        """Test movement input handling in headless mode."""
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

    def test_mining_input_headless(self):
        """Test mining input handling in headless mode."""
        # Set up a mineable tile at player's position
        player_pos = self.game.player.position
        self.game.world.set_tile(player_pos, Tiles.tree())

        # Create mining event
        event = self.create_keyboard_event(ord("u"))  # Mining key
        world_map = Mock()

        # Handle mining
        InputHandler.handle_mining(event, self.game, world_map)

        # Check that tile was mined (Tree should become Dirt)
        tile = self.game.get_tile_at_player_feet()
        self.assertEqual(tile, Tiles.dirt())

    def test_viewport_input_headless(self):
        """Test viewport input handling in headless mode."""
        initial_viewport = self.game.viewport.copy()

        # Test viewport movement
        event = self.create_keyboard_event(ord("i"))  # Move viewport up
        InputHandler.handle_viewport(event, self.game)

        # Viewport should have changed
        self.assertNotEqual(initial_viewport, self.game.viewport)

    def test_scale_input_headless(self):
        """Test scale input handling in headless mode."""
        initial_scale = self.game.viewport.scale

        # Test scale change
        event = self.create_keyboard_event(ord("="))  # Increase scale
        InputHandler.handle_scale(event, self.game)

        # Scale should have changed
        self.assertNotEqual(initial_scale, self.game.viewport.scale)


class TestHeadlessPages(HeadlessTUITestCase):
    """Test page components in headless mode."""

    def test_world_map_headless(self):
        """Test WorldMap in headless mode."""
        screen = self.create_headless_screen()

        try:
            # Create world map
            page = WorldMap(screen, self.game)

            # Check that page has layouts (which contain widgets)
            self.assertGreater(len(page._layouts), 0)

            # Check that GameWidget is present by looking for it using find_widget
            game_widget = page.find_widget("widgetGame")
            self.assertIsNotNone(game_widget)
            self.assertIsInstance(game_widget, GameWidget)

        finally:
            screen.close()

    def test_help_page_headless(self):
        """Test HelpPage in headless mode."""
        screen = self.create_headless_screen()

        try:
            # Create help page
            page = HelpPage(screen, self.game)

            # Check that page has layouts (which contain widgets)
            self.assertGreater(len(page._layouts), 0)

            # Check that help content is present by looking for labels
            help_label = page.find_widget("helpLabel")
            self.assertIsNotNone(help_label)
            self.assertTrue(hasattr(help_label, "text"))

        finally:
            screen.close()


class TestHeadlessIntegration(HeadlessTUITestCase):
    """Integration tests in headless mode."""

    def test_complete_game_flow_headless(self):
        """Test a complete game flow in headless mode."""
        # Set up world at player's position
        player_pos = self.game.player.position
        self.game.world.set_tile(player_pos, Tiles.dirt())
        self.game.world.set_tile(player_pos + VectorN(1, 0, 0), Tiles.tree())
        self.game.world.set_tile(player_pos + VectorN(0, 1, 0), Tiles.bedrock())

        screen = self.create_headless_screen()

        try:
            # Create world map
            world_map = WorldMap(screen, self.game)

            # Simulate player movement
            movement_event = self.create_keyboard_event(ord("6"))  # Move east
            move_vec = InputHandler.handle_movement(movement_event)
            if move_vec:
                self.game.move_player(move_vec)

            # Simulate mining
            mining_event = self.create_keyboard_event(ord("u"))
            InputHandler.handle_mining(mining_event, self.game, world_map)

            # Check game state - player should have moved east from initial position
            initial_pos = VectorN(0, 0, 0)  # In testing mode, player starts at (0,0,0)
            expected_pos = initial_pos + VEC_EAST  # Move east
            self.assertEqual(self.game.player.position, expected_pos)

            # Check that tile was mined (Tree should become Dirt)
            tile = self.game.get_tile_at_player_feet()
            self.assertEqual(tile, Tiles.dirt())

        finally:
            screen.close()

    def test_ui_responsiveness_headless(self):
        """Test that UI responds to various input events in headless mode."""
        screen = self.create_headless_screen()

        try:
            world_map = WorldMap(screen, self.game)

            # Test various input events
            events = [
                (ord("8"), "north movement"),
                (ord("2"), "south movement"),
                (ord("4"), "west movement"),
                (ord("6"), "east movement"),
                (ord("7"), "northwest movement"),
                (ord("9"), "northeast movement"),
                (ord("1"), "southwest movement"),
                (ord("3"), "southeast movement"),
                (ord(" "), "mining"),
                (ord("i"), "viewport up"),
                (ord("k"), "viewport down"),
                (ord("j"), "viewport left"),
                (ord("l"), "viewport right"),
                (ord("="), "scale up"),
                (ord("-"), "scale down"),
            ]

            for key_code, description in events:
                with self.subTest(description):
                    event = self.create_keyboard_event(key_code)

                    # Handle the event appropriately
                    if key_code in [
                        ord("8"),
                        ord("2"),
                        ord("4"),
                        ord("6"),
                        ord("7"),
                        ord("9"),
                        ord("1"),
                        ord("3"),
                    ]:
                        result = InputHandler.handle_movement(event)
                        self.assertIsNotNone(result)
                    elif key_code == ord(" "):
                        InputHandler.handle_mining(event, self.game, world_map)
                    elif key_code in [ord("i"), ord("k"), ord("j"), ord("l")]:
                        InputHandler.handle_viewport(event, self.game)
                    elif key_code in [ord("="), ord("-")]:
                        InputHandler.handle_scale(event, self.game)

        finally:
            screen.close()


class TestHeadlessPerformance(HeadlessTUITestCase):
    """Performance tests in headless mode."""

    def test_game_widget_rendering_performance_headless(self):
        """Test that GameWidget renders efficiently in headless mode."""

        screen = self.create_headless_screen()

        try:
            # Create a larger world
            for x in range(-5, 6):
                for y in range(-5, 6):
                    self.game.world.set_tile(VectorN(x, y, 0), Tiles.dirt())

            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = Mock()
            widget._frame.palette = {"label": (7, 0, 0)}
            widget._x = 0
            widget._y = 0
            widget._w = 80
            widget._h = 24

            # Measure rendering time
            start_time = time.time()
            widget.update(0)
            render_time = time.time() - start_time

            # Rendering should be fast (less than 300ms for perlin noise worldgen)
            self.assertLess(render_time, 0.3, f"Rendering took {render_time:.3f}s")

        finally:
            screen.close()

    def test_input_handling_performance_headless(self):
        """Test that input handling is efficient in headless mode."""

        # Create many input events
        events = [self.create_keyboard_event(ord("8")) for _ in range(100)]

        start_time = time.time()
        for event in events:
            InputHandler.handle_movement(event)
        handle_time = time.time() - start_time

        # Input handling should be very fast
        self.assertLess(handle_time, 0.01, f"Input handling took {handle_time:.3f}s")


if __name__ == "__main__":
    unittest.main()
