> What would be a good choice for making custom structures or maps? I know that @crates/client/assets/structures/small_temple.lrstructure/shape_layers.txt is one way to store it but...it's not really easy to visualize. Are there any low-res voxel 3D editors that I could then make a custom exporter for? I want to make the main questline's factory that's 10 miles away [- the first quest is to travel 10 miles to the robotics factory that made you to replace your damaged fusion core and fix your limbs. you remember the location. TODO add a maps and waypoint system.] and I want to design it by hand, but not in our shape_layers.txt  format because it's pretty limited.

I'd like to avoid MagicaVoxel because i primarily use linux to develop.

reply from ai:

  Best Options for Linux:

  1. Goxel - Open source, specifically designed for voxel editing, lightweight and perfect for game assets
  2. VoxEdit (The Sandbox) - Free voxel editor with good export options
  3. Qubicle - Professional voxel editor with Linux support (paid)
  4. Blender with voxel addons - Free, powerful, and you could use addons like "Voxel Tools" or "Extra Objects"

  Recommended Approach:
  Goxel would be my top recommendation - it's:
  - Native Linux support
  - Lightweight and fast
  - Exports to various formats (PLY, OBJ, etc.)
  - Simple enough for game structures but powerful enough for detail

  Custom Export Strategy:
  You could create a simple converter that:
  1. Exports from Goxel as PLY/OBJ
  2. Reads the mesh data
  3. Converts to your shape_layers.txt format or a new binary format
  4. Handles multiple layers/materials if needed
