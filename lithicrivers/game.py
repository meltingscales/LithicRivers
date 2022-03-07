import logging
import pickle
import pprint
from pathlib import Path
from typing import List, Dict, Union

import numpy

from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST
from lithicrivers.model.generictype import T
from lithicrivers.model.modelpleasemoveme import Viewport, RenderedData
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import DEFAULT_SIZE_RADIUS, DEFAULT_VIEWPORT, DEFAULT_PLAYER_POSITION


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


class Entities:
    @staticmethod
    def StumblingSheep(position=VectorN(0, 0, 0)):
        return Entity(name='Stumbling Sheep', position=position)


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
        return self.tileid == other.tileid

    def calc_drop(self):
        return weighted_choice_dict(self.drops)


def weighted_choice(weights: List[float], choices: List[T]) -> T:
    if len(weights) != len(choices):
        ve = ValueError(
            f"Weights={weights} and choices={choices} for {weighted_choice.__name__}() must be the same length!")
        logging.error(ve)
        raise ve

    weights = numpy.asarray(weights)

    normalizedWeights = weights

    if weights.sum() != 1:
        normalizedWeights = weights / weights.sum()

    return numpy.random.choice(choices, p=normalizedWeights)


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
                   Tiles.DaFuq()]

    if weights is None:
        weights = [5, 100, 1]

    # for now, generate these stubs for other z values
    if current_location:
        if current_location.z > 0:
            return Tiles.Cloud()
        elif current_location.z < 0:
            return Tiles.Bedrock()

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
                    drops={1.00: Items.Stick()})

    @staticmethod
    def DaFuq():
        return Tile("Dafuq is this?", drops={
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

    def get_height(self):
        return self.size.y

    def get_width(self):
        return self.size.x

    @staticmethod
    def gen_random_world_data(
            radius: VectorN,
            gen_function=generate_tile,
            gf_args=None,
            gf_kwargs=None) -> WorldData:
        """
        Generate world data.

        Note gen_function MUST accept *args and **kwargs.
        """

        if gf_kwargs is None:
            gf_kwargs = {}

        if gf_args is None:
            gf_args = []

        resultworld = WorldData()

        for z in range(-radius.z, radius.z):
            for y in range(-radius.y, radius.y):
                for x in range(-radius.x, radius.x):
                    pos = VectorN(x, y, z)
                    tile = gen_function(*gf_args, **gf_kwargs, current_location=pos)
                    resultworld.set_tile(pos, tile)

        return resultworld

    def __init__(self, name="Gaia", size=DEFAULT_SIZE_RADIUS):
        self.size = size
        self.name = name
        self.data = World.gen_random_world_data(size)
        self.gametick = 0

    def get_tile(self, pos: VectorN):
        return self.data.get_tile(pos)

    def set_tile(self, pos: VectorN, tile: Tile):
        self.data.set_tile(pos, tile)


class Game:
    def __init__(self, player: Player = None, world: World = None, viewport: Viewport = DEFAULT_VIEWPORT):

        self.viewport = viewport

        if player is None:
            player = Player()

        if world is None:
            world = World()

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
        :return: A list of tiles.
        """

        if not viewport:
            viewport = self.viewport

        ret: List[List[str]] = []

        z = self.player.position.z

        for y in range(viewport.top_left.y, (viewport.lower_right.y + 1)):
            retrow = []
            for x in range(viewport.top_left.x, (viewport.lower_right.x + 1)):
                pos = VectorN(x, y, z)
                tile = self.world.get_tile(pos)
                if not tile:
                    tile = Tiles.Empty()

                sprite = tile.render_sprite(scale=viewport.scale)

                # if we are here, render us!
                if (self.player.position.y == y) and (self.player.position.x == x):
                    sprite = self.player.render_sprite(scale=viewport.scale)

                # logging.debug('render: {}'.format(sprite))
                # done with a single sprite in a row
                retrow.append(sprite)
            ret.append(retrow)

        logging.debug("Returning this from render_world_viewport()")
        logging.debug(pprint.pformat(ret))

        return RenderedData(ret, scale=viewport.scale)

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
