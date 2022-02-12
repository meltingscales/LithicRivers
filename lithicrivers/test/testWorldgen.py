import unittest

from lithicrivers.game import World, Tiles
from lithicrivers.model import Vector2


class RenderStuff(unittest.TestCase):
    def testworldGen(self):
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
