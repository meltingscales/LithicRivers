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

    def __getitem__(self, item):
        if item == 0:
            return self.x
        if item == 1:
            return self.y
        raise IndexError(f"Cannot get item {item} of {self}!")

    def __str__(self):
        return f"<{self.__class__.__name__} (x={self.x},y={self.y})>"

    def insideBoundingBox(self, vec1, vec2):
        raise NotImplementedError("fuck i am lazy...")


class Viewport:
    """
    Please note that y grows downwards, and x grows rightwards.
    This is why topleft is "smaller" numerically than lowerright.
    """

    def __init__(self, topleft, lowerright):
        self.topleft = topleft
        self.lowerright = lowerright

    def get_height(self):
        return self.lowerright.y - self.topleft.y
