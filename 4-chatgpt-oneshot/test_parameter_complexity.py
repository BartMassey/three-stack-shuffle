import itertools
import json
from pathlib import Path
from time import monotonic
import unittest

from tools.campaign_complexity import active_size
from tools.parameter_complexity import bounded_plan
from three_stack import Machine


class ParameterComplexityTests(unittest.TestCase):
    def test_small_budget_decisions(self):
        root = Path(__file__).resolve().parent
        for n in range(1, 6):
            exact = json.loads((root / "results" / f"exact-n{n}.json").read_text())
            for target, distance in zip(itertools.permutations(range(n)),
                                        exact["target_distances"]):
                for excess in range(3):
                    word, _ = bounded_plan(target, excess, monotonic() + 5)
                    self.assertEqual(word is not None,
                                     distance <= 2 * active_size(target) + 2 * excess)
                    if word is not None:
                        machine = Machine(list(range(n)))
                        machine.run(word)
                        machine.verify(target)
                        self.assertEqual(len(word), distance)

    def test_temporary_return_before_initial_drain(self):
        target = (2, 3, 1, 4, 0)
        word, _ = bounded_plan(target, 1, monotonic() + 5)
        self.assertEqual(len(word), 12)
        machine = Machine(list(range(5)))
        machine.run(word)
        machine.verify(target)


if __name__ == "__main__":
    unittest.main()
