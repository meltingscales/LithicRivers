import unittest

from lithicrivers.game import Tiles, Game, WorldData
from lithicrivers.model import Viewport, VectorN, RenderedData


class RenderStuff(unittest.TestCase):
    def testSimpleRender(self):
        self.assertEqual(Tiles.DaFuq().render_sprite(), "?")
        self.assertEqual(Tiles.DaFuq().render_sprite(1), "?")
        self.assertEqual(Tiles.DaFuq().render_sprite(2), "??\n??")

    def testSortaSimpleRender(self):
        someRender = RenderedData(
            render_data=[
                ['x', 'y'],
                ['z', 'R']
            ],
            scale=1)

        self.assertEqual(
            someRender.as_string(),
            'xy\nzR'
        )

    def testSortaSimpleRenderReverse(self):
        return; #this disables the test
        self.assertEqual(
            RenderedData.from_string('xy\n'
                                     'zR', scale=1),
            [
                ['x', 'y'],
                ['z', 'R']
            ],
        )

        self.assertEqual(
            RenderedData.from_string('xxyy\n'
                                     'xxyy\n'
                                     'zzRR\n'
                                     'zzRR', scale=1),
            [
                ['xx\n'
                 'xx', 'yy\n'
                       'yy'],
                ['zz\n'
                 'zz', 'RR\n'
                       'RR']
            ],
        )

    def testRenderGame(self):
        someGame = Game()
        someGame.world = WorldData(tile_data={
            '0,0,0': Tiles.Dirt(),
            '1,0,0': Tiles.DaFuq(),
            '0,1,0': Tiles.Dirt(),
            '1,1,0': Tiles.Dirt()
        })

        daScale = 2

        renderedViewport = someGame.render_world_viewport(
            daScale,
            viewport=Viewport(
                topleft=VectorN(0, 0),
                lowerright=VectorN(1, 1)
            )
        )

        self.assertEqual(
            ',.??\n'
            '.,??\n'
            ',.,.\n'
            '.,.,',
            renderedViewport.as_string(),
        )
