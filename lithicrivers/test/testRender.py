import unittest

from lithicrivers.game import Tiles, Game, WorldData
from lithicrivers.model import Viewport, VectorN


class RenderStuff(unittest.TestCase):
    def testSimpleRender(self):
        self.assertEqual(Tiles.DaFuq().render_sprite(), "?")
        self.assertEqual(Tiles.DaFuq().render_sprite(1), "?")
        self.assertEqual(Tiles.DaFuq().render_sprite(2), "??\n??")

    def testRenderGame(self):
        someGame = Game()
        someGame.world = WorldData(tile_data={
            '0,0,0': Tiles.Dirt(),
            '1,0,0': Tiles.Dirt(),
            '0,1,0': Tiles.Dirt(),
            '1,1,0': Tiles.Dirt()
        })

        daScale = 2

        self.assertEqual(
            [
                [
                    Tiles.Dirt().render_sprite(daScale),
                    ',.\n'
                    '.,'
                ],
                [
                    ',.\n'
                    '.,',
                    ',.\n'
                    '.,'
                ],
            ],
            someGame.render_world_viewport(daScale, viewport=Viewport(topleft=(0, 0), lowerright=(1, 1))),
        )
