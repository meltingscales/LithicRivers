import itertools
import logging
from typing import Dict, FrozenSet, List, Union

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
from lithicrivers.textutil import associated, spaced_list


class Keymap:
    """Keymap class that loads keybinds from JSON configuration."""

    def __init__(self):

        # Load keybinds from config
        self._load_keybinds()

    def matches_movement_key(self, ke: KeyboardEvent) -> bool:
        """Check if the keyboard event is a movement key."""
        return chr(ke.key_code) in list(itertools.chain(*self.MOVEMENT_KEYS)) 

    def get_movement_vector(self, ke: KeyboardEvent) -> Union[None, VectorN]:
        """Get the movement vector for a keyboard event."""
        
        key_char = chr(ke.key_code)
        
        # Check each movement key list and return the corresponding vector
        for key_list, vector in self.MOVEMENT_MAPPING.items():
            if key_char in key_list:
                return vector
                
        return None

    def _load_keybinds(self):
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
        self.MOVEMENT_MAPPING: Dict[FrozenSet[str], VectorN] = {
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

    def reload_keybinds(self):
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
        retstr += "  RESET          {}\n".format(spaced_list(self.RESET_VIEWPORT))
        retstr += "  SLIDE WEST     {}\n".format(spaced_list(self.SLIDE_VIEWPORT_WEST))
        retstr += "  SLIDE EAST     {}\n".format(spaced_list(self.SLIDE_VIEWPORT_EAST))
        retstr += "  TOGGLE         {}\n".format(spaced_list(self.TOGGLE_VIEWPORT))

        # Add scale keys
        retstr += "\n=== SCALE ===\n"
        retstr += "Actions:      Keys:\n"
        retstr += "  SCALE UP       {}\n".format(spaced_list(self.SCALE_UP))
        retstr += "  SCALE DOWN     {}\n".format(spaced_list(self.SCALE_DOWN))

        # Add action keys
        retstr += "\n=== ACTIONS ===\n"
        retstr += "Actions:      Keys:\n"
        retstr += "  MINE           {}\n".format(spaced_list(self.MINE))
        retstr += "  INTERACT       {}\n".format(spaced_list(self.INTERACT))

        return retstr

    @staticmethod
    def char_from_keyboard_event(ke: KeyboardEvent) -> Union[None, str]:
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
    def key_code_from_keyboard_event(ke: KeyboardEvent) -> Union[None, int]:
        """Extract key code from keyboard event for special keys like numpad."""
        return ke.key_code

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

        ke_char = self.char_from_keyboard_event(ke)
        return ke_char in [k.lower() for k in key]

    def update_keybind(self, key_name: str, value: str):
        """Update a keybind and save to config file."""
        # Determine category based on key name
        category = self._get_category_for_key(key_name)
        if category:
            config_manager.update_keybind(category, key_name, value)
            # Reload keybinds to reflect changes
            self.reload_keybinds()

# Global keymap instance
KEYMAP = Keymap()
