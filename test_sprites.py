#!/usr/bin/env python3
"""
Test script to verify the new 2x and 3x sprites for Elder Oak, Crystal Shard, and Ancient Relic.
"""

import sys
import os
sys.path.append(os.path.dirname(os.path.abspath(__file__)))

from lithicrivers.game import Entities, VectorN

def test_sprites():
    """Test the sprite rendering for all three entities."""
    
    # Create the entities
    elder_oak = Entities.starter_npc()
    crystal_shard = Entities.test_entity1()
    ancient_relic = Entities.test_entity2()
    
    print("=== Testing Elder Oak (NPC) ===")
    print(f"1x sprite:\n{elder_oak.render_sprite(1)}")
    print(f"2x sprite:\n{elder_oak.render_sprite(2)}")
    print(f"3x sprite:\n{elder_oak.render_sprite(3)}")
    
    print("\n=== Testing Crystal Shard (InteractiveEntity) ===")
    print(f"1x sprite:\n{crystal_shard.render_sprite(1)}")
    print(f"2x sprite:\n{crystal_shard.render_sprite(2)}")
    print(f"3x sprite:\n{crystal_shard.render_sprite(3)}")
    
    print("\n=== Testing Ancient Relic (InteractiveEntity) ===")
    print(f"1x sprite:\n{ancient_relic.render_sprite(1)}")
    print(f"2x sprite:\n{ancient_relic.render_sprite(2)}")
    print(f"3x sprite:\n{ancient_relic.render_sprite(3)}")
    
    print("\n=== Color Information ===")
    print(f"Elder Oak color: {elder_oak.color}")
    print(f"Crystal Shard color: {crystal_shard.color}")
    print(f"Ancient Relic color: {ancient_relic.color}")

if __name__ == "__main__":
    test_sprites() 