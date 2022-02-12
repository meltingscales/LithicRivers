import unittest

from lithicrivers.game import Tiles, World
from lithicrivers.model import Vector2


class RenderStuff(unittest.TestCase):
    def testSimpleRender(self):
        self.assertEquals(Tiles.DaFuq().render_sprite(), "?")
        self.assertEquals(Tiles.DaFuq().render_sprite(1), "?")
        self.assertEquals(Tiles.DaFuq().render_sprite(2), "??\n??")

