"""
Tests for the body modularity system.
Copyright (c) 2024 HenryFBP. All rights reserved.
"""

from lithicrivers.game.core import Player
from lithicrivers.model.body import Body, BodyPart, BodyPartState, BodyPartType
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestBodySystem(OptimizedTestCase):
    """Test the body modularity system."""

    def test_default_damaged_body(self):
        """Test that the default body is created with the expected damaged state."""
        body = Body()

        # Check that all body parts exist
        assert BodyPartType.HEAD in body.parts
        assert BodyPartType.TORSO in body.parts
        assert BodyPartType.LEFT_ARM in body.parts
        assert BodyPartType.RIGHT_ARM in body.parts
        assert BodyPartType.LEFT_LEG in body.parts
        assert BodyPartType.RIGHT_LEG in body.parts

        # Check specific damaged state
        assert body.parts[BodyPartType.TORSO].state == BodyPartState.DAMAGED
        assert body.parts[BodyPartType.RIGHT_ARM].state == BodyPartState.MISSING
        assert body.parts[BodyPartType.RIGHT_LEG].state == BodyPartState.DAMAGED

        # Check functional parts
        assert body.parts[BodyPartType.HEAD].state == BodyPartState.FUNCTIONAL
        assert body.parts[BodyPartType.LEFT_ARM].state == BodyPartState.FUNCTIONAL
        assert body.parts[BodyPartType.LEFT_LEG].state == BodyPartState.FUNCTIONAL

    def test_walk_speed_modifiers(self):
        """Test walk speed modifiers based on leg condition."""
        body = Body()

        # With one damaged leg and one functional leg
        walk_speed = body.get_total_walk_speed_modifier()
        assert 0.1 <= walk_speed <= 0.8  # Should be impaired but not completely stopped

        # Test with both legs functional
        body.parts[BodyPartType.RIGHT_LEG].state = BodyPartState.FUNCTIONAL
        walk_speed = body.get_total_walk_speed_modifier()
        assert (
            walk_speed >= 0.7
        )  # Should be close to normal (0.75 with current normalization)

    def test_break_speed_modifiers(self):
        """Test break speed modifiers based on arm condition."""
        body = Body()

        # With one missing arm and one functional arm
        break_speed = body.get_total_break_speed_modifier()
        assert 0.1 <= break_speed <= 0.6  # Should be impaired due to missing arm

        # Test with both arms functional
        body.parts[BodyPartType.RIGHT_ARM].state = BodyPartState.FUNCTIONAL
        break_speed = body.get_total_break_speed_modifier()
        assert (
            break_speed >= 0.4
        )  # Should be close to normal (0.5 with current normalization)

    def test_action_capabilities(self):
        """Test that actions are properly restricted based on body parts."""
        body = Body()

        # Should be able to walk with one functional leg
        assert body.can_perform_action("walk")

        # Should be able to mine with one functional arm
        assert body.can_perform_action("mine")

        # Test with no functional arms
        body.parts[BodyPartType.LEFT_ARM].state = BodyPartState.MISSING
        assert not body.can_perform_action("mine")
        assert not body.can_perform_action("craft")

    def test_player_integration(self):
        """Test that the Player class properly integrates with the body system."""
        player = Player("TestPlayer")

        # Check that body is initialized
        assert hasattr(player, "body")
        assert isinstance(player.body, Body)

        # Check that stats are affected by body condition
        assert player.health < 150  # Should be reduced due to damaged parts
        assert player.stamina < 150  # Should be reduced due to damaged parts

        # Check speed modifiers
        walk_speed = player.get_walk_speed_modifier()
        break_speed = player.get_break_speed_modifier()

        assert 0.1 <= walk_speed <= 0.8  # Impaired but not stopped
        assert 0.1 <= break_speed <= 0.6  # Impaired due to missing arm

    def test_body_status_summary(self):
        """Test that body status summary is generated correctly."""
        body = Body()
        summary = body.get_body_status_summary()

        # Should contain status for all body parts
        assert "Head" in summary
        assert "Torso" in summary
        assert "Left Arm" in summary
        assert "Right Arm" in summary
        assert "Left Leg" in summary
        assert "Right Leg" in summary

        # Should contain state information
        assert "functional" in summary
        assert "damaged" in summary
        assert "missing" in summary

    def test_movement_penalty_description(self):
        """Test that movement penalty descriptions are generated correctly."""
        body = Body()
        description = body.get_movement_penalty_description()

        # Should indicate impaired movement due to damaged leg
        assert "Impaired" in description or "impaired" in description

    def test_repair_requirements(self):
        """Test that repair requirements are calculated correctly."""
        body = Body()
        requirements = body.get_repair_requirements()

        # Should have requirements for damaged and missing parts
        assert BodyPartType.TORSO in requirements
        assert BodyPartType.RIGHT_ARM in requirements
        assert BodyPartType.RIGHT_LEG in requirements

        # Should not have requirements for functional parts
        assert BodyPartType.HEAD not in requirements
        assert BodyPartType.LEFT_ARM not in requirements
        assert BodyPartType.LEFT_LEG not in requirements

    def test_body_part_states(self):
        """Test that body part states work correctly."""
        # Test missing state
        missing_part = BodyPart(
            part_type=BodyPartType.LEFT_ARM,
            state=BodyPartState.MISSING,
            name="Test Arm",
            description="Test",
        )
        assert missing_part.get_walk_speed_modifier() == 0.0
        assert missing_part.get_break_speed_modifier() == 0.0
        assert not missing_part.can_perform_action("mine")

        # Test damaged state
        damaged_part = BodyPart(
            part_type=BodyPartType.LEFT_ARM,
            state=BodyPartState.DAMAGED,
            name="Test Arm",
            description="Test",
        )
        assert damaged_part.get_break_speed_modifier() < 1.0
        assert damaged_part.can_perform_action("mine")

        # Test functional state
        functional_part = BodyPart(
            part_type=BodyPartType.LEFT_ARM,
            state=BodyPartState.FUNCTIONAL,
            name="Test Arm",
            description="Test",
        )
        assert functional_part.get_break_speed_modifier() == 1.0
        assert functional_part.can_perform_action("mine")


if __name__ == "__main__":
    pytest.main([__file__])
