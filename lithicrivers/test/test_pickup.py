"""
Test the pickup items functionality.
Copyright (c) 2024 Henry Post. All rights reserved.
"""


from asciimatics.event import KeyboardEvent

from lithicrivers.game.entities import DroppedItem, Items
from lithicrivers.keymap import KEYMAP
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SEED
from lithicrivers.test.test_fixtures import OptimizedTestCase


class TestPickupSystem(OptimizedTestCase):
    """Test the pickup items system."""

    def setUp(self):
        """Set up test fixtures."""
        self.game = self.get_game(seed=DEFAULT_SEED)

    def test_pickup_keybind(self):
        """Test that the pickup keybind is properly configured."""
        self.assertIsNotNone(KEYMAP.PICKUP_ITEMS)
        self.assertEqual(KEYMAP.PICKUP_ITEMS, frozenset(["g"]))

    def test_pickup_key_event(self):
        """Test that the pickup key creates the correct keyboard event."""
        # Create a mock keyboard event for the 'g' key
        event = KeyboardEvent(ord("g"))

        # Test that the keymap matches
        self.assertTrue(KEYMAP.matches("PICKUP_ITEMS", event))

    def test_dropped_item_creation(self):
        """Test that DroppedItem entities can be created."""
        item = Items.acorn()
        position = VectorN(10, 10, 0)
        dropped_item = DroppedItem(item, position)

        self.assertEqual(dropped_item.name, "Acorn")
        self.assertEqual(dropped_item.position, position)
        self.assertEqual(dropped_item.item, item)
        self.assertEqual(dropped_item.color, "yellow")

    def test_dropped_item_detection(self):
        """Test that dropped items are properly detected."""
        # Create a dropped item near the player
        item = Items.acorn()
        position = VectorN(1, 0, 0)  # Adjacent to player at (0, 0, 0)
        dropped_item = DroppedItem(item, position)
        self.game.world.add_entity(dropped_item)

        # Get adjacent entities
        adjacent_entities = self.game.world.get_adjacent_entities(
            self.game.player.position
        )

        # Check that the dropped item is detected
        entity_names = [name for name, pos, color in adjacent_entities]
        self.assertIn("Acorn", entity_names)

    def test_pickup_functionality(self):
        """Test that items can be picked up."""
        # Create a dropped item near the player
        item = Items.acorn()
        position = VectorN(1, 0, 0)  # Adjacent to player at (0, 0, 0)
        dropped_item = DroppedItem(item, position)
        self.game.world.add_entity(dropped_item)

        # Verify item is in world
        entities = self.game.world.get_entities(position)
        self.assertIn(dropped_item, entities)

        # Pick up the item
        self.game.player.inventory.add_item(dropped_item.item)
        self.game.world.remove_entity(dropped_item)

        # Verify item is in inventory
        self.assertIn(item, self.game.player.inventory.itemsdata)

        # Verify item is removed from world
        entities = self.game.world.get_entities(position)
        self.assertNotIn(dropped_item, entities)

    def test_no_items_to_pickup(self):
        """Test behavior when no items are available to pick up."""
        # Move player to a position with no dropped items
        self.game.player.position = VectorN(100, 100, 0)

        # Get adjacent entities
        adjacent_entities = self.game.world.get_adjacent_entities(
            self.game.player.position
        )

        # Filter for dropped items
        dropped_items = []
        for name, pos, color in adjacent_entities:
            entities = self.game.world.get_entities(pos)
            for entity in entities:
                if hasattr(entity, "item") and hasattr(entity, "name"):
                    dropped_items.append((entity.name, pos, color, entity))

        # Should be no dropped items
        self.assertEqual(len(dropped_items), 0)

    def test_multiple_dropped_items(self):
        """Test handling multiple dropped items."""
        # Create multiple dropped items near the player
        item1 = Items.acorn()
        item2 = Items.stick()
        position1 = VectorN(1, 0, 0)
        position2 = VectorN(0, 1, 0)

        dropped_item1 = DroppedItem(item1, position1)
        dropped_item2 = DroppedItem(item2, position2)

        self.game.world.add_entity(dropped_item1)
        self.game.world.add_entity(dropped_item2)

        # Get adjacent entities and filter for dropped items
        adjacent_entities = self.game.world.get_adjacent_entities(
            self.game.player.position
        )
        dropped_items = []

        for name, pos, color in adjacent_entities:
            entities = self.game.world.get_entities(pos)
            for entity in entities:
                if hasattr(entity, "item") and hasattr(entity, "name"):
                    dropped_items.append((entity.name, pos, color, entity))

        # Should find both dropped items
        self.assertEqual(len(dropped_items), 2)

        item_names = [name for name, pos, color, entity in dropped_items]
        self.assertIn("Acorn", item_names)
        self.assertIn("Stick", item_names)
