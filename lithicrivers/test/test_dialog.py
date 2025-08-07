from lithicrivers.game.game_save_manager import GameSaveManager
from lithicrivers.game.game import NPC, Entities, Game, InteractiveEntity
from lithicrivers.settings import DEFAULT_SEED
from lithicrivers.test.test_fixtures import OptimizedTestCase
from lithicrivers.model.vector import VectorN

class TestDialogSystem(OptimizedTestCase):
    """Test the dialog and interaction system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = self.get_game(seed=DEFAULT_SEED)

    def test_npc_creation(self):
        """Test that NPCs can be created with conversations."""
        npc = Entities.starter_npc()
        self.assertIsInstance(npc, NPC)
        self.assertEqual(npc.name, "Elder Oak")
        self.assertEqual(npc.color, "cyan")
        self.assertEqual(npc.sprite, "N")

        # Test conversation setup
        conversation = npc.get_conversation()
        self.assertIn("text", conversation)
        self.assertIn("options", conversation)
        self.assertIn("Tell me about this world", conversation["options"])

    def test_npc_conversation_flow(self):
        """Test NPC conversation flow."""
        npc = Entities.starter_npc()

        # Test initial conversation
        conv = npc.get_conversation("greeting")
        self.assertIn("Hello, traveler!", conv["text"])

        # Test response handling
        next_topic = npc.handle_response("Tell me about this world", "greeting")
        self.assertEqual(next_topic, "about_world")

        # Test next conversation
        next_conv = npc.get_conversation(next_topic)
        self.assertIn("endless possibilities", next_conv["text"])

    def test_interactive_entity_creation(self):
        """Test that interactive entities can be created."""
        entity1 = Entities.test_entity1()
        entity2 = Entities.test_entity2()

        self.assertIsInstance(entity1, InteractiveEntity)
        self.assertIsInstance(entity2, InteractiveEntity)

        self.assertEqual(entity1.name, "Crystal Shard")
        self.assertEqual(entity1.color, "blue")
        self.assertEqual(entity1.sprite, "C")

        self.assertEqual(entity2.name, "Ancient Relic")
        self.assertEqual(entity2.color, "red")
        self.assertEqual(entity2.sprite, "R")

    def test_world_entity_management(self):
        """Test that entities are properly managed in the world."""
        # Check that starter entities are added
        self.assertIsNotNone(self.game.world.get_entity(VectorN(5, 5, 0)))  # NPC
        self.assertIsNotNone(
            self.game.world.get_entity(VectorN(6, 5, 0))
        )  # Test entity 1
        self.assertIsNotNone(
            self.game.world.get_entity(VectorN(5, 6, 0))
        )  # Test entity 2

        # Test entity retrieval
        npc = self.game.world.get_entity(VectorN(5, 5, 0))
        self.assertIsInstance(npc, NPC)
        self.assertEqual(npc.name, "Elder Oak")

    def test_adjacent_entities_detection(self):
        """Test that adjacent entities are properly detected."""
        # Move player near the NPC
        self.game.player.position = VectorN(4, 5, 0)  # Adjacent to NPC at (5, 5, 0)

        adjacent = self.game.world.get_adjacent_entities(self.game.player.position)
        self.assertGreater(len(adjacent), 0)

        # Check that the NPC is in the adjacent list
        npc_found = False
        for name, _pos, _color in adjacent:
            if name == "Elder Oak":
                npc_found = True
                break
        self.assertTrue(npc_found)

    def test_entity_rendering_colors(self):
        """Test that entities render with correct colors."""
        entity1 = Entities.test_entity1()
        entity2 = Entities.test_entity2()

        # Test sprite rendering
        sprite1 = entity1.render_sprite(scale=1)
        sprite2 = entity2.render_sprite(scale=1)

        self.assertEqual(sprite1, "C")
        self.assertEqual(sprite2, "R")

        # Test scaled sprite rendering
        scaled_sprite1 = entity1.render_sprite(scale=2)
        self.assertIn("CC", scaled_sprite1)
        self.assertIn("\n", scaled_sprite1)

    def test_interaction_text(self):
        """Test that interactive entities have proper interaction text."""
        entity1 = Entities.test_entity1()
        entity2 = Entities.test_entity2()

        self.assertIn("crystal shard", entity1.interact().lower())
        self.assertIn("ancient relic", entity2.interact().lower())


if __name__ == "__main__":
    unittest.main()
