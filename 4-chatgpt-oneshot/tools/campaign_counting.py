import argparse
from collections import Counter, deque
from fractions import Fraction
import itertools
import json
import math
from pathlib import Path
from time import monotonic

from counting_bound import summarize
from structural_bound import instance_bound, partitions, tableau_count


MOVES = ((0, 1), (1, 0), (1, 2), (2, 1))
FORBIDDEN = ((0, 1), (1, 0), (2, 3), (3, 2),
             (2, 1, 3, 0), (1, 2, 0, 3, 1, 2, 0, 3))


def automaton(forbidden=FORBIDDEN):
    prefixes = sorted({()} | {word[:k] for word in forbidden
                             for k in range(1, len(word))})
    index = {word: i for i, word in enumerate(prefixes)}
    transitions = []
    for prefix in prefixes:
        row = []
        for move in range(4):
            word = prefix + (move,)
            if any(word[-len(factor):] == factor for factor in forbidden):
                row.append(-1)
            else:
                suffix = max((p for p in prefixes if not p or
                              word[-len(p):] == p), key=len)
                row.append(index[suffix])
        transitions.append(row)
    return transitions, index[()]


def canonical_counts(n, maximum_length, component_reflection=True):
    transitions, start = automaton()
    states = {(0, 0, start): 1}
    counts = [1]
    for length in range(maximum_length):
        following = {}
        for (a, b, suffix), count in states.items():
            candidates = []
            if a:
                candidates.append((a - 1, b, 0))
            if a + b < n:
                candidates.append((a + 1, b, 1))
                if not (a == b == 0 and (component_reflection or length == 0)):
                    candidates.append((a, b + 1, 2))
            if b:
                candidates.append((a, b - 1, 3))
            for na, nb, move in candidates:
                ns = transitions[suffix][move]
                if ns >= 0:
                    key = na, nb, ns
                    following[key] = following.get(key, 0) + count
        states = following
        counts.append(sum(count for (a, b, suffix), count in states.items()
                          if a == b == 0))
    return counts


def canonical_summary(n, counts):
    return summarize(n, [counts[0]] + [2 * value for value in counts[1:]])


def structural_distribution(n, max_seconds=60):
    started = monotonic()
    previous = Counter({0: 1})
    bounds = Counter({0: 1})
    stages = []
    for m in range(1, n + 1):
        current = Counter()
        factorial = math.factorial(m)
        for shape in partitions(m):
            weight = tableau_count(shape, factorial) ** 2
            current[sum(shape[:2])] += weight
        assert sum(current.values()) == factorial
        active_count = 0
        for i2, count in current.items():
            active = count - previous[i2 - 1]
            assert active >= 0
            bounds[4 * m - 2 * i2] += active
            active_count += active
        assert active_count == factorial - math.factorial(m - 1)
        stages.append({"m": m, "active_targets": str(active_count)})
        previous = current
        if monotonic() - started > max_seconds:
            raise TimeoutError("distribution calculation exceeded budget")
    assert sum(bounds.values()) == math.factorial(n)
    return bounds, stages


def fraction_record(value):
    return {"numerator": str(value.numerator),
            "denominator": str(value.denominator), "decimal": float(value)}


def combined_summary(n, distribution, cumulative_counts):
    targets = math.factorial(n)
    structural = 0
    deficit = 0
    rows = []
    for length in range(0, max(distribution) + 1, 2):
        structural += distribution[length]
        word_bound = cumulative_counts.get(length, targets)
        cover = min(targets, structural, word_bound)
        deficit += targets - cover
        rows.append({"length": length, "structural_cumulative": str(structural),
                     "word_cumulative": str(word_bound),
                     "combined_cumulative": str(cover)})
    mean = Fraction(2 * deficit, targets)
    structural_mean = Fraction(sum(k * v for k, v in distribution.items()), targets)
    return {"mean_lower_bound": fraction_record(mean),
            "structural_mean": fraction_record(structural_mean),
            "improvement": fraction_record(mean - structural_mean),
            "cumulative_bounds": rows}


def validate(through=6):
    root = Path(__file__).resolve().parent.parent
    transitions, start = automaton()
    checks = []
    for n in range(1, through + 1):
        exact = json.loads((root / "results" / f"exact-n{n}.json").read_text())
        distance = dict(zip(itertools.permutations(range(n)), exact["target_distances"]))
        distribution, _ = structural_distribution(n)
        expected = Counter(instance_bound(p) for p in distance)
        assert +distribution == expected
        initial = ((), tuple(range(n)), ())
        queue = deque([(initial, start)])
        seen = {(initial, start): 0}
        reached = {tuple(range(n)): 0}
        limit = max(distance.values())
        while queue:
            state, suffix = queue.popleft()
            length = seen[state, suffix]
            if length >= limit:
                continue
            for move, (source, destination) in enumerate(MOVES):
                ns = transitions[suffix][move]
                if ns < 0 or not state[source]:
                    continue
                if move == 2 and not state[0] and not state[2]:
                    continue
                following = list(state)
                following[source] = state[source][1:]
                following[destination] = (state[source][0],) + state[destination]
                following = tuple(following)
                key = following, ns
                if key in seen:
                    continue
                seen[key] = length + 1
                queue.append(key)
                if not following[0] and not following[2]:
                    reached.setdefault(following[1], length + 1)
        assert reached == distance, (n, len(reached), len(distance))
        checks.append({"n": n, "target_count": len(distance),
                       "product_states": len(seen), "shortest_coverage": True})
    return checks


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--check-through", type=int, default=6)
    parser.add_argument("--max-seconds", type=float, default=60)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if not 1 <= args.n <= 52 or not 0 <= args.check_through <= 6:
        parser.error("require 1<=n<=52 and 0<=check-through<=6")
    started = monotonic()
    checks = validate(args.check_through)
    distribution, stages = structural_distribution(args.n, args.max_seconds)
    counts = canonical_counts(args.n, max(distribution))
    summary = canonical_summary(args.n, counts)
    combined = combined_summary(args.n, distribution,
                                {k: int(v) for k, v in
                                 summary["target_cumulative_upper_bounds"]})
    result = {"n": args.n, "forbidden_words": FORBIDDEN,
              "move_order": ["AD", "DA", "DB", "BD"],
              "component_reflection": True, "validation": checks,
              "distribution_stages": stages,
              "structural_distribution": {str(k): str(v) for k, v in
                                          sorted(distribution.items()) if v},
              "canonical_counting": summary,
              "canonical_closed_counts": [str(x) for x in counts],
              "combined": combined, "elapsed_seconds": monotonic() - started}
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"n": args.n, "validation": checks,
                      "canonical_mean": summary["mean_lower_bound"],
                      "canonical_max": summary["maximum_lower_bound"],
                      "combined_mean": combined["mean_lower_bound"],
                      "improvement": combined["improvement"],
                      "elapsed_seconds": result["elapsed_seconds"]}, indent=2))


if __name__ == "__main__":
    main()
