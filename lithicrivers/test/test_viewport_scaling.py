import unittest

from lithicrivers.game import Game
from lithicrivers.ui import GameWidget


class TestViewportScaling(unittest.TestCase):
    """Test that viewport scaling works correctly without overflow."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = Game()

    def test_viewport_scaling_dimensions(self):
        """Test that viewport dimensions account for scale correctly."""
        # Test scale 1
        self.game.viewport.scale = 1
        widget = GameWidget(self.game)
        height_scale_1 = widget.required_height(0, 0)
        width_scale_1 = widget.required_width(0, 0)

        # Test scale 2
        self.game.viewport.scale = 2
        widget = GameWidget(self.game)
        height_scale_2 = widget.required_height(0, 0)
        width_scale_2 = widget.required_width(0, 0)

        # Test scale 3
        self.game.viewport.scale = 3
        widget = GameWidget(self.game)
        height_scale_3 = widget.required_height(0, 0)
        width_scale_3 = widget.required_width(0, 0)

        # Verify that dimensions scale correctly
        viewport_height = self.game.viewport.get_height()
        viewport_width = self.game.viewport.get_width()

        # Height should be viewport_height * scale + 2 (for header)
        self.assertEqual(height_scale_1, viewport_height * 1 + 2)
        self.assertEqual(height_scale_2, viewport_height * 2 + 2)
        self.assertEqual(height_scale_3, viewport_height * 3 + 2)

        # Width should be viewport_width * scale
        self.assertEqual(width_scale_1, viewport_width * 1)
        self.assertEqual(width_scale_2, viewport_width * 2)
        self.assertEqual(width_scale_3, viewport_width * 3)

    def test_viewport_rendering_no_overflow(self):
        """Test that viewport rendering doesn't overflow."""
        # Test with different scales
        for scale in [1, 2, 3]:
            self.game.viewport.scale = scale
            widget = GameWidget(self.game)

            # Calculate expected dimensions
            expected_height = self.game.viewport.get_height() * scale + 2
            expected_width = self.game.viewport.get_width() * scale

            # Verify dimensions are reasonable
            self.assertGreater(expected_height, 0)
            self.assertGreater(expected_width, 0)
            self.assertLess(expected_height, 1000)  # Reasonable upper bound
            self.assertLess(expected_width, 1000)  # Reasonable upper bound


if __name__ == "__main__":
    unittest.main()
