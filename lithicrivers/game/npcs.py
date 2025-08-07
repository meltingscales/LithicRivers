from lithicrivers.constants import VEC_NORTH, VEC_SOUTH, VEC_EAST, VEC_WEST
from lithicrivers.game.entities import Entity
from lithicrivers.game.interfaces import SpriteRenderable
from lithicrivers.model.vector import VectorN


class NPC(Entity, SpriteRenderable):
    """A non-player character that can have conversations."""

    def __init__(
        self, name: str, position: VectorN, sprite: str = "N", color: str = "cyan"
    ):
        super().__init__(name, position)
        self.sprite = sprite
        self.color = color
        self.conversations = {}
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(name.lower().replace(" ", "_"), "entities")

        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color
        self._setup_default_conversation()

    def _setup_default_conversation(self) -> None:
        self.conversations = {
            "greeting": {
                "text": f"Hello, I am {self.name}.",
                "options": ["Goodbye"],
            },
            "goodbye": {
                "text": "Goodbye!",
                "options": [],
            },
        }

    def get_conversation(self, topic: str = "greeting"):
        return self.conversations.get(topic, self.conversations["greeting"])

    def handle_response(self, response: str, topic: str = "greeting"):
        if response == "Goodbye":
            return "goodbye"
        return "greeting"


class ElderOak(NPC):
    def __init__(self, position: VectorN):
        super().__init__("Elder Oak", position, sprite="N", color="cyan")
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("elder_oak", "entities")

        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color
        self._setup_default_conversation()

    def _setup_default_conversation(self) -> None:
        self.conversations = {
            "greeting": {
                "text": f"Hello, traveler! I am {self.name}. Welcome to LithicRivers!",
                "options": ["Tell me about this world", "What can you do?", "Goodbye"],
            },
            "about_world": {
                "text": "This is a world of endless possibilities. You can mine, build, and explore to your heart's content. The world is procedurally generated, so there's always something new to discover!",
                "options": [
                    "Tell me more about mining",
                    "What about building?",
                    "Back to greeting",
                ],
            },
            "about_mining": {
                "text": "Mining is simple! Just press 'u' when standing on a mineable tile like Gold Ore. You'll get valuable resources that you can use for crafting and trading.",
                "options": ["What about building?", "Back to greeting"],
            },
            "about_building": {
                "text": "Building is coming soon! You'll be able to place blocks and create structures. For now, focus on gathering resources through mining.",
                "options": ["Tell me more about mining", "Back to greeting"],
            },
            "goodbye": {
                "text": "Farewell, traveler! May your adventures be fruitful!",
                "options": ["OK"],
            },
        }

    def handle_response(self, response: str, topic: str = "greeting"):
        if response == "Tell me about this world":
            return "about_world"
        elif response == "What can you do?" or response == "Tell me more about mining":
            return "about_mining"
        elif response == "What about building?":
            return "about_building"
        elif response == "Back to greeting":
            return "greeting"
        elif response == "Goodbye" or response == "OK":
            return "goodbye"
        else:
            return "greeting"


class InteractiveEntity(Entity, SpriteRenderable):
    """An entity that can be interacted with."""

    def __init__(
        self,
        name: str,
        position: VectorN,
        sprite: str = "E",
        color: str = "yellow",
        interaction_text: str = "This is an interactive entity.",
    ):
        super().__init__(name, position)
        self.sprite = sprite
        self.color = color
        self.interaction_text = interaction_text

        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(name.lower().replace(" ", "_"), "entities")

        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color

    def render_sprite(self, scale: int = 1) -> str:
        """Render the entity sprite."""
        # Use the SpriteRenderable's render_sprite method
        return super().render_sprite(scale)

    def interact(self) -> str:
        """Handle interaction with this entity."""
        return self.interaction_text


class CrystalShard(InteractiveEntity):
    def __init__(self, position: VectorN):
        super().__init__(
            "Crystal Shard",
            position,
            sprite="C",
            color="blue",
            interaction_text="This crystal shard glows with a soft blue light. It seems to pulse with energy.",
        )
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("crystal_shard", "entities")

        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color


class AncientRelic(InteractiveEntity):
    def __init__(self, position: VectorN):
        super().__init__(
            "Ancient Relic",
            position,
            sprite="R",
            color="red",
            interaction_text="This ancient relic is covered in mysterious runes. It radiates warmth.",
        )
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("ancient_relic", "entities")

        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color


class StumblingSheep(InteractiveEntity):
    """A sheep that stumbles around randomly."""

    def __init__(self, position: VectorN):
        super().__init__(
            name="Stumbling Sheep",
            position=position,
            sprite="S",
            color="white",
            interaction_text="The sheep stumbles around aimlessly, occasionally making confused noises.",
        )
        # Load sprites from external data
        from lithicrivers.sprite_loader import get_sprite_loader

        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("stumbling_sheep", "entities")

        # Use external sprite data
        self.sprite_sheet = sprite_data.sprites
        self.color = sprite_data.color
        self.speed = 0.2  # Sheep moves at 0.2x speed (tick every 5 frames)

    def tick(self) -> None:
        """Move randomly every few ticks."""
        import random

        # Use deterministic randomness based on world seed and tick
        # This ensures the same behavior for the same seed
        # Get gametick from world if available, otherwise use 0
        gametick = 0
        if hasattr(self, '_listeners'):
            for listener in self._listeners:
                if hasattr(listener, 'gametick'):
                    gametick = listener.gametick
                    break

        # Only use deterministic seeding if not in a test environment
        # This allows mocking to work in tests
        if not hasattr(random, '_test_mode'):
            random.seed(f"sheep_{self.position.serialize()}_{gametick}")

        if random.random() < 0.1:  # 10% chance to move each tick
            directions = [VEC_NORTH, VEC_SOUTH, VEC_EAST, VEC_WEST]
            direction = random.choice(directions)
            self.move(direction)
