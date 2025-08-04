"""
Text utilities for formatting and displaying game information.
"""

import random
import string
from typing import Union

# Import the new centralized color system
from lithicrivers.colors import COLOR_MANAGER

# Remove the circular import
# from lithicrivers.logging_config import get_logger
# logger = get_logger(__name__)


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


def render_tuple(
    tups: list[Union[int, float, tuple[Union[int, float], ...]]], places: int = 2
) -> str:
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
    return COLOR_MANAGER.get_ui_color(element_type)


def list_label(text: str, width: int = 5, align: str = ">") -> str:
    """Format a list label."""
    return "-[{:{}{}s}]: ".format(text, align, width)


def emphasizing(text: str) -> str:
    """Format emphasized text."""
    return f"[ {text} ]"
