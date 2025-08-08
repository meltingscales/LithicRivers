"""
Shared test fixtures for optimizing unit test performance.
This module provides pre-generated worlds and other expensive objects that can be reused across tests.
"""

import logging
import msgspec
import msgspec.msgpack
import unittest
from pathlib import Path

from lithicrivers.game.core import Game, World, Player
from lithicrivers.settings import DEFAULT_SEED


class SharedTestFixtures:
    """
    Class that manages shared test fixtures to reduce test execution time.
    """

    def __init__(self):
        self._test_saves_dir = Path("lithicrivers-test-saves")

        # Define seeds to pre-generate (most commonly used in tests)
        self.pregenerated_seeds = [DEFAULT_SEED, 42]
        self.pregen_chunk_radius = 4

        # Create directory if it doesn't exist
        self._test_saves_dir.mkdir(exist_ok=True)

    def save_game(self, game: Game, seed: int):
        """Save a game to a file."""
        save_path = self._test_saves_dir / f"seed_{seed}.pkl"

        # Shut down any background threads before pickling to prevent
        # "dictionary changed size during iteration" errors
        if hasattr(game.world, "data") and hasattr(game.world.data, "shutdown"):
            game.world.data.shutdown()

        with save_path.open("wb") as f:
            logging.info(f"Saving game for seed: {seed} to file {save_path}")
            f.write(msgspec.msgpack.encode(game))

    def load_game(self, seed: int) -> Game:
        """Load a game from a file."""
        save_path = self._test_saves_dir / f"seed_{seed}.pkl"
        with save_path.open("rb") as f:
            logging.info(f"Loading game for seed: {seed} from file {save_path}")
            return msgspec.msgpack.decode(f.read(), type=Game)

    def does_save_exist(self, seed: int) -> bool:
        """Check if a save exists for the given seed."""
        save_path = self._test_saves_dir / f"seed_{seed}.pkl"
        return save_path.exists()

    def _initialize_fixtures(self):
        """Initialize all shared fixtures. This is called only once."""
        import time

        start_time = time.time()
        logging.info("🏗️  Initializing shared test fixtures...")

        # Pre-populate files with common seeds
        for seed in self.pregenerated_seeds:
            logging.info(f"   Pre-generating fixture for seed: {seed}")

            # save to disk if it doesn't exist
            if not self.does_save_exist(seed):
                logging.info(
                    f"   Pre-generating chunks for seed: {seed} as it doesn't exist as a save"
                )
                game = Game.create(world=World.create(seed), player=Player.create())
                game.pregen_chunks(radius=self.pregen_chunk_radius)
                self.save_game(game, seed)
                logging.info(f"   Done pre-generating chunks for seed: {seed}")

        init_time = time.time() - start_time
        logging.info(f"✅ Shared test fixtures initialized in {init_time:.3f}s!")
        logging.info(f"   Pre-generated fixtures for seeds: {self.pregenerated_seeds}")

    def get_world(self, seed: int) -> World:
        """
        Load a world from a file with the specified seed.
        """
        if not self.does_save_exist(seed):
            self.save_game(Game.create(world=World.create(seed), player=Player.create()), seed)
        return self.load_game(seed).world

    def get_game(self, seed: int) -> Game:
        """
        Load a game from a file with the specified seed.
        """
        if not self.does_save_exist(seed):
            self.save_game(Game.create(world=World.create(seed), player=Player.create()), seed)
        return self.load_game(seed)


class OptimizedTestCase(unittest.TestCase):
    """
    Base test case class that provides access to shared fixtures.
    Inherit from this instead of unittest.TestCase for faster tests.
    """

    @classmethod
    def setUpClass(cls):
        """Set up shared fixtures once per test class."""
        super().setUpClass()
        cls.fixtures = SharedTestFixtures()

    def get_world(self, seed: int = DEFAULT_SEED) -> World:
        """Get a world for testing."""
        return self.fixtures.get_world(seed)

    def get_game(self, seed: int = DEFAULT_SEED) -> Game:
        """Get a game for testing."""
        return self.fixtures.get_game(seed)
