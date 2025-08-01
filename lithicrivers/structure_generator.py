"""
Structure generation system for placing predefined structures in the world.
"""

import json
import os
from pathlib import Path
from typing import Dict, List, Optional, Tuple

from lithicrivers.game import Tile, Tiles
from lithicrivers.model.vector import VectorN


class StructureDefinition:
    """Represents a structure definition loaded from files."""

    def __init__(self, name: str, blocks: Dict[str, str], layers: List[str], 
                 gen_biomes: str, gen_chance: float, y_layer_gen_range: List[int]):
        self.name = name
        self.blocks = blocks
        self.layers = layers
        self.gen_biomes = gen_biomes
        self.gen_chance = gen_chance
        self.y_layer_gen_range = y_layer_gen_range

    @classmethod
    def load_from_directory(cls, structure_dir: Path) -> "StructureDefinition":
        """Load a structure definition from a directory."""
        name = structure_dir.name.replace(".lrstructure", "")
        
        # Load data.json
        data_file = structure_dir / "data.json"
        with open(data_file, 'r') as f:
            data = json.load(f)
        
        # Load shape_layers.txt
        shape_file = structure_dir / "shape_layers.txt"
        with open(shape_file, 'r') as f:
            shape_content = f.read().strip()
        
        # Parse layers (separated by ~~~~~)
        layers = [layer.strip() for layer in shape_content.split("~~~~~") if layer.strip()]
        
        return cls(
            name=name,
            blocks=data["blocks"],
            layers=layers,
            gen_biomes=data["gen_biomes"],
            gen_chance=data["gen_chance"],
            y_layer_gen_range=data["y_layer_gen_range"]
        )

    def get_tile_for_symbol(self, symbol: str) -> Tile:
        """Get the tile corresponding to a symbol in the structure."""
        tile_name = self.blocks.get(symbol, "empty")
        
        # Map tile names to Tile objects
        tile_map = {
            "empty": Tiles.empty(),
            "iron_scrap": Tiles.iron_scrap(),
            "bone_block": Tiles.bone_block(),
            "door": Tiles.door(),
            "scrap_electronics": Tiles.scrap_electronics(),
            "treasure": Tiles.treasure(),
            "dirt": Tiles.dirt(),
            "tree": Tiles.tree(),
            "gold_ore": Tiles.gold_ore(),
            "bedrock": Tiles.bedrock(),
        }
        
        return tile_map.get(tile_name, Tiles.empty())

    def get_dimensions(self) -> Tuple[int, int, int]:
        """Get the dimensions of the structure (width, height, depth)."""
        if not self.layers:
            return (0, 0, 0)
        
        # Width is the length of the longest line
        width = max(len(line) for layer in self.layers for line in layer.split('\n'))
        # Height is the number of layers
        height = len(self.layers)
        # Depth is the maximum number of lines in any layer
        depth = max(len(layer.split('\n')) for layer in self.layers)
        
        return (width, height, depth)


class StructureManager:
    """Manages loading and placing structures in the world."""

    def __init__(self, structures_dir: Path):
        self.structures_dir = structures_dir
        self.structures: Dict[str, StructureDefinition] = {}
        self._load_structures()

    def _load_structures(self):
        """Load all structure definitions from the structures directory."""
        if not self.structures_dir.exists():
            return
        
        for structure_dir in self.structures_dir.iterdir():
            if structure_dir.is_dir() and structure_dir.name.endswith(".lrstructure"):
                try:
                    structure = StructureDefinition.load_from_directory(structure_dir)
                    self.structures[structure.name] = structure
                except Exception as e:
                    print(f"Failed to load structure {structure_dir.name}: {e}")

    def get_available_structures(self) -> List[str]:
        """Get list of available structure names."""
        return list(self.structures.keys())

    def place_structure(self, structure_name: str, world_data: Dict[str, Tile], 
                       base_position: VectorN, rng) -> bool:
        """
        Place a structure at the given position in the world.
        
        Args:
            structure_name: Name of the structure to place
            world_data: World data dictionary
            base_position: Base position to place the structure
            rng: Random number generator for consistency
            
        Returns:
            True if structure was placed successfully, False otherwise
        """
        if structure_name not in self.structures:
            return False
        
        structure = self.structures[structure_name]
        
        # Check if we should generate this structure based on chance
        if rng.random() > structure.gen_chance:
            return False
        
        # Check if base position is within the allowed y-layer range
        if not (structure.y_layer_gen_range[0] <= base_position.z <= structure.y_layer_gen_range[1]):
            return False
        
        # Place the structure
        tiles_placed = 0
        for layer_idx, layer in enumerate(structure.layers):
            layer_lines = layer.split('\n')
            for line_idx, line in enumerate(layer_lines):
                for char_idx, char in enumerate(line):
                    if char == '.':
                        continue  # Skip empty spaces
                    
                    # Calculate world position
                    world_pos = VectorN(
                        base_position.x + char_idx,
                        base_position.y + line_idx,
                        base_position.z + layer_idx
                    )
                    
                    # Get the tile for this character
                    tile = structure.get_tile_for_symbol(char)
                    
                    # Place the tile in the world
                    world_data[world_pos.serialize()] = tile
                    tiles_placed += 1
        
        print(f"Placed {tiles_placed} tiles for {structure_name} at {base_position}")
        return True

    def generate_structures_for_chunk(self, world_data: Dict[str, Tile], 
                                   chunk_center: VectorN, chunk_radius: int, rng) -> None:
        """
        Generate structures for a chunk of the world.
        
        Args:
            world_data: World data dictionary
            chunk_center: Center of the chunk
            chunk_radius: Radius of the chunk
            rng: Random number generator for consistency
        """
        # Try to place each structure
        for structure_name in self.structures:
            # Generate multiple potential positions within the chunk
            for _ in range(10):  # Try up to 10 times per structure for testing
                # Random position within chunk
                pos_x = chunk_center.x + rng.randint(-chunk_radius, chunk_radius)
                pos_y = chunk_center.y + rng.randint(-chunk_radius, chunk_radius)
                pos_z = chunk_center.z + rng.randint(-chunk_radius, chunk_radius)
                
                base_pos = VectorN(pos_x, pos_y, pos_z)
                self.place_structure(structure_name, world_data, base_pos, rng)


def create_structure_manager(structures_dir: Optional[Path] = None) -> StructureManager:
    """
    Create a structure manager with the default structures directory.
    
    Args:
        structures_dir: Optional custom structures directory
        
    Returns:
        A new StructureManager instance
    """
    if structures_dir is None:
        # Use the default structures directory
        current_dir = Path(__file__).parent
        structures_dir = current_dir / "data" / "structures"
    
    return StructureManager(structures_dir) 