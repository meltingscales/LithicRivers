from asciimatics.event import KeyboardEvent

from lithicrivers.keymap import KEYMAP
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SEED
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestInteractionSystem(OptimizedTestCase):
    """Test the interaction system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = self.get_game(seed=DEFAULT_SEED)

    def test_interact_keybind(self):
        """Test that the interact keybind is properly configured."""
        self.assertIsNotNone(KEYMAP.INTERACT)
        self.assertEqual(KEYMAP.INTERACT, frozenset(["i"]))

    def test_adjacent_entities_detection(self):
        """Test that adjacent entities are properly detected."""
        # Move player near the NPC
        self.game.player.position = VectorN.create(4, 5, 0)  # Adjacent to NPC at (5, 5, 0)

        adjacent = self.game.world.get_adjacent_entities(self.game.player.position)
        self.assertGreater(len(adjacent), 0)

        # Check that we have the expected entities
        entity_names = [name for name, pos, color in adjacent]
        self.assertIn("Elder Oak", entity_names)  # NPC
        self.assertIn("Ancient Relic", entity_names)  # Test entity 2

        # Move player to a position where all entities are adjacent
        self.game.player.position = VectorN.create(
            5, 5, 0 
        )  # At NPC position, should see others

        adjacent = self.game.world.get_adjacent_entities(self.game.player.position)
        entity_names = [name for name, pos, color in adjacent]
        self.assertIn("Crystal Shard", entity_names)  # Test entity 1
        self.assertIn("Ancient Relic", entity_names)  # Test entity 2

    def test_entity_colors(self):
        """Test that entities have the correct colors."""
        npc = self.game.world.get_entity(VectorN.create(5, 5, 0))
        entity1 = self.game.world.get_entity(VectorN.create(6, 5, 0))
        entity2 = self.game.world.get_entity(VectorN.create(5, 6, 0))

        self.assertEqual(npc.color, "cyan")
        self.assertEqual(entity1.color, "blue")
        self.assertEqual(entity2.color, "red")

    def test_interaction_key_event(self):
        """Test that the interaction key creates the correct keyboard event."""
        # Create a mock keyboard event for the 'i' key
        event = KeyboardEvent(ord("i"))

        # Test that the keymap matches
        self.assertTrue(KEYMAP.matches("INTERACT", event))

    def test_no_adjacent_entities(self):
        """Test behavior when no entities are adjacent."""
        # Move player far from any entities
        self.game.player.position = VectorN.create(100, 100, 0)

        adjacent = self.game.world.get_adjacent_entities(self.game.player.position)
        self.assertEqual(len(adjacent), 0)

    def test_entity_interaction_text(self):
        """Test that interactive entities have proper interaction text."""
        entity1 = self.game.world.get_entity(VectorN.create(6, 5, 0))
        entity2 = self.game.world.get_entity(VectorN.create(5, 6, 0))

        self.assertIn("crystal shard", entity1.interact().lower())
        self.assertIn("ancient relic", entity2.interact().lower())

    def test_npc_conversation_system(self):
        """Test that NPCs have proper conversation systems."""
        npc = self.game.world.get_entity(VectorN.create(5, 5, 0))

        # Test initial conversation
        conv = npc.get_conversation()
        self.assertIn("Hello, traveler!", conv["text"])
        self.assertIn("Tell me about this world", conv["options"])

        # Test conversation flow
        next_topic = npc.handle_response("Tell me about this world", "greeting")
        self.assertEqual(next_topic, "about_world")

        next_conv = npc.get_conversation(next_topic)
        self.assertIn("endless possibilities", next_conv["text"])
