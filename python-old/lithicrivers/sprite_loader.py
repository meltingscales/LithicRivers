"""
Sprite loader for external sprite data.
Similar to the structure loader but for sprites.
"""

import json
import logging
from pathlib import Path
from typing import Dict, List

logger = logging.getLogger(__name__)


class SpriteData:
    """Represents sprite data loaded from external files."""

    def __init__(self, name: str, color: str, description: str, sprites: List[str]):
        self.name = name
        self.color = color
        self.description = description
        self.sprites = sprites

    def get_sprite(self, scale: int = 1) -> str:
        """Get sprite for a given scale (1-based)."""
        if scale <= 0 or scale > len(self.sprites):
            # Return a fallback sprite if scale is out of bounds
            return "?"
        return self.sprites[scale - 1]


class SpriteLoader:
    """Loads sprite data from external files."""

    def __init__(self, data_path: Path = None):
        if data_path is None:
            data_path = Path(__file__).parent / "data" / "sprites"
        self.data_path = data_path
        self._sprite_cache: Dict[str, SpriteData] = {}

    def load_sprite(self, sprite_name: str, category: str = "fluids") -> SpriteData:
        """Load a sprite from external files."""
        cache_key = f"{category}/{sprite_name}"

        if cache_key in self._sprite_cache:
            return self._sprite_cache[cache_key]

        sprite_path = self.data_path / category / f"{sprite_name}.lrsprite"

        if not sprite_path.exists():
            available_sprites = self.get_available_sprites(category)
            raise ValueError(
                f"Sprite '{sprite_name}' not found in '{category}' category. Available sprites: {available_sprites}"
            )

        try:
            # Load metadata
            data_file = sprite_path / "data.json"
            if not data_file.exists():
                raise ValueError(f"Missing data.json for sprite: {sprite_path}")

            with open(data_file) as f:
                metadata = json.load(f)

            # Load sprite data
            sprites_file = sprite_path / "sprites.txt"
            if not sprites_file.exists():
                raise ValueError(f"Missing sprites.txt for sprite: {sprite_path}")

            with open(sprites_file) as f:
                content = f.read()
                lines = content.split("\n")

                # Note that we do not strip whitespace because we want to preserve
                # spaces if they're part of the artwork.

                # Group lines into sprites by scale
                # Format: line[0] for scale 1, lines[1:2] for scale 2, lines[3:5] for scale 3
                sprites = []
                if len(lines) >= 6:
                    # Scale 1: first line
                    sprites.append(lines[0])

                    # Scale 2: next 2 lines
                    scale2 = "\n".join(lines[1:3])
                    sprites.append(scale2)

                    # Scale 3: last 3 lines
                    scale3 = "\n".join(lines[3:6])
                    sprites.append(scale3)
                else:
                    # Fallback if not enough lines
                    sprites = lines

            # Validate sprite dimensions
            self._validate_sprite_dimensions(sprites, sprite_name, category)

            # Create sprite data
            sprite_data = SpriteData(
                name=metadata.get("name", sprite_name),
                color=metadata.get("color", "white"),
                description=metadata.get("description", ""),
                sprites=sprites,
            )

            self._sprite_cache[cache_key] = sprite_data
            return sprite_data

        except Exception as e:
            logger.error(f"Failed to load sprite {sprite_name}: {e}")
            raise ValueError(
                f"Failed to load sprite '{sprite_name}' from '{category}' category: {e}"
            )

    def _validate_sprite_dimensions(
        self, sprites: List[str], sprite_name: str, category: str
    ) -> None:
        """Validate that all sprites are square and not empty."""
        if not sprites:
            raise ValueError(
                f"Sprite '{sprite_name}' in '{category}' category has no sprites"
            )

        # Check each sprite for proper dimensions
        for i, sprite in enumerate(sprites):
            if not sprite:
                raise ValueError(
                    f"Sprite '{sprite_name}' in '{category}' category has empty sprite at scale {i + 1}"
                )

            lines = sprite.split("\n")
            if not lines:
                raise ValueError(
                    f"Sprite '{sprite_name}' in '{category}' category has empty sprite at scale {i + 1}"
                )

            # Check that all lines have the same width
            line_lengths = [len(line) for line in lines]
            if len(set(line_lengths)) > 1:
                raise ValueError(
                    f"Sprite '{sprite_name}' in '{category}' category has inconsistent line lengths at scale {i + 1}. "
                    f"All lines must have the same width for square sprites."
                )

            # Check that width equals height (square)
            width = line_lengths[0] if line_lengths else 0
            height = len(lines)

            if width != height:
                raise ValueError(
                    f"Sprite '{sprite_name}' in '{category}' category is not square at scale {i + 1}. "
                    f"Width: {width}, Height: {height}. Sprites must be square."
                )

    def get_available_sprites(self, category: str = "fluids") -> List[str]:
        """Get list of available sprites in a category."""
        category_path = self.data_path / category
        if not category_path.exists():
            return []

        sprites = []
        for sprite_dir in category_path.iterdir():
            if sprite_dir.is_dir() and sprite_dir.name.endswith(".lrsprite"):
                sprite_name = sprite_dir.name[:-8]  # Remove .lrsprite extension
                # Remove any trailing dots
                sprite_name = sprite_name.rstrip(".")
                sprites.append(sprite_name)

        return sprites

    def clear_cache(self):
        """Clear the sprite cache."""
        self._sprite_cache.clear()


# Global sprite loader instance
_sprite_loader = None


def get_sprite_loader() -> SpriteLoader:
    """Get the global sprite loader instance."""
    global _sprite_loader
    if _sprite_loader is None:
        _sprite_loader = SpriteLoader()
    return _sprite_loader
