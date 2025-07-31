"""
Headless TUI testing framework for LithicRivers.
This module provides realistic TUI testing using asciimatics' headless mode.
"""

import unittest
import asyncio
import time
from typing import List, Optional, Dict, Any
from unittest.mock import Mock, patch

import asciimatics.screen
from asciimatics.event import KeyboardEvent, MouseEvent
from asciimatics.exceptions import NextScene, ResizeScreenError

from lithicrivers.game_engine import GameEngine
from lithicrivers.game import Game, Tiles, Items
from lithicrivers.model.vector import VectorN
from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST
from lithicrivers.ui import GameWidget, RootPage, HelpPage, InputHandler, demo


class HeadlessTUITestCase(unittest.TestCase):
    """Base class for headless TUI tests."""
    
    def setUp(self):
        """Set up common test fixtures."""
        self.game_engine = GameEngine()
        self.game = Game()
        self.screen = None
    
    def create_headless_screen(self, width: int = 80, height: int = 24):
        """Create a headless screen for testing."""
        return asciimatics.screen.Screen.open(
            width=width,
            height=height,
            unicode_aware=True,
            catch_signals=False
        )
    
    def get_screen_content(self, screen) -> str:
        """Get the current content of the screen as a string."""
        # Capture the screen content
        content = []
        for y in range(screen.height):
            row = ""
            for x in range(screen.width):
                char = screen.get_from(x, y)
                row += char if char else " "
            content.append(row)
        return '\n'.join(content)
    
    def assert_screen_contains(self, screen, text: str):
        """Assert that the screen contains the given text."""
        content = self.get_screen_content(screen)
        self.assertIn(text, content, f"Expected '{text}' in screen content:\n{content}")
    
    def assert_screen_not_contains(self, screen, text: str):
        """Assert that the screen does not contain the given text."""
        content = self.get_screen_content(screen)
        self.assertNotIn(text, content, f"Expected '{text}' not in screen content:\n{content}")
    
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
            # Create a simple world
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
            self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
            
            # Create the widget
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            widget._frame.palette = screen.palette
            widget._x = 0
            widget._y = 0
            widget._w = 40
            widget._h = 20
            
            # Update the widget
            widget.update(0)
            
            # Check that content was rendered
            content = self.get_screen_content(screen)
            self.assertIn("Dirt", content)
            self.assertIn("Tree", content)
            
        finally:
            screen.close()
    
    def test_game_widget_dimensions_headless(self):
        """Test GameWidget dimensions in headless mode."""
        screen = self.create_headless_screen()
        
        try:
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            
            # Test required height calculation
            height = widget.required_height(0, 40)
            self.assertGreater(height, 0)
            
        finally:
            screen.close()


class TestHeadlessInputHandler(HeadlessTUITestCase):
    """Test InputHandler in headless mode."""
    
    def test_movement_input_headless(self):
        """Test movement input handling in headless mode."""
        # Test each movement direction
        movement_tests = [
            (ord('w'), VEC_NORTH),
            (ord('s'), VEC_SOUTH),
            (ord('a'), VEC_WEST),
            (ord('d'), VEC_EAST),
        ]
        
        for key_code, expected_direction in movement_tests:
            with self.subTest(f"key_code={key_code}"):
                event = self.create_keyboard_event(key_code)
                result = InputHandler.handle_movement(event)
                self.assertEqual(result, expected_direction)
    
    def test_mining_input_headless(self):
        """Test mining input handling in headless mode."""
        # Set up a mineable tile
        self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Tree())
        
        # Create mining event
        event = self.create_keyboard_event(ord(' '))  # Spacebar
        root_page = Mock()
        
        # Handle mining
        InputHandler.handle_mining(event, self.game, root_page)
        
        # Check that tile was mined
        tile = self.game.get_tile_at_player_feet()
        self.assertEqual(tile, Tiles.Empty())
    
    def test_viewport_input_headless(self):
        """Test viewport input handling in headless mode."""
        initial_viewport = self.game.viewport.copy()
        
        # Test viewport movement
        event = self.create_keyboard_event(ord('i'))  # Move viewport up
        InputHandler.handle_viewport(event, self.game)
        
        # Viewport should have changed
        self.assertNotEqual(initial_viewport, self.game.viewport)
    
    def test_scale_input_headless(self):
        """Test scale input handling in headless mode."""
        initial_scale = self.game.viewport.scale
        
        # Test scale change
        event = self.create_keyboard_event(ord('='))  # Increase scale
        InputHandler.handle_scale(event, self.game)
        
        # Scale should have changed
        self.assertNotEqual(initial_scale, self.game.viewport.scale)


class TestHeadlessPages(HeadlessTUITestCase):
    """Test page components in headless mode."""
    
    def test_root_page_headless(self):
        """Test RootPage in headless mode."""
        screen = self.create_headless_screen()
        
        try:
            # Create root page
            page = RootPage(screen, self.game)
            
            # Check that page has widgets
            self.assertGreater(len(page._widgets), 0)
            
            # Check that GameWidget is present
            game_widgets = [w for w in page._widgets if isinstance(w, GameWidget)]
            self.assertEqual(len(game_widgets), 1)
            
        finally:
            screen.close()
    
    def test_help_page_headless(self):
        """Test HelpPage in headless mode."""
        screen = self.create_headless_screen()
        
        try:
            # Create help page
            page = HelpPage(screen, self.game)
            
            # Check that page has widgets
            self.assertGreater(len(page._widgets), 0)
            
            # Check that help content is present
            help_labels = [w for w in page._widgets if hasattr(w, 'text')]
            self.assertGreater(len(help_labels), 0)
            
        finally:
            screen.close()


class TestHeadlessIntegration(HeadlessTUITestCase):
    """Integration tests in headless mode."""
    
    def test_complete_game_flow_headless(self):
        """Test a complete game flow in headless mode."""
        # Set up world
        self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
        self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
        self.game.world.set_tile(VectorN(0, 1, 0), Tiles.Bedrock())
        
        screen = self.create_headless_screen()
        
        try:
            # Create root page
            root_page = RootPage(screen, self.game)
            
            # Simulate player movement
            movement_event = self.create_keyboard_event(ord('d'))
            InputHandler.handle_movement(movement_event)
            
            # Simulate mining
            mining_event = self.create_keyboard_event(ord(' '))
            InputHandler.handle_mining(mining_event, self.game, root_page)
            
            # Check game state
            self.assertEqual(self.game.player.position, VectorN(1, 0, 0))
            
            # Check that tile was mined
            tile = self.game.get_tile_at_player_feet()
            self.assertEqual(tile, Tiles.Empty())
            
        finally:
            screen.close()
    
    def test_ui_responsiveness_headless(self):
        """Test that UI responds to various input events in headless mode."""
        screen = self.create_headless_screen()
        
        try:
            root_page = RootPage(screen, self.game)
            
            # Test various input events
            events = [
                (ord('w'), "north movement"),
                (ord('s'), "south movement"),
                (ord('a'), "west movement"),
                (ord('d'), "east movement"),
                (ord(' '), "mining"),
                (ord('i'), "viewport up"),
                (ord('k'), "viewport down"),
                (ord('j'), "viewport left"),
                (ord('l'), "viewport right"),
                (ord('='), "scale up"),
                (ord('-'), "scale down"),
            ]
            
            for key_code, description in events:
                with self.subTest(description):
                    event = self.create_keyboard_event(key_code)
                    
                    # Handle the event appropriately
                    if key_code in [ord('w'), ord('s'), ord('a'), ord('d')]:
                        result = InputHandler.handle_movement(event)
                        self.assertIsNotNone(result)
                    elif key_code == ord(' '):
                        InputHandler.handle_mining(event, self.game, root_page)
                    elif key_code in [ord('i'), ord('k'), ord('j'), ord('l')]:
                        InputHandler.handle_viewport(event, self.game)
                    elif key_code in [ord('='), ord('-')]:
                        InputHandler.handle_scale(event, self.game)
                        
        finally:
            screen.close()


class TestHeadlessPerformance(HeadlessTUITestCase):
    """Performance tests in headless mode."""
    
    def test_game_widget_rendering_performance_headless(self):
        """Test that GameWidget renders efficiently in headless mode."""
        import time
        
        screen = self.create_headless_screen()
        
        try:
            # Create a larger world
            for x in range(-5, 6):
                for y in range(-5, 6):
                    self.game.world.set_tile(VectorN(x, y, 0), Tiles.Dirt())
            
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            widget._frame.palette = screen.palette
            widget._x = 0
            widget._y = 0
            widget._w = 80
            widget._h = 24
            
            # Measure rendering time
            start_time = time.time()
            widget.update(0)
            render_time = time.time() - start_time
            
            # Rendering should be fast (less than 100ms)
            self.assertLess(render_time, 0.1, f"Rendering took {render_time:.3f}s")
            
        finally:
            screen.close()
    
    def test_input_handling_performance_headless(self):
        """Test that input handling is efficient in headless mode."""
        import time
        
        # Create many input events
        events = [self.create_keyboard_event(ord('w')) for _ in range(100)]
        
        start_time = time.time()
        for event in events:
            InputHandler.handle_movement(event)
        handle_time = time.time() - start_time
        
        # Input handling should be very fast
        self.assertLess(handle_time, 0.01, f"Input handling took {handle_time:.3f}s")


if __name__ == '__main__':
    unittest.main() 