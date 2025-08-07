import copy

from lithicrivers.textutil import generate_sprite_repeat


class ShutDownable:
    """Interface for objects that can be shut down. Safe to delete object after shutting down."""
    def shutdown(self) -> None:
        """Shut down this object. Release all locks and close all threads."""
        pass


class Cloneable:
    """Interface for objects that can be cloned."""
    def clone(self) -> "Cloneable":
        """Clone this object. Override if you want to not clone specific fields."""
        return copy.deepcopy(self)


class SpriteRenderable:
    def __init__(self, sprite_sheet: list[str]):
        self.sprite_sheet = sprite_sheet
        if not sprite_sheet:
            self.sprite_sheet = ["?", "??\n??", "???\n???\n???"]

    def render_sprite(self, scale: int = 1) -> str:
        normalized_scale = scale - 1

        if normalized_scale < 0:
            raise Exception(
                f"Cannot render {self} with normalized_scale = {normalized_scale}"
            )

        if normalized_scale >= len(self.sprite_sheet):
            # if they ask for a sprite too large, give them '?'
            return generate_sprite_repeat("?", scale)

            # raise Exception("Cannot render sprite with scale {} as it only has these sprites:\n{} ".format(
            #     len(self.sprite_sheet),
            #     self.sprite_sheet
            # ))

        return self.sprite_sheet[normalized_scale]


class ItemArtRenderable:
    """Extend this class if you want to be able to render item art."""

    def __init__(self, item_art: str):
        self.item_art = item_art

    @staticmethod
    def blank_item() -> str:
        """Return a 12x8 blank (all spaces) ASCII art string."""
        return (
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            \n"
            "            "
        )

    @staticmethod
    def missing_texture_item() -> str:
        """Return a 12x8 'missing texture' ASCII art string."""
        return (
            "  ╭────────╮  \n"
            "  │████████│  \n"
            "  │████████│  \n"
            "  │███??███│  \n"
            "  │███??███│  \n"
            "  │████████│  \n"
            "  │████████│  \n"
            "  ╰────────╯  "
        )
