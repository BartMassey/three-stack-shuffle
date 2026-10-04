import itertools
import random
import unittest

from endpoint_portfolio import _select, endpoint_candidates, endpoint_portfolio
from exact_solver import optimal
from hybrid_merge import _join, _swap
from oriented_merge import _central_merge
from parked_leaf import parked_optimal
from solver import ALGORITHMS
from three_stack import Machine


class EndpointPortfolioTests(unittest.TestCase):
    def verify(self, initial, target, endpoint, word):
        machine = Machine(initial)
        guards = {"A": -101, "B": -102, "D": -103}
        for side, guard in guards.items():
            machine.stacks[side].insert(0, guard)
        for operation in word:
            self.assertNotIn(machine.stacks[operation[0]][-1], guards.values())
            machine.move(operation)
        for side in "ADB":
            expected = list(target) if side == endpoint else []
            if endpoint == "D":
                expected.reverse()
            self.assertEqual(machine.stacks[side], [guards[side]] + expected)

    def test_small_exact_endpoints(self):
        for n in range(6):
            initial = list(range(n))
            for target in itertools.permutations(initial):
                for width in (2, 4):
                    for endpoint in "ADB":
                        words = endpoint_candidates(initial, target, width, endpoint)
                        self.assertLessEqual(len(words), width)
                        exact = optimal(initial, target) if endpoint == "D" else parked_optimal(initial, target)
                        self.assertEqual(len(words[0]), len(exact))
                        for word in words:
                            self.verify(initial, target, endpoint, word)

    def test_recursive_bases(self):
        rng = random.Random(2026100415)
        for n in (9, 13, 26, 52, 64):
            initial = rng.sample(range(n), n)
            target = rng.sample(initial, n)
            for endpoint in "ADB":
                for width in ((2, 4) if n <= 13 else (2,)):
                    for word in endpoint_candidates(initial, target, width, endpoint):
                        self.verify(initial, target, endpoint, word)

    def test_longer_distinct_boundary_survives(self):
        words = ((), ("DA", "DB", "AD", "BD"))
        self.assertEqual(_select(words, 2), words)

    def test_policy_default_and_selection(self):
        words = ((), ("DA", "DB", "DA"), ("DB",) * 5,
                 ("DA", "DB", "AD", "BD"))
        self.assertEqual(_select(words, 2), _select(words, 2, "shortest"))
        self.assertEqual(_select(words, 2, "boundary"), ((), ("DB",) * 5))
        initial = list(range(9))
        target = random.Random(2026100406).sample(initial, len(initial))
        for endpoint in "ADB":
            self.assertEqual(endpoint_candidates(initial, target, 4, endpoint),
                             endpoint_candidates(initial, target, 4, endpoint,
                                                 policy="shortest"))
        self.assertEqual(endpoint_portfolio(initial, target, width=4),
                         endpoint_portfolio(initial, target, width=4,
                                            policy="shortest"))

    def test_boundary_recursive_guarded_and_fallback(self):
        initial = list(range(13))
        target = random.Random(2026100406).sample(initial, len(initial))
        for endpoint in "ADB":
            for word in endpoint_candidates(initial, target, 4, endpoint,
                                            policy="boundary"):
                self.verify(initial, target, endpoint, word)
        for seconds in (0, 8):
            word = endpoint_portfolio(initial, target, width=4, seconds=seconds,
                                      policy="boundary")
            self.verify(initial, target, "D", word)
            self.assertLessEqual(len(word), len(ALGORITHMS["oriented_window"](initial, target)))

    def test_policy_validation(self):
        self.assertRaises(ValueError, _select, [], 4, "unknown")
        self.assertRaises(ValueError, endpoint_candidates, [0], [0], policy="unknown")
        self.assertRaises(ValueError, endpoint_portfolio, [0], [0], policy="unknown")

    def test_longer_child_wins_parent_cancellation(self):
        initial = [5, 2, 4, 3, 8, 6, 7, 1, 0]
        left = endpoint_candidates(initial[:4], sorted(initial[:4]), 4, "A")
        right = endpoint_candidates(initial[4:], sorted(initial[4:]), 4, "A")
        merge = _central_merge(sorted(initial[:4]), sorted(initial[4:]), False)
        shortest = [_join((first, _swap(second), merge))
                    for first in left for second in right
                    if len(first) == len(left[0]) and len(second) == len(right[0])]
        candidates = [(_join((first, _swap(second), merge)), first, second)
                      for first in left for second in right]
        word, first, second = min(candidates, key=lambda item: len(item[0]))
        self.assertEqual(min(map(len, shortest)), 28)
        self.assertEqual(len(word), 26)
        self.assertEqual((len(second), len(right[0])), (15, 13))
        self.verify(initial, list(range(9)), "D", word)

    def test_fallback_and_validation(self):
        initial = list(range(13))
        target = random.Random(15).sample(initial, len(initial))
        details = {}
        word = endpoint_portfolio(initial, target, seconds=0, details=details)
        self.assertEqual(word, ALGORITHMS["oriented_window"](initial, target))
        self.assertTrue(details["timed_out"])
        self.verify(initial, target, "D", word)
        for width in (2, 4):
            candidate = endpoint_portfolio(initial, target, width=width)
            self.assertLessEqual(len(candidate), len(word))
            self.verify(initial, target, "D", candidate)
        larger = list(range(65))
        self.assertEqual(endpoint_portfolio(larger, larger), [])
        self.assertRaises(ValueError, endpoint_candidates, [0, 0], [0, 1])
        self.assertRaises(ValueError, endpoint_candidates, [0], [0], width=3)
        self.assertRaises(ValueError, endpoint_candidates, [0], [0], endpoint="C")
        self.assertRaises(ValueError, endpoint_candidates, list(range(65)), list(range(65)))
        self.assertRaises(ValueError, endpoint_portfolio, [0], [0], seconds=-1)


if __name__ == "__main__":
    unittest.main()
