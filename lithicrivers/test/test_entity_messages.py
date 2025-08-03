#!/usr/bin/env python3

import unittest
from lithicrivers.model.vector import VectorN
from lithicrivers.ui import _generate_entity_selection_message


class TestEntityMessages(unittest.TestCase):
    """Test the entity selection message generation."""

    def test_empty_entities(self):
        """Test with no entities."""
        result = _generate_entity_selection_message([])
        self.assertEqual(result, "No entities nearby.")

    def test_single_entity(self):
        """Test with a single entity."""
        entities = [("Crystal Shard", VectorN(0, 0, 0), "blue")]
        result = _generate_entity_selection_message(entities)
        self.assertEqual(result, "Found a Crystal Shard nearby:")

    def test_multiple_same_entity(self):
        """Test with multiple of the same entity."""
        entities = [
            ("Stumbling Sheep", VectorN(0, 0, 0), "white"),
            ("Stumbling Sheep", VectorN(1, 0, 0), "white")
        ]
        result = _generate_entity_selection_message(entities)
        self.assertEqual(result, "Found 2 Stumbling Sheeps nearby:")

    def test_two_different_entities(self):
        """Test with two different entities."""
        entities = [
            ("Crystal Shard", VectorN(0, 0, 0), "blue"),
            ("Ancient Relic", VectorN(1, 0, 0), "red")
        ]
        result = _generate_entity_selection_message(entities)
        self.assertEqual(result, "Found a Crystal Shard and a Ancient Relic nearby:")

    def test_three_different_entities(self):
        """Test with three different entities."""
        entities = [
            ("Crystal Shard", VectorN(0, 0, 0), "blue"),
            ("Ancient Relic", VectorN(1, 0, 0), "red"),
            ("Elder Oak", VectorN(0, 1, 0), "cyan")
        ]
        result = _generate_entity_selection_message(entities)
        self.assertEqual(result, "Found a Crystal Shard, a Ancient Relic, and a Elder Oak nearby:")

    def test_mixed_quantities(self):
        """Test with mixed quantities of different entities."""
        entities = [
            ("Crystal Shard", VectorN(0, 0, 0), "blue"),
            ("Crystal Shard", VectorN(1, 0, 0), "blue"),
            ("Ancient Relic", VectorN(0, 1, 0), "red")
        ]
        result = _generate_entity_selection_message(entities)
        self.assertEqual(result, "Found 2 Crystal Shards and a Ancient Relic nearby:")


if __name__ == "__main__":
    unittest.main() 