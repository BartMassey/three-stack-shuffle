import itertools
import json
from pathlib import Path
import random
import unittest

from tools.campaign_structure import conditional_bound, conditional_obstructions


def reference_obstructions(target):
    positions = [target.index(card) for card in range(len(target))]
    rules = {(2, 1, 4, 3, 0): (0, 1), (2, 4, 1, 3, 0): (0, 2),
             (4, 1, 0, 3, 2): (4, 1), (4, 2, 0, 3, 1): (4, 2)}
    result = {}
    for cards in itertools.combinations(range(len(target)), 5):
        pattern = tuple(sorted(range(5), key=lambda i: positions[cards[i]]))
        if pattern not in rules:
            continue
        index, weight = rules[pattern]
        exceptional = cards[index]
        quartet = sum(1 << card for card in cards if card != exceptional)
        six, eight = result.get(quartet, (0, 0))
        six |= 1 << exceptional
        if weight == 2:
            eight |= 1 << exceptional
        result[quartet] = six, eight
    return result


class CampaignBoundTests(unittest.TestCase):
    def test_quartet_preprocessing(self):
        for n in range(1, 8):
            for target in itertools.permutations(range(n)):
                self.assertEqual(conditional_obstructions(target),
                                 reference_obstructions(target))
        target = random.Random(178).sample(range(52), 52)
        self.assertEqual(conditional_obstructions(target),
                         reference_obstructions(target))

    def test_small_conditional_bounds(self):
        root = Path(__file__).resolve().parent
        for n in range(1, 7):
            exact = json.loads((root / "results" / f"exact-n{n}.json").read_text())
            for target, distance in zip(itertools.permutations(range(n)),
                                        exact["target_distances"]):
                result = conditional_bound(target, deficit=2, seconds=5)
                self.assertIsNone(result["halted"])
                self.assertLessEqual(result["certified_lower_bound"], distance)
                self.assertGreaterEqual(result["certified_lower_bound"],
                                        result["structural_bound"])
                if n <= 5:
                    self.assertEqual(result["certified_lower_bound"], distance)

    def test_suffix_and_invalid_input(self):
        self.assertEqual(conditional_bound([0, 1, 2])["certified_lower_bound"], 0)
        a = conditional_bound([2, 1, 4, 3, 0])
        b = conditional_bound([2, 1, 4, 3, 0, 5, 6])
        self.assertEqual(a["certified_lower_bound"], b["certified_lower_bound"])
        with self.assertRaises(ValueError):
            conditional_bound([0, 0])


if __name__ == "__main__":
    unittest.main()
