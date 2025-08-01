import logging
import pickle
import pprint
import random
from pathlib import Path
from typing import List, Dict, Union, Optional, Tuple

from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST
from lithicrivers.model.generictype import T
from lithicrivers.model.modelpleasemoveme import Viewport, RenderedData
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SIZE_RADIUS, DEFAULT_VIEWPORT, DEFAULT_PLAYER_POSITION
from lithicrivers.textutil import get_color_for_tile, get_color_for_item, COLOR_MANAGER


def generate_sprite_repeat(char, scale: int = 1):
    normalized_scale = scale - 1

    if normalized_scale == 0:
        return char

    ret = ""
    for i in range(0, scale):
        ret += char * scale
        if i < (scale - 1):
            ret += '\n'

    return ret


class SpriteRenderable:
    def __init__(self, sprite_sheet):
        self.sprite_sheet = sprite_sheet
        if not sprite_sheet:
            self.sprite_sheet = ['?', '??\n'
                                      '??', '???\n'
                                            '???\n'
                                            '???']

    def render_sprite(self, scale: int = 1) -> str:
        normalized_scale = scale - 1

        if normalized_scale < 0:
            raise Exception("Cannot render {} with normalized_scale = {}".format(self, normalized_scale))

        if normalized_scale >= len(self.sprite_sheet):
            # if they ask for a sprite too large, give them '?'
            return generate_sprite_repeat('?', scale)

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

    def calcOffset(self, vec: VectorN) -> VectorN:
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
    
    def __init__(self, name: str, position: VectorN, sprite: str = "N", color: str = "cyan"):
        super().__init__(name, position)
        self.sprite = sprite
        self.color = color
        self.conversations = {}
        self._setup_default_conversation()
    
    def _setup_default_conversation(self):
        """Set up the default conversation for this NPC."""
        self.conversations = {
            "greeting": {
                "text": f"Hello, traveler! I am {self.name}. Welcome to LithicRivers!",
                "options": [
                    "Tell me about this world",
                    "What can you do?",
                    "Goodbye"
                ]
            },
            "about_world": {
                "text": "This is a world of endless possibilities. You can mine, build, and explore to your heart's content. The world is procedurally generated, so there's always something new to discover!",
                "options": [
                    "Tell me more about mining",
                    "What about building?",
                    "Back to greeting"
                ]
            },
            "about_mining": {
                "text": "Mining is simple! Just press 'u' when standing on a mineable tile like Gold Ore. You'll get valuable resources that you can use for crafting and trading.",
                "options": [
                    "What about building?",
                    "Back to greeting"
                ]
            },
            "about_building": {
                "text": "Building is coming soon! You'll be able to place blocks and create structures. For now, focus on gathering resources through mining.",
                "options": [
                    "Tell me about mining",
                    "Back to greeting"
                ]
            },
            "goodbye": {
                "text": "Farewell, traveler! May your adventures be fruitful!",
                "options": []
            }
        }
    
    def get_conversation(self, topic: str = "greeting"):
        """Get a conversation topic."""
        return self.conversations.get(topic, self.conversations["greeting"])
    
    def handle_response(self, response: str, topic: str = "greeting"):
        """Handle a conversation response and return the next topic."""
        if response == "Tell me about this world":
            return "about_world"
        elif response == "What can you do?":
            return "about_world"
        elif response == "Tell me more about mining":
            return "about_mining"
        elif response == "What about building?":
            return "about_building"
        elif response == "Back to greeting":
            return "greeting"
        elif response == "Goodbye":
            return "goodbye"
        else:
            return topic  # Stay on current topic
    
    def render_sprite(self, scale: int = 1) -> str:
        """Render the NPC sprite."""
        if scale == 1:
            return self.sprite
        elif scale == 2:
            return f"{self.sprite}{self.sprite}\n{self.sprite}{self.sprite}"
        elif scale == 3:
            return f"{self.sprite}{self.sprite}{self.sprite}\n{self.sprite}{self.sprite}{self.sprite}\n{self.sprite}{self.sprite}{self.sprite}"
        else:
            return self.sprite


class InteractiveEntity(Entity, SpriteRenderable):
    """An entity that can be interacted with."""
    
    def __init__(self, name: str, position: VectorN, sprite: str = "E", color: str = "yellow", interaction_text: str = "This is an interactive entity."):
        super().__init__(name, position)
        self.sprite = sprite
        self.color = color
        self.interaction_text = interaction_text
    
    def render_sprite(self, scale: int = 1) -> str:
        """Render the entity sprite."""
        if scale == 1:
            return self.sprite
        elif scale == 2:
            return f"{self.sprite}{self.sprite}\n{self.sprite}{self.sprite}"
        elif scale == 3:
            return f"{self.sprite}{self.sprite}{self.sprite}\n{self.sprite}{self.sprite}{self.sprite}\n{self.sprite}{self.sprite}{self.sprite}"
        else:
            return self.sprite
    
    def interact(self):
        """Handle interaction with this entity."""
        return self.interaction_text


class Entities:
    @staticmethod
    def StumblingSheep(position=VectorN(0, 0, 0)):
        return Entity("Stumbling Sheep", position)
    
    @staticmethod
    def StarterNPC(position=VectorN(5, 5, 0)):
        return NPC("Elder Oak", position, sprite="N", color="cyan")
    
    @staticmethod
    def TestEntity1(position=VectorN(6, 5, 0)):
        return InteractiveEntity("Crystal Shard", position, sprite="C", color="blue", 
                               interaction_text="This crystal shard glows with a soft blue light. It seems to pulse with energy.")
    
    @staticmethod
    def TestEntity2(position=VectorN(5, 6, 0)):
        return InteractiveEntity("Ancient Relic", position, sprite="R", color="red", 
                               interaction_text="This ancient relic is covered in mysterious runes. It radiates warmth.")


class Items:
    """
    A bunch of default items.
    """

    @staticmethod
    def Rock():
        return Item("Rock",
                    sprite_sheet=['*'])

    @staticmethod
    def Gold_Nugget():
        return Item("Gold Nugget",
                    sprite_sheet=['c'])

    @staticmethod
    def Stick():
        return Item("Stick",
                    sprite_sheet=['\\'])

    @staticmethod
    def Diamond():
        return Item("Diamond",
                    sprite_sheet=['d'])

    @staticmethod
    def Log():
        return Item("Log",
                    sprite_sheet=['|'])

    @staticmethod
    def Acorn():
        return Item("Acorn",
                    sprite_sheet=['o'])


class Item(SpriteRenderable):
    def __init__(self, name, sprite_sheet: List[str] = None):
        SpriteRenderable.__init__(self, sprite_sheet)
        self.name = name


class Inventory:
    def __init__(self, items: List[Item] = None):
        if items is None:
            items = []

        self.itemsdata = items

    def add_item(self, item):
        self.itemsdata.append(item)

    def __str__(self):
        return "<Inventory numItems={} summary={}>".format(len(self.itemsdata), self.summary())

    def count_items(self) -> Dict[str, int]:
        d = {}

        for item in self.itemsdata:
            if item.name in d:
                d[item.name] += 1
            else:
                d[item.name] = 1

        return d

    def summary(self) -> str:
        s = ''

        for k, v in self.count_items().items():
            s += '{}={}, '.format(k, v)

        return s[0:len(s) - 2]  # wow you lazy bastard, you cant even fucking format a string???? AAFSDFASDFADFAFSD
    
    def colored_summary(self) -> str:
        """Generate a colored summary of inventory items."""
        s = ''
        
        for k, v in self.count_items().items():
            # Get color for this item
            item_color = get_color_for_item(k)
            color_name = self._get_color_name(item_color)
            s += f'{k}={v} ({color_name}), '
        
        return s[0:len(s) - 2] if s else "Empty"
    
    def _get_color_name(self, color: Tuple[int, int, int]) -> str:
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
        SpriteRenderable.__init__(self, ['$', '[]\n'
                                              '%%', '_o_\n'
                                                    '/|\\\n'
                                                    '/_\\'])

        self.inventory = Inventory([Item("Cookie", ['o'])])


class Tile(SpriteRenderable):
    def __init__(self,
                 tileid: str,
                 desc: str = None,
                 sprite_sheet: List[str] = None,
                 drops: Dict[float, Item] = None):
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
        if random.random() < 0.9:
            num_acorns = 1
        else:
            # 10% chance of 2-3 acorns, with equal probability
            num_acorns = random.randint(2, 3)
        
        # Create list of items to return
        items = []
        
        # Add guaranteed acorns
        for _ in range(num_acorns):
            items.append(Items.Acorn())
        
        # Add other possible drops (stick, log) with original probabilities
        if random.random() < 0.5:
            items.append(Items.Stick())
        if random.random() < 0.3:
            items.append(Items.Log())
        
        return items


def weighted_choice(weights: List[float], choices: List[T]) -> T:
    if len(weights) != len(choices):
        ve = ValueError(
            f"Weights={weights} and choices={choices} for {weighted_choice.__name__}() must be the same length!")
        logging.error(ve)
        raise ve

    # Normalize weights to sum to 1
    total_weight = sum(weights)
    if total_weight == 0:
        raise ValueError("Weights cannot all be zero")
    
    normalized_weights = [w / total_weight for w in weights]
    
    # Use random.choices for weighted selection
    return random.choices(choices, weights=normalized_weights, k=1)[0]


def weighted_choice_dict(dictWeight: Dict[float, T]) -> T:
    weights = []
    choices = []
    for k, v in dictWeight.items():
        weights.append(k)
        choices.append(v)
    return weighted_choice(weights, choices)


def generate_tile(choices: List[Tile] = None, weights: List[int] = None, current_location: VectorN = None) -> Tile:
    if choices is None:
        choices = [Tiles.Tree(),
                   Tiles.Dirt(),
                   Tiles.Gold_Ore()]

    if weights is None:
        weights = [5, 100, 1]

    # for now, generate these stubs for other z values
    if current_location:
        if current_location.z > 0:
            # we are in da sky
            return Tiles.Cloud()

        elif current_location.z < 0:
            # we are underground
            return weighted_choice(
                [1, 0.2, 0.05],
                [Tiles.Bedrock(), Tiles.Dirt(), Tiles.Gold_Ore()]
            )

    return weighted_choice(weights, choices)


class Tiles:
    """
    A bunch of default tiles.
    """

    @staticmethod
    def Dirt():
        return Tile('Dirt',
                    sprite_sheet=[',', ',.\n'
                                       '.,', ',.,\n'
                                             '.,.\n'
                                             ',.,'],
                    drops={0.99: Items.Rock(),
                           0.01: Items.Gold_Nugget()})

    @staticmethod
    def Tree():
        return Tile('Tree',
                    sprite_sheet=['t', '/\\\n'
                                       '||', '/|\\\n'
                                             ';|;\n'
                                             '/|\\\n'],
                    drops={0.50: Items.Stick(),
                           0.30: Items.Log(),
                           0.20: Items.Acorn()})

    @staticmethod
    def Gold_Ore():
        return Tile("Gold Ore", drops={
            0.9: Items.Gold_Nugget(),
            0.1: Items.Diamond()
        })

    @staticmethod
    def Cloud():
        return Tile("Cloud", sprite_sheet=['~', '~o\n'
                                                'oo', '.~~\n'
                                                      '~~o\n'
                                                      '~oo'])

    @staticmethod
    def Bedrock():
        return Tile("Bedrock", sprite_sheet=['#', '|/\n'
                                                  '/|', '|,/\n'
                                                        '/|\\\n'
                                                        '|/|'])

    @staticmethod
    def Empty():
        return Tile("Empty", sprite_sheet=[' ', '  \n'
                                                '  '])


class WorldData:

    def serialize(self, filepath: Path) -> Path:
        with open(filepath, 'wb') as fh:
            pickle.dump(self, fh)

        return filepath

    @staticmethod
    def deserialize(filepath: Path):
        with open(filepath, 'rb') as fh:
            return pickle.load(fh)

    def __init__(self, tile_data: Dict[str, Tile] = None, entity_data: Dict[str, List[Entity]] = None):

        self.tile_data = tile_data
        if not self.tile_data:
            self.tile_data = {
                VectorN(0, 0, 0).serialize(): Tiles.Dirt()
            }

        self.entity_data = entity_data
        if not self.entity_data:
            self.entity_data = {
                VectorN(0, 0, 0).serialize(): [Entities.StumblingSheep()]
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
        for key, val in self.tile_data.items():
            yield key, val


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
            seed: Optional[int] = None) -> WorldData:
        """
        Generate world data.

        Note gen_function MUST accept *args and **kwargs.
        """

        if gf_kwargs is None:
            gf_kwargs = {}

        if gf_args is None:
            gf_args = []

        resultworld = WorldData()

        # Use seeded world generation if seed is provided
        if seed is not None:
            from lithicrivers.worldgen import generate_world_with_seed
            world_data = generate_world_with_seed(radius, seed)
            resultworld.tile_data = world_data
        else:
            # Fall back to original random generation
            for z in range(-radius.z, radius.z):
                for y in range(-radius.y, radius.y):
                    for x in range(-radius.x, radius.x):
                        pos = VectorN(x, y, z)
                        tile = gen_function(*gf_args, **gf_kwargs, current_location=pos)
                        resultworld.set_tile(pos, tile)

        return resultworld

    def __init__(self, name="Gaia", size=DEFAULT_SIZE_RADIUS, seed: Optional[int] = None):
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
        npc = Entities.StarterNPC()
        self.entities[npc.position.serialize()] = npc
        
        # Add test entities
        entity1 = Entities.TestEntity1()
        entity2 = Entities.TestEntity2()
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
    
    def get_adjacent_entities(self, pos: VectorN) -> List[Tuple[str, VectorN, str]]:
        """Get all entities adjacent to a position."""
        adjacent = []
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                if dx == 0 and dy == 0:
                    continue  # Skip the center position
                
                check_pos = VectorN(pos.x + dx, pos.y + dy, pos.z)
                entity = self.get_entity(check_pos)
                if entity:
                    if hasattr(entity, 'color'):
                        color = entity.color
                    else:
                        color = "white"
                    adjacent.append((entity.name, check_pos, color))
        
        return adjacent


class Game:
    def __init__(self, player: Player = None, world: World = None, viewport: Viewport = DEFAULT_VIEWPORT, seed: Optional[int] = None):

        self.viewport = viewport

        if player is None:
            player = Player()

        if world is None:
            world = World(seed=seed)

        self.player: Player = player
        self.world: World = world

        self.running = True

    def get_tile_at_player_feet(self) -> Tile:
        return self.world.get_tile(self.player.position)

    def render_world_viewport(
            self,
            viewport: Viewport = None
    ) -> RenderedData:
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

        ret: List[List[str]] = []
        color_data: List[List[Tuple[int, int, int]]] = []

        z = self.player.position.z

        for y in range(viewport.top_left.y, (viewport.lower_right.y + 1)):
            retrow = []
            color_row = []
            for x in range(viewport.top_left.x, (viewport.lower_right.x + 1)):
                pos = VectorN(x, y, z)
                tile = self.world.get_tile(pos)
                if not tile:
                    tile = Tiles.Empty()

                sprite = tile.render_sprite(scale=viewport.scale)
                tile_color = get_color_for_tile(tile.tileid)

                # Check for entities at this position
                entity = self.world.get_entity(pos)
                if entity:
                    sprite = entity.render_sprite(scale=viewport.scale)
                    # Use entity color if available, otherwise use tile color
                    if hasattr(entity, 'color'):
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

        logging.debug("Returning this from render_world_viewport()")
        logging.debug(pprint.pformat(ret))

        return RenderedData(ret, scale=viewport.scale, color_data=color_data)

    def move_player(self, vec: VectorN):
        possiblePosition = self.player.calcOffset(vec)

        # check bounds
        if self.world.get_tile(possiblePosition) is None:
            logging.debug("Tried to move OOB! {} would have resulted in {}".format(vec, possiblePosition))
            return

        self.player.move(vec)

    def player_outside_viewport(self, wiggle=0):
        return not self.player_inside_2d_viewport(wiggle=wiggle)

    def player_inside_2d_viewport(self, wiggle: int = 0):
        return self.player.position.trim(2).inside_bounding_rect(self.viewport.top_left.trim(2),
                                                                 self.viewport.lower_right.trim(2), wiggle=wiggle)

    def reset_viewport(self):
        # TODO: Does this work for even/odd numbered sizes? Test this...
        px, py, pz = self.player.position

        vpwidth, vpheight = self.viewport.get_size()

        vpwTL = vpwidth // 2
        vphTL = vpheight // 2

        vpwLR = vpwidth // 2
        vphLR = vpheight // 2

        # preserve oddness
        if (vpwidth % 2) != 0:
            vpwLR += 1

        if (vpheight % 2) != 0:
            vphLR += 1

        # make our bounds centered on the player position
        self.viewport.top_left = \
            VectorN(px - vpwTL, py - vphTL, pz)

        self.viewport.lower_right = \
            VectorN(px + vpwLR, py + vphLR, pz)

    def set_tile_at_player_feet(self, tile):
        self.world.set_tile(self.player.position, tile)

    def render_pretty_player_position(self):
        return str(self.player.position.as_short_string())
