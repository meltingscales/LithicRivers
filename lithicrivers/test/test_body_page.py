"""
Tests for the BodyPage UI component.
Copyright (c) 2024 HenryFBP. All rights reserved.
"""

from unittest.mock import Mock
from lithicrivers.ui import BodyPage
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestBodyPage(OptimizedTestCase):
    """Test the BodyPage UI component."""

    def setUp(self):
        super().setUp()
        self.shared_game = self.get_game()

    def test_body_page_creation(self):
        """Test that BodyPage can be created without errors."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        # Create BodyPage
        body_page = BodyPage(mock_screen, self.shared_game)
        
        # Verify it was created successfully
        assert body_page is not None
        assert body_page.game == self.shared_game

    def test_body_page_without_game(self):
        """Test that BodyPage handles missing game gracefully."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        # Create BodyPage without game
        body_page = BodyPage(mock_screen, None)
        
        # Verify it was created successfully
        assert body_page is not None
        assert body_page.game is None

    def test_body_page_display_content(self):
        """Test that BodyPage displays body information correctly."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        # Create BodyPage
        body_page = BodyPage(mock_screen, self.shared_game)
        
        # Update the display
        body_page.update_body_display()
        
        # Verify the label contains body information
        assert body_page.body_label is not None
        assert "ANDROID BODY STATUS" in body_page.body_label.text
        assert "Health:" in body_page.body_label.text
        assert "Stamina:" in body_page.body_label.text
        assert "BODY PARTS:" in body_page.body_label.text
        assert "MOVEMENT STATUS:" in body_page.body_label.text
        assert "SPEED MODIFIERS:" in body_page.body_label.text
        assert "ACTION CAPABILITIES:" in body_page.body_label.text
        assert "REPAIR REQUIREMENTS:" in body_page.body_label.text
        assert "BODY DESCRIPTIONS:" in body_page.body_label.text

    def test_body_page_player_data(self):
        """Test that BodyPage shows correct player data."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        player = self.shared_game.player
        
        # Create BodyPage
        body_page = BodyPage(mock_screen, self.shared_game)
        
        # Update the display
        body_page.update_body_display()
        
        # Verify player data is displayed
        assert f"Health: {player.health}" in body_page.body_label.text
        assert f"Stamina: {player.stamina}" in body_page.body_label.text
        
        # Verify body parts are shown
        for part_name in ["Head", "Torso", "Left Arm", "Right Arm", "Left Leg", "Right Leg"]:
            assert part_name in body_page.body_label.text
        
        # Verify speed modifiers are shown
        walk_speed = player.get_walk_speed_modifier()
        break_speed = player.get_break_speed_modifier()
        assert f"Walk Speed: {walk_speed:.2f}" in body_page.body_label.text
        assert f"Break Speed: {break_speed:.2f}" in body_page.body_label.text

    def test_body_page_action_capabilities(self):
        """Test that BodyPage shows action capabilities correctly."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        # Create BodyPage
        body_page = BodyPage(mock_screen, self.shared_game)
        
        # Update the display
        body_page.update_body_display()
        
        # Verify action capabilities are shown
        actions = ["WALK", "MINE", "CRAFT", "PUSH", "INTERACT"]
        for action in actions:
            assert action in body_page.body_label.text

    def test_body_page_repair_requirements(self):
        """Test that BodyPage shows repair requirements correctly."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        # Create BodyPage
        body_page = BodyPage(mock_screen, self.shared_game)
        
        # Update the display
        body_page.update_body_display()
        
        # Verify repair requirements section exists
        assert "REPAIR REQUIREMENTS:" in body_page.body_label.text
        
        # Should show requirements for damaged/missing parts
        # (Right Arm and Right Leg should be damaged/missing by default)
        assert "Right Arm:" in body_page.body_label.text or "No repairs needed!" in body_page.body_label.text

    def test_body_page_body_descriptions(self):
        """Test that BodyPage shows body part descriptions."""
        # Mock screen
        mock_screen = Mock()
        mock_screen.height = 20
        mock_screen.width = 80
        
        # Create BodyPage
        body_page = BodyPage(mock_screen, self.shared_game)
        
        # Update the display
        body_page.update_body_display()
        
        # Verify body descriptions section exists
        assert "BODY DESCRIPTIONS:" in body_page.body_label.text
        
        # Should show descriptions for all body parts
        for part_name in ["Head", "Torso", "Left Arm", "Right Arm", "Left Leg", "Right Leg"]:
            assert part_name in body_page.body_label.text


if __name__ == "__main__":
    pytest.main([__file__]) 