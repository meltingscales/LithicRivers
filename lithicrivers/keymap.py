import logging
from typing import Union, List
from asciimatics.event import KeyboardEvent



from lithicrivers.constants import (
    VEC_DOWN, VEC_UP, VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST,
    VEC_NORTHWEST, VEC_NORTHEAST, VEC_SOUTHWEST, VEC_SOUTHEAST,
    NUMPAD_7, NUMPAD_8, NUMPAD_9, NUMPAD_4, NUMPAD_6, NUMPAD_1, NUMPAD_2, NUMPAD_3, NUMPAD_5
)
from lithicrivers.config_manager import config_manager
from lithicrivers.textutil import associated


class Keymap:
    """Keymap class that loads keybinds from JSON configuration."""
    
    def __init__(self):
        # Cache for valid key names
        self._get_valid_key_names_cache = None
        
        # Load keybinds from config
        self._load_keybinds()
        
        # Build numpad movement vector map
        self.NUMPAD_MOVEMENT_VECTOR_MAP = {
            NUMPAD_8: VEC_NORTH,
            NUMPAD_2: VEC_SOUTH,
            NUMPAD_4: VEC_WEST,
            NUMPAD_6: VEC_EAST,
            NUMPAD_7: VEC_NORTHWEST,
            NUMPAD_9: VEC_NORTHEAST,
            NUMPAD_1: VEC_SOUTHWEST,
            NUMPAD_3: VEC_SOUTHEAST,
        }
    
    def _load_keybinds(self):
        """Load keybinds from config manager."""
        # Movement keys
        self.MOVE_NORTH = config_manager.get_keybind("movement", "MOVE_NORTH")
        self.MOVE_WEST = config_manager.get_keybind("movement", "MOVE_WEST")
        self.MOVE_SOUTH = config_manager.get_keybind("movement", "MOVE_SOUTH")
        self.MOVE_EAST = config_manager.get_keybind("movement", "MOVE_EAST")
        self.MOVE_UP = config_manager.get_keybind("movement", "MOVE_UP")
        self.MOVE_DOWN = config_manager.get_keybind("movement", "MOVE_DOWN")
        
        # Viewport keys
        self.RESET_VIEWPORT = config_manager.get_keybind("viewport", "RESET_VIEWPORT")
        self.SLIDE_VIEWPORT_WEST = config_manager.get_keybind("viewport", "SLIDE_VIEWPORT_WEST")
        self.SLIDE_VIEWPORT_EAST = config_manager.get_keybind("viewport", "SLIDE_VIEWPORT_EAST")
        self.TOGGLE_VIEWPORT = config_manager.get_keybind("viewport", "TOGGLE_VIEWPORT")
        
        # Scale keys
        self.SCALE_UP = config_manager.get_keybind("scale", "SCALE_UP")
        self.SCALE_DOWN = config_manager.get_keybind("scale", "SCALE_DOWN")
        
        # Action keys
        self.MINE = config_manager.get_keybind("action", "MINE")
        self.INTERACT = config_manager.get_keybind("action", "INTERACT")
        
        # Build movement vector map with both character and numpad support
        self.MOVEMENT_VECTOR_MAP = {}
        
        # Add character-based movement
        if self.MOVE_NORTH and self.MOVE_NORTH.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_NORTH] = VEC_NORTH
        if self.MOVE_WEST and self.MOVE_WEST.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_WEST] = VEC_WEST
        if self.MOVE_SOUTH and self.MOVE_SOUTH.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_SOUTH] = VEC_SOUTH
        if self.MOVE_EAST and self.MOVE_EAST.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_EAST] = VEC_EAST
        if self.MOVE_UP and self.MOVE_UP.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_UP] = VEC_UP
        if self.MOVE_DOWN and self.MOVE_DOWN.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_DOWN] = VEC_DOWN
    
    def reload_keybinds(self):
        """Reload keybinds from config files."""
        config_manager.keybinds = config_manager._load_keybinds()
        self._load_keybinds()
        
        # Rebuild movement vector map
        self.MOVEMENT_VECTOR_MAP = {}
        
        # Add character-based movement
        if self.MOVE_NORTH and self.MOVE_NORTH.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_NORTH] = VEC_NORTH
        if self.MOVE_WEST and self.MOVE_WEST.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_WEST] = VEC_WEST
        if self.MOVE_SOUTH and self.MOVE_SOUTH.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_SOUTH] = VEC_SOUTH
        if self.MOVE_EAST and self.MOVE_EAST.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_EAST] = VEC_EAST
        if self.MOVE_UP and self.MOVE_UP.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_UP] = VEC_UP
        if self.MOVE_DOWN and self.MOVE_DOWN.isalpha():
            self.MOVEMENT_VECTOR_MAP[self.MOVE_DOWN] = VEC_DOWN
    
    def get_valid_key_names(self) -> List[str]:
        """Get list of valid key names."""
        if self._get_valid_key_names_cache is None:
            all_names = dir(self)
            filtered_names = [
                name for name in all_names
                if (
                        (not name.startswith('__')) and
                        (isinstance(name, str)) and  # name must be string
                        (isinstance(self.__getattribute__(name), str))  # self.[name] must be string
                )
            ]
            self._get_valid_key_names_cache = filtered_names
        
        return self._get_valid_key_names_cache
    
    def generate_key_guide(self) -> str:
        """Generate human readable guide for keys."""
        retstr = ""
        keynames = self.get_valid_key_names()
        
        for keyname in keynames:
            retstr += associated(self.__getattribute__(keyname), keyname)
            retstr += '\n'
        
        # Add numpad movement keys
        retstr += "\n=== NUMPAD MOVEMENT ===\n"
        retstr += "Directions:      Keys:\n"
        retstr += "  NW N NE       7 8 9\n"
        retstr += "   W   E        4   6\n"
        retstr += "  SW S SE       1 2 3\n"
        
        return retstr
    
    @staticmethod
    def char_from_keyboard_event(ke: KeyboardEvent) -> Union[None, str]:
        """Extract character from keyboard event."""
        try:
            ke_char = chr(ke.key_code).lower()
            return ke_char
        except ValueError as ve:
            logging.log(5, "Could not handle this KeyboardEvent -- {} -- probably a special key: {}".format(ke.key_code, ke, ))
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
        except AttributeError as ae:
            raise AttributeError("No key named {} found.\nValid keys: {}".format(key_name, dir(self)))
        
        ke_char = Keymap.char_from_keyboard_event(ke)
        return ke_char == key.lower()
    
    def matches_numpad(self, ke: KeyboardEvent) -> bool:
        """
        Check if the keyboard event is a numpad movement key.
        :param ke: KeyboardEvent.
        :return: boolean
        """
        key_code = Keymap.key_code_from_keyboard_event(ke)
        return key_code in self.NUMPAD_MOVEMENT_VECTOR_MAP
    
    def get_numpad_movement_vector(self, ke: KeyboardEvent):
        """
        Get movement vector for numpad key.
        :param ke: KeyboardEvent.
        :return: VectorN or None
        """
        key_code = Keymap.key_code_from_keyboard_event(ke)
        return self.NUMPAD_MOVEMENT_VECTOR_MAP.get(key_code)
    
    def update_keybind(self, key_name: str, value: str):
        """Update a keybind and save to config file."""
        # Determine category based on key name
        category = self._get_category_for_key(key_name)
        if category:
            config_manager.update_keybind(category, key_name, value)
            # Reload keybinds to reflect changes
            self.reload_keybinds()
    
    def _get_category_for_key(self, key_name: str) -> str:
        """Get the category for a given key name."""
        category_mapping = {
            # Movement keys
            "MOVE_NORTH": "movement",
            "MOVE_WEST": "movement", 
            "MOVE_SOUTH": "movement",
            "MOVE_EAST": "movement",
            "MOVE_UP": "movement",
            "MOVE_DOWN": "movement",
            # Viewport keys
            "RESET_VIEWPORT": "viewport",
            "SLIDE_VIEWPORT_WEST": "viewport",
            "SLIDE_VIEWPORT_EAST": "viewport",
            "TOGGLE_VIEWPORT": "viewport",
            # Scale keys
            "SCALE_UP": "scale",
            "SCALE_DOWN": "scale",
            # Action keys
            "MINE": "action"
        }
        return category_mapping.get(key_name, "")


# Global keymap instance
KEYMAP = Keymap() 