import unittest

from tools.residual_bounds import full_distances, independent_bound, residual_bound


class ResidualBoundTests(unittest.TestCase):
    def test_all_small_states(self):
        for n in range(1, 6):
            target = tuple(range(n))
            for state, distance in full_distances(n).items():
                lower = residual_bound(*state, target)
                self.assertGreaterEqual(lower, independent_bound(*state, target))
                self.assertLessEqual(lower, distance)
                self.assertEqual(lower % 2, (len(state[0]) + len(state[2])) % 2)

    def test_relabeling_and_known_gap(self):
        self.assertEqual(residual_bound((2,), (0, 4, 3), (1,), tuple(range(5))), 10)
        self.assertEqual(residual_bound((30,), (10, 50, 40), (20,),
                                        (10, 20, 30, 40, 50)), 10)
        self.assertEqual(residual_bound((2, 1, 0), (), (), (0, 1, 2)), 3)
        self.assertEqual(residual_bound((), (0, 1, 2), (), (0, 1, 2)), 0)
        self.assertRaises(ValueError, residual_bound, (0,), (0,), (), (0, 1))


if __name__ == "__main__":
    unittest.main()
