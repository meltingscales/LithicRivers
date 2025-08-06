"""
Shared test fixtures for optimizing unit test performance.
This module provides pre-generated worlds and other expensive objects that can be reused across tests.
"""

import unittest
from typing import Optional

from lithicrivers.game import World, Game
from lithicrivers.game_engine import GameEngine
from lithicrivers.settings import DEFAULT_SEED

class SharedTestFixtures:
    """
    Singleton class that manages shared test fixtures to reduce test execution time.
    """
    
    _instance: Optional['SharedTestFixtures'] = None
    _initialized = False
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance
    
    def __init__(self):
        if not self._initialized:
            self._initialize_fixtures()
            SharedTestFixtures._initialized = True
    
    def _initialize_fixtures(self):
        """Initialize all shared fixtures. This is called only once."""
        import time
        start_time = time.time()
        print("🏗️  Initializing shared test fixtures...")

        
        # Define seeds to pre-generate (most commonly used in tests)
        self.pregenerated_seeds = [DEFAULT_SEED]
        
        # Use cache variables for all fixtures (pre-populated + dynamic)
        self._world_cache = {}
        self._game_cache = {}
        self._engine_cache = {}
        self._cache_max_size = 50  # Prevent unbounded cache growth
        
        # Pre-populate caches with common seeds
        for seed in self.pregenerated_seeds:
            print(f"   Pre-generating fixture for seed: {seed}")
            self.get_world(seed)
            self.get_game(seed)
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
            self._world_cache[seed] = World(seed=seed)
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
            print(f"Cache MISS: generating game for seed: {seed}")
            # Prevent unbounded cache growth
            if len(self._game_cache) >= self._cache_max_size:
                # Remove oldest entry (simple FIFO)
                oldest_key = next(iter(self._game_cache))
                del self._game_cache[oldest_key]
            self._game_cache[seed] = Game(seed=seed)
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
            self._engine_cache[seed] = GameEngine(seed=seed)
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
    
    def get_world(self, seed: int = DEFAULT_SEED, read_only: bool = True) -> World:
        """Get a world for testing."""
        return self.fixtures.get_world(seed, read_only)
    
    def get_game(self, seed: int = DEFAULT_SEED) -> Game:
        """Get a cloned game for testing."""
        return self.fixtures.get_game(seed)
    
    def get_engine(self, seed: int = DEFAULT_SEED) -> GameEngine:
        """Get a cloned game engine for testing."""
        return self.fixtures.get_engine(seed)
    




