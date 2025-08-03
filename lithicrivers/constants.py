from lithicrivers.model.vector import VectorN

VEC_DOWN = VectorN(0, 0, 1)
VEC_UP = VectorN(0, 0, -1)
VEC_NORTH = -VectorN(
    0, 1, 0
)  # negative because i am laaaaazy and my Y values are flipped
VEC_SOUTH = -VectorN(
    0, -1, 0
)  # negative because i am laaaaazy and my Y values are flipped
VEC_WEST = VectorN(-1, 0, 0)
VEC_EAST = VectorN(1, 0, 0)
VEC_ZERO = VectorN(0, 0, 0)

# Diagonal movement vectors for 8-way movement
VEC_NORTHWEST = VEC_NORTH + VEC_WEST
VEC_NORTHEAST = VEC_NORTH + VEC_EAST
VEC_SOUTHWEST = VEC_SOUTH + VEC_WEST
VEC_SOUTHEAST = VEC_SOUTH + VEC_EAST
