import os.path
import unittest
from pathlib import Path

from lithicrivers.game import Tiles, World, WorldData, weighted_choice
from lithicrivers.model.vector import VectorN

filename = Path("testWorld.lithicriversworlddata")


class TestSerialize(unittest.TestCase):
    def setUp(self) -> None:
        if os.path.exists(filename):
            os.remove(filename)

    def tearDown(self) -> None:
        if os.path.exists(filename):
            os.remove(filename)

    def test_simple_ser(self):
        for i in range(-5, 5):
            some_pos = VectorN(i, i, i)
            some_tile = weighted_choice(
                [1, 1, 1], [Tiles.tree(), Tiles.gold_ore(), Tiles.bedrock()]
            )
            some_tile.description = "Wow serialize test!!"

            wd = World.gen_random_world_data(radius=VectorN(5, 5, 5))
            wd.set_tile(some_pos, some_tile)
            # make a world and edit a random block

            # file shoudl not exist
            self.assertFalse(os.path.exists(filename))

            # serialize world to file
            wd.serialize(filename)

            # deserialize it
            unser_wd = WorldData.deserialize(filename)

            # assert the block we changed exists
            self.assertEqual(unser_wd.get_tile(some_pos), some_tile)

            # delete file
            os.remove(filename)
