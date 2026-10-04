import argparse
import json
from pathlib import Path
import random
import statistics
import sys
from time import perf_counter

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from oriented_merge import oriented_bounds, oriented_merge
from parked_leaf import parked_recommended
from three_stack import Machine, invert


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path,
                        default=Path("results/campaign-execution-development.json"))
    parser.add_argument("--count", type=int, default=20)
    parser.add_argument("--window", type=int, default=0)
    parser.add_argument("--fresh-seed", type=int)
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.fresh_seed is None:
        source = json.loads(args.input.read_text())
    else:
        rng = random.Random(args.fresh_seed)
        source = {"targets": [{"target": rng.sample(range(args.n), args.n)}
                              for _ in range(args.count)]}
    rows = []
    for item in source["targets"][:args.count]:
        target = item["target"]
        initial = list(range(len(target)))
        times = {}
        start = perf_counter()
        baseline = parked_recommended(initial, target)
        times["parked"] = perf_counter() - start
        start = perf_counter()
        forward = oriented_merge(initial, target, window=args.window)
        inverse = invert(oriented_merge(target, initial, window=args.window))
        times["oriented"] = perf_counter() - start
        words = {"parked": baseline, "oriented": min((forward, inverse), key=len)}
        words["portfolio"] = min(words.values(), key=len)
        for word in words.values():
            machine = Machine(initial)
            machine.run(word)
            machine.verify(target)
        assert len(words["oriented"]) <= oriented_bounds(len(target))[0]
        rows.append({"target": target,
                     "moves": {name: len(word) for name, word in words.items()},
                     "seconds": times, "word": words["portfolio"]})
    summary = {}
    for name in ("parked", "oriented", "portfolio"):
        moves = [row["moves"][name] for row in rows]
        summary[name] = {"mean": statistics.mean(moves), "maximum": max(moves),
                         "wins_vs_parked": sum(row["moves"][name] < row["moves"]["parked"]
                                               for row in rows)}
        if name != "portfolio":
            summary[name]["median_seconds"] = statistics.median(
                row["seconds"][name] for row in rows)
    result = {"input": str(args.input) if args.fresh_seed is None else None,
              "count": len(rows), "window": args.window,
              "fresh_seed": args.fresh_seed,
              "bound_52": oriented_bounds(52), "summary": summary, "targets": rows}
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
