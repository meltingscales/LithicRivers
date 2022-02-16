import logging
import pprint
from typing import List, Dict, Union

import numpy

from lithicrivers.model import VectorN, Viewport, T
from lithicrivers.settings import DEFAULT_SIZE_RADIUS, DEFAULT_VIEWPORT, VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST, \
    DEFAULT_PLAYER_POSITION


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
        return self.sprite_sheet[normalized_scale]


class Entity:

    def __init__(self, position: VectorN):
        self.position = position
        self.health = 100
        self.stamina = 100

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

    def __init__(self):
        Entity.__init__(self, position=DEFAULT_PLAYER_POSITION)
        SpriteRenderable.__init__(self, ['$', '[]\n'
                                              '%%'])

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
        return Tile("Cloud", sprite_sheet=['~'])

    @staticmethod
    def Bedrock():
        return Tile("Bedrock", sprite_sheet=['#'])

    @staticmethod
    def Empty():
        return Tile("Empty", sprite_sheet=[' '])


class WorldData:
    def __init__(self):
        self.data = {
            VectorN(0, 0, 0).serialize(): Tiles.Dirt()
        }

    def set_tile(self, pos: VectorN, t: Tile):
        self.data[pos.serialize()] = t

    def get_tile(self, pos: VectorN) -> Union[Tile, None]:
        p = pos.serialize()

        if p in self.data:
            return self.data[p]

        return None

    def __getitem__(self, *item: int):
        return self.get_tile(VectorN(*item))

    def __setitem__(self, *item: int):
        self.set_tile(VectorN(*item))

    def __iter__(self):
        for key, val in self.data.items():
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
            gf_kwargs=None) -> \
            WorldData:
        """
        Generate world data.

        Note gen_function MUST accept *args and **kwargs.
        :param radius:
        :param gen_function:
        :param gf_args:
        :param gf_kwargs:
        :return:
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
    def __init__(self, player: Player = None, world: World = None, viewport=DEFAULT_VIEWPORT):

        self.viewport = viewport

        if player is None:
            player = Player()

        if world is None:
            world = World()

        self.player: Player = player
        self.world: World = world

        self.running = True

    def slide_viewport(self, moveVec):
        self.viewport.topleft += moveVec
        self.viewport.lowerright += moveVec

    def slide_viewport_left(self):
        self.slide_viewport(VEC_WEST)

    def slide_viewport_right(self):
        self.slide_viewport(VEC_EAST)

    def get_tile_at_player_feet(self) -> Tile:
        return self.world.get_tile(self.player.position)

    def render_world_viewport(
            self,
            scale: int = 1,
            viewport: Viewport = None
    ) -> List[List[str]]:
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

        ret = []

        z = self.player.position.z

        for y in range(viewport.topleft.y, viewport.lowerright.y):
            retrow = []
            for x in range(viewport.topleft.x, viewport.lowerright.x):
                pos = VectorN(x, y, z)
                tile = self.world.get_tile(pos)
                if not tile:
                    tile = Tiles.Empty()

                sprite = tile.render_sprite(scale=scale)

                # if we are here, render us!
                if (self.player.position.y == y) and (self.player.position.x == x):
                    sprite = self.player.render_sprite(scale=scale)

                # logging.debug('render: {}'.format(sprite))
                # done with a single sprite in a row
                retrow.append(sprite)
            ret.append(retrow)
        logging.debug("Returning this from render_world_viewport()")
        logging.debug(pprint.pformat(ret))

        return ret

    def render_world(self, scale: int = 1) -> List[List[str]]:
        r"""
        :param scale: Scaling for sprites.      <br><br><pre><code>
        |   1 = x, 2 = \/, 3 = \ /, etc.        <br>
        |              /\       x               <br>
        |                      / \              <br></pre></code>
        :return: A list of tiles.
        """

        ret = []

        for row in self.world.data:
            retrow = []
            for tile in row:
                sprite = tile.render_sprite(scale=scale)
                # logging.debug('render: {}'.format(sprite))
                retrow.append(sprite)  # TODO: will break with scale>2... we need to print to a buffer
            ret.append(retrow)

        # TODO: Assert ret is well-formed

        logging.info("player x,y={:2d},{:2d}".format(self.player.position.x, self.player.position.y))
        ret[self.player.position.y][self.player.position.x] = self.player.render_sprite(scale=scale)

        return ret

    def move_player(self, vec: VectorN):
        possiblePosition = self.player.calcOffset(vec)

        # check bounds
        if self.world.get_tile(possiblePosition) is None:
            logging.debug("Tried to move OOB! {} would have resulted in {}".format(vec, possiblePosition))
            return

        self.player.move(vec)

    def player_outside_viewport(self):
        return not self.player_inside_2d_viewport()

    def player_inside_2d_viewport(self):
        return self.player.position.trim(2).insideBoundingRect(self.viewport.topleft.trim(2),
                                                               self.viewport.lowerright.trim(2))

    def reset_viewport(self):
        raise NotImplementedError("TODO: Reset viewport")

    def set_tile_at_player_feet(self, tile):
        self.world.set_tile(self.player.position, tile)

    def render_pretty_player_position(self):
        return "{}".format(self.player.position.as_short_string())
