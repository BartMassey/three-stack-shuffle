import copy
import importlib.util
import itertools
import json
import unittest

from tools.fourth_certificates import discover, fresh_targets, rational_certificate
from tools.third_ensemble import ROOT, verify_extra_certificate


class FourthCertificateTests(unittest.TestCase):
    def test_frozen_seventeen_rows(self):
        certificate = json.loads((ROOT / "results/third-ensemble-extras.json").read_text())
        self.assertEqual(len(certificate["dual"]), 17)
        self.assertEqual(verify_extra_certificate(certificate), 176)
        corrupt = copy.deepcopy(certificate)
        corrupt["even_lower_bound"] += 2
        with self.assertRaises(ValueError):
            verify_extra_certificate(corrupt)

    @unittest.skipUnless(importlib.util.find_spec("scipy"), "optional SciPy discovery")
    def test_tiny_exact_and_suffix(self):
        exact = json.loads((ROOT / "results/exact-n3.json").read_text())["target_distances"]
        for target, distance in zip(itertools.permutations(range(3)), exact):
            with self.subTest(target=target):
                result = discover(list(target), 2)
                self.assertLessEqual(result["certified_lower_bound"], distance)
                self.assertIn(result["status"], ("verified", "identity"))
                if "certificate" in result:
                    self.assertEqual(verify_extra_certificate(result["certificate"]),
                                     result["certified_lower_bound"])

    def test_rational_repair_and_seed(self):
        constraints = [({0: 1, 1: 1}, 2), ({0: 1}, 1), ({1: 1}, 1)]
        certificate = rational_certificate([1, 0], constraints,
                                           [-0.4999999, -0.5000002, -0.4999998], 1)
        self.assertLessEqual(verify_extra_certificate(certificate), 4)
        self.assertEqual(fresh_targets(52, 20, 2026100404),
                         fresh_targets(52, 20, 2026100404))
        self.assertEqual(len({tuple(p) for p in fresh_targets(52, 20, 2026100404)}), 20)

    @unittest.skipUnless(importlib.util.find_spec("scipy"), "optional SciPy discovery")
    def test_five_card_group_patterns(self):
        exact = json.loads((ROOT / "results/exact-n5.json").read_text())["target_distances"]
        distances = dict(zip(itertools.permutations(range(5)), exact))
        for target in ((2, 1, 4, 3, 0), (4, 3, 2, 1, 0), (2, 1, 4, 0, 3)):
            result = discover(list(target), 2)
            self.assertEqual(result["status"], "verified")
            self.assertLessEqual(verify_extra_certificate(result["certificate"]),
                                 distances[target])


if __name__ == "__main__":
    unittest.main()
