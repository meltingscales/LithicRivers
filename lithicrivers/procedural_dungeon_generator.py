"""
Procedural dungeon generation system for creating dynamic dungeons.
Uses seeded randomness for deterministic generation.
"""

import os
from lithicrivers.game.rng import SimpleRNG
from enum import Enum
from typing import Dict, List, Tuple

import msgspec

from lithicrivers.game.core import Tile, Tiles
from lithicrivers.logging_config import get_logger
from lithicrivers.model.vector import VectorN

logger = get_logger(__name__)


class DungeonType(Enum):
    """Types of procedural dungeons."""

    CAVE_SYSTEM = "cave_system"
    UNDERGROUND_FACILITY = "underground_facility"
    MINING_SHAFT = "mining_shaft"
    CRYPT = "crypt"
    LABORATORY = "laboratory"


class DungeonConfig(msgspec.Struct, frozen=False):
    """Configuration for a procedural dungeon."""

    dungeon_type: DungeonType
    min_size: int
    max_size: int
    depth_range: Tuple[int, int]  # Z-level range
    generation_chance: float
    biome_restrictions: List[str]

    # Dungeon-specific parameters
    room_count_range: Tuple[int, int]
    corridor_width_range: Tuple[int, int]
    special_room_chance: float


class ProceduralStructureGenerator(msgspec.Struct, frozen=False):
    """
    Generates procedural dungeons using seeded randomness.
    Follows the same deterministic principles as the existing structure system.
    """

    dungeon_configs: Dict[DungeonType, DungeonConfig] = None

    @classmethod
    def create(cls):
        instance = cls()
        instance._initialize_dungeon_configs()
        return instance

    def _initialize_dungeon_configs(self) -> None:
        """Initialize dungeon configurations."""
        self.dungeon_configs = {
            DungeonType.CAVE_SYSTEM: DungeonConfig(
                dungeon_type=DungeonType.CAVE_SYSTEM,
                min_size=8,
                max_size=20,
                depth_range=(-5, -1),
                generation_chance=0.15,
                biome_restrictions=["ALL"],
                room_count_range=(3, 8),
                corridor_width_range=(2, 4),
                special_room_chance=0.3,
            ),
            DungeonType.UNDERGROUND_FACILITY: DungeonConfig(
                dungeon_type=DungeonType.UNDERGROUND_FACILITY,
                min_size=12,
                max_size=25,
                depth_range=(-8, -3),
                generation_chance=0.08,
                biome_restrictions=["ALL"],
                room_count_range=(5, 12),
                corridor_width_range=(3, 5),
                special_room_chance=0.5,
            ),
            DungeonType.MINING_SHAFT: DungeonConfig(
                dungeon_type=DungeonType.MINING_SHAFT,
                min_size=6,
                max_size=15,
                depth_range=(-10, -2),
                generation_chance=0.12,
                biome_restrictions=["ALL"],
                room_count_range=(2, 6),
                corridor_width_range=(2, 3),
                special_room_chance=0.2,
            ),
            DungeonType.CRYPT: DungeonConfig(
                dungeon_type=DungeonType.CRYPT,
                min_size=10,
                max_size=18,
                depth_range=(-6, -2),
                generation_chance=0.06,
                biome_restrictions=["ALL"],
                room_count_range=(4, 9),
                corridor_width_range=(2, 4),
                special_room_chance=0.4,
            ),
            DungeonType.LABORATORY: DungeonConfig(
                dungeon_type=DungeonType.LABORATORY,
                min_size=15,
                max_size=30,
                depth_range=(-12, -4),
                generation_chance=0.04,
                biome_restrictions=["ALL"],
                room_count_range=(8, 15),
                corridor_width_range=(3, 6),
                special_room_chance=0.6,
            ),
        }

    def generate_dungeon(
        self,
        dungeon_type: DungeonType,
        base_position: VectorN,
        world_data: Dict[str, Tile],
        rng: SimpleRNG,
        force_placement: bool = False,
    ) -> bool:
        """
        Generate a procedural dungeon at the given position.

        Args:
            dungeon_type: Type of dungeon to generate
            base_position: Base position for the dungeon
            world_data: World data dictionary to modify
            rng: Seeded random number generator
            force_placement: If True, bypass random chance check

        Returns:
            True if dungeon was generated successfully, False otherwise
        """
        config = self.dungeon_configs.get(dungeon_type)
        if not config:
            return False

        # Check generation chance (unless forced)
        if not force_placement and rng.random() > config.generation_chance:
            return False

        # Check depth requirements
        base_z = base_position.z if base_position.z is not None else 0
        if not (config.depth_range[0] <= base_z <= config.depth_range[1]):
            return False

        # Generate dungeon layout
        dungeon_size = rng.randint(config.min_size, config.max_size)
        room_count = rng.randint(*config.room_count_range)

        # Create dungeon layout using seeded randomness
        layout = self._generate_dungeon_layout(
            dungeon_type, dungeon_size, room_count, base_position, rng
        )

        # Place the dungeon in the world
        tiles_placed = self._place_dungeon_in_world(
            layout, world_data, base_position, rng
        )

        logger.info(
            f"Generated {dungeon_type.value} at {base_position} with {tiles_placed} tiles"
        )
        return tiles_placed > 0

    def _generate_dungeon_layout(
        self,
        dungeon_type: DungeonType,
        size: int,
        room_count: int,
        base_position: VectorN,
        rng: SimpleRNG,
    ) -> Dict[VectorN, str]:
        """
        Generate the layout for a dungeon.

        Returns:
            Dictionary mapping relative positions to tile symbols
        """
        layout = {}

        if dungeon_type == DungeonType.CAVE_SYSTEM:
            layout = self._generate_cave_system(size, room_count, rng)
        elif dungeon_type == DungeonType.UNDERGROUND_FACILITY:
            layout = self._generate_underground_facility(size, room_count, rng)
        elif dungeon_type == DungeonType.MINING_SHAFT:
            layout = self._generate_mining_shaft(size, room_count, rng)
        elif dungeon_type == DungeonType.CRYPT:
            layout = self._generate_crypt(size, room_count, rng)
        elif dungeon_type == DungeonType.LABORATORY:
            layout = self._generate_laboratory(size, room_count, rng)

        return layout

    def _generate_cave_system(
        self, size: int, room_count: int, rng: SimpleRNG
    ) -> Dict[VectorN, str]:
        """Generate a natural cave system."""
        layout = {}

        # Create main cavern
        cavern_center = VectorN.from_args(size // 2, size // 2, 0)
        # Ensure cavern_radius is valid for small sizes
        max_radius = max(1, size // 3)
        cavern_radius = rng.randint(1, max_radius)

        # Generate irregular cavern using noise-like approach
        for x in range(size):
            for y in range(size):
                pos = VectorN.from_args(x, y, 0)
                distance = abs(x - cavern_center.x) + abs(y - cavern_center.y)

                # Add some randomness to make it more natural
                noise_factor = rng.random() * 0.3
                if distance < cavern_radius + noise_factor:
                    layout[pos] = "."
                else:
                    layout[pos] = "X"

        # Add smaller chambers
        for _ in range(room_count - 1):
            chamber_x = rng.randint(1, size - 2)
            chamber_y = rng.randint(1, size - 2)
            chamber_radius = rng.randint(2, 4)

            for dx in range(-chamber_radius, chamber_radius + 1):
                for dy in range(-chamber_radius, chamber_radius + 1):
                    nx, ny = chamber_x + dx, chamber_y + dy
                    if 0 <= nx < size and 0 <= ny < size:
                        if abs(dx) + abs(dy) < chamber_radius:
                            layout[VectorN.from_args(nx, ny, 0)] = "."

        # Add some ore deposits
        ore_count = rng.randint(2, 5)
        for _ in range(ore_count):
            ore_x = rng.randint(1, size - 2)
            ore_y = rng.randint(1, size - 2)
            if layout.get(VectorN.from_args(ore_x, ore_y, 0)) == ".":
                layout[VectorN.from_args(ore_x, ore_y, 0)] = "O"

        return layout

    def _generate_underground_facility(
        self, size: int, room_count: int, rng: SimpleRNG
    ) -> Dict[VectorN, str]:
        """Generate an underground facility with rooms and corridors."""
        layout = {}

        # Initialize with walls
        for x in range(size):
            for y in range(size):
                layout[VectorN.from_args(x, y, 0)] = "W"

        # Generate rooms
        rooms = []
        for i in range(room_count):
            room_width = rng.randint(4, 8)
            room_height = rng.randint(4, 8)
            room_x = rng.randint(1, size - room_width - 1)
            room_y = rng.randint(1, size - room_height - 1)

            # Create room
            for rx in range(room_width):
                for ry in range(room_height):
                    layout[VectorN.create(room_x + rx, room_y + ry, 0)] = "."

            rooms.append((room_x, room_y, room_width, room_height))

        # Connect rooms with corridors
        for i in range(len(rooms) - 1):
            room1 = rooms[i]
            room2 = rooms[i + 1]

            # Connect room centers
            center1 = (room1[0] + room1[2] // 2, room1[1] + room1[3] // 2)
            center2 = (room2[0] + room2[2] // 2, room2[1] + room2[3] // 2)

            # Horizontal corridor
            for x in range(
                min(center1[0], center2[0]), max(center1[0], center2[0]) + 1
            ):
                if 0 <= x < size:
                    layout[VectorN.create(x, center1[1], 0)] = "."

            # Vertical corridor
            for y in range(
                min(center1[1], center2[1]), max(center1[1], center2[1]) + 1
            ):
                if 0 <= y < size:
                    layout[VectorN.create(center2[0], y, 0)] = "."

        # Add some special rooms (treasure, machinery, etc.)
        special_room_count = rng.randint(1, 3)
        for _ in range(special_room_count):
            if rooms:
                room = rng.choice(rooms)
                room_center_x = room[0] + room[2] // 2
                room_center_y = room[1] + room[3] // 2
                layout[VectorN.create(room_center_x, room_center_y, 0)] = "T"

        return layout

    def _generate_mining_shaft(
        self, size: int, room_count: int, rng: SimpleRNG
    ) -> Dict[VectorN, str]:
        """Generate a mining shaft with branching tunnels."""
        layout = {}

        # Initialize with walls
        for x in range(size):
            for y in range(size):
                layout[VectorN.create(x, y, 0)] = "X"

        # Main shaft
        shaft_x = size // 2
        for y in range(1, size - 1):
            layout[VectorN.create(shaft_x, y, 0)] = "."

        # Add branching tunnels
        for _ in range(room_count):
            branch_y = rng.randint(2, size - 3)
            branch_length = rng.randint(3, 8)
            branch_direction = rng.choice([-1, 1])

            for i in range(branch_length):
                branch_x = shaft_x + (i * branch_direction)
                if 0 <= branch_x < size:
                    layout[VectorN.create(branch_x, branch_y, 0)] = "."

                    # Add small mining chambers
                    if i % 3 == 0 and i > 0:
                        chamber_radius = 2
                        for dx in range(-chamber_radius, chamber_radius + 1):
                            for dy in range(-chamber_radius, chamber_radius + 1):
                                nx, ny = branch_x + dx, branch_y + dy
                                if 0 <= nx < size and 0 <= ny < size:
                                    if abs(dx) + abs(dy) < chamber_radius:
                                        layout[VectorN.create(nx, ny, 0)] = "."

        # Add ore deposits
        ore_count = rng.randint(3, 8)
        for _ in range(ore_count):
            ore_x = rng.randint(1, size - 2)
            ore_y = rng.randint(1, size - 2)
            if layout.get(VectorN.create(ore_x, ore_y, 0)) == ".":
                layout[VectorN.create(ore_x, ore_y, 0)] = "O"

        return layout

    def _generate_crypt(
        self, size: int, room_count: int, rng: SimpleRNG
    ) -> Dict[VectorN, str]:
        """Generate a crypt with burial chambers."""
        layout = {}

        # Initialize with walls
        for x in range(size):
            for y in range(size):
                layout[VectorN.create(x, y, 0)] = "X"

        # Main corridor
        corridor_y = size // 2
        for x in range(1, size - 1):
            layout[VectorN.create(x, corridor_y, 0)] = "."

        # Burial chambers along the corridor
        chamber_spacing = size // (room_count + 1)
        for i in range(room_count):
            chamber_x = (i + 1) * chamber_spacing
            if chamber_x < size - 2:
                # Create burial chamber
                chamber_width = rng.randint(3, 6)
                chamber_height = rng.randint(3, 6)

                for cx in range(chamber_width):
                    for cy in range(chamber_height):
                        nx = chamber_x + cx
                        ny = corridor_y - chamber_height // 2 + cy
                        if 0 <= nx < size and 0 <= ny < size:
                            layout[VectorN.create(nx, ny, 0)] = "."

                # Add sarcophagus or treasure in chamber center
                center_x = chamber_x + chamber_width // 2
                center_y = corridor_y
                if 0 <= center_x < size:
                    layout[VectorN.create(center_x, center_y, 0)] = "T"

        return layout

    def _generate_laboratory(
        self, size: int, room_count: int, rng: SimpleRNG
    ) -> Dict[VectorN, str]:
        """Generate a laboratory with specialized rooms."""
        layout = {}

        # Initialize with walls
        for x in range(size):
            for y in range(size):
                layout[VectorN.create(x, y, 0)] = "X"

        # Generate a grid-based layout
        grid_size = int(size**0.5)
        cell_size = size // grid_size

        # Create main rooms in grid pattern
        for gx in range(grid_size):
            for gy in range(grid_size):
                room_x = gx * cell_size + 1
                room_y = gy * cell_size + 1
                room_width = cell_size - 2
                room_height = cell_size - 2

                # Create room
                for rx in range(room_width):
                    for ry in range(room_height):
                        nx, ny = room_x + rx, room_y + ry
                        if nx < size and ny < size:
                            layout[VectorN.create(nx, ny, 0)] = "."

                # Add doors between adjacent rooms
                if gx < grid_size - 1:
                    door_x = (gx + 1) * cell_size - 1
                    door_y = gy * cell_size + cell_size // 2
                    if door_y < size:
                        layout[VectorN.create(door_x, door_y, 0)] = "."

                if gy < grid_size - 1:
                    door_x = gx * cell_size + cell_size // 2
                    door_y = (gy + 1) * cell_size - 1
                    if door_x < size:
                        layout[VectorN.create(door_x, door_y, 0)] = "."

        # Add specialized equipment rooms
        equipment_count = rng.randint(2, 5)
        for _ in range(equipment_count):
            eq_x = rng.randint(1, size - 2)
            eq_y = rng.randint(1, size - 2)
            if layout.get(VectorN.create(eq_x, eq_y, 0)) == ".":
                layout[VectorN.create(eq_x, eq_y, 0)] = "E"

        # Add central control room
        center_x = size // 2
        center_y = size // 2
        if layout.get(VectorN.create(center_x, center_y, 0)) == ".":
            layout[VectorN.create(center_x, center_y, 0)] = "C"

        return layout

    def _place_dungeon_in_world(
        self,
        layout: Dict[VectorN, str],
        world_data: Dict[str, Tile],
        base_position: VectorN,
        rng: SimpleRNG,
    ) -> int:
        """
        Place the dungeon layout into the world data.

        Returns:
            Number of tiles placed
        """
        tiles_placed = 0

        # Define tile mappings
        tile_mappings = {
            ".": "empty",
            "X": "bedrock",
            "W": "iron_scrap",
            "O": "gold_ore",
            "T": "treasure",
            "E": "scrap_electronics",
            "C": "buried_treasure",
        }

        for rel_pos, symbol in layout.items():
            # Calculate absolute position
            abs_x = (
                base_position.x + rel_pos.x
                if base_position.x is not None
                else rel_pos.x
            )
            abs_y = (
                base_position.y + rel_pos.y
                if base_position.y is not None
                else rel_pos.y
            )
            abs_z = (
                base_position.z + rel_pos.z
                if base_position.z is not None
                else rel_pos.z
            )

            abs_pos = VectorN.create(abs_x, abs_y, abs_z)

            # Get tile type
            tile_name = tile_mappings.get(symbol, "empty")

            # Create tile
            if tile_name == "empty":
                tile = Tiles.empty()
            elif tile_name == "bedrock":
                tile = Tiles.bedrock()
            elif tile_name == "iron_scrap":
                tile = Tiles.iron_scrap()
            elif tile_name == "gold_ore":
                tile = Tiles.gold_ore()
            elif tile_name == "treasure":
                tile = Tiles.treasure()
            elif tile_name == "scrap_electronics":
                tile = Tiles.scrap_electronics()
            elif tile_name == "buried_treasure":
                tile = Tiles.treasure()
            else:
                tile = Tiles.empty()

            # Place tile in world
            world_data[abs_pos.serialize()] = tile
            tiles_placed += 1

        return tiles_placed

    def generate_dungeons_for_chunk(
        self,
        world_data: Dict[str, Tile],
        chunk_center: VectorN,
        chunk_radius: int,
        rng: SimpleRNG,
    ) -> None:
        """
        Generate procedural dungeons for a chunk of the world.

        Args:
            world_data: World data dictionary
            chunk_center: Center of the chunk
            chunk_radius: Radius of the chunk
            rng: Seeded random number generator
        """
        # Try to generate each dungeon type
        for dungeon_type in DungeonType:
            # Generate multiple potential positions within the chunk
            attempts = 3 if os.environ.get("TESTING") == "1" else 8

            for _ in range(attempts):
                # Random position within chunk
                center_x = chunk_center.x if chunk_center.x is not None else 0
                center_y = chunk_center.y if chunk_center.y is not None else 0
                center_z = chunk_center.z if chunk_center.z is not None else 0

                pos_x = center_x + rng.randint(-chunk_radius, chunk_radius)
                pos_y = center_y + rng.randint(-chunk_radius, chunk_radius)
                pos_z = center_z + rng.randint(-chunk_radius, chunk_radius)

                base_pos = VectorN.create(pos_x, pos_y, pos_z)
                self.generate_dungeon(dungeon_type, base_pos, world_data, rng)


# Global singleton instance
_global_procedural_generator = None


def create_procedural_generator() -> ProceduralStructureGenerator:
    """
    Get or create the singleton procedural generator instance.

    Returns:
        The singleton ProceduralStructureGenerator instance
    """
    global _global_procedural_generator

    if _global_procedural_generator is None:
        _global_procedural_generator = ProceduralStructureGenerator()
        logger.info("Created singleton ProceduralStructureGenerator instance")

    return _global_procedural_generator
