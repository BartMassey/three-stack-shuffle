import argparse
from fractions import Fraction
import json
import math
from pathlib import Path
import random
import statistics
from time import monotonic

import numpy as np
from scipy.optimize import linprog

from structural_bound import common_suffix, instance_bound


ROOT = Path(__file__).resolve().parent.parent


def rank(sequence):
    remaining = sorted(sequence)
    value = 0
    for card in sequence:
        index = remaining.index(card)
        value = value * len(remaining) + index
        remaining.pop(index)
    return value


def load_tables():
    return {n: json.loads((ROOT / "results" / f"exact-n{n}.json").read_text())
            ["target_distances"] for n in range(2, 10)}


def table_distance(target, subset, tables):
    return tables[len(subset)][rank([card for card in target if card in subset])]


def certificate_bound(target, certificate, tables):
    n = len(target)
    m = n - common_suffix(target)
    loads = [0] * n
    denominator = certificate["denominator"]
    assert denominator > 0
    objective = 0
    for entry in certificate["entries"]:
        subset = entry["cards"]
        numerator = entry["numerator"]
        assert len(set(subset)) == len(subset)
        assert numerator >= 0 and all(0 <= card < n for card in subset)
        if entry["kind"] == "forced":
            assert len(subset) == 1 and subset[0] < m
            distance = 2
        else:
            assert entry["kind"] == "pattern" and 2 <= len(subset) <= 9
            distance = table_distance(target, set(subset), tables)
        assert distance == entry["distance"]
        objective += numerator * distance
        for card in subset:
            loads[card] += numerator
    assert max(loads, default=0) <= denominator
    return Fraction(objective, denominator)


def solve_target(target, generator, tables, pattern_count=1500):
    started = monotonic()
    n = len(target)
    m = n - common_suffix(target)
    size = min(9, n)
    patterns = {}
    for _ in range(pattern_count):
        subset = tuple(sorted(generator.sample(range(n), size)))
        patterns[subset] = table_distance(target, set(subset), tables)
    for round_index in range(2):
        rows = [(subset, value, "pattern") for subset, value in patterns.items()]
        rows.extend(((card,), 2, "forced") for card in range(m))
        matrix = np.zeros((len(rows), n))
        rhs = np.empty(len(rows))
        for i, (subset, value, _) in enumerate(rows):
            matrix[i, list(subset)] = -1
            rhs[i] = -value
        result = linprog(np.ones(n), A_ub=matrix, b_ub=rhs,
                         bounds=(0, None), method="highs",
                         options={"time_limit": .5})
        if not result.success:
            return {"target": target, "status": result.message,
                    "structural_bound": instance_bound(target),
                    "seconds": monotonic() - started}
        if round_index == 0:
            prices = result.x
            promising = sorted(patterns, key=lambda subset:
                                patterns[subset] - sum(prices[list(subset)]),
                                reverse=True)[:20]
            for original in promising:
                subset = original
                score = patterns[subset] - sum(prices[list(subset)])
                for _ in range(60):
                    outside = list(set(range(n)) - set(subset))
                    if not outside:
                        break
                    candidate = list(subset)
                    candidate[generator.randrange(size)] = generator.choice(outside)
                    candidate = tuple(sorted(candidate))
                    value = table_distance(target, set(candidate), tables)
                    patterns[candidate] = value
                    next_score = value - sum(prices[list(candidate)])
                    if next_score >= score:
                        subset, score = candidate, next_score
    denominator = 10 ** 9
    entries = []
    loads = [0] * n
    for (subset, distance, kind), marginal in zip(rows, result.ineqlin.marginals):
        numerator = max(0, math.floor(-float(marginal) * denominator))
        if numerator:
            entries.append({"cards": list(subset), "kind": kind,
                            "distance": distance, "numerator": numerator})
            for card in subset:
                loads[card] += numerator
    denominator = max(denominator, max(loads, default=0))
    certificate = {"denominator": denominator, "entries": entries}
    bound = certificate_bound(target, certificate, tables)
    rounded = 2 * ((bound.numerator + 2 * bound.denominator - 1)
                   // (2 * bound.denominator))
    return {"target": target, "status": "verified dual",
            "structural_bound": instance_bound(target),
            "pattern_bound": rounded, "raw_numerator": str(bound.numerator),
            "raw_denominator": str(bound.denominator),
            "lp_objective": result.fun, "pattern_count": len(patterns),
            "certificate": certificate, "seconds": monotonic() - started}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--samples", type=int, default=100)
    parser.add_argument("--seed", type=int, default=20261003)
    parser.add_argument("--patterns", type=int, default=1500)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if not 2 <= args.n <= 52 or args.samples < 1:
        parser.error("require 2<=n<=52 and positive samples")
    generator = random.Random(args.seed)
    targets = [generator.sample(range(args.n), args.n) for _ in range(args.samples)]
    patterns = random.Random(args.seed + 1)
    tables = load_tables()
    rows = []
    for target in targets:
        row = solve_target(target, patterns, tables, args.patterns)
        if args.n <= 9 and "pattern_bound" in row:
            assert row["pattern_bound"] <= tables[args.n][rank(target)]
        rows.append(row)
    successes = [row for row in rows if "pattern_bound" in row]
    summary = {"n": args.n, "samples": args.samples, "seed": args.seed,
               "verified_certificates": len(successes),
               "structural_mean": statistics.mean(row["structural_bound"] for row in rows),
               "pattern_mean": statistics.mean(row["pattern_bound"] for row in successes)
               if successes else None,
               "combined_mean": statistics.mean(max(row["structural_bound"],
                                                    row.get("pattern_bound", 0))
                                                 for row in rows),
               "improved_targets": sum(row.get("pattern_bound", 0) > row["structural_bound"]
                                       for row in rows),
               "total_seconds": sum(row["seconds"] for row in rows),
               "max_seconds": max(row["seconds"] for row in rows)}
    if args.output:
        args.output.write_text(json.dumps({"summary": summary, "targets": rows},
                                         indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
