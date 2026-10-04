import argparse
from collections import deque
import itertools
import json
from pathlib import Path
import statistics
from time import monotonic

from structural_bound import instance_bound


def color_distances(colors, deadline):
    initial = ((), tuple(colors), ())
    distances = {initial: 0}
    queue = deque([initial])
    processed = 0
    while queue:
        state = queue.popleft()
        distance = distances[state] + 1
        for source, destination in ((0, 1), (1, 0), (1, 2), (2, 1)):
            if not state[source]:
                continue
            following = list(state)
            following[source] = state[source][1:]
            following[destination] = (state[source][0],) + state[destination]
            following = tuple(following)
            if following not in distances:
                distances[following] = distance
                queue.append(following)
        processed += 1
        if processed % 1000 == 0 and monotonic() > deadline:
            raise TimeoutError("color BFS exceeded time budget")
    endpoints = {state[1]: distance for state, distance in distances.items()
                 if not state[0] and not state[2]}
    return endpoints, len(distances)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=8)
    parser.add_argument("--seconds", type=float, default=50)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if not 2 <= args.n <= 9:
        parser.error("exhaustive color comparison requires 2<=n<=9")
    started = monotonic()
    n = args.n
    schemes = [tuple(i % k for i in range(n)) for k in (2, 3)]
    schemes += [tuple(min(k - 1, i * k // n) for i in range(n)) for k in (2, 3)]
    schemes += [tuple((i // 2) % k for i in range(n)) for k in (2, 3)]
    tables = []
    summaries = []
    for colors in dict.fromkeys(schemes):
        table, states = color_distances(colors, started + args.seconds)
        tables.append((colors, table))
        summaries.append({"colors": colors, "states": states,
                          "endpoints": len(table), "maximum": max(table.values())})
    root = Path(__file__).resolve().parent.parent
    exact = json.loads((root / "results" / f"exact-n{n}.json").read_text())["target_distances"]
    structural = []
    bounds = []
    improved = []
    for rank, target in enumerate(itertools.permutations(range(n))):
        bound = max(table[tuple(colors[card] for card in target)]
                    for colors, table in tables)
        base = instance_bound(target)
        assert bound <= exact[rank]
        structural.append(base)
        bounds.append(bound)
        if bound > base:
            improved.append({"target": target, "color_bound": bound,
                             "structural_bound": base, "exact": exact[rank]})
    result = {"n": n, "schemes": summaries, "targets": len(bounds),
              "color_mean": statistics.mean(bounds),
              "structural_mean": statistics.mean(structural),
              "combined_mean": statistics.mean(max(a, b) for a, b in zip(bounds, structural)),
              "exact_mean": statistics.mean(exact), "improved_count": len(improved),
              "improved_examples": improved[:10], "elapsed_seconds": monotonic() - started}
    if args.output:
        args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
