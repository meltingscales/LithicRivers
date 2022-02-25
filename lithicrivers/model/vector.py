from typing import Tuple, List, Any, Union


class VectorN:
    """
    Vector (point) that can be any dimension (X, or X/Y, or X/Y/Z, or X/Y/Z/W, etc)
    """

    dimPosMap = {'x': 0, 'y': 1, 'z': 2, 'w': 3}

    def __init__(self, *dimvals: int):

        # we're probably being passed a string, a list, or a VectorN object
        if not isinstance(dimvals, tuple):
            dimvals = VectorN.deserialize(dimvals).as_tuple()

        subelt = dimvals[0]
        # why is it nested? who knows :P
        if isinstance(subelt, tuple):
            dimvals = subelt

        self.x, self.y, self.z, self.w = (None, None, None, None,)
        self.dimension_values = dimvals

        # set x,y,z, etc
        for dimName, dimIdx in self.dimPosMap.items():
            if dimIdx < len(self.dimension_values):
                self.__setattr__(dimName, self.dimension_values[dimIdx])
            else:
                self.__setattr__(dimName, None)

    def trim(self, new_size: int):
        """Trim VectorN down to smaller size."""
        return VectorN(*self.as_list()[0:new_size])

    def dimension_order(self):
        """are we "1"d, "2"d, "3"d, etc"""
        return len(self.dimension_values)

    # noinspection PyRedundantParentheses
    def as_tuple(self) -> Tuple[int]:
        return (*self.dimension_values,)

    def as_list(self) -> List[int]:
        return [*self.dimension_values, ]

    def assert_same_dimension_order(self, other):
        other: VectorN
        if not (self.dimension_order() == other.dimension_order()):
            raise ValueError(
                "you cannot modify vector self ({}) with vector other ({}) as it is not the same dimension order!".format(
                    self,
                    other))

    def __neg__(self):
        return VectorN(*[
            (-1 * a) for a in self.dimension_values
        ])

    def __add__(self, other):
        other: VectorN
        self.assert_same_dimension_order(other)
        return VectorN(*[
            (a + b) for a, b in zip(self.dimension_values, other.dimension_values)
        ])

    def __sub__(self, other):
        other: VectorN
        self.assert_same_dimension_order(other)
        return VectorN(*[
            (a - b) for a, b in zip(self.dimension_values, other.dimension_values)
        ])

    def __mul__(self, other):
        other: VectorN
        if isinstance(other, VectorN):
            self.assert_same_dimension_order(other)
            return VectorN(*[
                (a * b) for a, b in zip(self.dimension_values, other.dimension_values)
            ])
        else:
            return VectorN(*[
                (a * other) for a in self.dimension_values
            ])

    def __eq__(self, other):
        other: VectorN
        return self.dimension_values == other.dimension_values

    def __str__(self):
        return "<Vec{} {}>".format(self.dimension_order(), self.dimension_values)

    def __repr__(self):
        return str(self)

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

    def insideBoundingRect(self, vec1, vec2, wiggle: int = 0):

        if not (self.dimension_order() == 2):
            raise Exception(
                f"Currently only implemented for 2d! Cannot determine if {self} is within {vec1} and {vec2}")

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
        return ((px - wiggle) >= x1) and \
               ((px + wiggle) < x2) and \
               ((py - wiggle) >= y1) and \
               ((py + wiggle) < y2)

    def serialize(self) -> str:
        return ','.join([str(x) for x in self.dimension_values])

    @staticmethod
    def deserialize(object: Union[str, list, tuple]):

        if isinstance(object, VectorN):
            return object

        if isinstance(object, list):
            return VectorN(*object)

        if isinstance(object, str):
            object = object.strip()
            toks = object.split(',')
            ints = [int(x.strip()) for x in toks]
            return VectorN(*ints)

    def as_short_string(self):
        return ','.join(str(x) for x in self.dimension_values)
