import math
from typing import Any, Optional

from lithicrivers.constants import VEC_EAST, VEC_WEST
from lithicrivers.model.vector import VectorN
from lithicrivers.textutil import (
    COLOR_MANAGER,
    render_tuple,
)


import msgspec

class ColoredRenderedData(msgspec.Struct, frozen=False):
    """A rendered list of objects with color information."""
    render_data: list[list[str]]
    color_data: list[list[tuple[int, int, int]]]
    scale: int = 1

    def as_string(self, eol: str = "\n") -> str:
        """Convert to string (without color information)."""
        ret = []

        for y in range(0, len(self.render_data)):
            render_row = self.render_data[y]
            for stripe_idx in range(0, self.scale):
                ret_slice = []
                for x in range(0, len(render_row)):
                    render_item = render_row[x]
                    render_item_chunk = render_item.split(eol)
                    slice = render_item_chunk[stripe_idx]
                    slice = slice.replace(eol, "")
                    ret_slice.append(slice)

                ret.append("".join(ret_slice))

        return eol.join(ret)

    def get_color_at(self, x: int, y: int) -> tuple[int, int, int]:
        """Get color information at a specific position."""
        if 0 <= y < len(self.color_data) and 0 <= x < len(self.color_data[y]):
            return self.color_data[y][x]
        return COLOR_MANAGER.get_color("DEFAULT")


class RenderedData(msgspec.Struct, frozen=False):
    """A rendered list of objects -- tile, sprite, etc.

    Scale is necessary to know so that objects can be "sliced" by how many
    columns/rows they inhabit...
    """
    render_data: list[list[str]]
    scale: int = 1
    color_data: Optional[list[list[tuple[int, int, int]]]] = None

    @classmethod
    def create(cls, render_data: Any, scale: int = 1, color_data: Optional[list[list[tuple[int, int, int]]]] = None) -> "RenderedData":
        # constructor flexibility
        if isinstance(render_data, str):
            render_data = [[render_data]]
        elif (
            isinstance(render_data, list)
            and len(render_data) > 0
            and isinstance(render_data[0], str)
        ):
            render_data = [render_data]
        # At this point, render_data should be list[list[str]]
        from typing import cast
        render_data = cast("list[list[str]]", render_data)
        if color_data is None:
            color_data = [
                [COLOR_MANAGER.get_color("DEFAULT") for _ in row] for row in render_data
            ]
        return cls(render_data=render_data, scale=scale, color_data=color_data)

    def as_string(self, eol: str = "\n") -> str:
        ret = []

        for y in range(0, len(self.render_data)):
            render_row = self.render_data[y]
            for stripe_idx in range(0, self.scale):
                ret_slice = []
                for x in range(0, len(render_row)):
                    render_item = render_row[x]
                    render_item_chunk = render_item.split(eol)
                    slice = render_item_chunk[stripe_idx]
                    slice = slice.replace(eol, "")
                    ret_slice.append(slice)

                ret.append("".join(ret_slice))

        return eol.join(ret)

    def get_color_at(self, x: int, y: int) -> tuple[int, int, int]:
        """Get color information at a specific position."""
        if 0 <= y < len(self.color_data) and 0 <= x < len(self.color_data[y]):
            return self.color_data[y][x]
        return COLOR_MANAGER.get_color("DEFAULT")


import msgspec

class Viewport(msgspec.Struct, frozen=False):
    """
    Please note that y grows downwards, and x grows rightwards.
    This is why top_left is "smaller" numerically than lower_right.

    The reason for this is...I lazily used list(list(...)) as my underlying data structure for World :P
    """
    top_left: VectorN
    lower_right: VectorN
    scale: int = 1
    original_size: VectorN = msgspec.field(default=None)

    @classmethod
    def create(cls, top_left: VectorN, lower_right: VectorN, scale: int = 1) -> "Viewport":
        vp = cls(top_left=top_left, lower_right=lower_right, scale=scale)
        vp.original_size = vp.get_size()
        return vp

    @staticmethod
    def generate_centered(
        center: VectorN, radius: VectorN, scale: int = 1
    ) -> "Viewport":
        """Generate a Viewport centered on `center` with `radius` as its lower and upper bounds.
        It doubles from `radius`."""
        return Viewport((center - radius), (center + radius), scale=scale)

    def clamp_scale(self) -> None:
        if self.scale < 1:
            self.scale = 1
        elif self.scale > 3:
            self.scale = 3

    def rescale_down(self, i: int = 1) -> None:
        self.rescale(-i)

    def rescale_up(self, i: int = 1) -> None:
        self.rescale(i)

    def rescale(self, i: int) -> None:
        self.scale += i
        self.clamp_scale()

        new_scale = self.scale

        factor: float = 1 / new_scale
        # this fucks up the viewport but we can just let the game reset it
        self.top_left = VectorN(0, 0)

        # Handle None values safely
        original_size_x = (
            self.original_size.x if self.original_size.x is not None else 0
        )
        original_size_y = (
            self.original_size.y if self.original_size.y is not None else 0
        )

        self.lower_right = VectorN(
            math.floor(factor * float(original_size_x)),
            math.floor(factor * float(original_size_y)),
        )

    def slide(self, move_vec: VectorN) -> None:
        self.top_left += move_vec
        self.lower_right += move_vec

    def shrink(self, n: int = 1) -> None:
        self.top_left += VectorN(n, n, 0)
        self.lower_right -= VectorN(n, n, 0)

    def shrink_horizontal(self, n: int = 1) -> None:
        self.top_left += VectorN(n, 0, 0)
        self.lower_right -= VectorN(n, 0, 0)

    def shrink_vertical(self, n: int = 1) -> None:
        self.top_left += VectorN(0, n, 0)
        self.lower_right -= VectorN(0, n, 0)

    def grow_horizontal(self, n: int = 1) -> None:
        self.shrink_horizontal(-n)

    def grow_vertical(self, n: int = 1) -> None:
        self.shrink_vertical(-n)

    def grow(self, n: int = 1) -> None:
        self.shrink(-n)

    def slide_left(self) -> None:
        self.slide(VEC_WEST)

    def slide_right(self) -> None:
        self.slide(VEC_EAST)

    def get_height(self) -> int:
        # Handle None values safely
        top_y = self.top_left.y if self.top_left.y is not None else 0
        bottom_y = self.lower_right.y if self.lower_right.y is not None else 0
        return abs(bottom_y - top_y)

    def get_width(self) -> int:
        # Handle None values safely
        left_x = self.top_left.x if self.top_left.x is not None else 0
        right_x = self.lower_right.x if self.lower_right.x is not None else 0
        return abs(right_x - left_x)

    def __str__(self) -> str:
        return f"<Viewport scale={self.scale} top_left=[{self.top_left}] lower_right=[{self.lower_right}] >"

    def __repr__(self) -> str:
        return str(self)

    def render_pretty(self) -> str:
        size_list = self.get_size().as_list()
        # Convert to the expected type for render_tuple
        size_tuple = tuple(size_list)
        left_list = self.top_left.trim(2).as_list()
        right_list = self.lower_right.trim(2).as_list()
        return f"<{self.scale}> [{render_tuple([size_tuple])}] ({render_tuple([tuple(left_list)])}, {render_tuple([tuple(right_list)])}) "

    def copy(self) -> "Viewport":
        """Create a copy of this viewport."""
        return Viewport(self.top_left, self.lower_right, self.scale)

    def get_size(self) -> VectorN:
        return VectorN(self.get_width(), self.get_height())


class StopGameError(Exception):
    pass
