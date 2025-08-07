"""
Shared test fixtures for optimizing unit test performance.
This module provides pre-generated worlds and other expensive objects that can be reused across tests.
"""

import unittest
from lithicrivers.game import World, Game
from lithicrivers.game_engine import GameEngine
from lithicrivers.settings import DEFAULT_SEED
from pathlib import Path
import pickle
class SharedTestFixtures:
    """
    Class that manages shared test fixtures to reduce test execution time.
    """
        
    def __init__(self):
        self._test_saves_dir = Path("lithicrivers-test-saves")
        self._test_saves_dir.mkdir(exist_ok=True)
        self._initialize_fixtures()

    def _save_game(self, game: Game, seed: int):
        """Save a game to a file."""
        save_path = self._test_saves_dir / f"seed_{seed}.pkl"
        with save_path.open("wb") as f:
            print(f"Saving game for seed: {seed} to file {save_path}")
            pickle.dump(game, f)
    
    def load_game(self, seed: int) -> Game:
        """Load a game from a file."""
        save_path = self._test_saves_dir / f"seed_{seed}.pkl"
        with save_path.open("rb") as f:
            print(f"Loading game for seed: {seed} from file {save_path}")
            return pickle.load(f)

    def does_save_exist(self, seed: int) -> bool:
        """Check if a save exists for the given seed."""
        save_path = self._test_saves_dir / f"seed_{seed}.pkl"
        return save_path.exists()

    def _initialize_fixtures(self):
        """Initialize all shared fixtures. This is called only once."""
        import time
        start_time = time.time()
        print("🏗️  Initializing shared test fixtures...")

        
        # Define seeds to pre-generate (most commonly used in tests)
        self.pregenerated_seeds = [DEFAULT_SEED, 42]
        self.pregen_chunk_radius = 4
        
        # Use cache variables for all fixtures (pre-populated + dynamic)
        self._world_cache = {}
        self._game_cache = {}
        self._engine_cache = {}
        self._cache_max_size = 50  # Prevent unbounded cache growth
        
        # Pre-populate caches with common seeds
        for seed in self.pregenerated_seeds:
            print(f"   Pre-generating fixture for seed: {seed}")

            # save to disk if it doesn't exist
            if not self.does_save_exist(seed):
                game = Game(seed)
                game.pregen_chunks(radius=self.pregen_chunk_radius)
                self.save_game(game, seed)

            # minor speedup by reusing the game's world
            self._world_cache[seed] = self.get_game(seed).world.clone()

            self.get_world(seed)
            self.get_engine(seed)

        init_time = time.time() - start_time
        print(f"✅ Shared test fixtures initialized in {init_time:.3f}s!")
        print(f"   Pre-generated fixtures for seeds: {self.pregenerated_seeds}")
        
    def get_world(self, seed: int) -> World:
        """
        Get a cloned world with the specified seed.
        This is much faster than creating a new world from scratch.
        """
        # Use cached instances (pre-populated + dynamic)
        if seed not in self._world_cache:
            print(f"Cache MISS: generating world for seed: {seed}")
            # Prevent unbounded cache growth
            if len(self._world_cache) >= self._cache_max_size:
                # Remove oldest entry (simple FIFO)
                oldest_key = next(iter(self._world_cache))
                del self._world_cache[oldest_key]
            self._world_cache[seed] = self.load_game(seed).world.clone()
        base_world = self._world_cache[seed]
        
        # Create a proper clone using the World's clone method
        return base_world.clone()
    
    def get_game(self, seed: int) -> Game:
        """
        Get a cloned game with the specified seed.
        This is much faster than creating a new game from scratch.
        """
        
        # Use cached instances (pre-populated + dynamic)
        if seed not in self._game_cache:

            # if save doesn't exist, create it
            if not self.does_save_exist(seed):
                # create a new game, and it'll get used later
                print(f"Cache MISS: creating and saving game for seed: {seed}")
                game = Game(seed=seed)
                game.pregen_chunks(radius=self.pregen_chunk_radius)
                self.save_game(game, seed)
                
            print(f"Cache MISS: loading game for seed: {seed}")
            # Prevent unbounded cache growth
            if len(self._game_cache) >= self._cache_max_size:
                # Remove oldest entry (simple FIFO)
                oldest_key = next(iter(self._game_cache))
                del self._game_cache[oldest_key]
            self._game_cache[seed] = self.load_game(seed)
        base_game = self._game_cache[seed]
        
        # Create a proper clone using the Game's clone method
        return base_game.clone()
    
    def get_engine(self, seed: int) -> GameEngine:
        """
        Get a cloned game engine with the specified seed.
        This is much faster than creating a new engine from scratch.
        """
        # Use cached instances (pre-populated + dynamic)
        if seed not in self._engine_cache:
            print(f"Cache MISS: generating engine for seed: {seed}")
            # Prevent unbounded cache growth
            if len(self._engine_cache) >= self._cache_max_size:
                # Remove oldest entry (simple FIFO)
                oldest_key = next(iter(self._engine_cache))
                del self._engine_cache[oldest_key]
            self._engine_cache[seed] = self.load_game(seed).engine
        base_engine = self._engine_cache[seed]
        
        # Create a proper clone using the GameEngine's clone method
        return base_engine.clone()


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
        """Get a cloned game for testing."""
        return self.fixtures.get_game(seed)
    
    def get_engine(self, seed: int = DEFAULT_SEED) -> GameEngine:
        """Get a cloned game engine for testing."""
        return self.fixtures.get_engine(seed)
    




