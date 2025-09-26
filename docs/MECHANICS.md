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

### Surface vs Underground
- **Surface Level (Z >= 0)**: Fog of war is disabled - all areas are fully visible
- **Underground (Z < 0)**: Full fog of war mechanics apply for atmospheric exploration

This system encourages careful exploration and resource management in underground areas, as torches provide significant tactical advantage in dark environments while consuming inventory space. Surface exploration remains unobstructed for easier navigation.

## Map

TODO: Populate with map description

## Combat

TODO: Populate with combat description