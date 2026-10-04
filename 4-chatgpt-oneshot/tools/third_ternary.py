import argparse
import hashlib
import json
from pathlib import Path
import random
import statistics
import sys
from time import perf_counter

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from oriented_merge import oriented_bounds, oriented_merge
from ternary_merge import ternary_bounds, ternary_merge
from three_stack import Machine, invert


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--sizes", type=int, nargs="+", default=(52, 128, 512, 4096))
    parser.add_argument("--count", type=int, default=3)
    parser.add_argument("--seed", type=int, default=2026100403)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    records = []
    for n in args.sizes:
        rng = random.Random(args.seed + n)
        initial = list(range(n))
        rows = []
        for _ in range(args.count):
            target = rng.sample(initial, n)
            row = {"target": target, "measurements": {}}
            for name, algorithm, bound in (("oriented", oriented_merge, oriented_bounds(n)[0]),
                                            ("ternary", ternary_merge, ternary_bounds(n)[0])):
                start = perf_counter()
                forward = algorithm(initial, target)
                reverse = invert(algorithm(target, initial))
                word = min((forward, reverse), key=len)
                seconds = perf_counter() - start
                machine = Machine(initial)
                machine.run(word)
                machine.verify(target)
                assert len(word) <= bound
                row["measurements"][name] = {
                    "moves": len(word), "seconds": seconds, "bound": bound,
                    "word_sha256": hashlib.sha256(" ".join(word).encode()).hexdigest()}
            rows.append(row)
        summary = {
            name: {"mean_moves": statistics.mean(row["measurements"][name]["moves"] for row in rows),
                   "max_moves": max(row["measurements"][name]["moves"] for row in rows),
                   "median_seconds": statistics.median(row["measurements"][name]["seconds"] for row in rows)}
            for name in ("oriented", "ternary")}
        records.append({"n": n, "summary": summary, "targets": rows})
        print(json.dumps({"n": n, "summary": summary}), flush=True)
    result = {"seed": args.seed, "count_per_size": args.count,
              "sampling": "Random(seed+n).sample(range(n), n)", "records": records}
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
