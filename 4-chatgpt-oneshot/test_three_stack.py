import itertools
import json
from pathlib import Path
import random
import unittest
from unittest.mock import patch

from exact_solver import optimal
from hybrid_merge import hybrid_merge, hybrid_dp
from shuffle import shuffled_target
from solver import ALGORITHMS, LEAF_PARKING_COST, move_bound
from three_stack import Machine, cancel, invert, reversal, selection


class MachineTests(unittest.TestCase):
    def test_rejects_illegal_moves(self):
        machine = Machine([1, 2])
        for operation in ("AB", "BA", "DD", "AD", "BD"):
            with self.assertRaises(ValueError):
                machine.move(operation)
        self.assertEqual(machine.operations, 0)

    def test_top_convention(self):
        machine = Machine([1, 2, 3])
        machine.run(["DA", "DB", "AD", "BD"])
        machine.verify([2, 1, 3])

    def test_distinct_cards(self):
        with self.assertRaises(ValueError):
            Machine([1, 1])

    def test_reversal(self):
        for n in range(1, 101):
            initial = list(range(n))
            operations = reversal(n)
            self.assertEqual(len(operations), 4 * (n - 1))
            machine = Machine(initial)
            machine.run(operations)
            machine.verify(initial[::-1])

    def test_selection_and_inverse(self):
        for n in range(7):
            initial = list(range(n))
            for target in itertools.permutations(initial):
                operations = selection(initial, target)
                machine = Machine(initial)
                machine.run(operations)
                machine.verify(target)
                machine = Machine(target)
                machine.run(invert(operations))
                machine.verify(initial)

    def test_cancellation(self):
        self.assertEqual(cancel(["DA", "DB", "BD", "AD"]), [])
        self.assertEqual(cancel(["DA", "DB", "AD", "BD"]),
                         ["DA", "DB", "AD", "BD"])

    def test_exact_tables(self):
        for n in range(1, 9):
            path = Path(__file__).parent / "results" / f"exact-n{n}.json"
            distances = json.loads(path.read_text())["target_distances"]
            initial = list(range(n))
            parking_cost = 0
            for target, distance in zip(itertools.permutations(initial), distances):
                operations = optimal(initial, target)
                self.assertEqual(len(operations), distance)
                tail = 0
                for operation in reversed(operations):
                    if operation != operations[-1]:
                        break
                    tail += 1
                parking_cost = max(parking_cost, len(operations) - 2 * tail)
                machine = Machine(initial)
                machine.run(operations)
                machine.verify(target)
            self.assertEqual(parking_cost, LEAF_PARKING_COST[n])

    def test_all_algorithms_small(self):
        for n in range(6):
            initial = list(range(n))
            for target in itertools.permutations(initial):
                for name, algorithm in ALGORITHMS.items():
                    operations = algorithm(initial, target)
                    machine = Machine(initial)
                    machine.run(operations)
                    machine.verify(target)
                    self.assertEqual(len(operations) % 2, 0, name)

    def test_arbitrary_initial_and_labels(self):
        generator = random.Random(42)
        for n in (8, 9, 16, 52):
            for _ in range(5):
                initial = generator.sample(range(100, 100 + n), n)
                target = generator.sample(initial, n)
                for algorithm in ALGORITHMS.values():
                    operations = algorithm(initial, target)
                    machine = Machine(initial)
                    machine.run(operations)
                    machine.verify(target)

    def test_invalid_targets(self):
        for algorithm in ALGORITHMS.values():
            for initial, target in (([1, 2], [1, 1]), ([1, 1], [1, 1]),
                                    ([1, 2], [1]), ([1, 2], [2, 3])):
                with self.assertRaises(ValueError):
                    algorithm(initial, target)

    def test_exact_plan_with_untouched_buffers(self):
        initial = list(range(8))
        target = [3, 7, 1, 5, 0, 6, 2, 4]
        machine = Machine(initial)
        machine.stacks["A"] = [-1]
        machine.stacks["B"] = [-2]
        machine.stacks["D"].insert(0, -3)
        machine.run(optimal(initial, target))
        self.assertEqual(machine.stacks["A"], [-1])
        self.assertEqual(machine.stacks["B"], [-2])
        self.assertEqual(machine.stacks["D"], [-3] + target[::-1])

    def test_recursive_hybrid(self):
        initial = list(range(6))
        for target in itertools.permutations(initial):
            for algorithm in (hybrid_merge, hybrid_dp):
                operations = algorithm(initial, target, leaf_limit=2)
                machine = Machine(initial)
                machine.run(operations)
                machine.verify(target)

    def test_proven_balanced_bound(self):
        self.assertEqual(move_bound(52), 444)
        generator = random.Random(104)
        for n in (9, 16, 31, 52, 64, 65, 1000):
            initial = list(range(n))
            target = generator.sample(initial, n)
            operations = ALGORITHMS["recommended"](initial, target)
            self.assertLessEqual(len(operations), move_bound(n))
            machine = Machine(initial)
            machine.run(operations)
            machine.verify(target)

    def test_uniform_target_choice_bijection(self):
        n = 6
        seen = set()
        for choices in itertools.product(*(range(k) for k in range(n, 1, -1))):
            with patch("shuffle.random.SystemRandom") as generator:
                generator.return_value.randrange.side_effect = choices
                target = shuffled_target(list(range(n)))
            seen.add(tuple(target))
        self.assertEqual(len(seen), 720)

    def test_disjoint_adjacent_swaps(self):
        initial = list(range(52))
        target = [index ^ 1 for index in initial]
        for name in ("fast", "recommended", "thorough"):
            operations = ALGORITHMS[name](initial, target)
            self.assertEqual(len(operations), 104)
            machine = Machine(initial)
            machine.run(operations)
            machine.verify(target)


if __name__ == "__main__":
    unittest.main()
