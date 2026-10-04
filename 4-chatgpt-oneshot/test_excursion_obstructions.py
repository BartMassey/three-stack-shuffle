import itertools
import json
from pathlib import Path
import unittest

from tools.excursion_obstructions import (
    brute_group_bound, catalog, conditional_costs, disjoint_groups,
    group_bound,
)


class ExcursionObstructionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog = catalog(5, 10)
        cls.rules = cls.catalog["rules"]

    def test_minimal_five_card_catalog(self):
        self.assertNotIn("halted", self.catalog)
        self.assertEqual(len(self.rules), 16)
        pairs = {(tuple(rule["target"]), tuple(rule["twice_cards"]))
                 for rule in self.rules if len(rule["twice_cards"]) == 2}
        self.assertEqual(pairs, {
            ((2, 4, 3, 1, 0), (0, 1)),
            ((4, 1, 3, 2, 0), (0, 4)),
            ((4, 3, 0, 2, 1), (3, 4)),
        })
        costs = conditional_costs([rule for rule in self.rules
                                   if len(rule["twice_cards"]) == 2], 10)
        self.assertNotIn("halted", costs)
        self.assertEqual([row["minimum_with_selected_twice"]
                          for row in costs["cases"]], [18, 18, 18])

    def test_group_charges_are_disjoint(self):
        value, packed = disjoint_groups({0b001100, 0b011000}, 0, 0, 0)
        self.assertEqual(value, 1)
        self.assertEqual(len(packed), 1)
        value, packed = disjoint_groups({0b001100, 0b110000}, 0, 0, 0)
        self.assertEqual(value, 2)
        self.assertEqual(len(packed), 2)
        value, packed = disjoint_groups({0b001100, 0b011000}, 0, 0, 0b000100)
        self.assertEqual(value, 1)
        self.assertEqual(packed, [])
        value, packed = disjoint_groups({0b001100, 0b110000},
                                       0b000100, 0b000100, 0)
        self.assertEqual(value, 3)
        self.assertEqual(packed, [0b110000])

    def test_all_six_card_targets(self):
        root = Path(__file__).resolve().parent
        exact = json.loads((root / "results" / "exact-n6.json").read_text())
        for target, distance in zip(itertools.permutations(range(6)),
                                    exact["target_distances"]):
            lower = group_bound(target, self.rules, 2, 10)
            brute = brute_group_bound(target, self.rules)
            self.assertIsNone(lower["halted"], target)
            self.assertLessEqual(lower["certified_lower_bound"], brute, target)
            self.assertLessEqual(brute, distance, target)
            self.assertEqual(lower["certified_lower_bound"], distance, target)
        self.assertEqual(group_bound((2, 5, 0, 3, 4, 1),
                                     self.rules, 2, 10)["certified_lower_bound"],
                         16)
        halted = group_bound((2, 5, 0, 3, 4, 1), self.rules, 2, -1)
        self.assertEqual(halted["certified_lower_bound"], 14)
        self.assertEqual(halted["subset_checks"], [])
        self.assertEqual(halted["halted"], "group preprocessing deadline")


if __name__ == "__main__":
    unittest.main()
