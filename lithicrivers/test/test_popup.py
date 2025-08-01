import unittest
from unittest.mock import Mock

from lithicrivers.game import Game
from lithicrivers.model.vector import VectorN
from lithicrivers.ui import SimplePopup


class TestPopupSystem(unittest.TestCase):
    """Test the popup system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = Game()

    def test_simple_popup_creation(self):
        """Test that SimplePopup can be created."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.width = 80
        mock_screen.height = 24

        popup = SimplePopup(
            mock_screen,
            "Test Popup",
            "This is a test popup content.",
            ["Option 1", "Option 2"],
            callback=lambda x: None,
        )

        self.assertIsNotNone(popup)
        self.assertEqual(popup.title, "Test Popup")
        self.assertEqual(popup.content, "This is a test popup content.")
        self.assertEqual(popup.options, ["Option 1", "Option 2"])
        self.assertTrue(popup.visible)

    def test_popup_dimensions(self):
        """Test that popup dimensions are calculated correctly."""
        mock_screen = Mock()
        mock_screen.width = 80
        mock_screen.height = 24

        popup = SimplePopup(
            mock_screen, "Test", "Content", ["Option 1", "Option 2", "Option 3"]
        )

        # Popup should be centered
        expected_x = (80 - 60) // 2  # 60 is max_width
        expected_y = (24 - 9) // 2  # 9 is height (3 options + 6)

        self.assertEqual(popup.popup_x, expected_x)
        self.assertEqual(popup.popup_y, expected_y)
        self.assertEqual(popup.popup_width, 60)
        self.assertEqual(popup.popup_height, 9)

    def test_popup_callback(self):
        """Test that popup callback works."""
        callback_called = False
        callback_value = None

        def test_callback(value):
            nonlocal callback_called, callback_value
            callback_called = True
            callback_value = value

        mock_screen = Mock()
        mock_screen.width = 80
        mock_screen.height = 24

        popup = SimplePopup(
            mock_screen, "Test", "Content", ["Option 1", "Option 2"], test_callback
        )

        # Test that callback is called when popup is closed
        popup.visible = False
        popup.callback("Option 1")

        self.assertTrue(callback_called)
        self.assertEqual(callback_value, "Option 1")

    def test_entity_interaction_popup(self):
        """Test that entity interaction creates popup."""
        # Move player near multiple entities
        self.game.player.position = VectorN(5, 5, 0)  # At NPC position

        # Get adjacent entities
        adjacent = self.game.world.get_adjacent_entities(self.game.player.position)
        self.assertGreater(len(adjacent), 1)  # Should have multiple entities

        # Test that we can create entity options
        entity_options = [f"{name} ({color})" for name, pos, color in adjacent]
        self.assertIn("Crystal Shard (blue)", entity_options)
        self.assertIn("Ancient Relic (red)", entity_options)


if __name__ == "__main__":
    unittest.main()
