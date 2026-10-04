from collections import deque
import itertools
import random
import unittest

from exact_solver import optimal
from hybrid_merge import _join, _swap, hybrid_merge
from parked_leaf import PARKED_LEAF_MAXIMUM, parked_bounds, parked_hybrid, parked_optimal
from three_stack import Machine


def reference_distances(n):
    goal = (tuple(reversed(range(n))), (), ())
    distances = {goal: 0}
    queue = deque([goal])
    while queue:
        state = queue.popleft()
        for source, destination in ((0, 1), (1, 0), (1, 2), (2, 1)):
            if not state[source]:
                continue
            stacks = list(state)
            stacks[source] = state[source][1:]
            stacks[destination] = (state[source][0],) + state[destination]
            next_state = tuple(stacks)
            if next_state not in distances:
                distances[next_state] = distances[state] + 1
                queue.append(next_state)
    return distances


class ParkedLeafTests(unittest.TestCase):
    def test_terminal_run_alternative(self):
        for n in range(1, 9):
            for initial in itertools.permutations(range(n)):
                target = list(range(n))
                old = parked_optimal(initial, target)
                word = parked_optimal(initial, target, prefer_tail=True)
                self.assertEqual(len(word), len(old))
                old_run = next((i for i, move in enumerate(reversed(old))
                                if move != "DA"), len(old))
                new_run = next((i for i, move in enumerate(reversed(word))
                                if move != "DA"), len(word))
                self.assertGreaterEqual(new_run, old_run)
                machine = Machine(initial)
                for operation in word:
                    machine.move(operation)
                self.assertEqual(machine.stacks, {"A": target, "D": [], "B": []})

    def test_independent_bfs(self):
        for n in range(1, 6):
            distances = reference_distances(n)
            tails = {}
            for state, distance in distances.items():
                best = 0
                for source, destination in ((0, 1), (1, 0), (1, 2), (2, 1)):
                    if not state[source]:
                        continue
                    stacks = list(state)
                    stacks[source] = state[source][1:]
                    stacks[destination] = (state[source][0],) + state[destination]
                    neighbor = tuple(stacks)
                    if distances[neighbor] + 1 == distance:
                        run = tails[neighbor]
                        if (source, destination) == (1, 0) and run == distances[neighbor]:
                            run += 1
                        best = max(best, run)
                tails[state] = best
            for initial in itertools.permutations(range(n)):
                word = parked_optimal(initial, list(range(n)))
                self.assertEqual(len(word), distances[((), initial, ())])
                alternative = parked_optimal(initial, list(range(n)), prefer_tail=True)
                run = next((i for i, move in enumerate(reversed(alternative))
                            if move != "DA"), len(alternative))
                self.assertEqual(run, tails[((), initial, ())])

    def test_all_protected_leaves(self):
        for n in range(1, 9):
            target = list(range(n))
            for initial in itertools.permutations(target):
                old = optimal(initial, target)
                for side in ("A", "B"):
                    word = parked_optimal(initial, target, side)
                    parking = ["D" + side] * n
                    self.assertLessEqual(len(word), min(
                        len(_join((old, parking))),
                        len(_join((_swap(old), parking)))))
                    bases = {"A": [-10, -11], "D": [-20],
                             "B": [-30, -31, -32]}
                    machine = Machine(initial)
                    for stack in bases:
                        machine.stacks[stack] = bases[stack] + machine.stacks[stack]
                    for operation in word:
                        self.assertGreater(len(machine.stacks[operation[0]]),
                                           len(bases[operation[0]]))
                        machine.move(operation)
                    for stack in bases:
                        expected = bases[stack] + (target if stack == side else [])
                        self.assertEqual(machine.stacks[stack], expected)

    def test_planners(self):
        generator = random.Random(20261003)
        for n in (0, 1, 2, 5, 8, 9, 10, 13, 26, 52, 65):
            initial = list(range(n))
            for _ in range(5):
                target = generator.sample(initial, n)
                for window in (0, 2):
                    word = parked_hybrid(initial, target, window=window)
                    machine = Machine(initial)
                    machine.run(word)
                    machine.verify(target)
                    self.assertLessEqual(len(word), len(hybrid_merge(initial, target)))

    def test_guaranteed_bounds(self):
        self.assertEqual(parked_bounds(52)[0], 410)
        for n in range(1, 9):
            maximum = max(len(parked_optimal(initial, list(range(n))))
                          for initial in itertools.permutations(range(n)))
            self.assertEqual(maximum, PARKED_LEAF_MAXIMUM[n])
        generator = random.Random(410)
        for n in (9, 13, 22, 30, 52, 64):
            for _ in range(10):
                initial = list(range(n))
                target = generator.sample(initial, n)
                word = parked_hybrid(initial, target, retain_baseline=False,
                                     bound_splits=True)
                self.assertLessEqual(len(word), parked_bounds(n)[0])
                machine = Machine(initial)
                machine.run(word)
                machine.verify(target)

    def test_validation_and_relabeling(self):
        with self.assertRaises(ValueError):
            parked_optimal([1, 1], [1, 1])
        with self.assertRaises(ValueError):
            parked_optimal([1], [1], "D")
        with self.assertRaises(ValueError):
            parked_optimal(list(range(9)), list(range(9)))
        labels = ["blue", "green", "red"]
        machine = Machine(labels)
        word = parked_optimal(labels, labels[::-1], "B")
        machine.run(word)
        self.assertEqual(machine.stacks["B"], labels[::-1])

    def test_registered_controller(self):
        from solver import ALGORITHMS, move_bound, recommended

        self.assertEqual(move_bound(52, "parked"), 410)
        self.assertEqual(move_bound(52), 444)
        self.assertEqual(move_bound(65, "parked"), move_bound(65))
        generator = random.Random(41052)
        for n in (0, 1, 4, 8, 13, 52, 65):
            initial = list(range(n))
            target = generator.sample(initial, n)
            word = ALGORITHMS["parked"](initial, target)
            machine = Machine(initial)
            machine.run(word)
            machine.verify(target)
            self.assertLessEqual(len(word), move_bound(n, "parked"))
            self.assertLessEqual(len(word), len(recommended(initial, target)))


if __name__ == "__main__":
    unittest.main()
