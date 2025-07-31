# Development Guide

This guide explains how to develop and test LithicRivers using the new modular architecture.

## Architecture Overview

### Modular Game Engine

The game has been refactored to use a modular architecture with clear separation of concerns:

- **`game_engine.py`**: Core game logic and state management
- **`ui.py`**: User interface components
- **`game.py`**: Legacy game classes (will be gradually migrated)

### Key Components

#### GameState
Immutable game state that can be easily serialized and tested:
```python
from lithicrivers.game_engine import GameState, VectorN

state = GameState(
    player_position=VectorN(0, 0, 0),
    player_health=100,
    player_stamina=100
)
```

#### GameEngine
Manages game state and applies actions:
```python
from lithicrivers.game_engine import GameEngine

engine = GameEngine()
engine.move_player(VEC_NORTH)
engine.mine_at_player_position()
```

#### GameActions
Immutable actions that transform game state:
```python
from lithicrivers.game_engine import MovePlayerAction, MineAction

action = MovePlayerAction(VEC_NORTH)
new_state = action.apply(current_state)
```

## Testing Framework

### Unit Testing Game Logic

The modular architecture makes it easy to test game logic without UI dependencies:

```python
import unittest
from lithicrivers.game_engine import GameEngine, GameState
from lithicrivers.game import Tiles
from lithicrivers.model.vector import VectorN

class TestGameLogic(unittest.TestCase):
    def setUp(self):
        self.engine = GameEngine()
    
    def test_player_movement(self):
        initial_pos = self.engine.state.player_position
        self.engine.move_player(VEC_NORTH)
        self.assertEqual(
            self.engine.state.player_position,
            initial_pos + VEC_NORTH
        )
    
    def test_mining(self):
        # Set up a mineable tile
        self.engine.set_tile(VectorN(0, 0, 0), Tiles.Tree())
        
        # Mine the tile
        self.engine.mine_at_player_position()
        
        # Check that tile was replaced
        tile = self.engine.get_tile_at_position(VectorN(0, 0, 0))
        self.assertEqual(tile, Tiles.Empty())
```

### UI Testing

The UI testing framework allows testing UI components without a real terminal:

```python
from lithicrivers.test.test_ui import UITestCase, MockScreen

class TestGameWidget(UITestCase):
    def test_widget_rendering(self):
        widget = self.GameWidget(self.game_engine)
        
        # Mock the frame and update widget
        widget._frame = Mock()
        widget._frame.canvas = self.mock_canvas
        widget.update(0)
        
        # Check rendered content
        content = self.get_rendered_content()
        self.assertNotEqual(content.strip(), '')
```

### Running Tests

```bash
# Run all tests
make test

# Run tests with coverage
make test-coverage

# Run specific test file
python -m unittest lithicrivers.test.test_game_engine

# Run tests
make test
```

## Development Workflow

### Using uv (Recommended)

```bash
# Install dependencies
make install

# Run the game
make run

# Run tests
make test

# Format code
make format

# Lint code
make lint
```

### Migrating from Poetry to uv

```bash
# Migrate to uv
make migrate-to-uv

# After migration, use standard commands
make install
make run
make test
```

**Note:** This project now uses uv exclusively. Poetry is no longer supported.



### Code Quality

The project uses pre-commit hooks for code quality:

```bash
# Install pre-commit hooks
make setup-dev

# Run pre-commit manually
pre-commit run --all-files
```

## Adding New Features

### 1. Game Logic

When adding new game features, implement them in the modular engine:

```python
# In game_engine.py
@dataclass
class CraftAction:
    recipe: str
    
    def apply(self, state: GameState) -> GameState:
        # Implement crafting logic
        new_state = state.copy()
        # ... crafting implementation
        return new_state

# In GameEngine class
def craft_item(self, recipe: str) -> GameState:
    action = CraftAction(recipe)
    return self.apply_action(action)
```

### 2. UI Components

When adding UI components, make them testable:

```python
# In ui.py
class CraftingWidget(asciimatics.widgets.Widget):
    def __init__(self, game_engine: GameEngine):
        self.game_engine = game_engine
    
    def update(self, frame_no):
        # Render crafting interface
        pass
```

### 3. Tests

Always write tests for new features:

```python
# In test_game_engine.py
def test_crafting(self):
    engine = GameEngine()
    
    # Set up inventory with materials
    engine.state.inventory.add_item(Items.Stick())
    engine.state.inventory.add_item(Items.Stick())
    
    # Craft an item
    new_state = engine.craft_item("wooden_pickaxe")
    
    # Check result
    self.assertIn("Wooden Pickaxe", new_state.inventory.items)
```

## Performance Testing

The modular architecture makes it easy to test performance:

```python
import time
from lithicrivers.game_engine import GameEngine

def test_world_generation_performance():
    start_time = time.time()
    
    engine = GameEngine()
    # Generate large world
    for i in range(1000):
        engine.set_tile(VectorN(i, i, 0), Tiles.Dirt())
    
    end_time = time.time()
    assert end_time - start_time < 1.0  # Should complete in under 1 second
```

## Debugging

### State Inspection

The modular design makes debugging easier:

```python
# Inspect current game state
print(f"Player position: {engine.state.player_position}")
print(f"Player health: {engine.state.player_health}")
print(f"World tiles: {len(engine.state.world_data)}")

# Check action history
for i, action in enumerate(engine.action_history):
    print(f"Action {i}: {type(action).__name__}")
```

### Save/Load for Debugging

```python
# Save current state for debugging
engine.save_state(Path("debug_save.pkl"))

# Load state in another session
new_engine = GameEngine()
new_engine.load_state(Path("debug_save.pkl"))
```

## Best Practices

1. **Always write tests** for new game logic
2. **Use the modular engine** for new features
3. **Keep UI and logic separate** - UI should only handle display and input
4. **Use immutable actions** for state changes
5. **Test UI components** with the mock framework
6. **Run pre-commit hooks** before committing
7. **Document new features** with examples

## Troubleshooting

### Common Issues

1. **Import errors**: Make sure to import from the correct modules
2. **Circular imports**: Use lazy imports in `game_engine.py`
3. **Test failures**: Check that mocks are set up correctly
4. **UI not updating**: Ensure the game engine state is being updated

### Getting Help

- Check existing tests for examples
- Look at the `test_game_engine.py` for comprehensive examples
- Use the mock framework for UI testing
- Run `make help` to see all available commands 