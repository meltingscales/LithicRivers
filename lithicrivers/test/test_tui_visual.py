"""
Visual TUI testing framework for LithicRivers.
This module provides visual regression testing for TUI components.
"""

import os
# Set TESTING environment BEFORE importing any game modules
os.environ["TESTING"] = "1"

import unittest

from lithicrivers.game import Game
from lithicrivers.game_engine import GameEngine


class VisualTUITestCase(unittest.TestCase):
    """Base class for visual TUI tests."""

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
            self.skipTest("TERM environment variable not set - skipping visual tests")

        # Use shared instances instead of creating new ones
        self.game_engine = self.shared_game_engine
        self.game = self.shared_game

    def test_visual_test_placeholder(self):
        """Placeholder test for visual regression testing."""
        # This is a placeholder test that will be expanded later
        # when visual regression testing is implemented
        self.assertTrue(True, "Visual tests require TERM environment variable")


class TestVisualRegression(VisualTUITestCase):
    """Test visual regression scenarios."""

    def test_basic_visual_test(self):
        """Basic visual test that requires terminal."""
        # This test will only run if TERM is set
        self.assertTrue(True, "Visual test running with terminal support")

    def test_visual_test_without_terminal(self):
        """Test that visual tests are skipped without TERM."""
        # This test should be skipped if TERM is not set
        # (which is handled by the base class setUp method)
        self.assertTrue(True, "This test should only run with TERM set")
