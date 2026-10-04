import itertools
import random
import unittest

from oriented_merge import oriented_bounds, oriented_merge
from three_stack import Machine


class OrientedMergeTests(unittest.TestCase):
    def verify(self, initial, target, endpoint="D", bases=False):
        word = oriented_merge(initial, target, endpoint=endpoint)
        machine = Machine(initial)
        if bases:
            machine.stacks["A"].insert(0, -101)
            machine.stacks["B"].insert(0, -102)
            machine.stacks["D"].insert(0, -103)
        machine.run(word)
        for side in "ADB":
            expected = list(target) if side == endpoint else []
            if endpoint == "D":
                expected.reverse()
            if bases:
                expected.insert(0, {"A": -101, "B": -102, "D": -103}[side])
            self.assertEqual(machine.stacks[side], expected)
        self.assertLessEqual(len(word), oriented_bounds(len(initial))[endpoint != "D"])

    def test_small(self):
        for n in range(7):
            for target in itertools.permutations(range(n)):
                for endpoint in "ADB":
                    self.verify(list(range(n)), target, endpoint, True)

    def test_recursive_endpoints(self):
        rng = random.Random(2026100402)
        for n in (9, 10, 13, 14, 16, 22, 26, 30, 52, 64, 65, 128, 257):
            initial = list(range(n))
            for target in (initial, initial[::-1], rng.sample(initial, n)):
                for endpoint in "ADB":
                    self.verify(initial, target, endpoint, True)

    def test_bounds_and_validation(self):
        self.assertEqual(oriented_bounds(52)[0], 352)
        self.assertEqual(oriented_bounds(52)[1], 368)
        self.assertRaises(ValueError, oriented_bounds, -1)
        self.assertRaises(ValueError, oriented_merge, [0, 0], [0, 1])
        self.assertRaises(ValueError, oriented_merge, [0], [0], endpoint="C")
        self.assertRaises(ValueError, oriented_merge, [0], [0], window=-1)

    def test_registered_controllers(self):
        from solver import ALGORITHMS, move_bound

        initial = list(range(52))
        target = random.Random(52).sample(initial, len(initial))
        for name in ("oriented", "oriented_window"):
            word = ALGORITHMS[name](initial, target)
            machine = Machine(initial)
            machine.run(word)
            machine.verify(target)
            self.assertLessEqual(len(word), move_bound(52, name))
            self.assertEqual(move_bound(52, name), 352)


if __name__ == "__main__":
    unittest.main()
