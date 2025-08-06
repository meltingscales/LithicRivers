# Cursor Rules for LithicRivers

Make sure to also read docs/AI_INSTRUCTIONS.md.

## Project Context
This is a Python-based game development project called "LithicRivers". The project appears to be a roguelike or simulation game with:
- Sprite-based graphics system
- Sound and music assets
- Entity management
- Tile-based world system
- Structure generation

## Code Style Guidelines
- Follow PEP 8 for Python code formatting
- Use type hints where appropriate
- Write docstrings for functions and classes
- Use descriptive variable and function names
- Keep functions focused and single-purpose

## Project Structure
- `lithicrivers/` - Main package directory
- `lithicrivers/data/` - Game assets (sprites, sounds, structures)
- `lithicrivers/model/` - Game logic and data models
- `lithicrivers/tui/` - Text-based user interface
- `lithicrivers/demo/` - Demo and example code
- `lithicrivers/scripts/` - Utility scripts
- `lithicrivers/test/` - Test files

## File Extensions
- `.lrsprite` - Custom sprite format files
- `.lrstructure` - Custom structure format files
- `.py` - Python source files

## Development Guidelines
- When working with game assets, respect the existing file structure
- Follow the naming conventions used in the sprite and structure directories
- Consider performance implications when modifying game logic
- Maintain compatibility with existing asset formats
- Use the project's dependency management (pyproject.toml, uv.lock)

## Testing
- Write tests for new functionality
- Follow existing test patterns in the `test/` directory
- Ensure tests are comprehensive and maintainable

## Documentation
- Update README.md and DEVELOPMENT.md when adding significant features
- Document any new asset formats or file structures
- Keep TODO.md updated with current development priorities 
