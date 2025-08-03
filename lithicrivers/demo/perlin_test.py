from lithicrivers.worldgen import PerlinNoise

noise = PerlinNoise(111)

print("Testing with fractional coordinates:")
for i in range(10):
    x = i + 0.5
    print(f"noise_2d({x}, {x}) = {noise.noise_2d(x, x)}")

print("\nTesting with small fractional values:")
for i in range(10):
    x = i * 0.1
    print(f"noise_2d({x}, {x}) = {noise.noise_2d(x, x)}")

print("\nTesting different Y values with fractional X:")
for y in range(-5, 6):
    print(f"noise_2d(10.5, {y}) = {noise.noise_2d(10.5, y)}")