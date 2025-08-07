"""
Game logic for LithicRivers.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import logging
import os
import pickle
import pprint
import random
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path
from typing import Optional, Union, Protocol, TypeVar, Callable

from lithicrivers.colors import COLOR_MANAGER
from lithicrivers.constants import VEC_EAST, VEC_NORTH, VEC_SOUTH, VEC_WEST
from lithicrivers.model.generictype import T
from lithicrivers.model.model import RenderedData, Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.model.body import Body, BodyPartState
from lithicrivers.settings import (
    DEFAULT_PLAYER_NAME,
    DEFAULT_PLAYER_POSITION,
    DEFAULT_VIEWPORT,
)
from lithicrivers.textutil import get_color_for_item, get_color_for_tile

# Event system for entity movement
class EntityEvent:
    """Base class for entity events."""
    pass

class EntityMovedEvent(EntityEvent):
    """Event fired when an entity moves."""
    def __init__(self, entity: "Entity", old_position: VectorN, new_position: VectorN):
        self.entity = entity
        self.old_position = old_position
        self.new_position = new_position

class EntityListener:
    """Interface for objects that listen to entity events."""
    def on_entity_moved(self, event: EntityMovedEvent) -> None:
        """Called when an entity moves."""
        pass


def generate_sprite_repeat(char: str, scale: int = 1) -> str:
    normalized_scale = scale - 1

    if normalized_scale == 0:
        return char

    ret = ""
    for i in range(0, scale):
        ret += char * scale
        if i < (scale - 1):
            ret += "\n"

    return ret


class SpriteRenderable:
    def __init__(self, sprite_sheet: list[str]):
        self.sprite_sheet = sprite_sheet
        if not sprite_sheet:
            self.sprite_sheet = ["?", "??\n??", "???\n???\n???"]

    def render_sprite(self, scale: int = 1) -> str:
        normalized_scale = scale - 1

        if normalized_scale < 0:
            raise Exception(
                f"Cannot render {self} with normalized_scale = {normalized_scale}"
            )

        if normalized_scale >= len(self.sprite_sheet):
            # if they ask for a sprite too large, give them '?'
            return generate_sprite_repeat("?", scale)

            # raise Exception("Cannot render sprite with scale {} as it only has these sprites:\n{} ".format(
            #     len(self.sprite_sheet),
            #     self.sprite_sheet
            # ))

        return self.sprite_sheet[normalized_scale]


class ItemArtRenderable:
    """Extend this class if you want to be able to render item art."""

    def __init__(self, item_art: str):
        self.item_art = item_art

    @staticmethod
    def blank_item() -> str:
        """Return a 12x8 blank (all spaces) ASCII art string."""
        return (
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            "
        )

    @staticmethod
    def missing_texture_item() -> str:
        """Return a 12x8 'missing texture' ASCII art string."""
        return (
            "  ╭────────╮  \n"
            "  │████████│  \n"
            "  │████████│  \n"
            "  │███??███│  \n"
            "  │███??███│  \n"
            "  │████████│  \n"
            "  │████████│  \n"
            "  ╰────────╯  "
        )


class Entity:
    def __init__(self, name: str, position: VectorN):
        self.name: str = name
        self.position: VectorN = position
        self.health: int = 100
        self.stamina: int = 100
        self._listeners: list[EntityListener] = []
    
    def __getstate__(self):
        """Custom pickle serialization that excludes listeners."""
        state = self.__dict__.copy()
        # Remove listeners - they'll be re-established when the world is loaded
        state['_listeners'] = []
        return state
    
    def __setstate__(self, state):
        """Custom pickle deserialization that initializes empty listeners."""
        self.__dict__.update(state)
        # Ensure listeners list exists (will be populated by World after loading)
        if '_listeners' not in self.__dict__:
            self._listeners = []

    def add_listener(self, listener: EntityListener) -> None:
        """Add an event listener."""
        self._listeners.append(listener)

    def remove_listener(self, listener: EntityListener) -> None:
        """Remove an event listener."""
        if listener in self._listeners:
            self._listeners.remove(listener)

    def _notify_moved(self, old_position: VectorN, new_position: VectorN) -> None:
        """Notify listeners of movement."""
        event = EntityMovedEvent(self, old_position, new_position)
        for listener in self._listeners:
            listener.on_entity_moved(event)

    def tick(self) -> None:
        """Called each game tick. Override in subclasses."""
        pass

    def move(self, vec: VectorN) -> None:
        """Move the entity and notify listeners."""
        old_position = self.position
        self.position += vec
        self._notify_moved(old_position, self.position)

    def calc_offset(self, vec: VectorN) -> VectorN:
        """Where would I move, if I did move?"""
        return self.position + vec

    def move_north(self) -> None:
        self.move(VEC_NORTH)

    def move_south(self) -> None:
        self.move(VEC_SOUTH)

    def move_west(self) -> None:
        self.move(VEC_WEST)

    def move_east(self) -> None:
        self.move(VEC_EAST)


class NPC(Entity, SpriteRenderable):
    """A non-player character that can have conversations."""

    def __init__(
        self, name: str, position: VectorN, sprite: str = "N", color: str = "cyan"
    ):
        super().__init__(name, position)
        self.sprite = sprite
        self.color = color
        self.conversations = {}
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(name.lower().replace(" ", "_"), "entities")
        
        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color
        self._setup_default_conversation()

    def _setup_default_conversation(self) -> None:
        self.conversations = {
            "greeting": {
                "text": f"Hello, I am {self.name}.",
                "options": ["Goodbye"],
            },
            "goodbye": {
                "text": "Goodbye!",
                "options": [],
            },
        }

    def get_conversation(self, topic: str = "greeting"):
        return self.conversations.get(topic, self.conversations["greeting"])

    def handle_response(self, response: str, topic: str = "greeting"):
        if response == "Goodbye":
            return "goodbye"
        return "greeting"


class ElderOak(NPC):
    def __init__(self, position: VectorN):
        super().__init__("Elder Oak", position, sprite="N", color="cyan")
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("elder_oak", "entities")
        
        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color
        self._setup_default_conversation()

    def _setup_default_conversation(self) -> None:
        self.conversations = {
            "greeting": {
                "text": f"Hello, traveler! I am {self.name}. Welcome to LithicRivers!",
                "options": ["Tell me about this world", "What can you do?", "Goodbye"],
            },
            "about_world": {
                "text": "This is a world of endless possibilities. You can mine, build, and explore to your heart's content. The world is procedurally generated, so there's always something new to discover!",
                "options": [
                    "Tell me more about mining",
                    "What about building?",
                    "Back to greeting",
                ],
            },
            "about_mining": {
                "text": "Mining is simple! Just press 'u' when standing on a mineable tile like Gold Ore. You'll get valuable resources that you can use for crafting and trading.",
                "options": ["What about building?", "Back to greeting"],
            },
            "about_building": {
                "text": "Building is coming soon! You'll be able to place blocks and create structures. For now, focus on gathering resources through mining.",
                "options": ["Tell me more about mining", "Back to greeting"],
            },
            "goodbye": {
                "text": "Farewell, traveler! May your adventures be fruitful!",
                "options": ["OK"],
            },
        }

    def handle_response(self, response: str, topic: str = "greeting"):
        if response == "Tell me about this world":
            return "about_world"
        elif response == "What can you do?" or response == "Tell me more about mining":
            return "about_mining"
        elif response == "What about building?":
            return "about_building"
        elif response == "Back to greeting":
            return "greeting"
        elif response == "Goodbye" or response == "OK":
            return "goodbye"
        else:
            return "greeting"


class InteractiveEntity(Entity, SpriteRenderable):
    """An entity that can be interacted with."""

    def __init__(
        self,
        name: str,
        position: VectorN,
        sprite: str = "E",
        color: str = "yellow",
        interaction_text: str = "This is an interactive entity.",
    ):
        super().__init__(name, position)
        self.sprite = sprite
        self.color = color
        self.interaction_text = interaction_text

        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(name.lower().replace(" ", "_"), "entities")
        
        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color

    def render_sprite(self, scale: int = 1) -> str:
        """Render the entity sprite."""
        # Use the SpriteRenderable's render_sprite method
        return super().render_sprite(scale)

    def interact(self) -> str:
        """Handle interaction with this entity."""
        return self.interaction_text


class CrystalShard(InteractiveEntity):
    def __init__(self, position: VectorN):
        super().__init__(
            "Crystal Shard",
            position,
            sprite="C",
            color="blue",
            interaction_text="This crystal shard glows with a soft blue light. It seems to pulse with energy.",
        )
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("crystal_shard", "entities")
        
        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color


class AncientRelic(InteractiveEntity):
    def __init__(self, position: VectorN):
        super().__init__(
            "Ancient Relic",
            position,
            sprite="R",
            color="red",
            interaction_text="This ancient relic is covered in mysterious runes. It radiates warmth.",
        )
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("ancient_relic", "entities")
        
        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color


class StumblingSheep(InteractiveEntity):
    """A sheep that stumbles around randomly."""

    def __init__(self, position: VectorN):
        super().__init__(
            name="Stumbling Sheep",
            position=position,
            sprite="S",
            color="white",
            interaction_text="The sheep stumbles around aimlessly, occasionally making confused noises.",
        )
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("stumbling_sheep", "entities")
        
        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color
        self.speed = 0.2  # Sheep moves at 0.2x speed (tick every 5 frames)

    def tick(self) -> None:
        """Move randomly every few ticks."""
        import random
        
        # Use deterministic randomness based on world seed and tick
        # This ensures the same behavior for the same seed
        # Get gametick from world if available, otherwise use 0
        gametick = 0
        if hasattr(self, '_listeners'):
            for listener in self._listeners:
                if hasattr(listener, 'gametick'):
                    gametick = listener.gametick
                    break
        
        # Only use deterministic seeding if not in a test environment
        # This allows mocking to work in tests
        if not hasattr(random, '_test_mode'):
            random.seed(f"sheep_{self.position.serialize()}_{gametick}")
        
        if random.random() < 0.1:  # 10% chance to move each tick
            directions = [VEC_NORTH, VEC_SOUTH, VEC_EAST, VEC_WEST]
            direction = random.choice(directions)
            self.move(direction)


class Fluid(Entity, SpriteRenderable):
    """
    A fluid entity that can flow and spread across the world.
    Fluids don't replace blocks but exist as separate entities.
    """
    
    def __init__(self, fluid_type: str, position: VectorN, amount: float = 1.0, viscosity: float = 1.0):
        super().__init__(name=f"{fluid_type}_fluid", position=position)
        self.fluid_type = fluid_type
        self.amount = amount  # Amount of fluid (0.0 to 1.0)
        self.viscosity = viscosity  # How slowly the fluid flows (higher = slower)
        self.max_amount = 1.0  # Maximum amount per tile
        self.spread_threshold = 0.8  # Amount at which fluid starts spreading
        # Initialize sprite sheet after fluid_type is set
        SpriteRenderable.__init__(self, self.get_sprites())
        

    def render_sprite(self, scale: int = 1) -> str:
        """Render the fluid sprite."""
        sprites = self.get_sprites()
        if self.fluid_type not in sprites:
            raise ValueError(f"Fluid type '{self.fluid_type}' not found in sprite data. Available types: {list(sprites.keys())}")
        
        sprite_list = sprites[self.fluid_type]
        if scale <= 0 or scale > len(sprite_list):
            raise ValueError(f"Scale {scale} is out of bounds for fluid '{self.fluid_type}'. Valid range: 1-{len(sprite_list)}")
        
        return sprite_list[scale-1]

    def get_sprites(self) -> dict[str, list[str]]:
        """Get all possible sprite representations of this fluid (for different scales, 1x1, 2x2, 3x3, etc.)"""
        # Try to load from external sprite data first
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(self.fluid_type, "fluids")
        
        if sprite_data:
            # Use external sprite data
            return {self.fluid_type: sprite_data.sprites}
        
        # No fallback - throw exception if external data not found
        raise ValueError(f"External sprite data not found for fluid type '{self.fluid_type}' in 'fluids' category")
    
    def get_color(self) -> str:
        """Get the color for this fluid."""
        # Try to load from external sprite data first
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(self.fluid_type, "fluids")
        
        if sprite_data:
            # Use external sprite data
            return sprite_data.color
        
        # No fallback - throw exception if external data not found
        raise ValueError(f"External sprite data not found for fluid type '{self.fluid_type}' in 'fluids' category")
    
    def tick(self) -> None:
        """Process fluid physics each tick."""
        # Fluid physics are handled by FluidManager
        pass
    
    def copy(self) -> "Fluid":
        """Create a copy of this fluid."""
        return Fluid(self.fluid_type, self.position, self.amount, self.viscosity)


class FluidManager:
    """
    Manages fluid physics and spreading across the world.
    All fluid calculations are deterministic based on world seed and tick.
    """
    
    def __init__(self, world: "World"):
        self.world = world
        self.fluids: dict[str, Fluid] = {}  # position_key -> Fluid
        self.flow_directions = [
            VectorN(0, 0, 1),   # Down (gravity) - deeper into earth
            VectorN(-1, 0, 0),  # Left
            VectorN(1, 0, 0),   # Right
            VectorN(0, -1, 0),  # North
            VectorN(0, 1, 0),   # South
        ]
    
    def add_fluid(self, fluid: Fluid) -> None:
        """Add a fluid to the manager."""
        pos_key = self._get_position_key(fluid.position)
        if pos_key in self.fluids:
            # Merge with existing fluid
            existing = self.fluids[pos_key]
            if existing.fluid_type == fluid.fluid_type:
                total_amount = existing.amount + fluid.amount
                if total_amount <= existing.max_amount:
                    existing.amount = total_amount
                else:
                    # Overflow - create new fluid entities
                    existing.amount = existing.max_amount
                    overflow = total_amount - existing.max_amount
                    if overflow > 0:
                        # Create overflow fluid that will spread
                        overflow_fluid = Fluid(fluid.fluid_type, fluid.position, overflow, fluid.viscosity)
                        self.fluids[pos_key] = overflow_fluid
        else:
            self.fluids[pos_key] = fluid
    
    def remove_fluid(self, position: VectorN) -> None:
        """Remove fluid from a position."""
        pos_key = self._get_position_key(position)
        if pos_key in self.fluids:
            del self.fluids[pos_key]
    
    def get_fluid(self, position: VectorN) -> Optional[Fluid]:
        """Get fluid at a position."""
        pos_key = self._get_position_key(position)
        return self.fluids.get(pos_key)
    
    def process_fluids(self, gametick: int) -> None:
        """Process all fluid physics for a given tick."""
        # Use deterministic randomness based on world seed and tick
        import random
        random.seed(f"fluids_{self.world.seed}_{gametick}")
        
        # Create a copy of fluids to avoid modifying during iteration
        fluids_to_process = list(self.fluids.items())
        
        for pos_key, fluid in fluids_to_process:
            if fluid.amount <= 0:
                # Remove empty fluids
                del self.fluids[pos_key]
                continue
            
            # Check if fluid should spread
            if fluid.amount >= fluid.spread_threshold:
                self._spread_fluid(fluid, gametick)
    
    def _spread_fluid(self, fluid: Fluid, gametick: int) -> None:
        """Spread fluid to adjacent tiles based on physics."""
        import random
        
        # Use deterministic randomness for this specific fluid
        random.seed(f"fluid_spread_{fluid.position.serialize()}_{gametick}")
        
        # Calculate how much to spread
        spread_amount = fluid.amount - fluid.spread_threshold
        fluid.amount = fluid.spread_threshold
        
        # Try to spread to adjacent positions
        valid_targets = []
        
        for direction in self.flow_directions:
            target_pos = fluid.position + direction
            target_tile = self.world.get_tile(target_pos)
            
            # Check if target position can hold fluid
            if self._can_hold_fluid(target_pos, target_tile):
                valid_targets.append(target_pos)
        
        if not valid_targets:
            return
        
        # Distribute fluid among valid targets
        amount_per_target = spread_amount / len(valid_targets)
        
        for target_pos in valid_targets:
            # Create new fluid at target position
            new_fluid = Fluid(fluid.fluid_type, target_pos, amount_per_target, fluid.viscosity)
            self.add_fluid(new_fluid)
    
    def _can_hold_fluid(self, position: VectorN, tile: Optional["Tile"]) -> bool:
        """Check if a position can hold fluid."""
        if tile is None:
            return True  # Empty space can hold fluid
        
        # Check if tile is solid (can't hold fluid)
        solid_tiles = ["bedrock", "door"]
        if tile.tileid.lower() in solid_tiles:
            return False
        
        # Check if there's already too much fluid at this position
        existing_fluid = self.get_fluid(position)
        if existing_fluid and existing_fluid.amount >= existing_fluid.max_amount:
            return False
        
        return True
    
    def _get_position_key(self, pos: VectorN) -> str:
        """Get a string key for a position."""
        return pos.serialize()
    
    def get_all_fluids(self) -> list[Fluid]:
        """Get all fluids in the manager."""
        return list(self.fluids.values())
    
    def clear(self) -> None:
        """Clear all fluids."""
        self.fluids.clear()


class Entities:
    @staticmethod
    def stumbling_sheep(position: VectorN = VectorN(0, 0, 0)) -> "StumblingSheep":
        return StumblingSheep(position)

    @staticmethod
    def starter_npc(position: VectorN = VectorN(5, 5, 0)) -> "ElderOak":
        return ElderOak(position)

    @staticmethod
    def test_entity1(position: VectorN = VectorN(6, 5, 0)) -> "CrystalShard":
        return CrystalShard(position)

    @staticmethod
    def test_entity2(position: VectorN = VectorN(5, 6, 0)) -> "AncientRelic":
        return AncientRelic(position)
    
    @staticmethod
    def water(position: VectorN = VectorN(0, 0, 0), amount: float = 1.0) -> "Fluid":
        return Fluid("water", position, amount, viscosity=100.0)
    
    @staticmethod
    def lava(position: VectorN = VectorN(0, 0, 0), amount: float = 1.0) -> "Fluid":
        return Fluid("lava", position, amount, viscosity=200.0)  # Lava flows slower
    
    @staticmethod
    def acid(position: VectorN = VectorN(0, 0, 0), amount: float = 1.0) -> "Fluid":
        return Fluid("acid", position, amount, viscosity=150.0)
    
    @staticmethod
    def oil(position: VectorN = VectorN(0, 0, 0), amount: float = 1.0) -> "Fluid":
        return Fluid("oil", position, amount, viscosity=50.0)  # Oil flows faster
    
    @staticmethod
    def blood(position: VectorN = VectorN(0, 0, 0), amount: float = 1.0) -> "Fluid":
        return Fluid("blood", position, amount, viscosity=120.0)


class Items:
    """
    A bunch of default items.
    """

    @staticmethod
    def rock() -> "Item":
        return Item("Rock")

    @staticmethod
    def gold_nugget() -> "Item":
        return Item("Gold Nugget")

    @staticmethod
    def stick() -> "Item":
        return Item("Stick")

    @staticmethod
    def diamond() -> "Item":
        return Item("Diamond")

    @staticmethod
    def log() -> "Item":
        return Item("Log")

    @staticmethod
    def acorn() -> "Item":
        return Item("Acorn")

    @staticmethod
    def iron_scrap() -> "Item":
        return Item("Iron Scrap")

    @staticmethod
    def scrap_electronics() -> "Item":
        return Item("Scrap Electronics")


class DroppedItem(Entity, SpriteRenderable):
    """An item that has been dropped in the world."""
    
    def __init__(self, item: "Item", position: VectorN):
        Entity.__init__(self, item.name, position)
        SpriteRenderable.__init__(self, item.sprite_sheet)
        self.item = item  # Store the actual item object
        self.color = "yellow"  # Default color for dropped items


class Item(ItemArtRenderable, SpriteRenderable):
    def __init__(self, name: str, sprite_sheet: Optional[list[str]] = None):
        # Load sprites from external data if not provided
        if sprite_sheet is None:
            from lithicrivers.sprite_loader import get_sprite_loader
            
            sprite_loader = get_sprite_loader()
            sprite_data = sprite_loader.load_sprite(name.lower().replace(" ", "_"), "items")
            sprite_sheet = sprite_data.sprites
        
        SpriteRenderable.__init__(self, sprite_sheet)
        self.name = name


class Inventory:
    def __init__(self, items: Optional[list[Item]] = None):
        if items is None:
            items = []

        self.itemsdata = items

    def add_item(self, item: "Item") -> None:
        self.itemsdata.append(item)

    def __str__(self) -> str:
        return f"<Inventory numItems={len(self.itemsdata)} summary={self.summary()}>"

    def count_items(self) -> dict[str, int]:
        d = {}

        for item in self.itemsdata:
            if item.name in d:
                d[item.name] += 1
            else:
                d[item.name] = 1

        return d

    def summary(self) -> str:
        s = ""

        for k, v in self.count_items().items():
            s += f"{k}={v}, "

        return s[0 : len(s) - 2]

    def colored_summary(self) -> str:
        """Generate a colored summary of inventory items."""
        s = ""

        for k, v in self.count_items().items():
            # Get color for this item
            item_color = get_color_for_item(k)
            color_name = COLOR_MANAGER.get_color_name(item_color)
            s += f"{k}={v} ({color_name}), "

        return s[0 : len(s) - 2] if s else "Empty"


class Player(Entity, SpriteRenderable):
    def __init__(self, name):
        Entity.__init__(self, name=name, position=DEFAULT_PLAYER_POSITION)
        
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader
        
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("player", "entities")
        
        # Use external sprite data
        SpriteRenderable.__init__(self, sprite_data.sprites)

        self.inventory = Inventory([Item("Cookie", ["o"])])
        self.body = Body()  # Initialize with damaged android body
        
        # Apply body modifiers to base stats
        self._update_stats_from_body()
        
        # Add debug items for body repairs
        self._add_debug_repair_items()

    def _update_stats_from_body(self) -> None:
        """Update player stats based on body condition."""
        health_modifier = self.body.get_total_health_modifier()
        stamina_modifier = self.body.get_total_stamina_modifier()
        
        # Apply modifiers to base stats (100 each)
        self.health = max(1, 100 + health_modifier)
        self.stamina = max(1, 100 + stamina_modifier)

    def _add_debug_repair_items(self) -> None:
        """Add double the necessary items to repair all body parts for debug purposes."""
        repair_requirements = self.body.get_repair_requirements()
        
        # Count total items needed
        total_items_needed = {}
        for part_type, costs in repair_requirements.items():
            for item_name, amount in costs.items():
                if item_name in total_items_needed:
                    total_items_needed[item_name] += amount
                else:
                    total_items_needed[item_name] = amount
        
        # Add double the required items to inventory
        for item_name, amount in total_items_needed.items():
            # Create the item based on name
            if item_name == "iron_scrap":
                item = Items.iron_scrap()
            elif item_name == "scrap_electronics":
                item = Items.scrap_electronics()
            else:
                # Fallback for unknown items
                item = Item(item_name, ["?"])
            
            # Add double the amount needed
            for _ in range(amount * 2):
                self.inventory.add_item(item)

    def get_walk_speed_modifier(self) -> float:
        """Get the player's walk speed modifier based on body condition."""
        return self.body.get_total_walk_speed_modifier()

    def get_break_speed_modifier(self) -> float:
        """Get the player's break speed modifier based on body condition."""
        return self.body.get_total_break_speed_modifier()

    def can_perform_action(self, action_type: str) -> bool:
        """Check if the player can perform a specific action based on body condition."""
        return self.body.can_perform_action(action_type)

    def get_action_speed(self, action_type: str) -> float:
        """Get the speed modifier for a specific action."""
        return self.body.get_action_speed(action_type)

    def get_body_status_summary(self) -> str:
        """Get a summary of the player's body condition."""
        return self.body.get_body_status_summary()

    def get_movement_penalty_description(self) -> str:
        """Get a description of current movement penalties."""
        return self.body.get_movement_penalty_description()

    def repair_body_part(self, part_type: str) -> bool:
        """Attempt to repair a body part. Returns True if successful."""
        # This will be implemented when we add crafting
        # For now, just return False
        return False

    def tick(self) -> None:
        """Called each game tick. Override when I add poison damage, for example."""
        pass


class Tile(SpriteRenderable):
    def __init__(
        self,
        tileid: str,
        desc: Optional[str] = None,
        sprite_sheet: Optional[list[str]] = None,
        drops: Optional[dict[float, Item]] = None,
    ):
        # Load sprites from external data if not provided
        if sprite_sheet is None:
            from lithicrivers.sprite_loader import get_sprite_loader
            
            sprite_loader = get_sprite_loader()
            sprite_data = sprite_loader.load_sprite(tileid.lower().replace(" ", "_"), "tiles")
            sprite_sheet = sprite_data.sprites
        
        SpriteRenderable.__init__(self, sprite_sheet)
        self.tileid = tileid
        self.description = desc
        self.drops = drops

    def __str__(self) -> str:
        return f"<Tile '{self.tileid}': [{self.render_sprite(1)}]>"

    def __eq__(self, other: object) -> bool:
        if other is None:
            return False
        return self.tileid == other.tileid

    def __hash__(self) -> int:
        return hash(self.tileid)

    def calc_drop(self) -> "Item":
        return weighted_choice_dict(self.drops)

    def calc_tree_drops(self) -> list["Item"]:
        """
        Special drop calculation for trees that guarantees 1-3 acorns.
        90% chance of exactly 1 acorn, 10% chance of 2-3 acorns.
        """
        import random

        # Determine number of acorns (1-3, with 90% chance of 1)
        num_acorns = 1 if random.random() < 0.9 else random.randint(2, 3)

        # Create list of items to return
        items = []

        # Add guaranteed acorns
        for _ in range(num_acorns):
            items.append(Items.acorn())

        # Add other possible drops (stick, log) with original probabilities
        if random.random() < 0.5:
            items.append(Items.stick())
        if random.random() < 0.3:
            items.append(Items.log())

        return items


def weighted_choice(weights: list[float], choices: list[T]) -> T:
    if len(weights) != len(choices):
        ve = ValueError(
            f"Weights={weights} and choices={choices} for {weighted_choice.__name__}() must be the same length!"
        )
        logging.error(ve)
        raise ve

    # Normalize weights to sum to 1
    total_weight = sum(weights)
    if total_weight == 0:
        raise ValueError("Weights cannot all be zero")

    normalized_weights = [w / total_weight for w in weights]

    # Use random.choices for weighted selection
    return random.choices(choices, weights=normalized_weights, k=1)[0]


def weighted_choice_dict(dict_weight: dict[float, T]) -> T:
    weights = []
    choices = []
    for k, v in dict_weight.items():
        weights.append(k)
        choices.append(v)
    return weighted_choice(weights, choices)


class Tiles:
    """
    A bunch of default tiles.
    """

    @staticmethod
    def dirt() -> "Tile":
        return Tile(
            "Dirt",
            drops={0.99: Items.rock(), 0.01: Items.gold_nugget()},
        )

    @staticmethod
    def tree() -> "Tile":
        return Tile(
            "Tree",
            drops={0.50: Items.stick(), 0.30: Items.log(), 0.20: Items.acorn()},
        )

    @staticmethod
    def gold_ore() -> "Tile":
        return Tile(
            "Gold Ore",
            drops={0.9: Items.gold_nugget(), 0.1: Items.diamond()},
        )

    @staticmethod
    def cloud() -> "Tile":
        return Tile("Cloud")

    @staticmethod
    def bedrock() -> "Tile":
        return Tile("Bedrock")

    @staticmethod
    def empty() -> "Tile":
        return Tile("Empty")

    @staticmethod
    def iron_scrap() -> "Tile":
        return Tile(
            "Iron Scrap",
            drops={0.8: Items.iron_scrap(), 0.2: Items.gold_nugget()},
        )

    @staticmethod
    def bone_block() -> "Tile":
        return Tile(
            "Bone Block",
            drops={0.7: Items.rock(), 0.3: Items.gold_nugget()},
        )

    @staticmethod
    def door() -> "Tile":
        return Tile(
            "Door",
            drops={0.5: Items.rock()},
        )

    @staticmethod
    def scrap_electronics() -> "Tile":
        return Tile(
            "Scrap Electronics",
            drops={0.6: Items.scrap_electronics(), 0.4: Items.gold_nugget()},
        )

    @staticmethod
    def treasure() -> "Tile":
        return Tile(
            "buried_treasure",
            drops={0.3: Items.gold_nugget(), 0.7: Items.diamond()},
        )


class TilePalette:
    """
    Efficient storage for tile types using integer IDs.
    Similar to Minecraft's block palette system.
    """

    def __init__(self) -> None:
        self.tile_to_id = {}  # tileid -> int
        self.id_to_tile = {}  # int -> Tile
        self.next_id = 0
        self._empty_tile = None

    def get_empty_tile(self) -> Tile:
        """Get the empty tile (ID 0) - used for ungenerated areas."""
        if self._empty_tile is None:
            self._empty_tile = Tiles.empty()
        return self._empty_tile

    def get_id(self, tile: Tile) -> int:
        """Get the integer ID for a tile, creating it if needed."""
        if tile is None:
            return 0  # Empty tile is always ID 0

        # Use tileid as the key instead of the Tile object to avoid hash issues
        tile_key = tile.tileid if hasattr(tile, "tileid") else str(tile)

        if tile_key not in self.tile_to_id:
            self.tile_to_id[tile_key] = self.next_id
            self.id_to_tile[self.next_id] = tile
            self.next_id += 1
        return self.tile_to_id[tile_key]

    def get_tile(self, tile_id: int) -> Tile:
        """Get the tile for a given integer ID."""
        if tile_id == 0:
            return self.get_empty_tile()
        return self.id_to_tile.get(tile_id, self.get_empty_tile())

    def __len__(self) -> int:
        """Number of unique tile types in the palette."""
        return len(self.tile_to_id)


class Chunk:
    """
    A 3D chunk of the world, storing tile data efficiently.
    Similar to Minecraft's chunk system.
    """

    def __init__(self, size: int = 16):
        self.size = size
        self.palette = TilePalette()
        # 3D array of tile IDs (integers)
        self.blocks = [[[0] * size for _ in range(size)] for _ in range(size)]
        self.is_generated = False

    def get_local_pos(self, world_pos: VectorN) -> tuple[int, int, int]:
        """Convert world position to local chunk position."""
        return (
            world_pos.x % self.size,
            world_pos.y % self.size,
            world_pos.z % self.size,
        )

    def get_tile(self, local_pos: tuple[int, int, int]) -> Tile:
        """Get tile at local position within this chunk."""
        tile_id = self.blocks[local_pos[0]][local_pos[1]][local_pos[2]]
        return self.palette.get_tile(tile_id)

    def set_tile(self, local_pos: tuple[int, int, int], tile: Tile) -> None:
        """Set tile at local position within this chunk."""
        tile_id = self.palette.get_id(tile)
        self.blocks[local_pos[0]][local_pos[1]][local_pos[2]] = tile_id

    def is_empty(self) -> bool:
        """Check if chunk is completely empty (all blocks are ID 0)."""
        for x in range(self.size):
            for y in range(self.size):
                for z in range(self.size):
                    if self.blocks[x][y][z] != 0:
                        return False
        return True

from lithicrivers.worldgen import SeededWorldGenerator

class ChunkedWorldData:
    """
    Efficient world data storage using chunked 3D arrays.
    Similar to Minecraft's world storage system.
    """

    def __init__(self, chunk_size: int = None, world_generator: "SeededWorldGenerator" =None):
        from lithicrivers.settings import CHUNK_SIZE
        if chunk_size is None:
            chunk_size = CHUNK_SIZE
        self.chunk_size = chunk_size
        self.chunks = {}  # (chunk_x, chunk_y, chunk_z) -> Chunk
        self.entity_data = {}  # Entity storage
        self._tile_cache = {}  # Cache for frequently accessed tiles
        self._cache_size = 1000  # Max cache size
        self.world_generator = world_generator  # Reference to world generator for structure generation
        self._generated_chunks = set()  # Track which chunks have had structures generated
        self._chunk_generation_lock = threading.Lock()  # Lock for thread-safe chunk generation
        from lithicrivers.settings import MAX_CPU_THREADS
        self._thread_pool = ThreadPoolExecutor(max_workers=MAX_CPU_THREADS)  # Thread pool for chunk generation

import concurrent.futures
from concurrent.futures import ProcessPoolExecutor

def _generate_chunk_data_for_process(seed, chunk_x, chunk_y, chunk_z):
    # This function runs in a separate process
    from lithicrivers.model.vector import VectorN
    generator = SeededWorldGenerator(seed)
    generator._generate_complete_chunk(chunk_x, chunk_y, chunk_z)
    # Get chunk data from the generator's chunk cache
    chunk_data = generator.chunk_cache.cache.get((chunk_x, chunk_y, chunk_z), {})
    # Return as a dict mapping pos_str to tile (must be pickleable)
    return (chunk_x, chunk_y, chunk_z, chunk_data)

class ChunkedWorldData:
    """
    Efficient world data storage using chunked 3D arrays.
    Similar to Minecraft's world storage system.
    """

    def __init__(self, chunk_size: int = None, world_generator=None):
        from lithicrivers.settings import CHUNK_SIZE
        if chunk_size is None:
            chunk_size = CHUNK_SIZE
        self.chunk_size = chunk_size
        self.chunks = {}  # (chunk_x, chunk_y, chunk_z) -> Chunk
        self.entity_data = {}  # Entity storage
        self._tile_cache = {}  # Cache for frequently accessed tiles
        self._cache_size = 1000  # Max cache size
        self.world_generator = world_generator  # Reference to world generator for structure generation
        self._generated_chunks = set()  # Track which chunks have had structures generated
        self._chunk_generation_lock = threading.Lock()  # Lock for thread-safe chunk generation
        from lithicrivers.settings import MAX_CPU_THREADS
        self._thread_pool = ThreadPoolExecutor(max_workers=MAX_CPU_THREADS)  # Thread pool for chunk generation

    def pregen_chunks(self, radius: int) -> None:
        """Pre-generate all chunks within a cubic radius around (0,0,0) using process pool. Waits for all processes to finish.
        This should only be called when the world generator is initialized, or during unit tests to speed them up if pickling after generating."""
        if not self.world_generator:
            raise Exception("World generator is not initialized")
        seed = getattr(self.world_generator, 'seed', None)
        if hasattr(seed, 'seed'):
            seed = seed.seed
        if seed is None:
            raise Exception("World generator must have a .seed or .seed.seed attribute for process pool pregen")
        chunk_size = self.chunk_size
        chunk_radius = (radius + chunk_size - 1) // chunk_size  # ceil division
        # Prepare all chunk coords to generate
        chunk_coords = [
            (cx, cy, cz)
            for cx in range(-chunk_radius, chunk_radius + 1)
            for cy in range(-chunk_radius, chunk_radius + 1)
            for cz in range(-chunk_radius, chunk_radius + 1)
            if (cx, cy, cz) not in self._generated_chunks
        ]
        # Mark as generated to avoid duplicate work
        for chunk_key in chunk_coords:
            self._generated_chunks.add(chunk_key)
        results = []
        with ProcessPoolExecutor() as pool:
            future_to_chunk = {
                pool.submit(_generate_chunk_data_for_process, seed, cx, cy, cz): (cx, cy, cz)
                for (cx, cy, cz) in chunk_coords
            }
            for future in concurrent.futures.as_completed(future_to_chunk):
                cx, cy, cz, chunk_data = future.result()
                # Insert chunk data into self.chunks
                from lithicrivers.model.vector import VectorN
                if (cx, cy, cz) not in self.chunks:
                    from lithicrivers.game import Chunk
                    self.chunks[(cx, cy, cz)] = Chunk(self.chunk_size)
                chunk = self.chunks[(cx, cy, cz)]
                for pos_str, tile in chunk_data.items():
                    pos_parts = [int(x) for x in pos_str.split(",")]
                    world_pos = VectorN(*pos_parts)
                    local_pos = chunk.get_local_pos(world_pos)
                    chunk.set_tile(local_pos, tile)
                    pos_key = (world_pos.x, world_pos.y, world_pos.z)
                    self._tile_cache[pos_key] = tile


    def __getstate__(self):
        state = self.__dict__.copy()
        # Remove unpickleable objects for pickling
        state.pop('_chunk_generation_lock', None)
        state.pop('_thread_pool', None)
        return state

    def __setstate__(self, state):
        self.__dict__.update(state)
        import threading
        from concurrent.futures import ThreadPoolExecutor
        from lithicrivers.settings import MAX_CPU_THREADS
        self._chunk_generation_lock = threading.Lock()
        self._thread_pool = ThreadPoolExecutor(max_workers=MAX_CPU_THREADS)

    def __deepcopy__(self, memo):
        """
        Custom deepcopy implementation that handles threading primitives properly.
        """
        import copy
        from lithicrivers.worldgen import SeededWorldGenerator
        
        # Create new ChunkedWorldData with same parameters
        cloned_data = ChunkedWorldData.__new__(ChunkedWorldData)
        cloned_data.chunk_size = self.chunk_size
        cloned_data._cache_size = self._cache_size
        
        # Create a new world generator with the same seed instead of deep copying
        if self.world_generator:
            cloned_data.world_generator = SeededWorldGenerator(self.world_generator.seed.seed)
        else:
            cloned_data.world_generator = None
        
        # Deep copy chunks
        cloned_data.chunks = copy.deepcopy(self.chunks, memo)
        
        # Deep copy other data structures
        cloned_data.entity_data = copy.deepcopy(self.entity_data, memo)
        cloned_data._tile_cache = copy.deepcopy(self._tile_cache, memo)
        cloned_data._generated_chunks = copy.deepcopy(self._generated_chunks, memo)
        
        # Create new threading primitives (can't be copied)
        cloned_data._chunk_generation_lock = threading.Lock()
        from lithicrivers.settings import MAX_CPU_THREADS
        cloned_data._thread_pool = ThreadPoolExecutor(max_workers=MAX_CPU_THREADS)
        
        return cloned_data

    def clone(self) -> "ChunkedWorldData":
        """
        Create a deep copy of this ChunkedWorldData for testing purposes.
        This is more efficient than regenerating all chunks from scratch.
        """
        import copy
        return copy.deepcopy(self)

    def get_chunk_key(self, pos: VectorN) -> tuple[int, int, int]:
        """Get chunk coordinates from world position."""
        return (
            pos.x // self.chunk_size,
            pos.y // self.chunk_size,
            pos.z // self.chunk_size,
        )

    def get_chunk(self, chunk_key: tuple[int, int, int]) -> Chunk:
        """Get or create a chunk."""
        if chunk_key not in self.chunks:
            self.chunks[chunk_key] = Chunk(self.chunk_size)
            # Generate structures for this chunk if it's the first time
            if self.world_generator and chunk_key not in self._generated_chunks:
                # Mark as generated first to prevent infinite recursion
                self._generated_chunks.add(chunk_key)
                # Use threaded chunk generation for lazy loading
                self._generate_chunk_threaded(chunk_key)
        return self.chunks[chunk_key]

    def _generate_chunk_threaded(self, chunk_key: tuple[int, int, int]) -> None:
        """
        Generate a chunk using threaded generation for lazy loading.
        
        Args:
            chunk_key: The chunk coordinates (chunk_x, chunk_y, chunk_z)
        """
        if not self.world_generator:
            return
            
        # Skip structure generation during testing to speed up tests
        if os.environ.get("TESTING") == "1":
            return
            
        chunk_x, chunk_y, chunk_z = chunk_key
        
        # Submit chunk generation to thread pool
        future = self._thread_pool.submit(self._generate_chunk_worker, chunk_x, chunk_y, chunk_z)
        
        # Store the future for later retrieval if needed
        # For now, we'll let it run in the background
        # In a more sophisticated implementation, we could track futures and wait for them

    def _generate_chunk_worker(self, chunk_x: int, chunk_y: int, chunk_z: int) -> None:
        """
        Worker method that runs in a separate thread to generate a chunk.
        
        Args:
            chunk_x: Chunk X coordinate
            chunk_y: Chunk Y coordinate
            chunk_z: Chunk Z coordinate
        """
        try:
            # Use the world generator's threaded chunk generation
            self.world_generator._generate_complete_chunk(chunk_x, chunk_y, chunk_z)
            
            # Apply the generated chunk data to our chunked world
            chunk_data = self.world_generator.chunk_cache.cache.get((chunk_x, chunk_y, chunk_z), {})
            
            with self._chunk_generation_lock:
                tiles_applied = 0
                for pos_str, tile in chunk_data.items():
                    pos_parts = pos_str.split(',')
                    world_pos = VectorN(int(pos_parts[0]), int(pos_parts[1]), int(pos_parts[2]))
                    
                    # Get chunk directly without triggering generation
                    chunk_key = self.get_chunk_key(world_pos)
                    if chunk_key in self.chunks:
                        chunk = self.chunks[chunk_key]
                        local_pos = chunk.get_local_pos(world_pos)
                        chunk.set_tile(local_pos, tile)
                        
                        # Update cache
                        pos_key = (world_pos.x, world_pos.y, world_pos.z)
                        self._tile_cache[pos_key] = tile
                        tiles_applied += 1
        except Exception as e:
            # Log any errors that occur during chunk generation
            import logging
            logging.warning(f"Error generating chunk ({chunk_x}, {chunk_y}, {chunk_z}): {e}")

    def get_tile(self, pos: VectorN) -> Union[Tile, None]:
        """Get tile at world position."""
        # Check cache first
        pos_key = (pos.x, pos.y, pos.z)
        if pos_key in self._tile_cache:
            return self._tile_cache[pos_key]

        # Get chunk and local position
        chunk_key = self.get_chunk_key(pos)
        chunk = self.get_chunk(chunk_key)
        local_pos = chunk.get_local_pos(pos)

        # Get tile from chunk
        tile = chunk.get_tile(local_pos)

        # Cache the result (but limit cache size)
        if len(self._tile_cache) < self._cache_size:
            self._tile_cache[pos_key] = tile

        return tile if tile != chunk.palette.get_empty_tile() else None

    def set_tile(self, pos: VectorN, tile: Tile) -> None:
        """Set tile at world position."""
        chunk_key = self.get_chunk_key(pos)
        chunk = self.get_chunk(chunk_key)
        local_pos = chunk.get_local_pos(pos)

        chunk.set_tile(local_pos, tile)

        # Update cache
        pos_key = (pos.x, pos.y, pos.z)
        self._tile_cache[pos_key] = tile

    def clear_cache(self) -> None:
        """Clear the tile cache."""
        self._tile_cache.clear()

    def shutdown(self) -> None:
        """Shutdown the thread pool and clean up resources."""
        if hasattr(self, '_thread_pool'):
            # Shutdown the thread pool and wait for all threads to complete
            # This prevents the game from hanging due to background threads
            self._thread_pool.shutdown(wait=True)
            print("Thread pool shutdown complete.")

    def get_chunk_stats(self) -> dict:
        """Get statistics about chunk usage."""
        total_chunks = len(self.chunks)
        empty_chunks = sum(1 for chunk in self.chunks.values() if chunk.is_empty())
        total_tiles = sum(len(chunk.palette) for chunk in self.chunks.values())

        return {
            "total_chunks": total_chunks,
            "empty_chunks": empty_chunks,
            "used_chunks": total_chunks - empty_chunks,
            "total_tile_types": total_tiles,
            "cache_size": len(self._tile_cache),
        }

    def serialize(self, filepath: Path) -> Path:
        """Serialize the chunked world data."""
        with open(filepath, "wb") as fh:
            pickle.dump(self, fh)
        return filepath

    @staticmethod
    def deserialize(filepath: Path):
        """Deserialize the chunked world data."""
        with open(filepath, "rb") as fh:
            return pickle.load(fh)

    def __getitem__(self, *item: int):
        return self.get_tile(VectorN(*item))

    def __setitem__(self, *item: int):
        self.set_tile(VectorN(*item))

    def __iter__(self):
        """Iterate over all tiles in all chunks."""
        for chunk_key, chunk in self.chunks.items():
            for x in range(self.chunk_size):
                for y in range(self.chunk_size):
                    for z in range(self.chunk_size):
                        tile = chunk.get_tile((x, y, z))
                        if tile != chunk.palette.get_empty_tile():
                            world_x = chunk_key[0] * self.chunk_size + x
                            world_y = chunk_key[1] * self.chunk_size + y
                            world_z = chunk_key[2] * self.chunk_size + z
                            pos = VectorN(world_x, world_y, world_z)
                            yield (pos.serialize(), tile)


class World(EntityListener):
    """
    A world contains world data and manages the world state.
    """

    def __init__(self, seed: int, name="Gaia"):
        self.name = name
        self.seed = seed
        
        # Create ONE generator that will be reused
        from lithicrivers.worldgen import SeededWorldGenerator

        self.generator = SeededWorldGenerator(seed)
        
        # Start with empty world data - everything will be generated lazily
        # Pass the generator so structures can be generated when chunks are loaded
        self.data = ChunkedWorldData(world_generator=self.generator)
        self.gametick = 0

        # Change entity storage to support multiple entities per position
        # Map position tuples to lists of entities
        self.entities_by_position = {}  # (x, y, z) -> list[Entity]
        
        # Initialize fluid manager
        self.fluid_manager = FluidManager(self)
        
        self._add_starter_entities()
        
        # Generate forced structures for quests and main story content
        self._generate_forced_structures()

    def pregen_chunks(self, radius: int) -> None:
        """Pre-generate chunks for the world."""
        self.data.pregen_chunks(radius)

    def clone(self) -> "World":
        """
        Create a deep copy of this World for testing purposes.
        This is more efficient than creating a new world from scratch.
        """
        import copy
        cloned_world = copy.deepcopy(self)
        # Re-establish entity listeners after cloning
        cloned_world._reestablish_entity_listeners()
        return cloned_world
    
    def _reestablish_entity_listeners(self) -> None:
        """Re-establish entity listeners after deserialization or cloning."""
        # Clear existing listeners and re-add the world as a listener to all entities
        for entities in self.entities_by_position.values():
            for entity in entities:
                entity._listeners.clear()
                entity.add_listener(self)
    
    def __getstate__(self):
        """Custom pickle serialization that handles threading primitives."""
        state = self.__dict__.copy()
        # Don't serialize the generator - we'll recreate it on load
        if 'generator' in state:
            del state['generator']
        return state
    
    def __setstate__(self, state):
        """Custom pickle deserialization that recreates threading primitives."""
        self.__dict__.update(state)
        # Recreate the world generator with the same seed
        from lithicrivers.worldgen import SeededWorldGenerator
        self.generator = SeededWorldGenerator(self.seed)
        # Re-establish entity listeners
        self._reestablish_entity_listeners()

    def _generate_forced_structures(self) -> None:
        """
        Generate forced structures for quests and main story content.
        
        This method creates specific structures at predetermined locations that are
        essential for the game's narrative and quest progression. These structures
        are generated during world initialization and are separate from the procedural
        structure generation that happens when chunks are loaded.
        
        Note: This is different from the procedural structure generation that happens
        when chunks are loaded. This method is for story-critical structures only.
        """

        print("Generating forced structures for seed: ", self.seed)
        # Force a ship to spawn at (20, 20, 0) - Main story location
        forced_ship_pos = VectorN(20, 20, 0)
        print(f"FORCING SHIP TO SPAWN AT {forced_ship_pos}")  # Debug output
        
        # Generate a small world around the ship to place it
        ship_radius = VectorN(25, 25, 2)
        ship_world_data = self.generator.generate_world_data(ship_radius)
        
        # Apply the ship world data to our chunked world
        for pos_str, tile in ship_world_data.items():
            pos_parts = pos_str.split(',')
            world_pos = VectorN(int(pos_parts[0]), int(pos_parts[1]), int(pos_parts[2]))
            self.data.set_tile(world_pos, tile)
        
        # Force a procedural dungeon to spawn at (50, 50, -3) - Quest location
        forced_dungeon_pos = VectorN(50, 50, -3)
        print(f"FORCING UNDERGROUND FACILITY TO SPAWN AT {forced_dungeon_pos}")  # Debug output
        
        # Generate a small world around the dungeon to place it
        dungeon_radius = VectorN(55, 55, 5)
        dungeon_world_data = self.generator.generate_world_data(dungeon_radius)
        
        # Apply the dungeon world data to our chunked world
        for pos_str, tile in dungeon_world_data.items():
            pos_parts = pos_str.split(',')
            world_pos = VectorN(int(pos_parts[0]), int(pos_parts[1]), int(pos_parts[2]))
            self.data.set_tile(world_pos, tile)

    def on_entity_moved(self, event: EntityMovedEvent) -> None:
        """Handle entity movement events."""
        # Remove from old position
        old_pos_key = self._get_position_key(event.old_position)
        if old_pos_key in self.entities_by_position:
            try:
                self.entities_by_position[old_pos_key].remove(event.entity)
                # Clean up empty position entries
                if not self.entities_by_position[old_pos_key]:
                    del self.entities_by_position[old_pos_key]
            except ValueError:
                # Entity not found at position, ignore
                pass
        
        # Add to new position
        new_pos_key = self._get_position_key(event.new_position)
        if new_pos_key not in self.entities_by_position:
            self.entities_by_position[new_pos_key] = []
        self.entities_by_position[new_pos_key].append(event.entity)

    def _add_starter_entities(self):
        """Add starter entities to the world."""
        # Add NPC
        npc = Entities.starter_npc()
        self.add_entity(npc)

        # Add test entities
        entity1 = Entities.test_entity1()
        entity2 = Entities.test_entity2()
        self.add_entity(entity1)
        self.add_entity(entity2)

        # Add StumblingSheep 2 blocks north of player spawn
        sheep_position = DEFAULT_PLAYER_POSITION + (VEC_NORTH * 2)
        sheep = Entities.stumbling_sheep(sheep_position)
        self.add_entity(sheep)
        
        # Add some test fluids to demonstrate the system
        # Water pool near the player
        water_pos = DEFAULT_PLAYER_POSITION + VectorN(3, 0, 0)
        water = Entities.water(water_pos, amount=1.0)
        self.fluid_manager.add_fluid(water)
        
        # Lava pool further away
        lava_pos = DEFAULT_PLAYER_POSITION + VectorN(-3, -1, 0)
        lava = Entities.lava(lava_pos, amount=1.0)
        self.fluid_manager.add_fluid(lava)
        
        # Acid pool
        acid_pos = DEFAULT_PLAYER_POSITION + VectorN(0, 2, 0)
        acid = Entities.acid(acid_pos, amount=1.0)
        self.fluid_manager.add_fluid(acid)

        # BIG oil pool further away
        oil_pos = DEFAULT_PLAYER_POSITION + VectorN(0, -10, 0)
        oil = Entities.oil(oil_pos, amount=100.0)
        self.fluid_manager.add_fluid(oil)

    def get_tile(self, pos: VectorN):
        tile = self.data.get_tile(pos)
        if tile is None:
            # Use the stored generator instead of creating a new one
            tile = self.generator.generate_tile_for_position(pos)
            self.data.set_tile(pos, tile)
        return tile
    
    def set_tile(self, pos: VectorN, tile: Tile):
        self.data.set_tile(pos, tile)

    def _get_position_key(self, pos: VectorN) -> tuple[int, int, int]:
        """Convert VectorN to tuple key for dictionary storage."""
        return (pos.x, pos.y, pos.z)

    def get_entities(self, pos: VectorN) -> list["Entity"]:
        """Get all entities at a position."""
        pos_key = self._get_position_key(pos)
        return self.entities_by_position.get(pos_key, [])

    def get_entity(self, pos: VectorN) -> Optional["Entity"]:
        """Get the first entity at a position (for backward compatibility)."""
        entities = self.get_entities(pos)
        return entities[0] if entities else None

    def add_entity(self, entity: "Entity") -> None:
        """Add an entity to the world."""
        pos_key = self._get_position_key(entity.position)
        if pos_key not in self.entities_by_position:
            self.entities_by_position[pos_key] = []
        self.entities_by_position[pos_key].append(entity)
        
        # Register as listener for movement events
        entity.add_listener(self)

    def remove_entity(self, entity: "Entity") -> None:
        """Remove an entity from the world."""
        pos_key = self._get_position_key(entity.position)
        if pos_key in self.entities_by_position:
            try:
                self.entities_by_position[pos_key].remove(entity)
                # Clean up empty position entries
                if not self.entities_by_position[pos_key]:
                    del self.entities_by_position[pos_key]
            except ValueError:
                # Entity not found at position, ignore
                pass
        
        # Unregister as listener
        entity.remove_listener(self)

    def move_entity(self, entity: "Entity", new_position: VectorN) -> None:
        """Move an entity from its current position to a new position."""
        # Remove from old position
        self.remove_entity(entity)
        
        # Update entity's position
        entity.position = new_position
        
        # Add to new position
        self.add_entity(entity)

    def get_adjacent_entities(self, pos: VectorN) -> list[tuple[str, VectorN, str]]:
        """Get all entities adjacent to a position."""
        adjacent = []
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                check_pos = VectorN(pos.x + dx, pos.y + dy, pos.z)
                entities = self.get_entities(check_pos)
                for entity in entities:
                    color = entity.color if hasattr(entity, "color") else "white"
                    adjacent.append((entity.name, check_pos, color))

        return adjacent

    def get_all_entities(self) -> list["Entity"]:
        """Get all entities in the world."""
        all_entities = []
        for entities in self.entities_by_position.values():
            all_entities.extend(entities)
        return all_entities

    def get_priority_entity(self, pos: VectorN) -> Optional["Entity"]:
        """
        Get the highest priority entity at a position for rendering.
        Priority order: Player > NPCs > Interactive Entities > Dropped Items
        """
        entities = self.get_entities(pos)
        if not entities:
            return None
        
        if len(entities) == 1:
            return entities[0]
        
        # Sort entities by priority
        def get_priority(entity):
            if hasattr(entity, 'get_conversation'):  # NPCs
                return 3
            elif hasattr(entity, 'interact'):  # Interactive entities
                return 2
            elif isinstance(entity, DroppedItem):  # Dropped items
                return 1
            else:
                return 0
        
        # Sort by priority (highest first) and return the first one
        sorted_entities = sorted(entities, key=get_priority, reverse=True)
        return sorted_entities[0]

    def get_entity_count(self, pos: VectorN) -> int:
        """Get the number of entities at a position."""
        return len(self.get_entities(pos))

    def test_multiple_entities(self) -> bool:
        """
        Test method to verify that multiple entities can exist at the same position.
        Returns True if the test passes.
        """
        # Create test position
        test_pos = VectorN(10, 10, 0)
        
        # Create multiple dropped items at the same position
        item1 = Items.rock()
        item2 = Items.gold_nugget()
        item3 = Items.stick()
        
        dropped1 = DroppedItem(item1, test_pos)
        dropped2 = DroppedItem(item2, test_pos)
        dropped3 = DroppedItem(item3, test_pos)
        
        # Add all entities
        self.add_entity(dropped1)
        self.add_entity(dropped2)
        self.add_entity(dropped3)
        
        # Verify we can retrieve all entities
        entities = self.get_entities(test_pos)
        if len(entities) != 3:
            return False
        
        # Verify entity names are correct
        entity_names = [e.name for e in entities]
        expected_names = ["Rock", "Gold Nugget", "Stick"]
        if set(entity_names) != set(expected_names):
            return False
        
        # Verify priority entity is correct (should be the first one added)
        priority_entity = self.get_priority_entity(test_pos)
        if priority_entity is None:
            return False
        
        # Clean up test entities
        self.remove_entity(dropped1)
        self.remove_entity(dropped2)
        self.remove_entity(dropped3)
        
        return True


class Game:
    """Main game class. Meant to hold all game state. Can be pickled to save the game."""
    def __init__(
        self,
        seed: int,
        player: Player = None,
        world: World = None,
        viewport: Viewport = DEFAULT_VIEWPORT,
    ):
        # Create a copy of the viewport to avoid shared state between tests
        if viewport is DEFAULT_VIEWPORT:
            from lithicrivers.model.model import Viewport
            from lithicrivers.model.vector import VectorN

            self.viewport = Viewport(
                top_left=VectorN(
                    viewport.top_left.x, viewport.top_left.y, viewport.top_left.z
                ),
                lower_right=VectorN(
                    viewport.lower_right.x,
                    viewport.lower_right.y,
                    viewport.lower_right.z,
                ),
                scale=viewport.scale,
            )
        else:
            self.viewport = viewport

        if player is None:
            player = Player(name=DEFAULT_PLAYER_NAME)

        if world is None:
            world = World(seed=seed)

        self.player: Player = player
        self.world: World = world

        self.running = True
        self.message_log = MessageLog(game=self)
        self.gametick = 0

        # Add initial welcome message
        self.message_log.add_message(
            "Welcome to LithicRivers! Your adventures will be logged here.", "info"
        )

    def pregen_chunks(self, radius: int) -> None:
        """Pre-generate chunks for the world."""
        self.world.pregen_chunks(radius)

    def shutdown(self) -> None:
        """Shutdown the game."""
        self.world.data.shutdown()

    def get_tile_at_player_feet(self) -> Tile:
        return self.world.get_tile(self.player.position)

    def render_world_viewport(self, viewport: Viewport = None) -> RenderedData:
        r"""
        :param viewport: Viewport to render.
            if not specified, defaults to self.viewport
        :param scale: Scaling for sprites.      <br><br><pre><code>
        |   1 = x, 2 = \/, 3 = \ /, etc.        <br>
        |              /\       x               <br>
        |                      / \              <br></pre></code>
        :return: A list of tiles with color information.
        """

        if not viewport:
            viewport = self.viewport

        ret: list[list[str]] = []
        color_data: list[list[tuple[int, int, int]]] = []

        z = self.player.position.z

        for y in range(viewport.top_left.y, (viewport.lower_right.y + 1)):
            retrow = []
            color_row = []
            for x in range(viewport.top_left.x, (viewport.lower_right.x + 1)):
                pos = VectorN(x, y, z)
                tile = self.world.get_tile(pos)
                if not tile:
                    tile = Tiles.empty()

                sprite = tile.render_sprite(scale=viewport.scale)
                tile_color = get_color_for_tile(tile.tileid)

                # Check for entities at this position
                entities = self.world.get_entities(pos)
                if entities:
                    # Get the highest priority entity for rendering
                    entity = self.world.get_priority_entity(pos)
                    sprite = entity.render_sprite(scale=viewport.scale)
                    # Use entity color if available, otherwise use tile color
                    if hasattr(entity, "color"):
                        tile_color = COLOR_MANAGER.get_entity_color(entity.color)
                    
                    # If there are multiple entities, modify the sprite to show count
                    if len(entities) > 1:
                        # For now, just use the priority entity's sprite
                        # TODO: Implement better multi-entity visualization (e.g., add a number)
                        pass
                
                # Check for fluids at this position (render on top of tiles but under entities)
                fluid = self.world.fluid_manager.get_fluid(pos)
                if fluid:
                    # Only render fluid if there's no entity at this position
                    if not entities:
                        sprite = fluid.render_sprite(scale=viewport.scale)
                        tile_color = COLOR_MANAGER.get_color(fluid.get_color())

                # if we are here, render us!
                if (self.player.position.y == y) and (self.player.position.x == x):
                    sprite = self.player.render_sprite(scale=viewport.scale)
                    tile_color = COLOR_MANAGER.get_color("PLAYER")

                # logging.debug('render: {}'.format(sprite))
                # done with a single sprite in a row
                retrow.append(sprite)
                color_row.append(tile_color)
            ret.append(retrow)
            color_data.append(color_row)

        logging.log(5, "Returning this from render_world_viewport()")
        logging.log(5, pprint.pformat(ret))

        return RenderedData(ret, scale=viewport.scale, color_data=color_data)

    def move_player(self, vec: VectorN):
        possible_position = self.player.calc_offset(vec)

        # check bounds
        if self.world.get_tile(possible_position) is None:
            logging.debug(
                f"Tried to move OOB! {vec} would have resulted in {possible_position}"
            )
            return

        self.player.move(vec)
        # Use action tick cost system instead of just incrementing by 1
        tick_cost = self.get_action_tick_cost("walk")
        for _ in range(tick_cost):
            self.increment_tick()

    def player_outside_viewport(self, wiggle=0):
        return not self.player_inside_2d_viewport(wiggle=wiggle)

    def player_inside_2d_viewport(self, wiggle: int = 0):
        return self.player.position.trim(2).inside_bounding_rect(
            self.viewport.top_left.trim(2),
            self.viewport.lower_right.trim(2),
            wiggle=wiggle,
        )

    def reset_viewport(self):
        # TODO: Does this work for even/odd numbered sizes? Test this...
        px, py, pz = self.player.position

        vpwidth, vpheight = self.viewport.get_size()

        vpw_tl = vpwidth // 2
        vph_tl = vpheight // 2

        vpw_lr = vpwidth // 2
        vph_lr = vpheight // 2

        # preserve oddness
        if (vpwidth % 2) != 0:
            vpw_lr += 1

        if (vpheight % 2) != 0:
            vph_lr += 1

        # make our bounds centered on the player position
        self.viewport.top_left = VectorN(px - vpw_tl, py - vph_tl, pz)

        self.viewport.lower_right = VectorN(px + vpw_lr, py + vph_lr, pz)

    def set_tile_at_player_feet(self, tile):
        self.world.set_tile(self.player.position, tile)

    def render_pretty_player_position(self):
        return str(self.player.position.as_short_string())

    def log_mining(self, tile_name: str, items_dropped: list[str]):
        """Log a mining event."""
        if items_dropped:
            items_str = ", ".join(items_dropped)
            self.message_log.add_message(
                f"Mined {tile_name} and found: {items_str}", "mining"
            )
        else:
            self.message_log.add_message(f"Mined {tile_name}", "mining")

    def log_interaction(self, entity_name: str, interaction_text: str):
        """Log an interaction event."""
        self.message_log.add_message(
            f"Interacted with {entity_name}: {interaction_text}", "interaction"
        )

    def log_pickup(self, item_name: str):
        """Log an item pickup event."""
        self.message_log.add_message(f"Picked up: {item_name}", "pickup")

    def log_dialog(self, speaker: str, message: str):
        """Log a dialog event."""
        self.message_log.add_message(f"{speaker}: {message}", "dialog")

    def log_info(self, message: str):
        """Log a general info message."""
        self.message_log.add_message(message, "info")

    def increment_tick(self):
        """Increment the game tick counter."""
        self.gametick += 1
        self.process_entity_ticks()
        # Process fluid physics
        self.world.fluid_manager.process_fluids(self.gametick)

    def get_tick_rate(self) -> int:
        """Get the current tick rate based on player body condition."""
        # Base tick rate is 200, but can be modified by body condition
        base_rate = 200
        walk_speed = self.player.get_walk_speed_modifier()
        break_speed = self.player.get_break_speed_modifier()
        
        # Average the speed modifiers to get overall performance
        avg_speed = (walk_speed + break_speed) / 2.0
        
        # Adjust tick rate based on performance (slower body = faster ticks for more granular control)
        if avg_speed < 0.5:
            return int(base_rate * 2)  # 400 ticks for severely impaired
        elif avg_speed < 0.8:
            return int(base_rate * 1.5)  # 300 ticks for impaired
        else:
            return base_rate  # 200 ticks for normal operation

    def get_action_tick_cost(self, action_type: str) -> int:
        """Get how many ticks an action should cost based on body condition."""
        speed_modifier = self.player.get_action_speed(action_type)
        
        # Define base costs for different action types
        base_costs = {
            "walk": 200,      # Walking is the baseline action
            "break": 300,      # Mining/breaking takes longer than walking
            "mine": 300,       # Alias for break
            "craft": 400,      # Crafting takes even longer
            "push": 250,       # Pushing objects takes some time
            "inventory": 50,   # Quick inventory operations
            "interact": 100,   # Quick interactions
            "pickup": 75,      # Quick pickup operations
        }
        
        # Get base cost for this action type, default to walk cost
        base_cost = base_costs.get(action_type, base_costs["walk"])
        
        # Adjust cost based on body condition
        if speed_modifier < 0.5:
            return max(1, int(base_cost / speed_modifier))  # More ticks for slower actions
        else:
            return base_cost

    def process_entity_ticks(self):
        """Process ticks for all entities that have a tick method."""
        # Get all entities in the world
        for entity in self.world.get_all_entities():
            if hasattr(entity, "tick") and callable(entity.tick):
                # Check if entity has a speed attribute and should tick this frame
                if hasattr(entity, "speed"):
                    # Entity speed affects how often it ticks
                    # Higher speed = more frequent ticks
                    if self.gametick % max(1, int(10 / entity.speed)) == 0:
                        entity.tick()
                else:
                    # Default behavior for entities without speed attribute
                    entity.tick()


class MessageLog:
    """A class to manage game messages for the message log pane."""

    def __init__(self, max_messages: int = 100, game: "Game" = None):
        self.messages = []
        self.max_messages = max_messages
        self.game = game

    def add_message(self, message: str, message_type: str = "info"):
        """Add a message to the log with timestamp, game tick, and type."""
        timestamp = datetime.now().strftime("%H:%M:%S")
        gametick = self.game.gametick if self.game else 0
        log_entry = {
            "timestamp": timestamp,
            "gametick": gametick,
            "message": message,
            "type": message_type,  # info, mining, interaction, pickup, dialog
        }

        self.messages.append(log_entry)

        # Keep only the most recent messages
        if len(self.messages) > self.max_messages:
            self.messages.pop(0)

    def get_messages(self, message_type: Optional[str] = None) -> list[dict]:
        """Get all messages, optionally filtered by type."""
        if message_type is None:
            return self.messages.copy()
        return [msg for msg in self.messages if msg["type"] == message_type]

    def get_recent_messages(self, count: int = 20) -> list[dict]:
        """Get the most recent messages."""
        return self.messages[-count:] if self.messages else []

    def clear(self):
        """Clear all messages."""
        self.messages.clear()
