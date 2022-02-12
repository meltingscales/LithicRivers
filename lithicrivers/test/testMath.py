from lithicrivers.model import Vector2

'''
god i hate math
'''

import unittest


class IHateMath(unittest.TestCase):
    def testVecBoundingBox(self):
        v1 = Vector2(1, 2)
        v2 = Vector2(6, 7)

        v3 = Vector2(4, 4)

        v4 = Vector2(1, 9)

        self.assertTrue(v3.insideBoundingRect(v1, v2))
        # swap args shouldnt matter, it just flips the rect by 90 degrees
        self.assertTrue(v3.insideBoundingRect(v2, v1))

        self.assertFalse(v4.insideBoundingRect(v1, v2))
        self.assertFalse(v4.insideBoundingRect(v2, v1))
