from lithicrivers.model import Vector2

'''
god i hate math
'''


def testVecBoundingBox():
    v1 = Vector2(0, 0)
    v2 = Vector2(5, 5)

    v3 = Vector2(4, 4)

    assert (v3.insideBoundingBox(v1, v2))
    assert (v3.insideBoundingBox(v2, v1))
