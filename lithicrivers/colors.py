"""
Centralized color management for LithicRivers.
Copyright (c) 2024 Henry Post. All rights reserved.
"""

from enum import Enum
from typing import Optional


class ColorEnum(Enum):
    """Centralized color definitions for the entire game."""
    
    # Basic colors (foreground, background, attributes)
    BLACK = (0, 0, 0)
    RED = (1, 0, 0)
    GREEN = (2, 0, 0)
    YELLOW = (3, 0, 0)
    BLUE = (4, 0, 0)
    MAGENTA = (5, 0, 0)
    CYAN = (6, 0, 0)
    WHITE = (7, 0, 0)
    GRAY = (8, 0, 0)
    
    # UI element colors
    DEFAULT = WHITE
    ERROR = RED
    SUCCESS = GREEN
    WARNING = YELLOW
    INFO = BLUE
    HIGHLIGHT = CYAN
    HEADER = (7, 0, 1)  # White with bold
    LABEL = WHITE
    BUTTON = WHITE
    BACKGROUND = BLACK
    BORDER = GRAY
    TITLE = (7, 0, 1)  # White with bold
    SUBTITLE = GRAY
    MESSAGE = WHITE
    STATUS = CYAN
    INVENTORY = YELLOW
    
    # Tile colors
    DIRT = YELLOW  # Earth
    TREE = GREEN   # Nature
    BEDROCK = GRAY  # Stone
    CLOUD = WHITE  # Sky
    EMPTY = BLACK  # Void
    GOLD_ORE = YELLOW  # Gold
    
    # Player colors
    PLAYER = CYAN
    
    # Item colors
    ROCK = GRAY
    GOLD = YELLOW
    DIAMOND = BLUE
    STICK = YELLOW  # Brown (yellow)
    LOG = YELLOW    # Brown (yellow)
    ACORN = GREEN
    COOKIE = YELLOW  # Brown (yellow)
    
    # Special colors
    RARE = MAGENTA  # Rare items
    COMMON = GRAY   # Common items
    VALUABLE = YELLOW  # Valuable items


class ColorManager:
    """Centralized color management for the entire game."""
    
    def __init__(self) -> None:
        # Create mapping from enum names to values
        self._color_map = {color.name: color.value for color in ColorEnum}
        
        # Create human-readable color names mapping
        self._color_names = {
            ColorEnum.RED.value: "red",
            ColorEnum.GREEN.value: "green", 
            ColorEnum.YELLOW.value: "yellow",
            ColorEnum.BLUE.value: "blue",
            ColorEnum.MAGENTA.value: "magenta",
            ColorEnum.CYAN.value: "cyan",
            ColorEnum.WHITE.value: "white",
            ColorEnum.GRAY.value: "gray",
            ColorEnum.BLACK.value: "black",
        }
        
        # Tile color mappings
        self._tile_colors = {
            "Dirt": ColorEnum.DIRT.value,
            "Tree": ColorEnum.TREE.value,
            "Bedrock": ColorEnum.BEDROCK.value,
            "Cloud": ColorEnum.CLOUD.value,
            "Empty": ColorEnum.EMPTY.value,
            "Gold Ore": ColorEnum.GOLD_ORE.value,
        }
        
        # Item color mappings
        self._item_colors = {
            "Rock": ColorEnum.ROCK.value,
            "Gold Nugget": ColorEnum.GOLD.value,
            "Diamond": ColorEnum.DIAMOND.value,
            "Stick": ColorEnum.STICK.value,
            "Log": ColorEnum.LOG.value,
            "Acorn": ColorEnum.ACORN.value,
            "Cookie": ColorEnum.COOKIE.value,
        }
    
    def get_color(self, color_name: str) -> tuple[int, int, int]:
        """Get color tuple for a color name."""
        return self._color_map.get(color_name, ColorEnum.DEFAULT.value)
    
    def get_tile_color(self, tile_id: str) -> tuple[int, int, int]:
        """Get color tuple for a tile type."""
        return self._tile_colors.get(tile_id, ColorEnum.DEFAULT.value)
    
    def get_item_color(self, item_name: str) -> tuple[int, int, int]:
        """Get color tuple for an item type."""
        return self._item_colors.get(item_name, ColorEnum.DEFAULT.value)
    
    def get_ui_color(self, element_type: str) -> tuple[int, int, int]:
        """Get color tuple for a UI element."""
        # First try to get the color directly from the enum
        try:
            color_enum = ColorEnum[element_type]
            return color_enum.value
        except KeyError:
            # Fall back to the color map
            return self._color_map.get(element_type, ColorEnum.DEFAULT.value)
    
    def get_color_name(self, color: tuple[int, int, int]) -> str:
        """Get a human-readable name for a color tuple."""
        return self._color_names.get(color, "default")
    
    def get_entity_color(self, color_name: str) -> tuple[int, int, int]:
        """Get color tuple for an entity color name."""
        entity_colors = {
            "cyan": ColorEnum.CYAN.value,
            "blue": ColorEnum.BLUE.value,
            "red": ColorEnum.RED.value,
            "yellow": ColorEnum.YELLOW.value,
            "white": ColorEnum.WHITE.value,
        }
        return entity_colors.get(color_name, ColorEnum.WHITE.value)


# Global color manager instance
COLOR_MANAGER = ColorManager() 