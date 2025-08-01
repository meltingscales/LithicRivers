import unittest

from lithicrivers.game import Tiles, Game, WorldData, generate_sprite_repeat
from lithicrivers.model.modelpleasemoveme import RenderedData, Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.textutil import COLOR_MANAGER


class RenderStuff(unittest.TestCase):

    def testgenerate_sprite_repeat(self):
        self.assertEqual('?', generate_sprite_repeat('?', 1))

        self.assertEqual('??\n'
                         '??', generate_sprite_repeat('?', 2), )

        self.assertEqual('???\n'
                         '???\n'
                         '???', generate_sprite_repeat('?', 3))




    def testSimpleRender(self):
        self.assertEqual(Tiles.Gold_Ore().render_sprite(), "?")
        self.assertEqual(Tiles.Gold_Ore().render_sprite(1), "?")
        self.assertEqual(Tiles.Gold_Ore().render_sprite(2), "??\n??")

    def testColorExpansionForScaledSprites(self):
        """Test that color data is properly expanded for scaled sprites."""
        # Create a simple rendered data with 2x2 tiles at scale 2
        render_data = [
            ['a', 'b'],
            ['c', 'd']
        ]
        
        # Create color data for the tiles
        color_data = [
            [(1, 0, 0), (2, 0, 0)],  # Red, Green
            [(3, 0, 0), (4, 0, 0)]   # Yellow, Blue
        ]
        
        rendered_data = RenderedData(render_data, scale=2, color_data=color_data)
        
        # Test that colors are properly accessible at scaled positions
        # For scale 2, each tile becomes 2x2 characters
        # Position (0,0) should have color (1,0,0) for all 4 characters
        self.assertEqual(rendered_data.get_color_at(0, 0), (1, 0, 0))
        self.assertEqual(rendered_data.get_color_at(1, 0), (2, 0, 0))
        self.assertEqual(rendered_data.get_color_at(0, 1), (3, 0, 0))
        self.assertEqual(rendered_data.get_color_at(1, 1), (4, 0, 0))


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
        return None  # this disables the test
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
            '1,0,0': Tiles.Gold_Ore(),
            '0,1,0': Tiles.Dirt(),
            '1,1,0': Tiles.Dirt()
        })

        daScale = 2

        renderedViewport = someGame.render_world_viewport(
            daScale,
            viewport=Viewport(
                top_left=VectorN(0, 0),
                lower_right=VectorN(1, 1)
            )
        )

        self.assertEqual(
            ',.??\n'
            '.,??\n'
            ',.,.\n'
            '.,.,',
            renderedViewport.as_string(),
        )
