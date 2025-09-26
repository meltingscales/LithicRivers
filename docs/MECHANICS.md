# Mechanics

Brief descriptions of game mechanics. To guide future development and tutorials.

## Cheat Console

The game features a comprehensive command system for debugging and testing:

### Command System
- **Access**: Press `/` to open the command console
- **Autocomplete**: Type partial commands and press Tab to autocomplete or cycle through suggestions
- **Dynamic Help**: Real-time suggestion display shows matching commands as you type

### Available Commands
- **`tp x y z`**: Teleport to specified coordinates (e.g., `/tp 10 5 -1`)
- **`noclip_toggle`**: Toggle noclip mode for walking through walls
- **`fogofwar_toggle`**: Toggle fog of war rendering (shows/hides '?' markers)

### Features
- **Programmatic Registration**: Commands are automatically registered and displayed
- **Tab Completion**: Press Tab to complete commands or cycle through options
- **Live Suggestions**: Shows matching commands as you type
- **Error-Safe**: Unknown commands are silently ignored

## Fog of War and Light Sources

The game features a fog of war system that creates atmospheric exploration:

### Light Sources
- **Default Light**: Players emit a base 2-tile radius light source
- **Torch**: Equipping a torch (if available in inventory) increases light radius to 8 tiles
- **Torch Toggle**: Press 't' to toggle torch on/off (requires torch in inventory)

### Fog of War States
- **Unvisited Areas**: Rendered as very dark grey '?' - completely unknown terrain
- **Visited but Dark**: Areas you've been to but aren't currently illuminated show in grayscale
- **Illuminated Areas**: Currently lit areas render in full color with normal visibility

### Realistic Light Mechanics
- **Raycasting**: Light rays are traced using Bresenham line algorithm for realistic shadows
- **Wall Occlusion**: Light does not pass through walls - creates realistic shadows
- **Line-of-Sight**: You can see walls that block your view, but not areas behind them
- **Circular Falloff**: Light has natural circular distance-based illumination

### Surface vs Underground
- **Surface Level (Z >= 0)**: Fog of war is disabled - all areas are fully visible
- **Underground (Z < 0)**: Full fog of war mechanics apply for atmospheric exploration

### Debug Features
- **Toggle Fog of War**: Use `/fogofwar_toggle` command to disable '?' rendering for debugging
- **Mode Indicators**: UI shows current torch status (T) and fog of war status (FOG)

This system encourages careful exploration and resource management in underground areas, as torches provide significant tactical advantage in dark environments while consuming inventory space. The realistic light physics add strategic depth to underground exploration.

## Map

TODO: Populate with map description

## Combat

TODO: Populate with combat description