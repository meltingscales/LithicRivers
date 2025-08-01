# LithicRivers Configuration System

This directory contains user-configurable JSON files for keybinds and game settings.

## Files

### `keybinds.json`
Contains all keyboard bindings organized by category:

- **movement**: Player movement controls (WASD + QE)
- **viewport**: Camera/viewport controls
- **scale**: Zoom in/out controls
- **action**: Game actions like mining

### `settings.json`
Contains game settings organized by category:

- **game**: Basic game information and logging
- **world**: World generation settings (separate for production/testing)
- **viewport**: Camera behavior settings

## How to Modify

### Keybinds
You can change any keybind by editing the `keybinds.json` file. For example, to change the mine key from 'u' to 'm':

```json
{
  "action": {
    "MINE": "m"
  }
}
```

### Settings
You can modify game settings by editing the `settings.json` file. For example, to change the world size:

```json
{
  "world": {
    "DEFAULT_SIZE_RADIUS": {
      "production": [100, 100, 5],
      "testing": [10, 10, 2]
    }
  }
}
```

## File Locations

### Development
When running from source, config files are located in:
- `lithicrivers/config/keybinds.json`
- `lithicrivers/config/settings.json`

### Executable
When running as a compiled executable, config files are located in:
- `./config/keybinds.json` (adjacent to the executable)
- `./config/settings.json` (adjacent to the executable)

## Auto-Creation
If the config files don't exist, the game will automatically create them with default values when first run.

## Environment-Specific Settings
Some settings (like world size) have different values for production vs testing environments. The game automatically uses the appropriate values based on the `TESTING` environment variable.

## Steam Integration
This configuration system is designed to work seamlessly with Steam distribution:
- Config files are packaged with the executable
- Users can easily modify settings without reinstalling
- Changes persist between game sessions
- No technical knowledge required to modify settings 