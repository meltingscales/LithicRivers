from typing import TYPE_CHECKING, Optional

from lithicrivers.constants import VEC_EAST, VEC_NORTH, VEC_SOUTH, VEC_WEST
from lithicrivers.game.events import EntityListener, EntityMovedEvent
from lithicrivers.game.interfaces import ItemArtRenderable, SpriteRenderable
from lithicrivers.model.vector import VectorN
import msgspec

if TYPE_CHECKING:
    from lithicrivers.game.npcs import (
        AncientRelic,
        CrystalShard,
        ElderOak,
        StumblingSheep,
    )


class Entity(msgspec.Struct, frozen=False):
    name: str
    position: VectorN
    health: int = 100
    stamina: int = 100
    _listeners: list[EntityListener] = msgspec.field(default_factory=list)

    def __getstate__(self):
        """Custom msgspec serialization that excludes listeners."""
        state = self.__dict__.copy()
        # Remove listeners - they'll be re-established when the world is loaded
        state["_listeners"] = []
        return state

    def __setstate__(self, state):
        """Custom msgspec deserialization that initializes empty listeners."""
        self.__dict__.update(state)
        # Ensure listeners list exists (will be populated by World after loading)
        if "_listeners" not in self.__dict__:
            self._listeners = []

    def add_listener(self, listener: EntityListener) -> None:
        """Add an event listener."""
        self._listeners.append(listener)

    def remove_listener(self, listener: EntityListener) -> None:
        """Remove an event listener."""
        if listener in self._listeners:
            self._listeners.remove(listener)

    def _notify_moved(self, old_position: VectorN, new_position: VectorN) -> None:
        """Notify listeners of movement."""
        event = EntityMovedEvent(self, old_position, new_position)
        for listener in self._listeners:
            listener.on_entity_moved(event)

    def tick(self) -> None:
        """Called each game tick. Override in subclasses."""
        pass

    def move(self, vec: VectorN) -> None:
        """Move the entity and notify listeners."""
        old_position = self.position
        self.position += vec
        self._notify_moved(old_position, self.position)

    def calc_offset(self, vec: VectorN) -> VectorN:
        """Where would I move, if I did move?"""
        return self.position + vec

    def move_north(self) -> None:
        self.move(VEC_NORTH)

    def move_south(self) -> None:
        self.move(VEC_SOUTH)

    def move_west(self) -> None:
        self.move(VEC_WEST)

    def move_east(self) -> None:
        self.move(VEC_EAST)


class Items:
    """
    A bunch of default items.
    """

    @staticmethod
    def rock() -> "Item":
        return Item("Rock")

    @staticmethod
    def gold_nugget() -> "Item":
        return Item("Gold Nugget")

    @staticmethod
    def stick() -> "Item":
        return Item("Stick")

    @staticmethod
    def diamond() -> "Item":
        return Item("Diamond")

    @staticmethod
    def log() -> "Item":
        return Item("Log")

    @staticmethod
    def acorn() -> "Item":
        return Item("Acorn")

    @staticmethod
    def iron_scrap() -> "Item":
        return Item("Iron Scrap")

    @staticmethod
    def scrap_electronics() -> "Item":
        return Item("Scrap Electronics")


class DroppedItem(Entity, msgspec.Struct, frozen=False, kw_only=True):
    """An item that has been dropped in the world."""
    item: "Item"
    color: str = "yellow"

    @classmethod
    def create(cls, item: "Item", position: VectorN) -> "DroppedItem":
        # Create the DroppedItem with correct fields
        return cls(name=item.name, position=position, health=100, stamina=100, item=item)


class Item(msgspec.Struct, frozen=False, kw_only=True):
    name: str
    sprite_sheet: Optional[list[str]] = None

    @classmethod
    def create(cls, name: str, sprite_sheet: Optional[list[str]] = None) -> "Item":
        if sprite_sheet is None:
            from lithicrivers.sprite_loader import get_sprite_loader
            sprite_loader = get_sprite_loader()
            sprite_data = sprite_loader.load_sprite(
                name.lower().replace(" ", "_"), "items"
            )
            sprite_sheet = sprite_data.sprites
        return cls(name=name, sprite_sheet=sprite_sheet)


class Fluid(Entity, msgspec.Struct, frozen=False, kw_only=True):
    """
    A fluid entity that can flow and spread across the world.
    Fluids don't replace blocks but exist as separate entities.
    """
    fluid_type: str
    amount: int = 1000
    viscosity: float = 1.0
    max_amount: int = 1000
    spread_threshold: int = 800
    settled: bool = False
    last_spread_tick: int = -1
    stability_counter: int = 0
    settlement_threshold: int = 5

    @classmethod
    def create(cls, fluid_type: str, position: VectorN, amount: int = 1000, viscosity: float = 1.0) -> "Fluid":
        # Clamp amount
        amount = max(1, min(1000, int(amount)))
        return cls(name=f"{fluid_type}_fluid", position=position, fluid_type=fluid_type, amount=amount, viscosity=viscosity)

    def render_sprite(self, scale: int = 1) -> str:
        """Render the fluid sprite."""
        sprites = self.get_sprites()
        if self.fluid_type not in sprites:
            raise ValueError(
                f"Fluid type '{self.fluid_type}' not found in sprite data. Available types: {list(sprites.keys())}"
            )

        sprite_list = sprites[self.fluid_type]
        if scale <= 0 or scale > len(sprite_list):
            raise ValueError(
                f"Scale {scale} is out of bounds for fluid '{self.fluid_type}'. Valid range: 1-{len(sprite_list)}"
            )

        return sprite_list[scale - 1]

    def get_sprites(self) -> dict[str, list[str]]:
        """Get all possible sprite representations of this fluid (for different scales, 1x1, 2x2, 3x3, etc.)"""
        # Try to load from external sprite data first
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(self.fluid_type, "fluids")

        if sprite_data:
            # Use external sprite data
            return {self.fluid_type: sprite_data.sprites}

        # No fallback - throw exception if external data not found
        raise ValueError(
            f"External sprite data not found for fluid type '{self.fluid_type}' in 'fluids' category"
        )

    def get_color(self) -> str:
        """Get the color for this fluid."""
        # Try to load from external sprite data first
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(self.fluid_type, "fluids")

        if sprite_data:
            # Use external sprite data
            return sprite_data.color

        # No fallback - throw exception if external data not found
        raise ValueError(
            f"External sprite data not found for fluid type '{self.fluid_type}' in 'fluids' category"
        )

    def tick(self) -> None:
        """Process fluid physics each tick."""
        # Fluid physics are handled by FluidManager
        pass

    def copy(self) -> "Fluid":
        """Create a copy of this fluid."""
        return Fluid(self.fluid_type, self.position, self.amount, self.viscosity)


class Entities:
    @staticmethod
    def stumbling_sheep(position: VectorN = VectorN.from_args(0, 0, 0)) -> "StumblingSheep":
        from lithicrivers.game.npcs import StumblingSheep

        return StumblingSheep.create(position)

    @staticmethod
    def starter_npc(position: VectorN = VectorN.from_args(5, 5, 0)) -> "ElderOak":
        from lithicrivers.game.npcs import ElderOak

        return ElderOak.create(position)

    @staticmethod
    def test_entity1(position: VectorN = VectorN.from_args(6, 5, 0)) -> "CrystalShard":
        from lithicrivers.game.npcs import CrystalShard

        return CrystalShard.create(position)

    @staticmethod
    def test_entity2(position: VectorN = VectorN.from_args(5, 6, 0)) -> "AncientRelic":
        from lithicrivers.game.npcs import AncientRelic

        return AncientRelic.create(position)

    @staticmethod
    def water(position: VectorN = VectorN.from_args(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid.create("water", position, amount, viscosity=100.0)

    @staticmethod
    def lava(position: VectorN = VectorN.from_args(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid.create("lava", position, amount, viscosity=200.0)  # Lava flows slower

    @staticmethod
    def acid(position: VectorN = VectorN.from_args(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid.create("acid", position, amount, viscosity=150.0)

    @staticmethod
    def oil(position: VectorN = VectorN.from_args(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid.create("oil", position, amount, viscosity=50.0)  # Oil flows faster

    @staticmethod
    def blood(position: VectorN = VectorN.from_args(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid.create("blood", position, amount, viscosity=120.0)
