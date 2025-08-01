import unittest

from lithicrivers.game import Game
from lithicrivers.model.vector import VectorN


class TestInteractionMessages(unittest.TestCase):
    """Test that interaction messages are displayed correctly."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = Game()

    def test_npc_interaction_message(self):
        """Test that NPC interaction shows the correct message."""
        # Move player near the NPC
        self.game.player.position = VectorN(4, 5, 0)  # Adjacent to NPC at (5, 5, 0)

        # Get the NPC
        npc = self.game.world.get_entity(VectorN(5, 5, 0))
        self.assertIsNotNone(npc)

        # Get the conversation
        conversation = npc.get_conversation()
        self.assertIn("Hello, traveler!", conversation["text"])
        self.assertIn("Tell me about this world", conversation["options"])

    def test_entity_interaction_message(self):
        """Test that entity interaction shows the correct message."""
        # Move player near the test entities
        self.game.player.position = VectorN(
            5, 5, 0
        )  # At NPC position, should see others

        # Get adjacent entities
        adjacent = self.game.world.get_adjacent_entities(self.game.player.position)
        self.assertGreater(len(adjacent), 0)

        # Check that we have the expected entities
        entity_names = [name for name, pos, color in adjacent]
        self.assertIn("Crystal Shard", entity_names)
        self.assertIn("Ancient Relic", entity_names)

    def test_interaction_text_format(self):
        """Test that interaction text is properly formatted."""
        entity1 = self.game.world.get_entity(VectorN(6, 5, 0))  # Crystal Shard
        entity2 = self.game.world.get_entity(VectorN(5, 6, 0))  # Ancient Relic

        # Test interaction text
        text1 = entity1.interact()
        text2 = entity2.interact()

        self.assertIn("crystal shard", text1.lower())
        self.assertIn("ancient relic", text2.lower())
        self.assertIn("glows", text1.lower())
        self.assertIn("runes", text2.lower())


if __name__ == "__main__":
    unittest.main()
