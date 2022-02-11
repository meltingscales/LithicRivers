import logging
import pprint
from typing import List, Tuple, Dict

import numpy

from lithicrivers.model import Vector2, Viewport
from lithicrivers.settings import DEFAULT_SIZE, DEFAULT_VIEWPORT, VEC_UP, VEC_DOWN, VEC_LEFT, VEC_RIGHT, \
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

    def __init__(self):
        self.position = DEFAULT_PLAYER_POSITION
        self.health = 100
        self.stamina = 100

    def move(self, vec: Vector2):
        self.position += vec

    def calcOffset(self, vec: Vector2) -> Vector2:
        """Where would I move, if I did move?"""
        return self.position + vec

    def moveUp(self):
        self.move(VEC_UP)

    def moveDown(self):
        self.move(VEC_DOWN)

    def moveLeft(self):
        self.move(VEC_LEFT)

    def moveRight(self):
        self.move(VEC_RIGHT)


class Player(Entity, SpriteRenderable):

    def __init__(self):
        Entity.__init__(self)
        SpriteRenderable.__init__(self, ['$', '[]\n'
                                              '%%'])


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


class Item(SpriteRenderable):
    def __init__(self, name, sprite_sheet: List[str] = None):
        SpriteRenderable.__init__(self, sprite_sheet)
        self.name = name


class Tile(SpriteRenderable):
    def __init__(self, tileid: str, sprite_sheet: List[str] = None, drops: Dict[float, Item] = None):
        SpriteRenderable.__init__(self, sprite_sheet)
        self.tileid = tileid
        self.drops = drops

    def __str__(self):
        return f"<Tile '{self.tileid}': [{self.render_sprite(1)}]>"

    def __eq__(self, other):
        return self.tileid == other.tileid


def gen_tile(choices: List[Tile] = None, weights: List[int] = None) -> Tile:
    if choices is None:
        choices = [Tiles.Tree(),
                   Tiles.Dirt(),
                   Tiles.DaFuq()]

    if weights is None:
        weights = [5, 100, 1]

    if len(weights) != len(choices):
        ve = ValueError(f"Weights and choices for {gen_tile.__name__}() must be the same length!")
        logging.error(ve)
        raise ve

    weights = numpy.asarray(weights)

    normalizedWeights = weights

    if weights.sum() != 1:
        normalizedWeights = weights / weights.sum()

    return numpy.random.choice(choices, p=normalizedWeights)


class Tiles:
    """
    A bunch of default tiles.
    """

    @staticmethod
    def Dirt():
        return Tile('Dirt',
                    [',', ',.\n'
                          '.,', ',.,\n'
                                '.,.\n'
                                ',.,'],
                    {0.99: Items.Rock(),
                     0.01: Items.Gold_Nugget()})

    @staticmethod
    def Tree():
        return Tile('Tree',
                    ['t', '/\\\n'
                          '||', '/|\\\n'
                                ';|;\n'
                                '/|\\\n'],
                    {1.00: Items.Stick()})

    @staticmethod
    def DaFuq():
        return Tile("Dafuq is this?", drops={1.00: Items.Gold_Nugget()})


class World:

    def get_height(self):
        return self.size.y

    def get_width(self):
        return self.size.x

    @staticmethod
    def gen_random_world(size: Vector2) -> List[List[Tile]]:
        width, height = size
        resultworld = []
        for y in range(0, height):
            row = []
            for x in range(0, width):
                row.append(gen_tile())
            resultworld.append(row)
        return resultworld

    def __init__(self, name="Gaia", size=DEFAULT_SIZE):
        self.size = size
        self.name = name
        self.data = World.gen_random_world(size)
        self.gametick = 0

    def get_tile_at(self, pos: Vector2):
        return self.data[pos.y][pos.x]


class Game:
    def __init__(self, player: Player = None, world: World = None, viewport=DEFAULT_VIEWPORT):

        self.viewport = viewport

        if player is None:
            player = Player()

        if world is None:
            world = World()

        self.player = player
        self.world = world

        self.running = True

    def slide_viewport_left(self):
        self.viewport.topleft.x -= 1
        self.viewport.lowerright.x -= 1

    def slide_viewport_right(self):
        self.viewport.topleft.x += 1
        self.viewport.lowerright.x += 1

    def get_tile_at_player_feet(self) -> Tile:
        return self.world.get_tile_at(self.player.position)

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

        for y in range(viewport.topleft.y, viewport.lowerright.y):

            # if this viewport y is out of bounds, skip...
            if (y >= self.world.get_height()) or (y < 0):
                continue

            row = self.world.data[y]
            retrow = []

            for x in range(viewport.topleft.x, viewport.lowerright.x):

                if (x >= self.world.get_width()) or (x < 0):
                    continue

                tile = row[x]
                sprite = tile.render_sprite(scale=scale)

                # if we are here, render us!
                if (self.player.position.y == y) and (self.player.position.x == x):
                    sprite = self.player.render_sprite(scale=scale)

                # logging.debug('render: {}'.format(sprite))
                # done with a single sprite in a row
                retrow.append(sprite)

            # done with a row
            if len(retrow) > 0:
                ret.append(retrow)

        # TODO: Assert ret is well-formed

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

    def move_player(self, vec: Vector2):
        possiblePosition = self.player.calcOffset(vec)

        # check bounds
        if (
                (possiblePosition.x < 0) or
                (possiblePosition.y < 0) or
                (possiblePosition.x >= self.world.get_width()) or
                (possiblePosition.y >= self.world.get_height())
        ):
            logging.debug("Tried to move OOB! {} would have resulted in {}".format(vec, possiblePosition))
            return

        self.player.move(vec)
