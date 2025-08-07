from lithicrivers.model.vector import VectorN


class EntityEvent:
    """Base class for entity events."""
    pass


class EntityMovedEvent(EntityEvent):
    """Event fired when an entity moves."""
    def __init__(self, entity: "Entity", old_position: VectorN, new_position: VectorN):
        self.entity = entity
        self.old_position = old_position
        self.new_position = new_position


class EntityListener:
    """Interface for objects that listen to entity events."""
    def on_entity_moved(self, event: EntityMovedEvent) -> None:
        """Called when an entity moves."""
        pass
