from lithicrivers.model.vector import VectorN

VEC_DOWN = VectorN(0, 0, 1)
VEC_UP = VectorN(0, 0, -1)
VEC_NORTH = -VectorN(0, 1, 0)  # negative because i am laaaaazy and my Y values are flipped
VEC_SOUTH = -VectorN(0, -1, 0)  # negative because i am laaaaazy and my Y values are flipped
VEC_WEST = VectorN(-1, 0, 0)
VEC_EAST = VectorN(1, 0, 0)
NESW_MNEMONIC = \
    '''
      N
    W   E
      S
    '''