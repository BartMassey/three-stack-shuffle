import itertools
import json
from pathlib import Path
import unittest
from time import monotonic

from tools.campaign_structure import distances, feasible, graph, witness
from tools.third_pairs import packed_feasible
from three_stack import Machine


class ThirdPairTests(unittest.TestCase):
    def test_saved_seven_card_witnesses(self):
        root = Path(__file__).resolve().parent
        distinct = set()
        targets = set()
        total = 0
        for mode in ("hard", "structured", "miss"):
            result = json.loads((root / "results" /
                                 f"third-pairs-{mode}-n7.json").read_text())
            self.assertNotIn("halted", result)
            for row in result["cases"]:
                target = tuple(row["target"])
                targets.add(target)
                expected = {pair for pair in itertools.combinations(range(7), 2)
                            if target.index(pair[0]) < target.index(pair[1])}
                self.assertEqual({tuple(pair["selected"])
                                  for pair in row["pairs"]}, expected)
                for pair in row["pairs"]:
                    self.assertEqual(pair["status"], "feasible")
                    machine = Machine(range(7))
                    counts = [0] * 7
                    for move in pair["witness"]["moves"]:
                        counts[machine.stacks[move[0]][-1]] += 1
                        machine.move(move)
                    machine.verify(target)
                    self.assertEqual(counts, pair["witness"]["counts"])
                    self.assertEqual(machine.operations,
                                     pair["witness"]["cost"])
                    self.assertTrue(all(count <= cap for count, cap in zip(
                        counts, pair["budgets"])))
                    self.assertTrue(all(counts[card] == 2
                                        for card in pair["selected"]))
                    distinct.add((target, tuple(pair["selected"])))
                    total += 1
        self.assertEqual((len(targets), len(distinct), total), (94, 371, 409))

    def test_independent_five_card_pair_catalog(self):
        deadline = monotonic() + 45
        states, ids, edges = graph(5, deadline)
        failures = set()
        checks = 0
        for target in itertools.permutations(range(5)):
            goal = ids[((), target, ())]
            distance = distances(edges, goal)
            for selected in itertools.combinations(range(5), 2):
                budgets = [2 if card in selected else 4 for card in range(5)]
                plan, _, _ = packed_feasible(
                    edges, goal, distance, budgets, deadline)
                reference, _ = feasible(
                    edges, goal, distance, budgets, sum(budgets), deadline)
                self.assertEqual(plan is None, reference is None)
                if plan is None:
                    failures.add((target, selected))
                else:
                    verified = witness(target, plan)
                    self.assertTrue(all(count <= cap for count, cap in zip(
                        verified["counts"], budgets)))
                checks += 1
        self.assertEqual(checks, 1200)
        self.assertEqual(failures, {
            ((2, 4, 3, 1, 0), (0, 1)),
            ((4, 1, 3, 2, 0), (0, 4)),
            ((4, 3, 0, 2, 1), (3, 4)),
        })

    def test_interruption_is_not_infeasibility(self):
        deadline = monotonic() + 10
        _, ids, edges = graph(5, deadline)
        goal = ids[((), (2, 4, 3, 1, 0), ())]
        distance = distances(edges, goal)
        with self.assertRaises(TimeoutError):
            packed_feasible(edges, goal, distance, [2, 2, 4, 4, 4],
                            monotonic() - 1)
        with self.assertRaises(TimeoutError):
            packed_feasible(edges, goal, distance, [2, 2, 4, 4, 4],
                            deadline, max_states=0)


if __name__ == "__main__":
    unittest.main()
