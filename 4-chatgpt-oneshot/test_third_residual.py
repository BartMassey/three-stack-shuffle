from itertools import combinations
import unittest

from tools.residual_bounds import full_distances, neighbors, residual_bound
from tools.third_residual import (
    fast_residual_bound, partition_bound, selected_partition_bound,
)


class ThirdResidualTests(unittest.TestCase):
    def test_all_states_through_six(self):
        for n in range(1, 7):
            distances = full_distances(n)
            target = tuple(range(n))
            values = {}
            for state, exact in distances.items():
                value = fast_residual_bound(*state, target)
                self.assertEqual(value, residual_bound(*state, target))
                self.assertLessEqual(value, exact)
                values[state] = value
            for state, value in values.items():
                for following in neighbors(state):
                    self.assertLessEqual(value, 1 + values[following])

    def test_disjoint_partition_all_states_through_five(self):
        for n in range(1, 6):
            k = min(3, n)
            database = full_distances(k)
            patterns = tuple(combinations(range(n), k))
            for state, exact in full_distances(n).items():
                self.assertLessEqual(partition_bound(state, patterns, database), exact)
                self.assertLessEqual(selected_partition_bound(state, patterns, database), exact)


if __name__ == "__main__":
    unittest.main()
