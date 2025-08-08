import concurrent.futures
import logging
import os
import msgspec
import msgspec.msgpack
import pprint
import threading
from concurrent.futures import ProcessPoolExecutor, ThreadPoolExecutor
from datetime import datetime
from pathlib import Path
from typing import Optional, Union, TYPE_CHECKING

from lithicrivers.colors import COLOR_MANAGER
from lithicrivers.constants import VEC_NORTH
from lithicrivers.game.entities import DroppedItem, Entities, Entity, Item, Items
from lithicrivers.game.events import EntityListener, EntityMovedEvent
from lithicrivers.game.fluids import FluidManager
from lithicrivers.game.interfaces import Cloneable, ShutDownable, SpriteRenderable
from lithicrivers.game.tiles import Tile, Tiles
from lithicrivers.model.body import Body
from lithicrivers.model.model import RenderedData, Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.settings import (
    DEFAULT_PLAYER_NAME,
    DEFAULT_PLAYER_POSITION,
    DEFAULT_VIEWPORT,
)
from lithicrivers.textutil import get_color_for_item, get_color_for_tile
from lithicrivers.worldgen import Chunk, SeededWorldGenerator

from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from lithicrivers.game.game_save_manager import GameSaveManager

# Import GameSaveManager at runtime for msgspec compatibility
try:
    from lithicrivers.game.game_save_manager import GameSaveManager
except ImportError:
    GameSaveManager = None

class Player(Entity, msgspec.Struct, frozen=False):
    name: str
    position: VectorN = msgspec.field(default_factory=lambda: DEFAULT_PLAYER_POSITION)
    sprite_sheet: Optional[list[str]] = None

    @classmethod
    def create(cls, name: str = DEFAULT_PLAYER_NAME, position: VectorN = DEFAULT_PLAYER_POSITION) -> "Player":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("player", "entities")
        return cls(name=name, position=position, sprite_sheet=sprite_data.sprites)

        # Use external sprite data
        SpriteRenderable.__init__(self, sprite_data.sprites)

        self.inventory = Inventory([Item("Cookie", ["o"])])
        self.body = Body()  # Initialize with damaged android body

        # Apply body modifiers to base stats
        self._update_stats_from_body()

        # Add debug items for body repairs
        self._add_debug_repair_items()

    def _update_stats_from_body(self) -> None:
        """Update player stats based on body condition."""
        health_modifier = self.body.get_total_health_modifier()
        stamina_modifier = self.body.get_total_stamina_modifier()

        # Apply modifiers to base stats (100 each)
        self.health = max(1, 100 + health_modifier)
        self.stamina = max(1, 100 + stamina_modifier)

    def _add_debug_repair_items(self) -> None:
        """Add double the necessary items to repair all body parts for debug purposes."""
        repair_requirements = self.body.get_repair_requirements()

        # Count total items needed
        total_items_needed = {}
        for part_type, costs in repair_requirements.items():
            for item_name, amount in costs.items():
                if item_name in total_items_needed:
                    total_items_needed[item_name] += amount
                else:
                    total_items_needed[item_name] = amount

        # Add double the required items to inventory
        for item_name, amount in total_items_needed.items():
            # Create the item based on name
            if item_name == "iron_scrap":
                item = Items.iron_scrap()
            elif item_name == "scrap_electronics":
                item = Items.scrap_electronics()
            else:
                # Fallback for unknown items
                item = Item(item_name, ["?"])

            # Add double the amount needed
            for _ in range(amount * 2):
                self.inventory.add_item(item)

    def get_walk_speed_modifier(self) -> float:
        """Get the player's walk speed modifier based on body condition."""
        return self.body.get_total_walk_speed_modifier()

    def get_break_speed_modifier(self) -> float:
        """Get the player's break speed modifier based on body condition."""
        return self.body.get_total_break_speed_modifier()

    def can_perform_action(self, action_type: str) -> bool:
        """Check if the player can perform a specific action based on body condition."""
        return self.body.can_perform_action(action_type)

    def get_action_speed(self, action_type: str) -> float:
        """Get the speed modifier for a specific action."""
        return self.body.get_action_speed(action_type)

    def get_body_status_summary(self) -> str:
        """Get a summary of the player's body condition."""
        return self.body.get_body_status_summary()

    def get_movement_penalty_description(self) -> str:
        """Get a description of current movement penalties."""
        return self.body.get_movement_penalty_description()

    def repair_body_part(self, part_type: str) -> bool:
        """Attempt to repair a body part. Returns True if successful."""
        # This will be implemented when we add crafting
        # For now, just return False
        return False

    def tick(self) -> None:
        """Called each game tick. Override when I add poison damage, for example."""
        pass


class MessageLog(msgspec.Struct, frozen=False):
    """A class to manage game messages for the message log pane."""
    messages: list[dict] = msgspec.field(default_factory=list)
    max_messages: int = 100
    game: Optional["Game"] = None

    @classmethod
    def create(cls, max_messages: int = 100, game: Optional["Game"] = None) -> "MessageLog":
        return cls(messages=[], max_messages=max_messages, game=game)

    def add_message(self, message: str, message_type: str = "info"):
        """Add a message to the log with timestamp, game tick, and type."""
        timestamp = datetime.now().strftime("%H:%M:%S")
        gametick = self.game.gametick if self.game else 0
        log_entry = {
            "timestamp": timestamp,
            "gametick": gametick,
            "message": message,
            "type": message_type,  # info, mining, interaction, pickup, dialog
        }

        self.messages.append(log_entry)

        # Keep only the most recent messages
        if len(self.messages) > self.max_messages:
            self.messages.pop(0)

    def get_messages(self, message_type: Optional[str] = None) -> list[dict]:
        """Get all messages, optionally filtered by type."""
        if message_type is None:
            return self.messages.copy()
        return [msg for msg in self.messages if msg["type"] == message_type]

    def get_recent_messages(self, count: int = 20) -> list[dict]:
        """Get the most recent messages."""
        return self.messages[-count:] if self.messages else []

    def clear(self):
        """Clear all messages."""
        self.messages.clear()

class ChunkedWorldData(Cloneable, ShutDownable, msgspec.Struct, frozen=False):
    """
    Efficient world data storage using chunked 3D arrays.
    Similar to Minecraft's world storage system.
    """

    chunk_size: int
    chunks: dict[tuple[int, int, int], Chunk]
    entity_data: dict[str, Entity]
    _tile_cache: dict[str, Tile]
    _cache_size: int
    world_generator: "SeededWorldGenerator"
    _generated_chunks: set[tuple[int, int, int]]
    _chunk_generation_lock: threading.Lock
    _thread_pool: ThreadPoolExecutor

    @classmethod
    def create(cls, chunk_size: int = None, world_generator=None):
        instance = cls(chunk_size=chunk_size, world_generator=world_generator)

        instance.chunks = {}  # (chunk_x, chunk_y, chunk_z) -> Chunk
        instance.entity_data = {}  # Entity storage
        instance._tile_cache = {}  # Cache for frequently accessed tiles
        instance._cache_size = 1000  # Max cache size
        instance.world_generator = (
            world_generator  # Reference to world generator for structure generation
        )
        instance._generated_chunks = (
            set()
        )  # Track which chunks have had structures generated
        instance._chunk_generation_lock = (
            threading.Lock()
        )  # Lock for thread-safe chunk generation
        from lithicrivers.settings import MAX_CPU_THREADS

        instance._thread_pool = ThreadPoolExecutor(
            max_workers=MAX_CPU_THREADS
        )  # Thread pool for chunk generation
        return instance

    def pregen_chunks(self, radius: int) -> None:
        """Pre-generate all chunks within a cubic radius around (0,0,0) using process pool. Waits for all processes to finish.
        This should only be called when the world generator is initialized, or during unit tests to speed them up if pickling after generating."""
        if not self.world_generator:
            raise Exception("World generator is not initialized")
        seed = getattr(self.world_generator, "seed", None)
        if hasattr(seed, "seed"):
            seed = seed.seed
        if seed is None:
            raise Exception(
                "World generator must have a .seed or .seed.seed attribute for process pool pregen"
            )
        chunk_size = self.chunk_size
        chunk_radius = (radius + chunk_size - 1) // chunk_size  # ceil division
        # Prepare all chunk coords to generate
        chunk_coords = [
            (cx, cy, cz)
            for cx in range(-chunk_radius, chunk_radius + 1)
            for cy in range(-chunk_radius, chunk_radius + 1)
            for cz in range(-chunk_radius, chunk_radius + 1)
            if (cx, cy, cz) not in self._generated_chunks
        ]
        # Mark as generated to avoid duplicate work
        for chunk_key in chunk_coords:
            self._generated_chunks.add(chunk_key)
        with ProcessPoolExecutor() as pool:
            future_to_chunk = {
                pool.submit(_generate_chunk_data_for_process, seed, cx, cy, cz): (
                    cx,
                    cy,
                    cz,
                )
                for (cx, cy, cz) in chunk_coords
            }
            for future in concurrent.futures.as_completed(future_to_chunk):
                cx, cy, cz, chunk_data = future.result()
                # Insert chunk data into self.chunks
                if (cx, cy, cz) not in self.chunks:
                    self.chunks[(cx, cy, cz)] = Chunk(self.chunk_size)
                chunk = self.chunks[(cx, cy, cz)]
                for pos_str, tile in chunk_data.items():
                    pos_parts = [int(x) for x in pos_str.split(",")]
                    world_pos = VectorN.from_args(*pos_parts)
                    local_pos = chunk.get_local_pos(world_pos)
                    chunk.set_tile(local_pos, tile)
                    pos_key = (world_pos.x, world_pos.y, world_pos.z)
                    self._tile_cache[pos_key] = tile

    def __getstate__(self):
        state = self.__dict__.copy()
        # Remove unserializable objects for msgspec serialization
        state.pop("_chunk_generation_lock", None)
        state.pop("_thread_pool", None)
        return state

    def __setstate__(self, state):
        self.__dict__.update(state)
        import threading
        from concurrent.futures import ThreadPoolExecutor

        from lithicrivers.settings import MAX_CPU_THREADS

        self._chunk_generation_lock = threading.Lock()
        self._thread_pool = ThreadPoolExecutor(max_workers=MAX_CPU_THREADS)

    def __deepcopy__(self, memo):
        """
        Custom deepcopy implementation that handles threading primitives properly.
        """
        import copy

        from lithicrivers.worldgen import SeededWorldGenerator

        # Create new ChunkedWorldData with same parameters
        cloned_data = ChunkedWorldData.__new__(ChunkedWorldData)
        cloned_data.chunk_size = self.chunk_size
        cloned_data._cache_size = self._cache_size

        # Create a new world generator with the same seed instead of deep copying
        if self.world_generator:
            cloned_data.world_generator = SeededWorldGenerator(
                self.world_generator.seed.seed
            )
        else:
            cloned_data.world_generator = None

        # Deep copy chunks
        cloned_data.chunks = copy.deepcopy(self.chunks, memo)

        # Deep copy other data structures
        cloned_data.entity_data = copy.deepcopy(self.entity_data, memo)
        cloned_data._tile_cache = copy.deepcopy(self._tile_cache, memo)
        cloned_data._generated_chunks = copy.deepcopy(self._generated_chunks, memo)

        # Create new threading primitives (can't be copied)
        cloned_data._chunk_generation_lock = threading.Lock()
        from lithicrivers.settings import MAX_CPU_THREADS

        cloned_data._thread_pool = ThreadPoolExecutor(max_workers=MAX_CPU_THREADS)

        return cloned_data

    def clone(self) -> "ChunkedWorldData":
        """
        Create a deep copy of this ChunkedWorldData for testing purposes.
        This is more efficient than regenerating all chunks from scratch.
        """
        import copy

        return copy.deepcopy(self)

    def get_chunk_key(self, pos: VectorN) -> tuple[int, int, int]:
        """Get chunk coordinates from world position."""
        return (
            pos.x // self.chunk_size,
            pos.y // self.chunk_size,
            pos.z // self.chunk_size,
        )

    def get_chunk(self, chunk_key: tuple[int, int, int]) -> Chunk:
        """Get or create a chunk."""
        if chunk_key not in self.chunks:
            self.chunks[chunk_key] = Chunk(self.chunk_size)
            # Generate structures for this chunk if it's the first time
            if self.world_generator and chunk_key not in self._generated_chunks:
                # Mark as generated first to prevent infinite recursion
                self._generated_chunks.add(chunk_key)
                # Use threaded chunk generation for lazy loading
                self._generate_chunk_threaded(chunk_key)
        return self.chunks[chunk_key]

    def _generate_chunk_threaded(self, chunk_key: tuple[int, int, int]) -> None:
        """
        Generate a chunk using threaded generation for lazy loading.

        Args:
            chunk_key: The chunk coordinates (chunk_x, chunk_y, chunk_z)
        """
        if not self.world_generator:
            return

        # Skip structure generation during testing to speed up tests
        if os.environ.get("TESTING") == "1":
            return

        chunk_x, chunk_y, chunk_z = chunk_key

        # Submit chunk generation to thread pool
        future = self._thread_pool.submit(
            self._generate_chunk_worker, chunk_x, chunk_y, chunk_z
        )

        # Store the future for later retrieval if needed
        # For now, we'll let it run in the background
        # In a more sophisticated implementation, we could track futures and wait for them

    def _generate_chunk_worker(self, chunk_x: int, chunk_y: int, chunk_z: int) -> None:
        """
        Worker method that runs in a separate thread to generate a chunk.

        Args:
            chunk_x: Chunk X coordinate
            chunk_y: Chunk Y coordinate
            chunk_z: Chunk Z coordinate
        """
        try:
            # Use the world generator's threaded chunk generation
            self.world_generator._generate_complete_chunk(chunk_x, chunk_y, chunk_z)

            # Apply the generated chunk data to our chunked world
            chunk_data = self.world_generator.chunk_cache.cache.get(
                (chunk_x, chunk_y, chunk_z), {}
            )

            with self._chunk_generation_lock:
                tiles_applied = 0
                for pos_str, tile in chunk_data.items():
                    pos_parts = pos_str.split(",")
                    world_pos = VectorN(
                        int(pos_parts[0]), int(pos_parts[1]), int(pos_parts[2])
                    )

                    # Get chunk directly without triggering generation
                    chunk_key = self.get_chunk_key(world_pos)
                    if chunk_key in self.chunks:
                        chunk = self.chunks[chunk_key]
                        local_pos = chunk.get_local_pos(world_pos)
                        chunk.set_tile(local_pos, tile)

                        # Update cache
                        pos_key = (world_pos.x, world_pos.y, world_pos.z)
                        self._tile_cache[pos_key] = tile
                        tiles_applied += 1
        except Exception as e:
            # Log any errors that occur during chunk generation
            import logging

            logging.warning(
                f"Error generating chunk ({chunk_x}, {chunk_y}, {chunk_z}): {e}"
            )

    def get_tile(self, pos: VectorN) -> Union[Tile, None]:
        """Get tile at world position."""
        # Check cache first
        pos_key = (pos.x, pos.y, pos.z)
        if pos_key in self._tile_cache:
            return self._tile_cache[pos_key]

        # Get chunk and local position
        chunk_key = self.get_chunk_key(pos)
        chunk = self.get_chunk(chunk_key)
        local_pos = chunk.get_local_pos(pos)

        # Get tile from chunk
        tile = chunk.get_tile(local_pos)

        # Cache the result (but limit cache size)
        if len(self._tile_cache) < self._cache_size:
            self._tile_cache[pos_key] = tile

        return tile if tile != chunk.palette.get_empty_tile() else None

    def set_tile(self, pos: VectorN, tile: Tile) -> None:
        """Set tile at world position."""
        chunk_key = self.get_chunk_key(pos)
        chunk = self.get_chunk(chunk_key)
        local_pos = chunk.get_local_pos(pos)

        chunk.set_tile(local_pos, tile)

        # Update cache
        pos_key = (pos.x, pos.y, pos.z)
        self._tile_cache[pos_key] = tile

    def clear_cache(self) -> None:
        """Clear the tile cache."""
        self._tile_cache.clear()

    def shutdown(self) -> None:
        """Shutdown the thread pool and clean up resources."""
        if hasattr(self, "_thread_pool"):
            # Shutdown the thread pool and wait for all threads to complete
            # This prevents the game from hanging due to background threads
            print("Shutting down thread pool...")
            self._thread_pool.shutdown(wait=True)
            print("Thread pool shutdown complete.")

    def get_chunk_stats(self) -> dict:
        """Get statistics about chunk usage."""
        total_chunks = len(self.chunks)
        empty_chunks = sum(1 for chunk in self.chunks.values() if chunk.is_empty())
        total_tiles = sum(len(chunk.palette) for chunk in self.chunks.values())

        return {
            "total_chunks": total_chunks,
            "empty_chunks": empty_chunks,
            "used_chunks": total_chunks - empty_chunks,
            "total_tile_types": total_tiles,
            "cache_size": len(self._tile_cache),
        }

    def serialize(self, filepath: Path) -> Path:
        """Serialize the chunked world data."""
        with open(filepath, "wb") as fh:
            fh.write(msgspec.encode(self))
        return filepath

    @staticmethod
    def deserialize(filepath: Path):
        """Deserialize the chunked world data."""
        with open(filepath, "rb") as fh:
            return msgspec.msgpack.decode(fh.read(), type=type(self))

    def __getitem__(self, *item: int):
        return self.get_tile(VectorN.from_args(*item))

    def __setitem__(self, *item: int):
        self.set_tile(VectorN.from_args(*item))

    def __iter__(self):
        """Iterate over all tiles in all chunks."""
        for chunk_key, chunk in self.chunks.items():
            for x in range(self.chunk_size):
                for y in range(self.chunk_size):
                    for z in range(self.chunk_size):
                        tile = chunk.get_tile((x, y, z))
                        if tile != chunk.palette.get_empty_tile():
                            world_x = chunk_key[0] * self.chunk_size + x
                            world_y = chunk_key[1] * self.chunk_size + y
                            world_z = chunk_key[2] * self.chunk_size + z
                            pos = VectorN.from_args(world_x, world_y, world_z)
                            yield (pos.serialize(), tile)


class Inventory(msgspec.Struct, frozen=False):
    itemsdata: list["Item"] = msgspec.field(default_factory=list)

    @classmethod
    def create(cls, items: Optional[list["Item"]] = None) -> "Inventory":
        if items is None:
            items = []
        return cls(itemsdata=items)

    def add_item(self, item: "Item") -> None:
        self.itemsdata.append(item)

    def __str__(self) -> str:
        return f"<Inventory numItems={len(self.itemsdata)} summary={self.summary()}>"

    def count_items(self) -> dict[str, int]:
        d = {}

        for item in self.itemsdata:
            if item.name in d:
                d[item.name] += 1
            else:
                d[item.name] = 1

        return d

    def summary(self) -> str:
        s = ""

        for k, v in self.count_items().items():
            s += f"{k}={v}, "

        return s[0 : len(s) - 2]

    def colored_summary(self) -> str:
        """Generate a colored summary of inventory items."""
        s = ""

        for k, v in self.count_items().items():
            # Get color for this item
            item_color = get_color_for_item(k)
            color_name = COLOR_MANAGER.get_color_name(item_color)
            s += f"{k}={v} ({color_name}), "

        return s[0 : len(s) - 2] if s else "Empty"


def _generate_chunk_data_for_process(seed, chunk_x, chunk_y, chunk_z):
    # This function runs in a separate process
    generator = SeededWorldGenerator(seed)
    generator._generate_complete_chunk(chunk_x, chunk_y, chunk_z)
    # Get chunk data from the generator's chunk cache
    chunk_data = generator.chunk_cache.cache.get((chunk_x, chunk_y, chunk_z), {})
    # Return as a dict mapping pos_str to tile (must be serializable)
    return (chunk_x, chunk_y, chunk_z, chunk_data)


class World(Cloneable, ShutDownable, EntityListener, msgspec.Struct, frozen=False):
    """
    A world contains world data and manages the world state.
    """

    name: str
    seed: int
    generator: "SeededWorldGenerator"
    data: "ChunkedWorldData"
    gametick: int
    entities_by_position: dict[tuple[int, int, int], list[Entity]]
    fluid_manager: "FluidManager"

    @classmethod
    def create(cls, seed: int, name="Gaia") -> "World":
        instance = cls(seed=seed, name=name)
        instance._add_starter_entities()
        instance._generate_forced_structures()
        instance.generator = SeededWorldGenerator(seed)
        instance.data = ChunkedWorldData(world_generator=instance.generator)
        return instance

    def __init___disabled_due_to_msgspec(self, seed: int, name="Gaia"):
        self.name = name
        self.seed = seed

        # Create ONE generator that will be reused
        from lithicrivers.worldgen import SeededWorldGenerator

        self.generator = SeededWorldGenerator(seed)

        # Start with empty world data - everything will be generated lazily
        # Pass the generator so structures can be generated when chunks are loaded
        self.data = ChunkedWorldData(world_generator=self.generator)
        self.gametick = 0

        # Change entity storage to support multiple entities per position
        # Map position tuples to lists of entities
        self.entities_by_position = {}  # (x, y, z) -> list[Entity]

        # Initialize fluid manager
        self.fluid_manager = FluidManager(self)

        self._add_starter_entities()

        # Generate forced structures for quests and main story content
        self._generate_forced_structures()

    def pregen_chunks(self, radius: int) -> None:
        """Pre-generate chunks for the world."""
        self.data.pregen_chunks(radius)

    def clone(self) -> "World":
        """
        Create a deep copy of this World for testing purposes.
        This is more efficient than creating a new world from scratch.
        """
        import copy

        cloned_world = copy.deepcopy(self)
        # Re-establish entity listeners after cloning
        cloned_world._reestablish_entity_listeners()
        return cloned_world

    def shutdown(self) -> None:
        """Shutdown the world."""
        self.data.shutdown()

    def _reestablish_entity_listeners(self) -> None:
        """Re-establish entity listeners after deserialization or cloning."""
        # Clear existing listeners and re-add the world as a listener to all entities
        for entities in self.entities_by_position.values():
            for entity in entities:
                entity._listeners.clear()
                entity.add_listener(self)

    def __getstate__(self):
        """Custom pickle serialization that handles threading primitives."""
        state = self.__dict__.copy()
        # Don't serialize the generator - we'll recreate it on load
        if "generator" in state:
            del state["generator"]
        return state

    def __setstate__(self, state):
        """Custom pickle deserialization that recreates threading primitives."""
        self.__dict__.update(state)
        # Recreate the world generator with the same seed
        from lithicrivers.worldgen import SeededWorldGenerator

        self.generator = SeededWorldGenerator(self.seed)
        # Re-establish entity listeners
        self._reestablish_entity_listeners()

    def _generate_forced_structures(self) -> None:
        """
        Generate forced structures for quests and main story content.

        This method creates specific structures at predetermined locations that are
        essential for the game's narrative and quest progression. These structures
        are generated during world initialization and are separate from the procedural
        structure generation that happens when chunks are loaded.

        Note: This is different from the procedural structure generation that happens
        when chunks are loaded. This method is for story-critical structures only.
        """

        # TODO actually fix this, see TODO.md
        '''
        - todo: MANUAL TASK: Go into `worldgen.py` and make a way to force a specific structure to generate, then fix `_generate_forced_structures`... Don't use AI as it seems to get confused.
        '''

        # print("Generating forced structures for seed: ", self.seed)
        # # Force a ship to spawn at (20, 20, 0) - Main story location
        # forced_ship_pos = VectorN(20, 20, 0)
        # print(f"FORCING SHIP TO SPAWN AT {forced_ship_pos}")  # Debug output

        # # Generate a small world around the ship to place it
        # ship_radius = VectorN(25, 25, 2)
        # ship_world_data = self.generator.generate_world_data(ship_radius)
        #
        # # Apply the ship world data to our chunked world
        # for pos_str, tile in ship_world_data.items():
        #     pos_parts = pos_str.split(",")
        #     world_pos = VectorN(int(pos_parts[0]), int(pos_parts[1]), int(pos_parts[2]))
        #     self.data.set_tile(world_pos, tile)
        #
        # # Force a procedural dungeon to spawn at (50, 50, -3) - Quest location
        # forced_dungeon_pos = VectorN(50, 50, -3)
        # print(
        #     f"FORCING UNDERGROUND FACILITY TO SPAWN AT {forced_dungeon_pos}"
        # )  # Debug output
        #
        # # Generate a small world around the dungeon to place it
        # dungeon_radius = VectorN(55, 55, 5)
        # dungeon_world_data = self.generator.generate_world_data(dungeon_radius)
        #
        # # Apply the dungeon world data to our chunked world
        # for pos_str, tile in dungeon_world_data.items():
        #     pos_parts = pos_str.split(",")
        #     world_pos = VectorN(int(pos_parts[0]), int(pos_parts[1]), int(pos_parts[2]))
        #     self.data.set_tile(world_pos, tile)

    def on_entity_moved(self, event: EntityMovedEvent) -> None:
        """Handle entity movement events."""
        # Remove from old position
        old_pos_key = self._get_position_key(event.old_position)
        if old_pos_key in self.entities_by_position:
            try:
                self.entities_by_position[old_pos_key].remove(event.entity)
                # Clean up empty position entries
                if not self.entities_by_position[old_pos_key]:
                    del self.entities_by_position[old_pos_key]
            except ValueError:
                # Entity not found at position, ignore
                pass

        # Add to new position
        new_pos_key = self._get_position_key(event.new_position)
        if new_pos_key not in self.entities_by_position:
            self.entities_by_position[new_pos_key] = []
        self.entities_by_position[new_pos_key].append(event.entity)

    def _add_starter_entities(self):
        """Add starter entities to the world."""
        # Add NPC
        npc = Entities.starter_npc()
        self.add_entity(npc)

        # Add test entities
        entity1 = Entities.test_entity1()
        entity2 = Entities.test_entity2()
        self.add_entity(entity1)
        self.add_entity(entity2)

        # Add StumblingSheep 2 blocks north of player spawn
        sheep_position = DEFAULT_PLAYER_POSITION + (VEC_NORTH * 2)
        sheep = Entities.stumbling_sheep(sheep_position)
        self.add_entity(sheep)

        # Add some test fluids to demonstrate the system
        # Water pool near the player
        water_pos = DEFAULT_PLAYER_POSITION + VectorN(1, 0, 0)
        water = Entities.water(water_pos, amount=1000)
        self.fluid_manager.add_fluid(water)

        # Lava pool further away
        lava_pos = DEFAULT_PLAYER_POSITION + VectorN(-3, -1, 0)
        lava = Entities.lava(lava_pos, amount=1000)
        self.fluid_manager.add_fluid(lava)

        # Acid pool
        acid_pos = DEFAULT_PLAYER_POSITION + VectorN(0, 2, 0)
        acid = Entities.acid(acid_pos, amount=1000)
        self.fluid_manager.add_fluid(acid)

        # BIG oil pool further away (10x normal amount)
        oil_pos = DEFAULT_PLAYER_POSITION + VectorN(0, -10, 0)
        oil = Entities.oil(oil_pos, amount=1000)  # Max amount per tile
        self.fluid_manager.add_fluid(oil)

        # Add multiple oil tiles to create a big pool
        for dx in range(-1, 2):
            for dy in range(-1, 2):
                if dx != 0 or dy != 0:  # Skip the center tile (already added)
                    oil_tile_pos = oil_pos + VectorN(dx, dy, 0)
                    oil_tile = Entities.oil(oil_tile_pos, amount=1000)
                    self.fluid_manager.add_fluid(oil_tile)

    def get_tile(self, pos: VectorN):
        tile = self.data.get_tile(pos)
        if tile is None:
            # Use the stored generator instead of creating a new one
            tile = self.generator.generate_tile_for_position(pos)
            self.data.set_tile(pos, tile)
        return tile

    def set_tile(self, pos: VectorN, tile: Tile):
        self.data.set_tile(pos, tile)

    def _get_position_key(self, pos: VectorN) -> tuple[int, int, int]:
        """Convert VectorN to tuple key for dictionary storage."""
        return (pos.x, pos.y, pos.z)

    def get_entities(self, pos: VectorN) -> list["Entity"]:
        """Get all entities at a position."""
        pos_key = self._get_position_key(pos)
        return self.entities_by_position.get(pos_key, [])

    def get_entity(self, pos: VectorN) -> Optional["Entity"]:
        """Get the first entity at a position (for backward compatibility)."""
        entities = self.get_entities(pos)
        return entities[0] if entities else None

    def add_entity(self, entity: "Entity") -> None:
        """Add an entity to the world."""
        pos_key = self._get_position_key(entity.position)
        if pos_key not in self.entities_by_position:
            self.entities_by_position[pos_key] = []
        self.entities_by_position[pos_key].append(entity)

        # Register as listener for movement events
        entity.add_listener(self)

    def remove_entity(self, entity: "Entity") -> None:
        """Remove an entity from the world."""
        pos_key = self._get_position_key(entity.position)
        if pos_key in self.entities_by_position:
            try:
                self.entities_by_position[pos_key].remove(entity)
                # Clean up empty position entries
                if not self.entities_by_position[pos_key]:
                    del self.entities_by_position[pos_key]
            except ValueError:
                # Entity not found at position, ignore
                pass

        # Unregister as listener
        entity.remove_listener(self)

    def move_entity(self, entity: "Entity", new_position: VectorN) -> None:
        """Move an entity from its current position to a new position."""
        # Remove from old position
        self.remove_entity(entity)

        # Update entity's position
        entity.position = new_position

        # Add to new position
        self.add_entity(entity)

    def get_adjacent_entities(self, pos: VectorN) -> list[tuple[str, VectorN, str]]:
        """Get all entities adjacent to a position."""
        adjacent = []
        for dx in [-1, 0, 1]:
            for dy in [-1, 0, 1]:
                check_pos = VectorN(pos.x + dx, pos.y + dy, pos.z)
                entities = self.get_entities(check_pos)
                for entity in entities:
                    color = entity.color if hasattr(entity, "color") else "white"
                    adjacent.append((entity.name, check_pos, color))

        return adjacent

    def get_all_entities(self) -> list["Entity"]:
        """Get all entities in the world."""
        all_entities = []
        for entities in self.entities_by_position.values():
            all_entities.extend(entities)
        return all_entities

    def get_priority_entity(self, pos: VectorN) -> Optional["Entity"]:
        """
        Get the highest priority entity at a position for rendering.
        Priority order: Player > NPCs > Interactive Entities > Dropped Items
        """
        entities = self.get_entities(pos)
        if not entities:
            return None

        if len(entities) == 1:
            return entities[0]

        # Sort entities by priority
        def get_priority(entity):
            if hasattr(entity, "get_conversation"):  # NPCs
                return 3
            elif hasattr(entity, "interact"):  # Interactive entities
                return 2
            elif isinstance(entity, DroppedItem):  # Dropped items
                return 1
            else:
                return 0

        # Sort by priority (highest first) and return the first one
        sorted_entities = sorted(entities, key=get_priority, reverse=True)
        return sorted_entities[0]

    def get_entity_count(self, pos: VectorN) -> int:
        """Get the number of entities at a position."""
        return len(self.get_entities(pos))

    def test_multiple_entities(self) -> bool:
        """
        Test method to verify that multiple entities can exist at the same position.
        Returns True if the test passes.
        """
        # Create test position
        test_pos = VectorN(10, 10, 0)

        # Create multiple dropped items at the same position
        item1 = Items.rock()
        item2 = Items.gold_nugget()
        item3 = Items.stick()

        dropped1 = DroppedItem(item1, test_pos)
        dropped2 = DroppedItem(item2, test_pos)
        dropped3 = DroppedItem(item3, test_pos)

        # Add all entities
        self.add_entity(dropped1)
        self.add_entity(dropped2)
        self.add_entity(dropped3)

        # Verify we can retrieve all entities
        entities = self.get_entities(test_pos)
        if len(entities) != 3:
            return False

        # Verify entity names are correct
        entity_names = [e.name for e in entities]
        expected_names = ["Rock", "Gold Nugget", "Stick"]
        if set(entity_names) != set(expected_names):
            return False

        # Verify priority entity is correct (should be the first one added)
        priority_entity = self.get_priority_entity(test_pos)
        if priority_entity is None:
            return False

        # Clean up test entities
        self.remove_entity(dropped1)
        self.remove_entity(dropped2)
        self.remove_entity(dropped3)

        return True

class Game(Cloneable, ShutDownable, msgspec.Struct, frozen=False):
    """Main game class. Meant to hold all game state. Can be pickled to save the game."""

    player: Player
    world: World
    viewport: Viewport
    save_manager: "GameSaveManager"
    running: bool
    message_log: "MessageLog"
    gametick: int

    @classmethod
    def create(cls, seed: int, player: Player = None, world: World = None, viewport: Viewport = DEFAULT_VIEWPORT, save_manager: "GameSaveManager" = None) -> "Game":
        instance = cls(seed=seed, player=player, world=world, viewport=viewport, save_manager=save_manager)

        instance.running = True
        instance.message_log = MessageLog(game=instance)
        instance.gametick = 0 

        if viewport is DEFAULT_VIEWPORT:
            instance.viewport = Viewport(
                top_left=VectorN(
                    viewport.top_left.x, viewport.top_left.y, viewport.top_left.z
                ),
                lower_right=VectorN(
                    viewport.lower_right.x,
                    viewport.lower_right.y,
                    viewport.lower_right.z,
                ),
                scale=viewport.scale,
            )
        else:
            instance.viewport = viewport

        # Add initial welcome message
        instance.message_log.add_message(
            "Welcome to LithicRivers! Your adventures will be logged here.", "info"
        )
        return instance

    def clone(self) -> "Game":
        """Create a deep copy of this Game for pickling or testing."""
        import copy

        cloned_game = copy.deepcopy(self)
        # Re-establish entity listeners after cloning
        cloned_game.world._reestablish_entity_listeners()
        return cloned_game

    def pregen_chunks(self, radius: int) -> None:
        """Pre-generate chunks for the world."""
        self.world.pregen_chunks(radius)

    def shutdown(self) -> None:
        """Shutdown the game. Does not destroy any resources. Meant to be called before serializing."""
        self.world.shutdown()
        self.running = False

    def get_tile_at_player_feet(self) -> Tile:
        return self.world.get_tile(self.player.position)

    def render_world_viewport(self, viewport: Viewport = None) -> RenderedData:
        r"""
        :param viewport: Viewport to render.
            if not specified, defaults to self.viewport
        :param scale: Scaling for sprites.      <br><br><pre><code>
        |   1 = x, 2 = \/, 3 = \ /, etc.        <br>
        |              /\       x               <br>
        |                      / \              <br></pre></code>
        :return: A list of tiles with color information.
        """

        if not viewport:
            viewport = self.viewport

        ret: list[list[str]] = []
        color_data: list[list[tuple[int, int, int]]] = []

        z = self.player.position.z

        for y in range(viewport.top_left.y, (viewport.lower_right.y + 1)):
            retrow = []
            color_row = []
            for x in range(viewport.top_left.x, (viewport.lower_right.x + 1)):
                pos = VectorN(x, y, z)
                tile = self.world.get_tile(pos)
                if not tile:
                    tile = Tiles.empty()

                sprite = tile.render_sprite(scale=viewport.scale)
                tile_color = get_color_for_tile(tile.tileid)

                # Check for entities at this position
                entities = self.world.get_entities(pos)
                if entities:
                    # Get the highest priority entity for rendering
                    entity = self.world.get_priority_entity(pos)
                    sprite = entity.render_sprite(scale=viewport.scale)
                    # Use entity color if available, otherwise use tile color
                    if hasattr(entity, "color"):
                        tile_color = COLOR_MANAGER.get_entity_color(entity.color)

                    # If there are multiple entities, modify the sprite to show count
                    if len(entities) > 1:
                        # For now, just use the priority entity's sprite
                        # TODO: Implement better multi-entity visualization (e.g., add a number)
                        pass

                # Check for fluids at this position (render on top of tiles but under entities)
                fluid = self.world.fluid_manager.get_fluid(pos)
                if fluid:
                    # Only render fluid if there's no entity at this position
                    if not entities:
                        sprite = fluid.render_sprite(scale=viewport.scale)
                        tile_color = COLOR_MANAGER.get_color(fluid.get_color())

                # if we are here, render us!
                if (self.player.position.y == y) and (self.player.position.x == x):
                    sprite = self.player.render_sprite(scale=viewport.scale)
                    tile_color = COLOR_MANAGER.get_color("PLAYER")

                # logging.debug('render: {}'.format(sprite))
                # done with a single sprite in a row
                retrow.append(sprite)
                color_row.append(tile_color)
            ret.append(retrow)
            color_data.append(color_row)

        logging.log(5, "Returning this from render_world_viewport()")
        logging.log(5, pprint.pformat(ret))

        return RenderedData(ret, scale=viewport.scale, color_data=color_data)

    def move_player(self, vec: VectorN):
        possible_position = self.player.calc_offset(vec)

        # check bounds
        if self.world.get_tile(possible_position) is None:
            logging.debug(
                f"Tried to move OOB! {vec} would have resulted in {possible_position}"
            )
            return

        self.player.move(vec)
        # Use action tick cost system instead of just incrementing by 1
        tick_cost = self.get_action_tick_cost("walk")
        for _ in range(tick_cost):
            self.increment_tick()

    def player_outside_viewport(self, wiggle=0):
        return not self.player_inside_2d_viewport(wiggle=wiggle)

    def player_inside_2d_viewport(self, wiggle: int = 0):
        return self.player.position.trim(2).inside_bounding_rect(
            self.viewport.top_left.trim(2),
            self.viewport.lower_right.trim(2),
            wiggle=wiggle,
        )

    def reset_viewport(self):
        # TODO: Does this work for even/odd numbered sizes? Test this...
        px, py, pz = self.player.position

        vpwidth, vpheight = self.viewport.get_size()

        vpw_tl = vpwidth // 2
        vph_tl = vpheight // 2

        vpw_lr = vpwidth // 2
        vph_lr = vpheight // 2

        # preserve oddness
        if (vpwidth % 2) != 0:
            vpw_lr += 1

        if (vpheight % 2) != 0:
            vph_lr += 1

        # make our bounds centered on the player position
        self.viewport.top_left = VectorN(px - vpw_tl, py - vph_tl, pz)

        self.viewport.lower_right = VectorN(px + vpw_lr, py + vph_lr, pz)

    def set_tile_at_player_feet(self, tile):
        self.world.set_tile(self.player.position, tile)

    def render_pretty_player_position(self):
        return str(self.player.position.as_short_string())

    def log_mining(self, tile_name: str, items_dropped: list[str]):
        """Log a mining event."""
        if items_dropped:
            items_str = ", ".join(items_dropped)
            self.message_log.add_message(
                f"Mined {tile_name} and found: {items_str}", "mining"
            )
        else:
            self.message_log.add_message(f"Mined {tile_name}", "mining")

    def log_interaction(self, entity_name: str, interaction_text: str):
        """Log an interaction event."""
        self.message_log.add_message(
            f"Interacted with {entity_name}: {interaction_text}", "interaction"
        )

    def log_pickup(self, item_name: str):
        """Log an item pickup event."""
        self.message_log.add_message(f"Picked up: {item_name}", "pickup")

    def log_dialog(self, speaker: str, message: str):
        """Log a dialog event."""
        self.message_log.add_message(f"{speaker}: {message}", "dialog")

    def log_info(self, message: str):
        """Log a general info message."""
        self.message_log.add_message(message, "info")

    def increment_tick(self):
        """Increment the game tick counter."""
        self.gametick += 1
        self.process_entity_ticks()
        # Process fluid physics
        self.world.fluid_manager.process_fluids(self.gametick)

    def get_tick_rate(self) -> int:
        """Get the current tick rate based on player body condition."""
        # Base tick rate is 200, but can be modified by body condition
        base_rate = 200
        walk_speed = self.player.get_walk_speed_modifier()
        break_speed = self.player.get_break_speed_modifier()

        # Average the speed modifiers to get overall performance
        avg_speed = (walk_speed + break_speed) / 2.0

        # Adjust tick rate based on performance (slower body = faster ticks for more granular control)
        if avg_speed < 0.5:
            return int(base_rate * 2)  # 400 ticks for severely impaired
        elif avg_speed < 0.8:
            return int(base_rate * 1.5)  # 300 ticks for impaired
        else:
            return base_rate  # 200 ticks for normal operation

    def get_action_tick_cost(self, action_type: str) -> int:
        """Get how many ticks an action should cost based on body condition."""
        speed_modifier = self.player.get_action_speed(action_type)

        # Define base costs for different action types
        base_costs = {
            "walk": 200,  # Walking is the baseline action
            "break": 300,  # Mining/breaking takes longer than walking
            "mine": 300,  # Alias for break
            "craft": 400,  # Crafting takes even longer
            "push": 250,  # Pushing objects takes some time
            "inventory": 50,  # Quick inventory operations
            "interact": 100,  # Quick interactions
            "pickup": 75,  # Quick pickup operations
        }

        # Get base cost for this action type, default to walk cost
        base_cost = base_costs.get(action_type, base_costs["walk"])

        # Adjust cost based on body condition
        if speed_modifier < 0.5:
            return max(
                1, int(base_cost / speed_modifier)
            )  # More ticks for slower actions
        else:
            return base_cost

    def process_entity_ticks(self):
        """Process ticks for all entities that have a tick method."""
        # Get all entities in the world
        for entity in self.world.get_all_entities():
            if hasattr(entity, "tick") and callable(entity.tick):
                # Check if entity has a speed attribute and should tick this frame
                if hasattr(entity, "speed"):
                    # Entity speed affects how often it ticks
                    # Higher speed = more frequent ticks
                    if self.gametick % max(1, int(10 / entity.speed)) == 0:
                        entity.tick()
                else:
                    # Default behavior for entities without speed attribute
                    entity.tick()
