"""
Demo script for the pickup items functionality.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

from lithicrivers.game.game import Game, DroppedItem, Items
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SEED


def demo_pickup_functionality():
    """Demonstrate the pickup functionality."""
    print("=== LithicRivers Pickup Demo ===")
    print()
    
    # Create a game instance
    game = Game(seed=DEFAULT_SEED)
    
    print(f"Player starting position: {game.player.position}")
    print(f"Initial inventory: {game.player.inventory.summary()}")
    print()
    
    # Create some dropped items near the player
    print("Creating dropped items...")
    
    # Item 1: Acorn at (1, 0, 0)
    acorn = Items.acorn()
    acorn_dropped = DroppedItem(acorn, VectorN(1, 0, 0))
    game.world.add_entity(acorn_dropped)
    print(f"  - Dropped {acorn.name} at (1, 0, 0)")
    
    # Item 2: Stick at (0, 1, 0)
    stick = Items.stick()
    stick_dropped = DroppedItem(stick, VectorN(0, 1, 0))
    game.world.add_entity(stick_dropped)
    print(f"  - Dropped {stick.name} at (0, 1, 0)")
    
    # Item 3: Log at (1, 1, 0)
    log = Items.log()
    log_dropped = DroppedItem(log, VectorN(1, 1, 0))
    game.world.add_entity(log_dropped)
    print(f"  - Dropped {log.name} at (1, 1, 0)")
    
    print()
    
    # Show adjacent entities
    print("Checking adjacent entities...")
    adjacent_entities = game.world.get_adjacent_entities(game.player.position)
    print(f"Found {len(adjacent_entities)} adjacent entities:")
    
    for name, pos, color in adjacent_entities:
        print(f"  - {name} at {pos} (color: {color})")
    
    print()
    
    # Filter for dropped items only
    print("Filtering for dropped items only...")
    dropped_items = []
    
    for name, pos, color in adjacent_entities:
        entities = game.world.get_entities(pos)
        for entity in entities:
            if hasattr(entity, 'item') and hasattr(entity, 'name'):
                dropped_items.append((entity.name, pos, color, entity))
    
    print(f"Found {len(dropped_items)} dropped items:")
    for name, pos, color, entity in dropped_items:
        print(f"  - {name} at {pos}")
    
    print()
    
    # Demonstrate picking up all items
    print("Picking up all items...")
    picked_up_items = []
    
    for name, pos, color, entity in dropped_items:
        # Add to inventory
        game.player.inventory.add_item(entity.item)
        # Remove from world
        game.world.remove_entity(entity)
        picked_up_items.append(name)
        print(f"  - Picked up {name}")
    
    print()
    print(f"Final inventory: {game.player.inventory.summary()}")
    print(f"Items picked up: {', '.join(picked_up_items)}")
    
    # Verify items are removed from world
    print()
    print("Verifying items are removed from world...")
    adjacent_entities_after = game.world.get_adjacent_entities(game.player.position)
    dropped_items_after = []
    
    for name, pos, color in adjacent_entities_after:
        entities = game.world.get_entities(pos)
        for entity in entities:
            if hasattr(entity, 'item') and hasattr(entity, 'name'):
                dropped_items_after.append((entity.name, pos, color, entity))
    
    print(f"Remaining dropped items: {len(dropped_items_after)}")
    
    print()
    print("=== Demo Complete ===")
    print()
    print("In the actual game:")
    print("- Press 'g' to open the pickup menu")
    print("- Choose 'Pick Up All' to grab all nearby items")
    print("- Or choose individual items to pick up")
    print("- Items are automatically added to your inventory")


if __name__ == "__main__":
    demo_pickup_functionality() 