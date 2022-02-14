import logging
from typing import Tuple, List, TypeVar, Any

T = TypeVar('T')


class VectorN:
    """
    Vector (point) that can be any dimension (X, or X/Y, or X/Y/Z, or X/Y/Z/W, etc)
    """

    dimPosMap = {'x': 0, 'y': 1, 'z': 2, 'w': 3}

    def __init__(self, *values: int):

        self.x, self.y, self.z, self.w = (None, None, None, None,)
        self.dimension_values = [*values, ]

        # set x,y,z, etc
        for dimName, dimIdx in self.dimPosMap.items():
            if dimIdx < len(self.dimension_values):
                self.__setattr__(dimName, self.dimension_values[dimIdx])
            else:
                self.__setattr__(dimName, None)

    def apply_op(self, other, op):
        other: VectorN
        return VectorN(*[
            (op(a, b)) for a, b in zip(self.dimension_values, other.dimension_values)
        ])

    def __neg__(self):
        return VectorN(*[
            (-1 * a) for a in self.dimension_values
        ])

    def __add__(self, other):
        other: VectorN
        self.assert_same_dimension_order(other)
        return self.apply_op(other, int.__add__)

    def __sub__(self, other):
        other: VectorN
        self.assert_same_dimension_order(other)
        return self.apply_op(other, int.__sub__)

    def __mul__(self, other):
        other: VectorN
        self.assert_same_dimension_order(other)
        return self.apply_op(other, int.__mul__)

    def __eq__(self, other):
        other: VectorN
        return self.dimension_values == other.dimension_values

    def assert_same_dimension_order(self, other):
        other: VectorN
        assert (self.dimension_order() == other.dimension_order())

    def dimension_order(self):
        """are we "1"d, "2"d, "3"d, etc"""
        return len(self.dimension_values)

    # noinspection PyRedundantParentheses
    def as_tuple(self) -> Tuple[int]:
        return (*self.dimension_values,)

    def as_list(self) -> List[int]:
        return [*self.dimension_values, ]

    def __getitem__(self, item: Any):
        # indexing us like `self[1]`
        if isinstance(item, int):
            item: int
            if item < len(self.dimension_values):
                return self.dimension_values[item]
            else:
                raise IndexError("This is only a {} dimensional vector!".format(self.dimension_order()))

        # indexing us like `self['y']`
        if item in self.dimPosMap.keys():
            return self.dimension_values[self.dimPosMap[item]]

    def insideBoundingRect(self, vec1, vec2):

        if not (self.dimension_order() == 2):
            raise Exception("Currently only implemented for 2d!")

        vec1: VectorN
        vec2: VectorN

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

    def __init__(self, topleft: VectorN, lowerright: VectorN):
        self.topleft = topleft
        self.lowerright = lowerright

    @staticmethod
    def generate_centered(center, radius: VectorN):
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
