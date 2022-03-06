import logging
from typing import Union, List

from asciimatics.event import KeyboardEvent

from lithicrivers.constants import VEC_DOWN, VEC_UP, VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST
from lithicrivers.model.modelpleasemoveme import Viewport
from lithicrivers.model.vector import VectorN
from lithicrivers.textutil import associated

GAME_NAME = 'LithicRivers'
# DEFAULT_SIZE = (50, 15, 2)
DEFAULT_SIZE_RADIUS = VectorN(50, 50, 3)
DEFAULT_PLAYER_POSITION = VectorN(25, 25, 0)
DEFAULT_VIEWPORT = Viewport.generate_centered(DEFAULT_PLAYER_POSITION, radius=VectorN(15, 15, 0))
LOGFILENAME = GAME_NAME + '.log'
LOGGINGLEVEL = logging.INFO


class Keymap:
    def __init__(self):

        # cache
        self._get_valid_key_names_cache = None

        self.MOVE_NORTH = 'w'
        self.MOVE_WEST = 'a'
        self.MOVE_SOUTH = 's'
        self.MOVE_EAST = 'd'
        self.MOVE_UP = 'q'
        self.MOVE_DOWN = 'e'

        self.MOVEMENT_VECTOR_MAP = {
            self.MOVE_NORTH: VEC_NORTH,
            self.MOVE_WEST: VEC_WEST,
            self.MOVE_SOUTH: VEC_SOUTH,
            self.MOVE_EAST: VEC_EAST,
            self.MOVE_UP: VEC_UP,
            self.MOVE_DOWN: VEC_DOWN,
        }

        self.SLIDE_VIEWPORT_WEST = '['
        self.SLIDE_VIEWPORT_EAST = ']'

        self.SCALE_UP = '='
        self.SCALE_DOWN = '-'

        self.MINE = 'u'

    def get_valid_key_names(self) -> List[str]:

        all_names = dir(self)
        filtered_names = [
            name for name in all_names
            if (
                    (not name.startswith('__')) and
                    (isinstance(name, str)) and  # name must be string
                    (isinstance(self.__getattribute__(name), str))  # self.[name] must be string
            )
        ]

        return filtered_names

    def generate_key_guide(self) -> str:
        """
        :return: Human readable guide for keys
        """
        retstr = ""

        keynames = self.get_valid_key_names()

        for keyname in keynames:
            retstr += associated(self.__getattribute__(keyname), keyname)
            retstr += '\n'

        return retstr

    @staticmethod
    def char_from_keyboard_event(ke: KeyboardEvent) -> Union[None, str]:
        try:
            ke_char = chr(ke.key_code).lower()
            return ke_char
        except ValueError as ve:
            logging.debug(
                "Could not handle this KeyboardEvent -- {} -- probably a special key: {}".format(ke.key_code, ke, ))
            return None

    def matches(self, key_name: str, ke: KeyboardEvent):
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


KEYMAP = Keymap()
