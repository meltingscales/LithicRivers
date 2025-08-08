from lithicrivers.constants import VEC_EAST, VEC_NORTH, VEC_SOUTH, VEC_WEST
from lithicrivers.game.entities import Entity
from lithicrivers.game.interfaces import SpriteRenderable
from lithicrivers.model.vector import VectorN


class NPC(Entity, SpriteRenderable):
    """A non-player character that can have conversations."""
    sprite: str = "N"
    color: str = "cyan"
    sprite_sheet: list[str] | None = None
    conversations: dict | None = None

    @classmethod
    def create(cls, name: str, position: VectorN, sprite: str = "N", color: str = "cyan") -> "NPC":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(
            name.lower().replace(" ", "_"), "entities"
        )
        npc = cls(name=name, position=position, sprite=sprite_data.sprites[0] if sprite_data.sprites else sprite, color=sprite_data.color)
        npc.sprite_sheet = sprite_data.sprites
        npc._setup_default_conversation()
        return npc

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
    @classmethod
    def create(cls, position: VectorN) -> "ElderOak":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("elder_oak", "entities")
        npc = cls(name="Elder Oak", position=position, sprite=sprite_data.sprites[0] if sprite_data.sprites else "N", color=sprite_data.color)
        npc.sprite_sheet = sprite_data.sprites
        npc._setup_default_conversation()
        return npc

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
    sprite: str = "E"
    color: str = "yellow"
    sprite_sheet: list[str] | None = None
    interaction_text: str = "This is an interactive entity."

    @classmethod
    def create(cls, name: str, position: VectorN, sprite: str = "E", color: str = "yellow", interaction_text: str = "This is an interactive entity.") -> "InteractiveEntity":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite(
            name.lower().replace(" ", "_"), "entities"
        )
        entity = cls(name=name, position=position, sprite=sprite_data.sprites[0] if sprite_data.sprites else sprite, color=sprite_data.color, interaction_text=interaction_text)
        entity.sprite_sheet = sprite_data.sprites
        return entity

    def render_sprite(self, scale: int = 1) -> str:
        """Render the entity sprite."""
        # Use the SpriteRenderable's render_sprite method
        return super().render_sprite(scale)

    def interact(self) -> str:
        """Handle interaction with this entity."""
        return self.interaction_text


class CrystalShard(InteractiveEntity):
    @classmethod
    def create(cls, position: VectorN) -> "CrystalShard":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("crystal_shard", "entities")
        entity = cls(name="Crystal Shard", position=position, sprite=sprite_data.sprites[0] if sprite_data.sprites else "C", color=sprite_data.color, interaction_text="This crystal shard glows with a soft blue light. It seems to pulse with energy.")
        entity.sprite_sheet = sprite_data.sprites
        return entity


class AncientRelic(InteractiveEntity):
    @classmethod
    def create(cls, position: VectorN) -> "AncientRelic":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("ancient_relic", "entities")
        entity = cls(name="Ancient Relic", position=position, sprite=sprite_data.sprites[0] if sprite_data.sprites else "R", color=sprite_data.color, interaction_text="This ancient relic is covered in mysterious runes. It radiates warmth.")
        entity.sprite_sheet = sprite_data.sprites
        return entity


class StumblingSheep(InteractiveEntity):
    """A sheep that stumbles around randomly."""

    @classmethod
    def create(cls, position: VectorN) -> "StumblingSheep":
        from lithicrivers.sprite_loader import get_sprite_loader
        sprite_loader = get_sprite_loader()
        sprite_data = sprite_loader.load_sprite("stumbling_sheep", "entities")
        entity = cls(name="Stumbling Sheep", position=position, sprite=sprite_data.sprites[0] if sprite_data.sprites else "S", color=sprite_data.color, interaction_text="The sheep stumbles around aimlessly, occasionally making confused noises.")
        entity.sprite_sheet = sprite_data.sprites
        entity.speed = 0.2  # Sheep moves at 0.2x speed (tick every 5 frames)
        return entity

    def tick(self) -> None:
        """Move randomly every few ticks."""
        from lithicrivers.game.rng import SimpleRNG

        # Use deterministic randomness based on world seed and tick
        # This ensures the same behavior for the same seed
        # Get gametick from world if available, otherwise use 0
        gametick = 0
        if hasattr(self, "_listeners"):
            for listener in self._listeners:
                if hasattr(listener, "gametick"):
                    gametick = listener.gametick
                    break

        # Only use deterministic seeding if not in a test environment
        # This allows mocking to work in tests
        if not hasattr(SimpleRNG, "_test_mode"):
            rng = SimpleRNG.create(f"sheep_{self.position.serialize()}_{gametick}")

        if rng.random() < 0.1:  # 10% chance to move each tick
            directions = [VEC_NORTH, VEC_SOUTH, VEC_EAST, VEC_WEST]
            direction = rng.choice(directions)
            self.move(direction)
