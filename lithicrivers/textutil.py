"""
Text utilities for formatting and displaying game information.
"""

import random
import string
from enum import Enum
from typing import Union

# Remove the circular import
# from lithicrivers.logging_config import get_logger
# logger = get_logger(__name__)


class ColorScheme(Enum):
    """Color schemes for different UI elements."""

    # Basic colors (foreground, background, attributes)
    DEFAULT = (7, 0, 0)  # White on black
    ERROR = (1, 0, 0)  # Red on black
    SUCCESS = (2, 0, 0)  # Green on black
    WARNING = (3, 0, 0)  # Yellow on black
    INFO = (4, 0, 0)  # Blue on black
    HIGHLIGHT = (6, 0, 0)  # Cyan on black

    # Tile colors
    DIRT = (3, 0, 0)  # Yellow (earth)
    TREE = (2, 0, 0)  # Green (nature)
    BEDROCK = (8, 0, 0)  # Gray (stone)
    CLOUD = (7, 0, 0)  # White (sky)
    EMPTY = (0, 0, 0)  # Black (void)
    GOLD_ORE = (3, 0, 0)  # Yellow (gold)

    # Player colors
    PLAYER = (6, 0, 0)  # Cyan (player)

    # UI element colors
    HEADER = (7, 0, 1)  # White with bold
    LABEL = (7, 0, 0)  # White
    BUTTON = (7, 0, 0)  # White
    BACKGROUND = (0, 0, 0)  # Black
    BORDER = (8, 0, 0)  # Gray
    TITLE = (7, 0, 1)  # White with bold
    SUBTITLE = (8, 0, 0)  # Gray
    MESSAGE = (7, 0, 0)  # White
    STATUS = (6, 0, 0)  # Cyan
    INVENTORY = (3, 0, 0)  # Yellow

    # Item colors
    ROCK = (8, 0, 0)  # Gray
    GOLD = (3, 0, 0)  # Yellow
    DIAMOND = (4, 0, 0)  # Blue
    STICK = (3, 0, 0)  # Brown (yellow)
    LOG = (3, 0, 0)  # Brown (yellow)
    ACORN = (2, 0, 0)  # Green
    COOKIE = (3, 0, 0)  # Brown (yellow)

    # Special colors
    RARE = (5, 0, 0)  # Magenta (rare items)
    COMMON = (8, 0, 0)  # Gray (common items)
    VALUABLE = (3, 0, 0)  # Yellow (valuable items)


class ColorManager:
    """Manages color schemes for different game elements."""

    def __init__(self) -> None:
        self.color_schemes = {scheme.name: scheme.value for scheme in ColorScheme}

    def get_color(self, scheme_name: str) -> tuple[int, int, int]:
        """Get color tuple for a scheme name."""
        return self.color_schemes.get(scheme_name, ColorScheme.DEFAULT.value)

    def get_tile_color(self, tile_id: str) -> tuple[int, int, int]:
        """Get color tuple for a tile type."""
        tile_colors = {
            "Dirt": ColorScheme.DIRT.value,
            "Tree": ColorScheme.TREE.value,
            "Bedrock": ColorScheme.BEDROCK.value,
            "Cloud": ColorScheme.CLOUD.value,
            "Empty": ColorScheme.EMPTY.value,
            "Gold Ore": ColorScheme.GOLD_ORE.value,
        }
        return tile_colors.get(tile_id, ColorScheme.DEFAULT.value)

    def get_item_color(self, item_name: str) -> tuple[int, int, int]:
        """Get color tuple for an item type."""
        item_colors = {
            "Rock": ColorScheme.ROCK.value,
            "Gold Nugget": ColorScheme.GOLD.value,
            "Diamond": ColorScheme.DIAMOND.value,
            "Stick": ColorScheme.STICK.value,
            "Log": ColorScheme.LOG.value,
            "Acorn": ColorScheme.ACORN.value,
            "Cookie": ColorScheme.COOKIE.value,
        }
        return item_colors.get(item_name, ColorScheme.DEFAULT.value)


# Global color manager instance
COLOR_MANAGER = ColorManager()


def corrupt_text(text: str, corruption_rate: float = 0.01) -> str:
    """
    Randomly replaces characters in text with a random glitchy ASCII symbol
    at a given corruption rate, preserving newlines exactly.
    """
    # Glitch symbols have a higher probability than standard printable characters
    glitch_symbols = list("█▓▒░#%@*&$<>/\\=+-~^?!")
    normal_chars = [
        c for c in string.printable if not c.isspace() and c not in glitch_symbols
    ]
    corruption_pool = (
        glitch_symbols * 6 + normal_chars
    )  # Bias toward glitch symbols 6:1

    corrupted_chars = []

    for char in text:
        if char in "\n\r":
            # Preserve newlines exactly
            corrupted_chars.append(char)
        elif random.random() < corruption_rate:
            corrupted_chars.append(random.choice(corruption_pool))
        else:
            corrupted_chars.append(char)

    return "".join(corrupted_chars)


def presenting(text: str) -> str:
    return f"~ {text} ~"


def spaced_list(items: list[str]) -> str:
    return " ".join(items)


def render_tuple(tups: list[Union[int, float, tuple[Union[int, float], ...]]], places: int = 2) -> str:
    fstr = ""
    for i, tup in enumerate(tups):
        fstr += "("
        # Handle both integers and tuples
        if isinstance(tup, (int, float)):
            # Single value
            fstr += "{:.{}f}".format(tup, places)
        else:
            # Tuple or list of values
            for j, val in enumerate(tup):
                fstr += "{:.{}f}".format(val, places)
                if j < len(tup) - 1:
                    fstr += ", "
        fstr += ")"
        if i < len(tups) - 1:
            fstr += ", "
    return fstr


def associated(key: str, value: str) -> str:
    """Format a key-value pair for display."""
    return f"{key} => {value}"


def colorize_text(text: str, color_scheme: str) -> str:
    """Add color escape sequences to text (for terminals that support it)."""
    # This is a placeholder for terminal color support
    # In asciimatics, colors are handled by the canvas.paint method
    return text


def get_color_for_tile(tile_id: str) -> tuple[int, int, int]:
    """Get color tuple for a tile type."""
    return COLOR_MANAGER.get_tile_color(tile_id)


def get_color_for_item(item_name: str) -> tuple[int, int, int]:
    """Get color tuple for an item type."""
    return COLOR_MANAGER.get_item_color(item_name)


def get_color_for_ui_element(element_type: str) -> tuple[int, int, int]:
    """Get color tuple for a UI element."""
    return COLOR_MANAGER.get_color(element_type)


def list_label(text: str, width: int = 5, align: str = ">") -> str:
    """Format a list label."""
    return "-[{:{}{}s}]: ".format(text, align, width)


def emphasizing(text: str) -> str:
    """Format emphasized text."""
    return f"[ {text} ]"
