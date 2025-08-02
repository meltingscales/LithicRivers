"""
Game logic for LithicRivers.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

import logging
import pickle
import pprint
import random
from datetime import datetime
from pathlib import Path
from typing import Optional, Union

from lithicrivers.constants import VEC_EAST, VEC_NORTH, VEC_SOUTH, VEC_WEST
from lithicrivers.model.generictype import T
from lithicrivers.model.model import RenderedData, Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import (
    DEFAULT_PLAYER_POSITION,
    DEFAULT_SIZE_RADIUS,
    DEFAULT_VIEWPORT,
)
from lithicrivers.textutil import COLOR_MANAGER, get_color_for_item, get_color_for_tile


def generate_sprite_repeat(char, scale: int = 1):
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
    def __init__(self, sprite_sheet):
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


class Entity:
    def __init__(self, name: str, position: VectorN):
        self.name: str = name
        self.position: VectorN = position
        self.health: int = 100
        self.stamina: int = 100

    def move(self, vec: VectorN):
        self.position += vec

    def calc_offset(self, vec: VectorN) -> VectorN:
        """Where would I move, if I did move?"""
        return self.position + vec

    def move_north(self):
        self.move(VEC_NORTH)

    def move_south(self):
        self.move(VEC_SOUTH)

    def move_west(self):
        self.move(VEC_WEST)

    def move_east(self):
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
        # Default sprite sheet for all NPCs
        self.sprite_sheet = [
            sprite,  # 1x1
            f"{sprite}{sprite}\n{sprite}{sprite}",  # 2x2
            f"{sprite}{sprite}{sprite}\n{sprite}{sprite}{sprite}\n{sprite}{sprite}{sprite}"  # 3x3
        ]
        self._setup_default_conversation()

    def _setup_default_conversation(self):
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
        self.sprite_sheet = [
            "N",  # 1x1
            "NN\nNN",  # 2x2 - Simple 2x2 tree
            " N \nNNN\n N "  # 3x3 - Tree with trunk and branches
        ]
        self._setup_default_conversation()

    def _setup_default_conversation(self):
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
        elif response == "What can you do?":
            return "about_mining"
        elif response == "Tell me more about mining":
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
        
        # Default sprite sheet for all interactive entities
        self.sprite_sheet = [
            sprite,  # 1x1
            f"{sprite}{sprite}\n{sprite}{sprite}",  # 2x2
            f"{sprite}{sprite}{sprite}\n{sprite}{sprite}{sprite}\n{sprite}{sprite}{sprite}"  # 3x3
        ]

    def render_sprite(self, scale: int = 1) -> str:
        """Render the entity sprite."""
        # Use the SpriteRenderable's render_sprite method
        return super().render_sprite(scale)

    def interact(self):
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
        self.sprite_sheet = [
            "C",  # 1x1
            "CC\nCC",  # 2x2 - Simple 2x2 crystal
            " C \nCCC\n C "  # 3x3 - Crystal with facets
        ]

class AncientRelic(InteractiveEntity):
    def __init__(self, position: VectorN):
        super().__init__(
            "Ancient Relic",
            position,
            sprite="R",
            color="red",
            interaction_text="This ancient relic is covered in mysterious runes. It radiates warmth.",
        )
        self.sprite_sheet = [
            "R",  # 1x1
            "RR\nRR",  # 2x2 - Simple 2x2 relic
            " R \nRRR\n R "  # 3x3 - Relic with ornate details
        ]


class Entities:
    @staticmethod
    def stumbling_sheep(position=VectorN(0, 0, 0)):
        return Entity("Stumbling Sheep", position)

    @staticmethod
    def starter_npc(position=VectorN(5, 5, 0)):
        return ElderOak(position)

    @staticmethod
    def test_entity1(position=VectorN(6, 5, 0)):
        return CrystalShard(position)

    @staticmethod
    def test_entity2(position=VectorN(5, 6, 0)):
        return AncientRelic(position)


class Items:
    """
    A bunch of default items.
    """

    @staticmethod
    def rock():
        return Item("Rock", sprite_sheet=["*"])

    @staticmethod
    def gold_nugget():
        return Item("Gold Nugget", sprite_sheet=["c"])

    @staticmethod
    def stick():
        return Item("Stick", sprite_sheet=["\\"])

    @staticmethod
    def diamond():
        return Item("Diamond", sprite_sheet=["d"])

    @staticmethod
    def log():
        return Item("Log", sprite_sheet=["|"])

    @staticmethod
    def acorn():
        return Item("Acorn", sprite_sheet=["o"])


class Item(SpriteRenderable):
    def __init__(self, name, sprite_sheet: Optional[list[str]] = None):
        SpriteRenderable.__init__(self, sprite_sheet)
        self.name = name


class Inventory:
    def __init__(self, items: Optional[list[Item]] = None):
        if items is None:
            items = []

        self.itemsdata = items

    def add_item(self, item):
        self.itemsdata.append(item)

    def __str__(self):
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
            color_name = self._get_color_name(item_color)
            s += f"{k}={v} ({color_name}), "

        return s[0 : len(s) - 2] if s else "Empty"

    def _get_color_name(self, color: tuple[int, int, int]) -> str:
        """Get a human-readable name for a color."""
        color_names = {
            (1, 0, 0): "red",
            (2, 0, 0): "green",
            (3, 0, 0): "yellow",
            (4, 0, 0): "blue",
            (5, 0, 0): "magenta",
            (6, 0, 0): "cyan",
            (7, 0, 0): "white",
            (8, 0, 0): "gray",
            (0, 0, 0): "black",
        }
        return color_names.get(color, "default")


class Player(Entity, SpriteRenderable):
    def __init__(self, name="Inigo Montoya"):
        Entity.__init__(self, name=name, position=DEFAULT_PLAYER_POSITION)
        SpriteRenderable.__init__(self, ["$", "[]\n%%", "_o_\n/|\\\n/_\\"])

        self.inventory = Inventory([Item("Cookie", ["o"])])


class Tile(SpriteRenderable):
    def __init__(
        self,
        tileid: str,
        desc: Optional[str] = None,
        sprite_sheet: Optional[list[str]] = None,
        drops: Optional[dict[float, Item]] = None,
    ):
        SpriteRenderable.__init__(self, sprite_sheet)
        self.tileid = tileid
        self.description = desc
        self.drops = drops

    def __str__(self):
        return f"<Tile '{self.tileid}': [{self.render_sprite(1)}]>"

    def __eq__(self, other):
        if other is None:
            return False
        return self.tileid == other.tileid

    def calc_drop(self):
        return weighted_choice_dict(self.drops)

    def calc_tree_drops(self):
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


def generate_tile(
    choices: Optional[list[Tile]] = None,
    weights: Optional[list[int]] = None,
    current_location: VectorN = None,
) -> Tile:
    if choices is None:
        choices = [Tiles.tree(), Tiles.dirt(), Tiles.gold_ore()]

    if weights is None:
        weights = [5, 100, 1]

    # for now, generate these stubs for other z values
    if current_location:
        if current_location.z > 0:
            # we are in da sky
            return Tiles.cloud()

        elif current_location.z < 0:
            # we are underground
            return weighted_choice(
                [1, 0.2, 0.05], [Tiles.bedrock(), Tiles.dirt(), Tiles.gold_ore()]
            )

    return weighted_choice(weights, choices)


class Tiles:
    """
    A bunch of default tiles.
    """

    @staticmethod
    def dirt():
        return Tile(
            "Dirt",
            sprite_sheet=[",", ",.\n.,", ",.,\n.,.\n,.,"],
            drops={0.99: Items.rock(), 0.01: Items.gold_nugget()},
        )

    @staticmethod
    def tree():
        return Tile(
            "Tree",
            sprite_sheet=["t", "/\\\n||", "/|\\\n;|;\n/|\\\n"],
            drops={0.50: Items.stick(), 0.30: Items.log(), 0.20: Items.acorn()},
        )

    @staticmethod
    def gold_ore():
        return Tile(
            "Gold Ore",
            sprite_sheet=["?", "??\n??", "???\n???\n???"],
            drops={0.9: Items.gold_nugget(), 0.1: Items.diamond()},
        )

    @staticmethod
    def cloud():
        return Tile("Cloud", sprite_sheet=["~", "~o\noo", ".~~\n~~o\n~oo"])

    @staticmethod
    def bedrock():
        return Tile("Bedrock", sprite_sheet=["#", "|/\n/|", "|,/\n/|\\\n|/|"])

    @staticmethod
    def empty():
        return Tile("Empty", sprite_sheet=[" ", "  \n  "])

    @staticmethod
    def iron_scrap():
        return Tile(
            "Iron Scrap",
            sprite_sheet=["=", "==\n==", "===\n===\n==="],
            drops={0.8: Items.rock(), 0.2: Items.gold_nugget()},
        )

    @staticmethod
    def bone_block():
        return Tile(
            "Bone Block",
            sprite_sheet=["|", "||\n||", "|||\n|||\n|||"],
            drops={0.7: Items.rock(), 0.3: Items.gold_nugget()},
        )

    @staticmethod
    def door():
        return Tile(
            "Door",
            sprite_sheet=["D", "DD\nDD", "DDD\nDDD\nDDD"],
            drops={0.5: Items.rock()},
        )

    @staticmethod
    def scrap_electronics():
        return Tile(
            "Scrap Electronics",
            sprite_sheet=["e", "ee\nee", "eee\neee\neee"],
            drops={0.6: Items.rock(), 0.4: Items.gold_nugget()},
        )

    @staticmethod
    def treasure():
        return Tile(
            "Buried Treasure",
            sprite_sheet=["$", "$$\n$$", "$$$\n$$$\n$$$"],
            drops={0.3: Items.gold_nugget(), 0.7: Items.diamond()},
        )


class WorldData:
    def serialize(self, filepath: Path) -> Path:
        with open(filepath, "wb") as fh:
            pickle.dump(self, fh)

        return filepath

    @staticmethod
    def deserialize(filepath: Path):
        with open(filepath, "rb") as fh:
            return pickle.load(fh)

    def __init__(
        self,
        tile_data: Optional[dict[str, Tile]] = None,
        entity_data: Optional[dict[str, list[Entity]]] = None,
    ):
        self.tile_data = tile_data
        if not self.tile_data:
            self.tile_data = {VectorN(0, 0, 0).serialize(): Tiles.dirt()}

        self.entity_data = entity_data
        if not self.entity_data:
            self.entity_data = {
                VectorN(0, 0, 0).serialize(): [Entities.stumbling_sheep()]
            }

    def set_tile(self, pos: VectorN, t: Tile):
        self.tile_data[pos.serialize()] = t

    def get_tile(self, pos: VectorN) -> Union[Tile, None]:
        p = pos.serialize()

        if p in self.tile_data:
            return self.tile_data[p]

        return None

    def __getitem__(self, *item: int):
        return self.get_tile(VectorN(*item))

    def __setitem__(self, *item: int):
        self.set_tile(VectorN(*item))

    def __iter__(self):
        yield from self.tile_data.items()


class World:
    """
    A world contains world data and manages the world state.
    """

    def get_height(self):
        return self.size.y

    def get_width(self):
        return self.size.x

    @staticmethod
    def gen_random_world_data(
        radius: VectorN,
        gen_function=generate_tile,
        gf_args=None,
        gf_kwargs=None,
        seed: Optional[int] = None,
    ) -> WorldData:
        """
        Generate world data.

        Note gen_function MUST accept *args and **kwargs.
        """

        if gf_kwargs is None:
            gf_kwargs = {}

        if gf_args is None:
            gf_args = []

        resultworld = WorldData()

        # Always use seeded world generation for consistent structure placement
        from lithicrivers.worldgen import generate_world_with_seed
        from lithicrivers.settings import DEFAULT_SEED

        # Use provided seed or default seed for consistency
        if seed is None:
            seed = DEFAULT_SEED

        world_data = generate_world_with_seed(radius, seed)
        resultworld.tile_data = world_data

        return resultworld

    def __init__(
        self, name="Gaia", size=DEFAULT_SIZE_RADIUS, seed: Optional[int] = None
    ):
        self.size = size
        self.name = name
        self.seed = seed
        self.data = World.gen_random_world_data(size, seed=seed)
        self.gametick = 0

        # Add some starter entities
        self.entities = {}
        self._add_starter_entities()

    def _add_starter_entities(self):
        """Add starter entities to the world."""
        # Add NPC
        npc = Entities.starter_npc()
        self.entities[npc.position.serialize()] = npc

        # Add test entities
        entity1 = Entities.test_entity1()
        entity2 = Entities.test_entity2()
        self.entities[entity1.position.serialize()] = entity1
        self.entities[entity2.position.serialize()] = entity2

    def get_tile(self, pos: VectorN):
        tile = self.data.get_tile(pos)
        if tile is None:
            # Generate tile on-demand if it doesn't exist
            tile = generate_tile(current_location=pos)
            self.data.set_tile(pos, tile)
        return tile

    def set_tile(self, pos: VectorN, tile: Tile):
        self.data.set_tile(pos, tile)

    def get_entity(self, pos: VectorN):
        """Get an entity at a position."""
        return self.entities.get(pos.serialize())

    def add_entity(self, entity: Entity):
        """Add an entity to the world."""
        self.entities[entity.position.serialize()] = entity

    def remove_entity(self, pos: VectorN):
        """Remove an entity from the world."""
        key = pos.serialize()
        if key in self.entities:
            del self.entities[key]

    def get_adjacent_entities(self, pos: VectorN) -> list[tuple[str, VectorN, str]]:
        """Get all entities adjacent to a position."""
        adjacent = []
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                check_pos = VectorN(pos.x + dx, pos.y + dy, pos.z)
                entity = self.get_entity(check_pos)
                if entity:
                    color = entity.color if hasattr(entity, "color") else "white"
                    adjacent.append((entity.name, check_pos, color))

        return adjacent


class Game:
    def __init__(
        self,
        player: Player = None,
        world: World = None,
        viewport: Viewport = DEFAULT_VIEWPORT,
        seed: Optional[int] = None,
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
            player = Player()

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
                entity = None
                if hasattr(self.world, "get_entity"):
                    entity = self.world.get_entity(pos)
                if entity:
                    sprite = entity.render_sprite(scale=viewport.scale)
                    # Use entity color if available, otherwise use tile color
                    if hasattr(entity, "color"):
                        if entity.color == "cyan":
                            tile_color = (6, 0, 0)  # Cyan
                        elif entity.color == "blue":
                            tile_color = (4, 0, 0)  # Blue
                        elif entity.color == "red":
                            tile_color = (1, 0, 0)  # Red
                        elif entity.color == "yellow":
                            tile_color = (3, 0, 0)  # Yellow
                        else:
                            tile_color = (7, 0, 0)  # White

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
