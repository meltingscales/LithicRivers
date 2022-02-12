import logging
from typing import Tuple, List


class Vector2:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def tuple(self) -> Tuple[int, int]:
        return self.x, self.y,

    def list(self) -> List[int]:
        return [self.x, self.y, ]

    def __add__(self, other):
        return Vector2(x=(self.x + other.x), y=(self.y + other.y))

    def __mul__(self, other):
        return Vector2(x=(self.x * other.x), y=(self.y * other.y))

    def __eq__(self, other):
        return (self.x == other.x) and (self.y == other.y)

    def __getitem__(self, item):
        if item == 0:
            return self.x
        if item == 1:
            return self.y
        raise IndexError(f"Cannot get item {item} of {self}!")

    def __str__(self):
        return f"<{self.__class__.__name__} (x={self.x},y={self.y})>"

    def insideBoundingBox(self, vec1, vec2):

        bound1 = Vector2(vec1.x, vec1.y)
        bound2 = Vector2(vec2.x, vec1.y)
        bound3 = Vector2(vec1.x, vec2.y)
        bound4 = Vector2(vec2.x, vec2.y)

        raise NotImplementedError("fuck i am lazy...")


class Viewport:
    """
    Please note that y grows downwards, and x grows rightwards.
    This is why topleft is "smaller" numerically than lowerright.

    The reason for this is...I lazily used list(list(...)) as my underlying data structure for World :P
    """

    def __init__(self, topleft: Vector2, lowerright: Vector2):
        self.topleft = topleft
        self.lowerright = lowerright

    def get_height(self):
        logging.debug(
            "returning self.lowerright.y - self.topleft.y = {} - {}".format(self.lowerright.y, self.topleft.y))
        return self.lowerright.y - self.topleft.y

    def __str__(self):
        return "<Viewport topleft=[{}] lowerright=[{}] >".format(self.topleft, self.lowerright)


class StopGame(Exception):
    pass
