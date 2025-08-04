import json
import logging
import platform
from pathlib import Path
from typing import Optional

from asciimatics.event import KeyboardEvent

from lithicrivers.config_manager import config_manager
from lithicrivers.constants import (
    VEC_DOWN,
    VEC_EAST,
    VEC_NORTH,
    VEC_NORTHEAST,
    VEC_NORTHWEST,
    VEC_SOUTH,
    VEC_SOUTHEAST,
    VEC_SOUTHWEST,
    VEC_UP,
    VEC_WEST,
    VEC_ZERO,
)
from lithicrivers.model.vector import VectorN
from lithicrivers.textutil import spaced_list


class Keymap:
    """Keymap class that loads keybinds from JSON configuration."""

    def __init__(self) -> None:
        # Load keybinds from config
        self._load_keybinds()

        # Load platform-specific keychords
        self._load_keychords()

    def matches_movement_key(self, ke: KeyboardEvent) -> bool:
        """Check if the keyboard event is a movement key."""
        keycode = ke.key_code

        # Check each movement key against the keychords
        for key_list in self.MOVEMENT_KEYS:
            for key_name in key_list:
                if key_name in self.keychords:
                    keychord = self.keychords[key_name]
                    if keychord and keycode in keychord:
                        return True
        return False

    def get_movement_vector(self, ke: KeyboardEvent) -> Optional[VectorN]:
        """Get the movement vector for a keyboard event."""

        keycode = ke.key_code

        # Check each movement key list and return the corresponding vector
        for key_list, vector in self.MOVEMENT_MAPPING.items():
            for key_name in key_list:
                if key_name in self.keychords:
                    keychord = self.keychords[key_name]
                    if keychord and keycode in keychord:
                        return vector

        return None

    def _load_keybinds(self) -> None:
        """Load keybinds from config manager."""
        # Movement keys
        self.MOVE_NORTH = config_manager.get_keybind("movement", "MOVE_NORTH")
        self.MOVE_NORTHWEST = config_manager.get_keybind("movement", "MOVE_NORTHWEST")
        self.MOVE_NORTHEAST = config_manager.get_keybind("movement", "MOVE_NORTHEAST")
        self.MOVE_WEST = config_manager.get_keybind("movement", "MOVE_WEST")
        self.MOVE_SOUTH = config_manager.get_keybind("movement", "MOVE_SOUTH")
        self.MOVE_SOUTHWEST = config_manager.get_keybind("movement", "MOVE_SOUTHWEST")
        self.MOVE_SOUTHEAST = config_manager.get_keybind("movement", "MOVE_SOUTHEAST")
        self.MOVE_EAST = config_manager.get_keybind("movement", "MOVE_EAST")
        self.MOVE_UP = config_manager.get_keybind("movement", "MOVE_UP")
        self.MOVE_DOWN = config_manager.get_keybind("movement", "MOVE_DOWN")
        self.WAIT = config_manager.get_keybind("movement", "WAIT")

        self.MOVEMENT_KEYS = [
            self.MOVE_NORTHWEST,
            self.MOVE_NORTH,
            self.MOVE_NORTHEAST,
            self.MOVE_WEST,
            self.WAIT,
            self.MOVE_EAST,
            self.MOVE_SOUTHWEST,
            self.MOVE_SOUTH,
            self.MOVE_SOUTHEAST,
            self.MOVE_UP,
            self.MOVE_DOWN,
        ]

        # Create a mapping of movement key lists to their corresponding vectors
        self.MOVEMENT_MAPPING: dict[frozenset[str], VectorN] = {
            self.MOVE_NORTHWEST: VEC_NORTHWEST,
            self.MOVE_NORTH: VEC_NORTH,
            self.MOVE_NORTHEAST: VEC_NORTHEAST,
            self.MOVE_WEST: VEC_WEST,
            self.WAIT: VEC_ZERO,
            self.MOVE_EAST: VEC_EAST,
            self.MOVE_SOUTHWEST: VEC_SOUTHWEST,
            self.MOVE_SOUTH: VEC_SOUTH,
            self.MOVE_SOUTHEAST: VEC_SOUTHEAST,
            self.MOVE_UP: VEC_UP,
            self.MOVE_DOWN: VEC_DOWN,
        }

        # Viewport keys
        self.RESET_VIEWPORT = config_manager.get_keybind("viewport", "RESET_VIEWPORT")
        self.SLIDE_VIEWPORT_WEST = config_manager.get_keybind(
            "viewport", "SLIDE_VIEWPORT_WEST"
        )
        self.SLIDE_VIEWPORT_EAST = config_manager.get_keybind(
            "viewport", "SLIDE_VIEWPORT_EAST"
        )
        self.TOGGLE_VIEWPORT = config_manager.get_keybind("viewport", "TOGGLE_VIEWPORT")

        # Scale keys
        self.SCALE_UP = config_manager.get_keybind("scale", "SCALE_UP")
        self.SCALE_DOWN = config_manager.get_keybind("scale", "SCALE_DOWN")

        # Action keys
        self.MINE = config_manager.get_keybind("action", "MINE")
        self.INTERACT = config_manager.get_keybind("action", "INTERACT")

        # UI keys
        self.CLOSE_HELP_MENU = config_manager.get_keybind("ui", "CLOSE_HELP_MENU")

        # Inventory keys
        self.DROP_ITEM = config_manager.get_keybind("inventory", "DROP_ITEM")
        self.DESTROY_ITEM = config_manager.get_keybind("inventory", "DESTROY_ITEM")
        self.CHEAT_DUPLICATE_ITEM = config_manager.get_keybind("inventory", "CHEAT_DUPLICATE_ITEM")

    def _load_keychords(self) -> None:
        """Load platform-specific keychords from JSON file."""
        platform_name = platform.system().lower()

        # Detect OS type
        os_type = "unknown"
        try:
            with open("/etc/os-release") as f:
                for line in f:
                    if line.startswith("ID="):
                        os_type = line.split("=")[1].strip().strip('"')
                        break
        except FileNotFoundError:
            # Fallback for systems without /etc/os-release
            if platform_name == "linux":
                os_type = "linux"
            elif platform_name == "darwin":
                os_type = "macos"
            elif platform_name == "windows":
                os_type = "windows"

        filename = f"keychords.{platform_name}.{os_type}.json"
        config_dir = Path("config")
        keychords_file = config_dir / filename

        self.keychords = {}
        if keychords_file.exists():
            try:
                with open(keychords_file) as f:
                    self.keychords = json.load(f)
                logging.info(
                    f"Loaded {len(self.keychords)} keychords from {keychords_file}"
                )
            except Exception as e:
                logging.warning(f"Could not load keychords from {keychords_file}: {e}")
        else:
            logging.warning(f"No keychords file found at {keychords_file}")

    def reload_keybinds(self) -> None:
        """Reload keybinds from config files."""
        config_manager.keybinds = config_manager._load_keybinds()
        self._load_keybinds()

    def generate_categorized_key_guide(self) -> str:
        """Generate a categorized keybind list sorted by category."""

        # Generate categorized output
        retstr = ""

        # Add numpad movement keys
        retstr += "\n=== NUMPAD MOVEMENT ===\n"
        retstr += "Directions:      Keys:\n"
        retstr += "  NW N NE       7 8 9\n"
        retstr += "   W   E        4   6\n"
        retstr += "  SW S SE       1 2 3\n"

        # Add viewport keys
        retstr += "\n=== VIEWPORT ===\n"
        retstr += "Actions:      Keys:\n"
        retstr += f"  RESET          {spaced_list(list(self.RESET_VIEWPORT))}\n"
        retstr += f"  SLIDE WEST     {spaced_list(list(self.SLIDE_VIEWPORT_WEST))}\n"
        retstr += f"  SLIDE EAST     {spaced_list(list(self.SLIDE_VIEWPORT_EAST))}\n"
        retstr += f"  TOGGLE         {spaced_list(list(self.TOGGLE_VIEWPORT))}\n"

        # Add scale keys
        retstr += "\n=== SCALE ===\n"
        retstr += "Actions:      Keys:\n"
        retstr += f"  SCALE UP       {spaced_list(list(self.SCALE_UP))}\n"
        retstr += f"  SCALE DOWN     {spaced_list(list(self.SCALE_DOWN))}\n"

        # Add action keys
        retstr += "\n=== ACTIONS ===\n"
        retstr += "Actions:      Keys:\n"
        retstr += f"  MINE           {spaced_list(list(self.MINE))}\n"
        retstr += f"  INTERACT       {spaced_list(list(self.INTERACT))}\n"

        return retstr

    @staticmethod
    def char_from_keyboard_event(ke: KeyboardEvent) -> Optional[str]:
        """Extract character from keyboard event."""
        try:
            ke_char = chr(ke.key_code).lower()
            return ke_char
        except ValueError:
            logging.log(
                5,
                f"Could not handle this KeyboardEvent -- {ke.key_code} -- probably a special key: {ke}",
            )
            return None

    @staticmethod
    def key_code_from_keyboard_event(ke: KeyboardEvent) -> Optional[int]:
        """Extract key code from keyboard event for special keys like numpad."""
        key_code = ke.key_code
        if isinstance(key_code, int):
            return key_code
        else:
            logging.warning(f"Invalid key_code type: {type(key_code)}")
            return None

    def matches(self, key_name: str, ke: KeyboardEvent) -> bool:
        """
        Does this KeyboardEvent match a name of a key we have registered?
        :param key_name: Name of a key -- i.e. 'MOVE_NORTH'
        :param ke: KeyboardEvent.
        :return: boolean
        """
        try:
            key = self.__getattribute__(key_name)
        except AttributeError as err:
            raise AttributeError(
                f"No key named {key_name} found.\nValid keys: {dir(self)}"
            ) from err

        keycode = ke.key_code

        # Check each key name in the key list against the keychords
        for key_name_str in key:
            if key_name_str in self.keychords:
                keychord = self.keychords[key_name_str]
                if keychord and keycode in keychord:
                    return True
        return False

    def _get_category_for_key(self, key_name: str) -> Optional[str]:
        """Determine the category for a given key name."""
        # Define key categories
        categories = {
            "movement": [
                "MOVE_NORTH",
                "MOVE_SOUTH",
                "MOVE_EAST",
                "MOVE_WEST",
                "MOVE_UP",
                "MOVE_DOWN",
                "MOVE_NORTHWEST",
                "MOVE_NORTHEAST",
                "MOVE_SOUTHWEST",
                "MOVE_SOUTHEAST",
                "WAIT",
            ],
            "viewport": [
                "RESET_VIEWPORT",
                "SLIDE_VIEWPORT_WEST",
                "SLIDE_VIEWPORT_EAST",
                "TOGGLE_VIEWPORT",
            ],
            "scale": ["SCALE_UP", "SCALE_DOWN"],
            "action": ["MINE", "INTERACT"],
            "ui": ["CLOSE_HELP_MENU"],
        }

        for category, keys in categories.items():
            if key_name in keys:
                return category
        return None

    def update_keybind(self, key_name: str, value: str) -> None:
        """Update a keybind and save to config file."""
        # Determine category based on key name
        category = self._get_category_for_key(key_name)
        if category:
            # Convert single string to list for config manager
            value_list = [value] if isinstance(value, str) else value
            config_manager.update_keybind(category, key_name, value_list)
            # Reload keybinds to reflect changes
            self.reload_keybinds()


# Global keymap instance
KEYMAP = Keymap()
