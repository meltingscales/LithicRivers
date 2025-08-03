from lithicrivers.worldgen import PerlinNoise

noise = PerlinNoise(111)

for i in range(200):
    print(noise.noise_2d(i, i), end=" ")