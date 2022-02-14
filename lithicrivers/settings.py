import logging
from typing import Union, List

from asciimatics.event import KeyboardEvent

from lithicrivers.model import VectorN, Viewport
from lithicrivers.textutil import associated

GAME_NAME = 'LithicRivers'
# DEFAULT_SIZE = (50, 15)
DEFAULT_SIZE = VectorN(500, 500)
DEFAULT_PLAYER_POSITION = VectorN(250, 250)
DEFAULT_VIEWPORT = Viewport.generate_centered(DEFAULT_PLAYER_POSITION, VectorN(40, 10))
LOGFILENAME = GAME_NAME + '.log'

VEC2_NORTH = -VectorN(0, 1)  # negative because i am laaaaazy
VEC2_SOUTH = -VectorN(0, -1)  # negative because i am laaaaazy
VEC2_WEST = VectorN(-1, 0)
VEC2_EAST = VectorN(1, 0)

NESW_MNEMONIC = \
    '''
      N
    W   E
      S
    '''


class Keymap:
    def __init__(self):

        # cache
        self._get_valid_key_names_cache = None

        self.MOVE_NORTH = 'w'
        self.MOVE_WEST = 'a'
        self.MOVE_SOUTH = 's'
        self.MOVE_EAST = 'd'

        self.MOVEMENT_VECTOR_MAP = {
            self.MOVE_NORTH: VEC2_NORTH,
            self.MOVE_WEST: VEC2_WEST,
            self.MOVE_SOUTH: VEC2_SOUTH,
            self.MOVE_EAST: VEC2_EAST
        }

        self.SLIDE_VIEWPORT_WEST = '['
        self.SLIDE_VIEWPORT_EAST = ']'

        self.MINE = 'u'

    def get_valid_key_names(self) -> List[str]:
        # TODO: Make dis more efficient...
        # TODO filtered_names = filter(lambda x: ..., all_names)
        # TODO Will return an iterable, which can only be
        # TODO consumed once unless you do list(filter(...)), but is more efficient for large collections

        if self._get_valid_key_names_cache:
            return self._get_valid_key_names_cache

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
