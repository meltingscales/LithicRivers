"""
Shared test fixtures for optimizing unit test performance.
This module provides pre-generated worlds and other expensive objects that can be reused across tests.
"""

import unittest
from typing import Optional

from lithicrivers.game import World, Game
from lithicrivers.game_engine import GameEngine


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
        
        # Import DEFAULT_SEED for optimization
        from lithicrivers.settings import DEFAULT_SEED
        
        # Create shared worlds with different seeds
        self.world_seed_42 = World(seed=42)
        self.world_seed_12345 = World(seed=12345)
        self.world_seed_999 = World(seed=999)
        self.world_seed_default = World(seed=DEFAULT_SEED)
        
        # Create shared games
        self.game_seed_42 = Game(seed=42)
        self.game_seed_12345 = Game(seed=12345)
        self.game_seed_default = Game(seed=DEFAULT_SEED)
        
        # Create shared game engines
        self.engine_seed_42 = GameEngine(seed=42)
        self.engine_seed_12345 = GameEngine(seed=12345)
        self.engine_seed_default = GameEngine(seed=DEFAULT_SEED)
        
        # Cache for dynamically created fixtures with size limits
        self._world_cache = {}
        self._game_cache = {}
        self._engine_cache = {}
        self._cache_max_size = 50  # Prevent unbounded cache growth
        
        init_time = time.time() - start_time
        print(f"✅ Shared test fixtures initialized in {init_time:.3f}s!")
    
    def get_shared_world(self, read_only: bool = True) -> World:
        """Get a shared world instance for testing.
        
        Args:
            read_only: If True, returns the shared instance directly (read-only).
                      If False, returns a cloned copy that can be modified.
        """
        if read_only:
            return self.world_seed_42
        else:
            # For tests that need to modify the world, clone the shared instance
            # This is much faster than generating from scratch
            return self.world_seed_42.clone()
    
    def get_world(self, seed: int, read_only: bool = True) -> World:
        """
        Get a world with the specified seed.
        If read_only=True, returns the shared instance (faster but don't modify).
        If read_only=False, creates a new world (slower but safe to modify).
        """
        if read_only:
            # Return shared instance for read-only operations
            from lithicrivers.settings import DEFAULT_SEED
            if seed == 42:
                return self.world_seed_42
            elif seed == 12345:
                return self.world_seed_12345
            elif seed == 999:
                return self.world_seed_999
            elif seed == DEFAULT_SEED:
                return self.world_seed_default
            else:
                # Cache uncommon seeds for read-only access
                if seed not in self._world_cache:
                    # Prevent unbounded cache growth
                    if len(self._world_cache) >= self._cache_max_size:
                        # Remove oldest entry (simple FIFO)
                        oldest_key = next(iter(self._world_cache))
                        del self._world_cache[oldest_key]
                    self._world_cache[seed] = World(seed=seed)
                return self._world_cache[seed]
        else:
            # Always create a new world for modification
            return World(seed=seed)
    
    def get_game(self, seed: int) -> Game:
        """
        Get a cloned game with the specified seed.
        This is much faster than creating a new game from scratch.
        """
        import copy
        
        # Use shared instances for common seeds
        from lithicrivers.settings import DEFAULT_SEED
        if seed == 42:
            base_game = self.game_seed_42
        elif seed == 12345:
            base_game = self.game_seed_12345
        elif seed == DEFAULT_SEED:
            base_game = self.game_seed_default
        else:
            # Cache uncommon seeds for reuse
            if seed not in self._game_cache:
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
        # Use shared instances for common seeds
        from lithicrivers.settings import DEFAULT_SEED
        if seed == 42:
            base_engine = self.engine_seed_42
        elif seed == 12345:
            base_engine = self.engine_seed_12345
        elif seed == DEFAULT_SEED:
            base_engine = self.engine_seed_default
        else:
            # Cache uncommon seeds for reuse
            if seed not in self._engine_cache:
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
    
    def get_world(self, seed: int = 42, read_only: bool = True) -> World:
        """Get a world for testing."""
        return self.fixtures.get_world(seed, read_only)
    
    def get_game(self, seed: int = 42) -> Game:
        """Get a cloned game for testing."""
        return self.fixtures.get_game(seed)
    
    def get_engine(self, seed: int = 42) -> GameEngine:
        """Get a cloned game engine for testing."""
        return self.fixtures.get_engine(seed)
    




