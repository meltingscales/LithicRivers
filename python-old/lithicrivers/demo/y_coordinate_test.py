from lithicrivers.model.vector import VectorN
from lithicrivers.worldgen import SeededWorldGenerator

# Create a world generator with a fixed seed
generator = SeededWorldGenerator(seed=42)

print("Testing Y coordinate variation for underground terrain (Z < 0):")
print("=" * 60)

# Test different Y values at a fixed X and Z
x_coord = 10
z_coord = -1  # Underground

for y in range(-10, 11):
    pos = VectorN.create(x_coord, y, z_coord)
    tile = generator.generate_tile_for_position(pos)
    print(f"Position ({x_coord}, {y}, {z_coord}) -> {tile}")

print("\nTesting Y coordinate variation for surface terrain (Z = 0):")
print("=" * 60)

# Test different Y values at surface level
z_coord = 0  # Surface

for y in range(-10, 11):
    pos = VectorN.create(x_coord, y, z_coord)
    tile = generator.generate_tile_for_position(pos)
    print(f"Position ({x_coord}, {y}, {z_coord}) -> {tile}")

print("\nTesting X coordinate variation for comparison:")
print("=" * 60)

# Test different X values at a fixed Y and Z
y_coord = 5
z_coord = -1  # Underground

for x in range(-10, 11):
    pos = VectorN.create(x, y_coord, z_coord)
    tile = generator.generate_tile_for_position(pos)
    print(f"Position ({x}, {y_coord}, {z_coord}) -> {tile}")
