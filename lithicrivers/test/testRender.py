import unittest

from lithicrivers.game import Tiles, World
from lithicrivers.model import Vector2


class RenderStuff(unittest.TestCase):
    def testSimpleRender(self):
        self.assertEqual(Tiles.DaFuq().render_sprite(), "?")
        self.assertEqual(Tiles.DaFuq().render_sprite(1), "?")
        self.assertEqual(Tiles.DaFuq().render_sprite(2), "??\n??")

