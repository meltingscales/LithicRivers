from typing import Any, ClassVar, Union


import msgspec

class VectorN(msgspec.Struct, frozen=False):
    """
    Vector (point) that can be any dimension (X, or X/Y, or X/Y/Z, or X/Y/Z/W, etc)
    Use VectorN.from_args(0, 0, 1) for ergonomic construction.
    """

    dimension_values: tuple[int, ...] = ()
    x: int | None = None
    y: int | None = None
    z: int | None = None
    w: int | None = None

    dim_pos_map: ClassVar[dict[str, int]] = {"x": 0, "y": 1, "z": 2, "w": 3}

    @classmethod
    def from_args(cls, *args: int) -> "VectorN":
        return cls(dimension_values=tuple(args))

    def __post_init__(self):
        # set x,y,z, etc based on available dimensions
        for dim_name, dim_idx in self.dim_pos_map.items():
            if dim_idx < len(self.dimension_values):
                setattr(self, dim_name, self.dimension_values[dim_idx])

    def trim(self, new_size: int) -> "VectorN":
        """Trim VectorN down to smaller size."""
        return VectorN.from_args(*self.as_list()[0:new_size])

    def dimension_order(self) -> int:
        """are we "1"d, "2"d, "3"d, etc"""
        return len(self.dimension_values)

    def as_tuple(self) -> tuple[int, ...]:
        return (*self.dimension_values,)

    def as_list(self) -> list[int]:
        return [
            *self.dimension_values,
        ]

    def assert_same_dimension_order(self, other: "VectorN") -> None:
        if not (self.dimension_order() == other.dimension_order()):
            raise ValueError(
                f"you cannot perform an operation on vector `self` ({self}) with vector `other` ({other}) "
                "as it is not the same dimension order!"
            )

    def __neg__(self) -> "VectorN":
        return VectorN.from_args(*[(-1 * a) for a in self.dimension_values])

    def __add__(self, other: "VectorN") -> "VectorN":
        self.assert_same_dimension_order(other)
        return VectorN.from_args(
            *[(a + b) for a, b in zip(self.dimension_values, other.dimension_values)]
        )

    def __sub__(self, other: "VectorN") -> "VectorN":
        self.assert_same_dimension_order(other)
        return VectorN.from_args(*[(a - b) for a, b in zip(self.dimension_values, other.dimension_values)])

    def __mul__(self, other: Union[Any, int]) -> "VectorN":
        if isinstance(other, VectorN):
            self.assert_same_dimension_order(other)
            return VectorN.from_args(*[(a * b) for a, b in zip(self.dimension_values, other.dimension_values)])
        else:
            return VectorN.from_args(*[(a * other) for a in self.dimension_values])

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, VectorN):
            return False
        return self.dimension_values == other.dimension_values

    def __hash__(self) -> int:
        """Make VectorN hashable for use as dictionary keys and in sets."""
        return hash(self.dimension_values)

    def __str__(self) -> str:
        return f"<Vec{self.dimension_order()} {self.dimension_values}>"

    def __repr__(self) -> str:
        return str(self)

    def __getitem__(self, item: Any) -> int:
        # indexing us like `self[1]`
        if isinstance(item, int):
            if item < len(self.dimension_values):
                return self.dimension_values[item]
            else:
                raise IndexError(
                    f"This is only a {self.dimension_order()} dimensional vector!"
                )

        # indexing us like `self['y']`
        if isinstance(item, str) and item in self.dim_pos_map:
            return self.dimension_values[self.dim_pos_map[item]]

        raise KeyError(f"Invalid key: {item}")

    def inside_bounding_rect(
        self, vec1: "VectorN", vec2: "VectorN", wiggle: int = 0
    ) -> bool:
        if not (self.dimension_order() == 2):
            raise Exception(
                f"Currently only implemented for 2d! Cannot determine if {self} is within {vec1} and {vec2}"
            )

        # Get coordinates, handling None values
        px = self.x if self.x is not None else 0
        py = self.y if self.y is not None else 0
        x1 = vec1.x if vec1.x is not None else 0
        x2 = vec2.x if vec2.x is not None else 0
        y1 = vec1.y if vec1.y is not None else 0
        y2 = vec2.y if vec2.y is not None else 0

        # if our two points are flipped, flip em again :P
        if (x1 >= x2) or (y1 >= y2):
            vec2, vec1 = vec1, vec2
            # Update coordinates after flipping
            x1 = vec1.x if vec1.x is not None else 0
            x2 = vec2.x if vec2.x is not None else 0
            y1 = vec1.y if vec1.y is not None else 0
            y2 = vec2.y if vec2.y is not None else 0

        # YOINK from https://www.programming-idioms.org/idiom/178/check-if-point-is-inside-rectangle/2615/python
        # Assuming that x1 < x2 and y1 < y2...
        return (
            ((px - wiggle) >= x1)
            and ((px + wiggle) < x2)
            and ((py - wiggle) >= y1)
            and ((py + wiggle) < y2)
        )

    def serialize(self) -> str:
        return ",".join([str(x) for x in self.dimension_values])

    @staticmethod
    def deserialize(obj: Union[str, list, tuple]) -> "VectorN":
        if isinstance(obj, VectorN):
            return obj
        elif isinstance(obj, list):
            return VectorN(*obj)
        elif isinstance(obj, str):
            obj = obj.strip()
            tokens = obj.split(",")
            ints = [int(x.strip()) for x in tokens]
            return VectorN(*ints)
        elif isinstance(obj, tuple):
            return VectorN(*obj)
        else:
            raise ValueError(f"Cannot deserialize object of type {type(obj)}")

    def as_short_string(self) -> str:
        return ",".join(str(x) for x in self.dimension_values)
