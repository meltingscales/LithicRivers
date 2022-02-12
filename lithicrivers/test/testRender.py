import unittest

from lithicrivers.game import Tiles, World
from lithicrivers.model import Vector2


class RenderStuff(unittest.TestCase):
    def testSimpleRender(self):
        self.assertEquals(Tiles.DaFuq().render_sprite(), "?")
        self.assertEquals(Tiles.DaFuq().render_sprite(1), "?")
        self.assertEquals(Tiles.DaFuq().render_sprite(2), "??\n??")

    def testworldRendering(self):
        for _ in range(0, 100):
            self.assertEquals(
                World.gen_random_world_data(Vector2(1, 1),
                                            gen_function=lambda: Tiles.DaFuq())[0][0],
                Tiles.DaFuq()
            )
