from typing import List

from lithicrivers.constants import VEC_WEST, VEC_EAST
from lithicrivers.model.vector import VectorN


class RenderedData:
    """A rendered list of objects -- tile, sprite, etc.

    Scale is necessary to know so that objects can be "sliced" by how many
    columns/rows they inhabit...
    """

    def __init__(self, render_data: List[List[str]], scale=1):

        # constructor flexibility
        if isinstance(render_data, str):
            render_data = list(list(render_data))

        if isinstance(render_data[0], str):
            render_data = list(render_data)

        self.render_data = render_data
        self.scale = scale

    def as_string(self, eol='\n') -> str:

        ret = []

        for y in range(0, len(self.render_data)):
            render_row = self.render_data[y]
            for stripe_idx in range(0, self.scale):
                retSlice = []
                for x in range(0, len(render_row)):
                    render_item = render_row[x]
                    render_item_chunk = render_item.split(eol)
                    slice = render_item_chunk[stripe_idx]
                    slice = slice.replace(eol, '')
                    retSlice.append(slice)

                ret.append(''.join(retSlice))

        return eol.join(ret)

    @staticmethod
    def from_string(string: str, scale: int = 1, eol='\n'):

        raise NotImplementedError("Lazy!")

        split = string.split(eol)
        ret = []

        for i in range(0, len(split)):
            tok = split[i]
            retslices = (list() for _ in range(0, ))
            for stripe_idx in range(0, scale):
                stripe = tok[0:stripe_idx]
                print(stripe)

        return ret


class Viewport:
    """
    Please note that y grows downwards, and x grows rightwards.
    This is why top_left is "smaller" numerically than lower_right.

    The reason for this is...I lazily used list(list(...)) as my underlying data structure for World :P
    """

    def __init__(self, top_left: VectorN, lower_right: VectorN, scale=1):
        self.top_left = top_left
        self.lower_right = lower_right
        self.original_size = self.get_size()
        self.scale = scale

    @staticmethod
    def generate_centered(center: VectorN, radius: VectorN, scale=1):
        """Generate a Viewport centered on `center` with `radius` as its lower and upper bounds.
        It doubles from `radius`."""
        return Viewport(
            (center - radius),
            (center + radius),
            scale=scale
        )

    def clamp_scale(self):
        if self.scale < 1:
            self.scale = 1

    def rescale_down(self, i=1):
        self.rescale(-i)

    def rescale_up(self, i=1):
        self.rescale(i)

    def rescale(self, i: int):
        original_scale = self.scale
        self.scale += i
        self.clamp_scale()
        if self.scale != original_scale:
            # how much do we want to scale the viewport?
            scale_factor: float = (original_scale / self.scale)
            scale_direction = (-1 if scale_factor > 1 else 1)

            # TODO lol... this is NOT the right way to do it but it "works"
            self.shrink(3 * scale_direction)

            # return
            #
            # raise NotImplementedError(
            #     "We must scale the viewport by {} to accommodate the new render! "
            #     "Scale used to be {} but are now {}!".format(
            #         scale_factor,
            #         original_scale, self.scale))

    def slide(self, move_vec: VectorN):
        self.top_left += move_vec
        self.lower_right += move_vec

    def shrink(self, n=1):
        self.top_left += VectorN(n, n, 0)
        self.lower_right += -VectorN(n, n, 0)

    def grow(self, n=1):
        self.top_left += -VectorN(n, n, 0)
        self.lower_right += VectorN(n, n, 0)

    def slide_left(self):
        self.slide(VEC_WEST)

    def slide_right(self):
        self.slide(VEC_EAST)

    def get_height(self) -> int:
        return abs(self.lower_right.y - self.top_left.y)

    def get_width(self) -> int:
        return abs(self.lower_right.x - self.top_left.x)

    def __str__(self):
        return "<Viewport scale={} top_left=[{}] lower_right=[{}] >".format(self.scale, self.top_left, self.lower_right)

    def __repr__(self):
        return str(self)

    def render_pretty(self):
        return "scale={scale} ({x1:2d},{y1:2d}), ({x2:2d},{y2:2d}) ".format(
            x1=self.top_left.x, y1=self.top_left.y,
            x2=self.lower_right.x, y2=self.lower_right.y,
            scale=self.scale
        )

    def get_size(self) -> VectorN:
        return VectorN(self.get_width(), self.get_height())


class StopGame(Exception):
    pass
