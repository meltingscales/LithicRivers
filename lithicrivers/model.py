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

    def __sub__(self, other):
        return Vector2(x=(self.x - other.x), y=(self.y - other.y))

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

    def insideBoundingRect(self, vec1, vec2):

        # if our two points are flipped, flip em again :P
        if (vec1.x >= vec2.x) or (vec1.y >= vec2.y):
            vec2, vec1 = vec1, vec2

        px = self.x
        py = self.y
        x1 = vec1.x
        x2 = vec2.x
        y1 = vec1.y
        y2 = vec2.y

        # YOINK from https://www.programming-idioms.org/idiom/178/check-if-point-is-inside-rectangle/2615/python
        # Assuming that x1 < x2 and y1 < y2...
        return (px >= x1) and (px < x2) and (py >= y1) and (py < y2)


class Viewport:
    """
    Please note that y grows downwards, and x grows rightwards.
    This is why topleft is "smaller" numerically than lowerright.

    The reason for this is...I lazily used list(list(...)) as my underlying data structure for World :P
    """

    def __init__(self, topleft: Vector2, lowerright: Vector2):
        self.topleft = topleft
        self.lowerright = lowerright

    @staticmethod
    def generate_centered(center, radius: Vector2):
        """Generate a Viewport centered on `center` with `radius` as its lower and upper bounds.
        It doubles from `radius`."""
        return Viewport(
            (center - radius),
            (center + radius)
        )

    def get_height(self):
        logging.debug(
            "returning self.lowerright.y - self.topleft.y = {} - {}".format(self.lowerright.y, self.topleft.y))
        return self.lowerright.y - self.topleft.y

    def __str__(self):
        return "<Viewport topleft=[{}] lowerright=[{}] >".format(self.topleft, self.lowerright)


class StopGame(Exception):
    pass
