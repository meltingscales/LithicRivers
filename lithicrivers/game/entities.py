from typing import Optional, TYPE_CHECKING

from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_WEST, VEC_EAST
from lithicrivers.game.events import EntityListener, EntityMovedEvent
from lithicrivers.game.interfaces import SpriteRenderable, ItemArtRenderable
from lithicrivers.model.vector import VectorN

if TYPE_CHECKING:
    from lithicrivers.game.npcs import ElderOak, CrystalShard, AncientRelic, StumblingSheep


class Entity:
    def __init__(self, name: str, position: VectorN):
        self.name: str = name
        self.position: VectorN = position
        self.health: int = 100
        self.stamina: int = 100
        self._listeners: list[EntityListener] = []

    def __getstate__(self):
        """Custom pickle serialization that excludes listeners."""
        state = self.__dict__.copy()
        # Remove listeners - they'll be re-established when the world is loaded
        state['_listeners'] = []
        return state

    def __setstate__(self, state):
        """Custom pickle deserialization that initializes empty listeners."""
        self.__dict__.update(state)
        # Ensure listeners list exists (will be populated by World after loading)
        if '_listeners' not in self.__dict__:
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


class DroppedItem(Entity, SpriteRenderable):
    """An item that has been dropped in the world."""

    def __init__(self, item: "Item", position: VectorN):
        Entity.__init__(self, item.name, position)
        SpriteRenderable.__init__(self, item.sprite_sheet)
        self.item = item  # Store the actual item object
        self.color = "yellow"  # Default color for dropped items


class Item(ItemArtRenderable, SpriteRenderable):
    def __init__(self, name: str, sprite_sheet: Optional[list[str]] = None):
        # Load sprites from external data if not provided
        if sprite_sheet is None:
            from lithicrivers.sprite_loader import get_sprite_loader

            sprite_loader = get_sprite_loader()
            sprite_data = sprite_loader.load_sprite(name.lower().replace(" ", "_"), "items")
            sprite_sheet = sprite_data.sprites

        SpriteRenderable.__init__(self, sprite_sheet)
        self.name = name


class Fluid(Entity, SpriteRenderable):
    """
    A fluid entity that can flow and spread across the world.
    Fluids don't replace blocks but exist as separate entities.
    """

    def __init__(self, fluid_type: str, position: VectorN, amount: int = 1000, viscosity: float = 1.0):
        super().__init__(name=f"{fluid_type}_fluid", position=position)
        self.fluid_type = fluid_type
        # Convert to integer amount (1-1000) to eliminate floating-point precision issues
        self.amount = max(1, min(1000, int(amount)))  # Clamp between 1-1000
        self.viscosity = viscosity  # How slowly the fluid flows (higher = slower)
        self.max_amount = 1000  # Maximum amount per tile (integer)
        self.spread_threshold = 800  # Amount at which fluid starts spreading (80% of max)

        # Settlement optimization properties
        self.settled = False  # Whether this fluid has reached equilibrium
        self.last_spread_tick = -1  # Last tick when this fluid spread
        self.stability_counter = 0  # How many ticks this fluid has been stable
        self.settlement_threshold = 5  # Ticks of stability before marking as settled

        # Initialize sprite sheet after fluid_type is set
        SpriteRenderable.__init__(self, self.get_sprites())


    def render_sprite(self, scale: int = 1) -> str:
        """Render the fluid sprite."""
        sprites = self.get_sprites()
        if self.fluid_type not in sprites:
            raise ValueError(f"Fluid type '{self.fluid_type}' not found in sprite data. Available types: {list(sprites.keys())}")

        sprite_list = sprites[self.fluid_type]
        if scale <= 0 or scale > len(sprite_list):
            raise ValueError(f"Scale {scale} is out of bounds for fluid '{self.fluid_type}'. Valid range: 1-{len(sprite_list)}")

        return sprite_list[scale-1]

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
        raise ValueError(f"External sprite data not found for fluid type '{self.fluid_type}' in 'fluids' category")

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
        raise ValueError(f"External sprite data not found for fluid type '{self.fluid_type}' in 'fluids' category")

    def tick(self) -> None:
        """Process fluid physics each tick."""
        # Fluid physics are handled by FluidManager
        pass

    def copy(self) -> "Fluid":
        """Create a copy of this fluid."""
        return Fluid(self.fluid_type, self.position, self.amount, self.viscosity)


class Entities:
    @staticmethod
    def stumbling_sheep(position: VectorN = VectorN(0, 0, 0)) -> "StumblingSheep":
        from lithicrivers.game.npcs import StumblingSheep
        return StumblingSheep(position)

    @staticmethod
    def starter_npc(position: VectorN = VectorN(5, 5, 0)) -> "ElderOak":
        from lithicrivers.game.npcs import ElderOak
        return ElderOak(position)

    @staticmethod
    def test_entity1(position: VectorN = VectorN(6, 5, 0)) -> "CrystalShard":
        from lithicrivers.game.npcs import CrystalShard
        return CrystalShard(position)

    @staticmethod
    def test_entity2(position: VectorN = VectorN(5, 6, 0)) -> "AncientRelic":
        from lithicrivers.game.npcs import AncientRelic
        return AncientRelic(position)

    @staticmethod
    def water(position: VectorN = VectorN(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid("water", position, amount, viscosity=100.0)

    @staticmethod
    def lava(position: VectorN = VectorN(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid("lava", position, amount, viscosity=200.0)  # Lava flows slower

    @staticmethod
    def acid(position: VectorN = VectorN(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid("acid", position, amount, viscosity=150.0)

    @staticmethod
    def oil(position: VectorN = VectorN(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid("oil", position, amount, viscosity=50.0)  # Oil flows faster

    @staticmethod
    def blood(position: VectorN = VectorN(0, 0, 0), amount: int = 1000) -> "Fluid":
        return Fluid("blood", position, amount, viscosity=120.0)
