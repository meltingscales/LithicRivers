import unittest
from pprint import pprint

from lithicrivers.game import World, Tiles
from lithicrivers.model import Vector2


class RenderStuff(unittest.TestCase):
    def testworldGenConsistent(self):
        for _ in range(0, 100):
            self.assertEqual(
                World.gen_random_world_data(
                    Vector2(3, 3),
                    gen_function=lambda: Tiles.DaFuq()
                ),
                [
                    [
                        Tiles.DaFuq(),
                        Tiles.DaFuq(),
                        Tiles.DaFuq()
                    ],
                    [
                        Tiles.DaFuq(),
                        Tiles.DaFuq(),
                        Tiles.DaFuq()
                    ],
                    [
                        Tiles.DaFuq(),
                        Tiles.DaFuq(),
                        Tiles.DaFuq()
                    ]
                ]
            )

    def testWorldGenRandomSpread(self):

        daSize = 300

        world_data = World.gen_random_world_data(
            size=Vector2(daSize, daSize),
            gf_kwargs={
                "choices": [Tiles.Tree(), Tiles.Dirt()],
                "weights": [50, 50]
            }
        )

        count_tiles = {}
        for row in world_data:
            for tile in row:

                if tile.tileid not in count_tiles:
                    count_tiles[tile.tileid] = 0
                count_tiles[tile.tileid] += 1

        pprint(count_tiles)

        numDirt = count_tiles[Tiles.Dirt().tileid]
        numTree = count_tiles[Tiles.Tree().tileid]

        self.assertAlmostEqual(numDirt / (daSize * daSize),
                               numTree / (daSize * daSize),
                               1)
