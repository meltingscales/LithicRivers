import logging

from asciimatics.event import KeyboardEvent

from lithicrivers.model import Vector2, Viewport

GAME_NAME = 'LithicRivers'
# DEFAULT_SIZE = (50, 15)
DEFAULT_SIZE = Vector2(20, 20)
DEFAULT_VIEWPORT = Viewport(Vector2(0, 0), Vector2(5, 5))
LOGFILENAME = GAME_NAME + '.log'


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

    def matches_keyboard_event(self, key_name: str, ke: KeyboardEvent):

        try:
            key = self.__getattribute__(key_name)
        except AttributeError as ae:
            raise AttributeError("No key named {} found.\nValid keys: {}".format(key_name, dir(self)))

        try:
            ke_char = chr(ke.key_code).lower()
        except ValueError as ve:
            logging.debug(
                "Could not handle this KeyboardEvent -- {} -- probably a special key: {}".format(ke.key_code, ke, ))

        return ke_char == key.lower()


KEYMAP = Keymap()
