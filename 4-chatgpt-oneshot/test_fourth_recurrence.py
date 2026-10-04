import math
import unittest

from oriented_merge import oriented_bounds
from ternary_merge import ternary_bounds
from tools.fourth_recurrence import (
    COEFFICIENT, CUTOFF, DELTA, LEAF_COSTS, leaf_shift,
    optimized_recurrence, potential,
)


class FourthRecurrenceTests(unittest.TestCase):
    def test_frozen_leaf_constants_and_exact_envelopes(self):
        self.assertEqual(len(LEAF_COSTS), CUTOFF + 1)
        for n, pair in enumerate(LEAF_COSTS):
            self.assertEqual(pair, oriented_bounds(n)[:2])
            self.assertGreaterEqual(pair[0], 0)
            self.assertGreaterEqual(pair[1], n)
            self.assertLessEqual(pair[0], 8 * n)
            self.assertLessEqual(pair[1], 8 * n)
            if n >= 2:
                self.assertLessEqual(pair[0], n + LEAF_COSTS[n // 2][1]
                                     + LEAF_COSTS[n - n // 2][1])

    def test_empty_singleton_and_invalid_sizes(self):
        for repark in (False, True):
            self.assertEqual(optimized_recurrence(0, repark), ([0], [0]))
            self.assertEqual(optimized_recurrence(1, repark), ([0, 0], [0, 1]))
        self.assertEqual(potential(0, 0), 0)
        self.assertEqual(potential(0, 1), 0)
        with self.assertRaises(ValueError):
            optimized_recurrence(-1)

    def test_finite_potential_shift(self):
        shift, n, endpoint = leaf_shift()
        self.assertEqual((n, endpoint), (24, 1))
        self.assertAlmostEqual(shift, 5.5 - COEFFICIENT * math.log2(24) - DELTA)
        for n, pair in enumerate(LEAF_COSTS):
            for endpoint in (0, 1):
                self.assertGreaterEqual(pair[endpoint] + 1e-10,
                                        potential(n, endpoint, shift))
                self.assertGreaterEqual(pair[endpoint], potential(n, endpoint))

    def test_inductive_envelopes_and_repark_order(self):
        plain = optimized_recurrence(512, repark=False)
        credited = optimized_recurrence(512, repark=True)
        shift = leaf_shift()[0]
        for values in (plain, credited):
            central, parked = values
            for n in range(CUTOFF + 1, len(central)):
                for endpoint in (0, 1):
                    self.assertLessEqual(values[endpoint][n],
                                         ternary_bounds(n)[endpoint])
                    self.assertGreaterEqual(values[endpoint][n] + 1e-9,
                                            potential(n, endpoint, shift))
                    self.assertLessEqual(credited[endpoint][n], plain[endpoint][n])
            for n, pair in ((65, (474, 477)), (128, (1064, 1124)),
                            (512, (5608, 5720))):
                self.assertEqual((central[n], parked[n]), pair)
        self.assertGreater((2 - COEFFICIENT) * (CUTOFF + 1) - 2, 0)


if __name__ == "__main__":
    unittest.main()
