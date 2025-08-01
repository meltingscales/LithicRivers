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

# Diagonal movement vectors for 8-way movement
VEC_NORTHWEST = VEC_NORTH + VEC_WEST
VEC_NORTHEAST = VEC_NORTH + VEC_EAST
VEC_SOUTHWEST = VEC_SOUTH + VEC_WEST
VEC_SOUTHEAST = VEC_SOUTH + VEC_EAST

# Numpad key codes for 8-way movement
NUMPAD_7 = 55  # Northwest
NUMPAD_8 = 56  # North
NUMPAD_9 = 57  # Northeast
NUMPAD_4 = 52  # West
NUMPAD_6 = 54  # East
NUMPAD_1 = 49  # Southwest
NUMPAD_2 = 50  # South
NUMPAD_3 = 51  # Southeast
NUMPAD_5 = 53  # Center (no movement)

NESW_MNEMONIC = """
      N
    W   E
      S
    """

# 8-way movement mnemonic
EIGHT_WAY_MNEMONIC = """
    7 8 9
    4   6
    1 2 3
    """
