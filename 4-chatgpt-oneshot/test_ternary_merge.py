import math
import random
import unittest

from oriented_merge import oriented_merge
from ternary_merge import ternary_bounds, ternary_merge
from three_stack import Machine


class TernaryMergeTests(unittest.TestCase):
    def test_preserves_small_plans(self):
        rng = random.Random(2026100403)
        for n in (0, 1, 2, 5, 8, 9, 16, 31, 52, 64):
            initial = list(range(n))
            target = rng.sample(initial, n)
            for endpoint in "ADB":
                self.assertEqual(ternary_merge(initial, target, endpoint=endpoint),
                                 oriented_merge(initial, target, endpoint=endpoint))

    def test_recursive_protected_bases(self):
        rng = random.Random(2026100403)
        for n in (65, 66, 67, 97, 128, 257, 729):
            initial = list(range(n))
            for target in (initial, initial[::-1], rng.sample(initial, n)):
                for endpoint in "ADB":
                    word = ternary_merge(initial, target, endpoint=endpoint)
                    machine = Machine(initial)
                    for side, base in zip("ADB", (-1, -2, -3)):
                        machine.stacks[side].insert(0, base)
                    for move in word:
                        self.assertGreaterEqual(machine.stacks[move[0]][-1], 0)
                        machine.move(move)
                    for side, base in zip("ADB", (-1, -2, -3)):
                        cards = (list(reversed(target)) if endpoint == "D"
                                 else list(target)) if side == endpoint else []
                        self.assertEqual(machine.stacks[side], [base] + cards)
                    self.assertLessEqual(len(word), ternary_bounds(n)[endpoint != "D"])

    def test_bounds_and_validation(self):
        self.assertEqual(ternary_bounds(52)[0], 352)
        for n in range(1, 10001):
            self.assertLessEqual(ternary_bounds(n)[1], 2 * n * math.ceil(math.log(n, 3)) + 4 * n)
        self.assertRaises(ValueError, ternary_bounds, -1)
        self.assertRaises(ValueError, ternary_merge, [0], [0], endpoint="C")
        self.assertRaises(ValueError, ternary_merge, [0], [0], window=-1)

    def test_registered_controller(self):
        from solver import ALGORITHMS, move_bound

        initial = list(range(257))
        target = random.Random(257).sample(initial, len(initial))
        word = ALGORITHMS["ternary"](initial, target)
        machine = Machine(initial)
        machine.run(word)
        machine.verify(target)
        self.assertLessEqual(len(word), move_bound(257, "ternary"))
        self.assertEqual(move_bound(52, "ternary"), 352)


if __name__ == "__main__":
    unittest.main()
