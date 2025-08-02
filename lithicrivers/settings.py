import logging
import os
from typing import Union

from asciimatics.event import KeyboardEvent

from lithicrivers.config_manager import config_manager
from lithicrivers.model.model import Viewport

# Load settings from config manager
GAME_NAME = config_manager.get_setting("game", "GAME_NAME")
LOGFILENAME = config_manager.get_setting("game", "LOGFILENAME")
DEVELOPER_MODE = config_manager.get_setting("game", "DEVELOPER_MODE")
DEFAULT_SEED = config_manager.get_setting("game", "DEFAULT_SEED")

# Load logging level with support for custom levels
log_level_str = config_manager.get_setting("game", "LOGGINGLEVEL")
if log_level_str.isdigit():
    LOGGINGLEVEL = int(log_level_str)
else:
    LOGGINGLEVEL = getattr(logging, log_level_str)


# Load world settings based on environment
if os.environ.get("TESTING") == "1":
    DEFAULT_SIZE_RADIUS = config_manager.get_vector_setting(
        "world", "DEFAULT_SIZE_RADIUS", "testing"
    )
    DEFAULT_PLAYER_POSITION = config_manager.get_vector_setting(
        "world", "DEFAULT_PLAYER_POSITION", "testing"
    )
else:
    DEFAULT_SIZE_RADIUS = config_manager.get_vector_setting(
        "world", "DEFAULT_SIZE_RADIUS", "production"
    )
    DEFAULT_PLAYER_POSITION = config_manager.get_vector_setting(
        "world", "DEFAULT_PLAYER_POSITION", "production"
    )

# Load viewport settings
VIEWPORT_RADIUS = config_manager.get_vector_setting("viewport", "VIEWPORT_RADIUS")
VIEWPORT_WIGGLE = config_manager.get_setting("viewport", "VIEWPORT_WIGGLE")

# Create default viewport
DEFAULT_VIEWPORT = Viewport.generate_centered(
    DEFAULT_PLAYER_POSITION, radius=VIEWPORT_RADIUS
)


class Keymap:
    """Legacy Keymap class - now uses config manager internally."""

    def __init__(self):
        # Import the new Keymap class
        from lithicrivers.keymap import Keymap as NewKeymap

        self._keymap = NewKeymap()

        # Copy all attributes for backward compatibility
        for attr_name in dir(self._keymap):
            if not attr_name.startswith("_"):
                setattr(self, attr_name, getattr(self._keymap, attr_name))

    def get_valid_key_names(self) -> list[str]:
        return self._keymap.get_valid_key_names()

    def generate_key_guide(self) -> str:
        return self._keymap.generate_key_guide()

    @staticmethod
    def char_from_keyboard_event(ke: KeyboardEvent) -> Union[None, str]:
        from lithicrivers.keymap import KEYMAP as NEW_KEYMAP

        return NEW_KEYMAP.char_from_keyboard_event(ke)

    def matches(self, key_name: str, ke: KeyboardEvent):
        return self._keymap.matches(key_name, ke)


# Create global keymap instance
KEYMAP = Keymap()
