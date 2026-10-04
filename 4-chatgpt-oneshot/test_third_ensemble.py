from collections import Counter
import copy
from fractions import Fraction
import itertools
import json
import math
import unittest

from tools.structural_bound import common_suffix, shape
from tools.third_ensemble import (
    ROOT, confidence, counterexamples, exact_lower_probability, lp_population,
    verify_extra_certificate,
)


class ThirdEnsembleTests(unittest.TestCase):
    def test_binomial_inversion_has_exact_coverage(self):
        alpha = Fraction(1, 20)
        for n in range(1, 9):
            lower = [exact_lower_probability(n, k, alpha, 1000)[0]
                     for k in range(n + 1)]
            self.assertEqual(lower, sorted(lower))
            for numerator in range(11):
                p = Fraction(numerator, 10)
                failure = sum(Fraction(math.comb(n, k)) * p**k * (1-p)**(n-k)
                              for k in range(n + 1) if lower[k] > p)
                self.assertLessEqual(failure, alpha)

    def test_active_population_matches_permutations(self):
        for n in range(1, 8):
            counts = Counter()
            for p in itertools.permutations(range(n)):
                m = n - common_suffix(p)
                i2 = sum(shape(p)[:2]) - (n - m)
                if m == 0 or 2 * i2 <= m:
                    counts[m] += 1
            result = lp_population(n, 10)
            self.assertEqual(int(result["certified_stuck_targets"]), sum(counts.values()))
            for stage in result["stages"]:
                self.assertEqual(int(stage["stuck_count"]), counts[stage["active_cards"]])

    def test_frozen_gain_certificate(self):
        result = confidence()
        self.assertEqual(result["sample_count"], 100)
        self.assertEqual(result["mean_gain_lower_confidence"]["decimal"], 5.210214)
        self.assertFalse(result["deterministic_mean_theorem_improved"])

    def test_counterexamples(self):
        result = counterexamples()
        example = result["gap_not_pattern_monotone"]
        self.assertGreater(example["smaller_distance"], example["smaller_bound"])
        self.assertEqual(example["larger_distance"], example["larger_bound"])
        strata = result["fixed_five_rank_pattern_I2_strata"]
        self.assertEqual(sum(row[0] for row in strata.values()), 720)
        self.assertEqual(sum(row[1] for row in strata.values()), 6)

    def test_sparse_rational_chain_group_certificate(self):
        certificate = json.loads((ROOT / "results/third-ensemble-extras.json").read_text())
        self.assertEqual(verify_extra_certificate(certificate), 176)
        self.assertEqual(len(certificate["dual"]), 17)
        mutations = []
        changed = copy.deepcopy(certificate)
        changed["dual"][0]["rhs"] = 0
        mutations.append(changed)
        changed = copy.deepcopy(certificate)
        changed["dual"][0]["weight"] = {"numerator": "-1", "denominator": "1"}
        mutations.append(changed)
        changed = copy.deepcopy(certificate)
        changed["dual"][8]["coefficients"] = {"0": 1, "1": 1, "2": 1}
        mutations.append(changed)
        changed = copy.deepcopy(certificate)
        changed["even_lower_bound"] = 178
        mutations.append(changed)
        changed = copy.deepcopy(certificate)
        changed["dual"][0]["rhs"] = True
        mutations.append(changed)
        changed = copy.deepcopy(certificate)
        changed["dual"][0]["coefficients"]["17"] = 1.0
        mutations.append(changed)
        changed = copy.deepcopy(certificate)
        changed["target"][0] = changed["target"][1]
        mutations.append(changed)
        for changed in mutations:
            with self.subTest(changed=changed):
                with self.assertRaises(ValueError):
                    verify_extra_certificate(changed)


if __name__ == "__main__":
    unittest.main()
