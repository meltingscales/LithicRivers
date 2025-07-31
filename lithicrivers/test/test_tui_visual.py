"""
Visual TUI testing framework for LithicRivers.
This module provides visual regression testing for TUI components.
"""

import unittest
import os
import tempfile
import hashlib
from typing import List, Optional, Dict, Any
from unittest.mock import Mock, patch

import asciimatics.screen
from asciimatics.event import KeyboardEvent, MouseEvent

from lithicrivers.game_engine import GameEngine
from lithicrivers.game import Game, Tiles, Items
from lithicrivers.model.vector import VectorN
from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST
from lithicrivers.ui import GameWidget, RootPage, HelpPage, InputHandler


class VisualTUITestCase(unittest.TestCase):
    """Base class for visual TUI tests."""
    
    def setUp(self):
        """Set up common test fixtures."""
        self.game_engine = GameEngine()
        self.game = Game()
        self.screen = None
        self.test_output_dir = os.path.join(tempfile.gettempdir(), "lithicrivers_visual_tests")
        os.makedirs(self.test_output_dir, exist_ok=True)
    
    def create_headless_screen(self, width: int = 80, height: int = 24):
        """Create a headless screen for testing."""
        return asciimatics.screen.Screen.open(
            width=width,
            height=height,
            unicode_aware=True,
            catch_signals=False
        )
    
    def capture_screen_content(self, screen, test_name: str) -> str:
        """Capture the screen content and save it to a file."""
        content = []
        for y in range(screen.height):
            row = ""
            for x in range(screen.width):
                char = screen.get_from(x, y)
                row += char if char else " "
            content.append(row)
        
        screen_content = '\n'.join(content)
        
        # Save to file
        filename = os.path.join(self.test_output_dir, f"{test_name}.txt")
        with open(filename, 'w') as f:
            f.write(screen_content)
        
        return screen_content
    
    def get_screen_hash(self, screen_content: str) -> str:
        """Get a hash of the screen content for comparison."""
        return hashlib.md5(screen_content.encode()).hexdigest()
    
    def assert_screen_matches_baseline(self, screen_content: str, test_name: str):
        """Assert that the screen content matches a baseline."""
        baseline_file = os.path.join(self.test_output_dir, f"{test_name}_baseline.txt")
        
        if not os.path.exists(baseline_file):
            # Create baseline if it doesn't exist
            with open(baseline_file, 'w') as f:
                f.write(screen_content)
            self.skipTest(f"Created baseline for {test_name}")
        
        # Read baseline
        with open(baseline_file, 'r') as f:
            baseline_content = f.read()
        
        # Compare content
        self.assertEqual(screen_content, baseline_content, 
                        f"Screen content doesn't match baseline for {test_name}")
    
    def assert_screen_hash_matches(self, screen_content: str, test_name: str):
        """Assert that the screen content hash matches a baseline hash."""
        baseline_hash_file = os.path.join(self.test_output_dir, f"{test_name}_hash.txt")
        current_hash = self.get_screen_hash(screen_content)
        
        if not os.path.exists(baseline_hash_file):
            # Create baseline hash if it doesn't exist
            with open(baseline_hash_file, 'w') as f:
                f.write(current_hash)
            self.skipTest(f"Created baseline hash for {test_name}")
        
        # Read baseline hash
        with open(baseline_hash_file, 'r') as f:
            baseline_hash = f.read().strip()
        
        # Compare hashes
        self.assertEqual(current_hash, baseline_hash,
                        f"Screen hash doesn't match baseline for {test_name}")
    
    def create_keyboard_event(self, key_code: int) -> KeyboardEvent:
        """Create a keyboard event."""
        event = KeyboardEvent(key_code)
        return event


class TestVisualGameWidget(VisualTUITestCase):
    """Visual tests for GameWidget."""
    
    def test_game_widget_initial_render(self):
        """Test the initial render of GameWidget."""
        screen = self.create_headless_screen()
        
        try:
            # Set up a simple world
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
            
            # Capture and test screen content
            content = self.capture_screen_content(screen, "game_widget_initial")
            self.assert_screen_hash_matches(content, "game_widget_initial")
            
        finally:
            screen.close()
    
    def test_game_widget_after_movement(self):
        """Test GameWidget render after player movement."""
        screen = self.create_headless_screen()
        
        try:
            # Set up world
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
            self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
            
            # Create widget
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            widget._frame.palette = screen.palette
            widget._x = 0
            widget._y = 0
            widget._w = 40
            widget._h = 20
            
            # Initial render
            widget.update(0)
            
            # Move player
            self.game.move_player(VEC_EAST)
            
            # Re-render
            widget.update(0)
            
            # Capture and test screen content
            content = self.capture_screen_content(screen, "game_widget_after_movement")
            self.assert_screen_hash_matches(content, "game_widget_after_movement")
            
        finally:
            screen.close()
    
    def test_game_widget_after_mining(self):
        """Test GameWidget render after mining."""
        screen = self.create_headless_screen()
        
        try:
            # Set up world with mineable tile
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Tree())
            
            # Create widget
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            widget._frame.palette = screen.palette
            widget._x = 0
            widget._y = 0
            widget._w = 40
            widget._h = 20
            
            # Initial render
            widget.update(0)
            
            # Mine the tile
            mining_event = self.create_keyboard_event(ord(' '))
            root_page = Mock()
            InputHandler.handle_mining(mining_event, self.game, root_page)
            
            # Re-render
            widget.update(0)
            
            # Capture and test screen content
            content = self.capture_screen_content(screen, "game_widget_after_mining")
            self.assert_screen_hash_matches(content, "game_widget_after_mining")
            
        finally:
            screen.close()


class TestVisualPages(VisualTUITestCase):
    """Visual tests for page components."""
    
    def test_root_page_visual(self):
        """Test the visual appearance of RootPage."""
        screen = self.create_headless_screen()
        
        try:
            # Create root page
            page = RootPage(screen, self.game)
            
            # Render the page
            page.update(0)
            
            # Capture and test screen content
            content = self.capture_screen_content(screen, "root_page")
            self.assert_screen_hash_matches(content, "root_page")
            
        finally:
            screen.close()
    
    def test_help_page_visual(self):
        """Test the visual appearance of HelpPage."""
        screen = self.create_headless_screen()
        
        try:
            # Create help page
            page = HelpPage(screen, self.game)
            
            # Render the page
            page.update(0)
            
            # Capture and test screen content
            content = self.capture_screen_content(screen, "help_page")
            self.assert_screen_hash_matches(content, "help_page")
            
        finally:
            screen.close()


class TestVisualIntegration(VisualTUITestCase):
    """Visual integration tests."""
    
    def test_complete_game_flow_visual(self):
        """Test visual appearance during a complete game flow."""
        screen = self.create_headless_screen()
        
        try:
            # Set up world
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
            self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
            self.game.world.set_tile(VectorN(0, 1, 0), Tiles.Bedrock())
            
            # Create root page
            root_page = RootPage(screen, self.game)
            
            # Initial render
            root_page.update(0)
            initial_content = self.capture_screen_content(screen, "game_flow_initial")
            self.assert_screen_hash_matches(initial_content, "game_flow_initial")
            
            # Move player
            movement_event = self.create_keyboard_event(ord('d'))
            InputHandler.handle_movement(movement_event)
            root_page.update(0)
            
            after_movement_content = self.capture_screen_content(screen, "game_flow_after_movement")
            self.assert_screen_hash_matches(after_movement_content, "game_flow_after_movement")
            
            # Mine tile
            mining_event = self.create_keyboard_event(ord(' '))
            InputHandler.handle_mining(mining_event, self.game, root_page)
            root_page.update(0)
            
            after_mining_content = self.capture_screen_content(screen, "game_flow_after_mining")
            self.assert_screen_hash_matches(after_mining_content, "game_flow_after_mining")
            
        finally:
            screen.close()
    
    def test_viewport_changes_visual(self):
        """Test visual appearance when viewport changes."""
        screen = self.create_headless_screen()
        
        try:
            # Set up world
            for x in range(-5, 6):
                for y in range(-5, 6):
                    self.game.world.set_tile(VectorN(x, y, 0), Tiles.Dirt())
            
            # Create root page
            root_page = RootPage(screen, self.game)
            
            # Initial render
            root_page.update(0)
            initial_content = self.capture_screen_content(screen, "viewport_initial")
            self.assert_screen_hash_matches(initial_content, "viewport_initial")
            
            # Move viewport
            viewport_event = self.create_keyboard_event(ord('i'))
            InputHandler.handle_viewport(viewport_event, self.game)
            root_page.update(0)
            
            after_viewport_content = self.capture_screen_content(screen, "viewport_after_move")
            self.assert_screen_hash_matches(after_viewport_content, "viewport_after_move")
            
        finally:
            screen.close()
    
    def test_scale_changes_visual(self):
        """Test visual appearance when scale changes."""
        screen = self.create_headless_screen()
        
        try:
            # Set up world
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
            self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
            
            # Create root page
            root_page = RootPage(screen, self.game)
            
            # Initial render
            root_page.update(0)
            initial_content = self.capture_screen_content(screen, "scale_initial")
            self.assert_screen_hash_matches(initial_content, "scale_initial")
            
            # Change scale
            scale_event = self.create_keyboard_event(ord('='))
            InputHandler.handle_scale(scale_event, self.game)
            root_page.update(0)
            
            after_scale_content = self.capture_screen_content(screen, "scale_after_change")
            self.assert_screen_hash_matches(after_scale_content, "scale_after_change")
            
        finally:
            screen.close()


class TestVisualRegression(VisualTUITestCase):
    """Visual regression tests."""
    
    def test_no_unexpected_changes(self):
        """Test that no unexpected visual changes occur."""
        screen = self.create_headless_screen()
        
        try:
            # Set up a consistent world
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
            self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
            
            # Create widget
            widget = GameWidget(self.game)
            widget._frame = Mock()
            widget._frame.canvas = screen.canvas
            widget._frame.palette = screen.palette
            widget._x = 0
            widget._y = 0
            widget._w = 40
            widget._h = 20
            
            # Multiple renders should produce identical output
            widget.update(0)
            content1 = self.capture_screen_content(screen, "regression_test_1")
            
            widget.update(0)
            content2 = self.capture_screen_content(screen, "regression_test_2")
            
            # Content should be identical
            self.assertEqual(content1, content2, "Multiple renders produced different output")
            
        finally:
            screen.close()
    
    def test_deterministic_rendering(self):
        """Test that rendering is deterministic."""
        screen1 = self.create_headless_screen()
        screen2 = self.create_headless_screen()
        
        try:
            # Set up identical worlds
            self.game.world.set_tile(VectorN(0, 0, 0), Tiles.Dirt())
            self.game.world.set_tile(VectorN(1, 0, 0), Tiles.Tree())
            
            # Create identical widgets
            widget1 = GameWidget(self.game)
            widget1._frame = Mock()
            widget1._frame.canvas = screen1.canvas
            widget1._frame.palette = screen1.palette
            widget1._x = 0
            widget1._y = 0
            widget1._w = 40
            widget1._h = 20
            
            widget2 = GameWidget(self.game)
            widget2._frame = Mock()
            widget2._frame.canvas = screen2.canvas
            widget2._frame.palette = screen2.palette
            widget2._x = 0
            widget2._y = 0
            widget2._w = 40
            widget2._h = 20
            
            # Render both widgets
            widget1.update(0)
            widget2.update(0)
            
            # Capture content
            content1 = self.capture_screen_content(screen1, "deterministic_1")
            content2 = self.capture_screen_content(screen2, "deterministic_2")
            
            # Content should be identical
            self.assertEqual(content1, content2, "Identical widgets produced different output")
            
        finally:
            screen1.close()
            screen2.close()


if __name__ == '__main__':
    unittest.main() 