import logging
from typing import Union, List

from asciimatics.event import KeyboardEvent

from lithicrivers.model import Vector2, Viewport

GAME_NAME = 'LithicRivers'
# DEFAULT_SIZE = (50, 15)
DEFAULT_SIZE = Vector2(500, 500)
DEFAULT_VIEWPORT = Viewport(Vector2(0, 0), Vector2(60, 20))
DEFAULT_PLAYER_POSITION = Vector2(0, 0)
LOGFILENAME = GAME_NAME + '.log'

VEC_UP = Vector2(0, 1)
VEC_DOWN = Vector2(0, -1)
VEC_LEFT = Vector2(-1, 0)
VEC_RIGHT = Vector2(1, 0)


class Keymap:
    def __init__(self):
        self.MOVE_UP = 'w'
        self.MOVE_LEFT = 'a'
        self.MOVE_DOWN = 's'
        self.MOVE_RIGHT = 'd'

        self.MOVEMENT_VECTOR_MAP = {
            self.MOVE_UP: Vector2(0, -1),
            self.MOVE_LEFT: Vector2(-1, 0),
            self.MOVE_DOWN: Vector2(0, 1),
            self.MOVE_RIGHT: Vector2(1, 0)
        }

        self.VIEWPORT_SLIDE_LEFT = '['
        self.VIEWPORT_SLIDE_RIGHT = ']'

        self.MINE = 'u'

    def get_valid_key_names(self) -> List[str]:
        # TODO: Make dis more efficient...
        # TODO filtered_names = filter(lambda x: ..., all_names)
        # TODO Will return an iterable, which can only be
        # TODO consumed once unless you do list(filter(...)), but is more efficient for large collections

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
            retstr += "'[{}]' - {}\n".format(self.__getattribute__(keyname), keyname)

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
        :param key_name: Name of a key -- i.e. 'MOVE_LEFT'
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
