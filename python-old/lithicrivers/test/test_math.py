import unittest

from lithicrivers.model.vector import VectorN


class TestVectorN(unittest.TestCase):
    def test_simple(self):
        v1 = VectorN.create(1, 2, 0)
        self.assertEqual(v1.x, 1)
        self.assertEqual(v1.y, 2)
        self.assertEqual(v1.z, 0)
        self.assertEqual(v1.w, None)

        self.assertEqual(v1[0], 1)
        self.assertEqual(v1[1], 2)
        self.assertEqual(v1[2], 0)

        da_error = "Didn't throw error."
        try:
            self.assertEqual(v1[3], None)
        except IndexError as e:
            da_error = e
        finally:
            self.assertIsInstance(da_error, IndexError)

    def test_serialize(self):
        v1 = VectorN.create(1, 2, 3, 4)
        self.assertEqual(v1.serialize(), "1,2,3,4")
        self.assertEqual(VectorN.deserialize("1,2,3,4"), v1)

        v2 = VectorN.create(1)
        self.assertEqual(v2.serialize(), "1")
        self.assertEqual(VectorN.deserialize("1"), v2)

    def test_math(self):
        v1 = VectorN.create(1, 2, 3)

        self.assertEqual(v1 + v1, VectorN.create(2, 4, 6))
        self.assertEqual(v1 - v1, VectorN.create(0, 0, 0))
        self.assertEqual(v1 * v1, VectorN.create(1, 4, 9))

    def test_vector_trim(self):
        v1 = VectorN.create(1, 2, 3).trim(2)
        v2 = VectorN.create(1, 2)

        # print(v1)
        # print(v2)

        self.assertEqual(v1, v2)

    def test_vec_n_bounding_box(self):
        v1 = VectorN.create(1, 2)
        v2 = VectorN.create(6, 7)

        v3 = VectorN.create(4, 4)

        v4 = VectorN.create(1, 9)

        self.assertTrue(v3.inside_bounding_rect(v1, v2))
        # swap args shouldnt matter, it just flips the rect by 90 degrees
        self.assertTrue(v3.inside_bounding_rect(v2, v1))

        self.assertFalse(v4.inside_bounding_rect(v1, v2))
        self.assertFalse(v4.inside_bounding_rect(v2, v1))
