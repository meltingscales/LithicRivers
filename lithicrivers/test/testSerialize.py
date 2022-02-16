import os.path
import unittest
from pathlib import Path

from lithicrivers.game import WorldData, World, Tiles, weighted_choice
from lithicrivers.model import VectorN

filename = Path('testWorld.lithicriversworlddata')


class TestSerialize(unittest.TestCase):

    def setUp(self) -> None:
        if os.path.exists(filename):
            os.remove(filename)

    def tearDown(self) -> None:
        if os.path.exists(filename):
            os.remove(filename)

    def testSimpleSer(self):
        for i in range(-5, 5):
            somePos = VectorN(i, i, i)
            someTile = weighted_choice([1, 1, 1], [Tiles.Tree(), Tiles.DaFuq(), Tiles.Bedrock()])
            someTile.description = "Wow serialize test!!"

            wd = World.gen_random_world_data(radius=VectorN(5, 5, 5))
            wd.set_tile(somePos, someTile)
            # make a world and edit a random block

            # file shoudl not exist
            self.assertFalse(os.path.exists(filename))

            # serialize world to file
            wd.serialize(filename)

            # deserialize it
            unserWD = WorldData.deserialize(filename)

            # assert the block we changed exists
            self.assertEqual(unserWD.get_tile(somePos), someTile)

            # delete file
            os.remove(filename)

