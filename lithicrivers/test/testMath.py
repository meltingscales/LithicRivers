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

        # self.assert_(v3.insideBoundingBox(v1, v2))
        self.assert_(v3.insideBoundingBox(v2, v1))
