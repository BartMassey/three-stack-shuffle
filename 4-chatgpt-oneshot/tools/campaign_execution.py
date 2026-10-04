import argparse
from collections import Counter
import itertools
import json
from pathlib import Path
import random
import resource
import statistics
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from exact_solver import optimal
from hybrid_merge import _join, _swap, hybrid_merge
from parked_leaf import parked_bounds, parked_hybrid, parked_optimal, parked_recommended
from solver import recommended
from three_stack import Machine


def quantile(values, fraction):
    return sorted(values)[max(0, int(len(values) * fraction + 0.999999) - 1)]


def leaves():
    rows = []
    for n in range(1, 9):
        gains = Counter()
        starts = Counter()
        ends = Counter()
        distances = Counter()
        for initial in itertools.permutations(range(n)):
            target = list(range(n))
            old = optimal(initial, target)
            parking = ["DA"] * n
            old_length = min(len(_join((old, parking))),
                             len(_join((_swap(old), parking))))
            word = parked_optimal(initial, target)
            gains[old_length - len(word)] += 1
            distances[len(word)] += 1
            start_run = next((i for i, move in enumerate(word)
                              if move != word[0]), len(word))
            end_run = next((i for i, move in enumerate(reversed(word))
                            if move != word[-1]), len(word))
            starts[word[0] + ":" + str(start_run)] += 1
            ends[word[-1] + ":" + str(end_run)] += 1
        rows.append({"n": n, "count": sum(distances.values()),
                     "maximum": max(distances), "distance_histogram": dict(distances),
                     "gain_histogram": dict(gains), "initial_runs": dict(starts),
                     "final_runs": dict(ends)})
    return {"leaves": rows, "bound_52": parked_bounds(52)[0],
            "bounds": [{"n": n, "central": parked_bounds(n)[0],
                        "parked": parked_bounds(n)[1], "split": parked_bounds(n)[2]}
                       for n in range(1, 101)]}


def benchmark(count, seed, mode):
    initial = list(range(52))
    rng = random.Random(seed)
    methods = {"recommended": recommended,
               "parked_recommended": parked_recommended}
    if mode == "development":
        methods.update({"balanced": hybrid_merge,
                        "parked_balanced": parked_hybrid,
                        "parked_bound": lambda a, b: parked_hybrid(a, b, bound_splits=True),
                        "parked_window2": lambda a, b: parked_recommended(a, b, window=2)})
    rows = []
    for index in range(count):
        target = rng.sample(initial, len(initial))
        row = {"target": target, "measurements": {}}
        for name, algorithm in methods.items():
            started = time.perf_counter()
            word = algorithm(initial, target)
            elapsed = time.perf_counter() - started
            machine = Machine(initial)
            machine.run(word)
            machine.verify(target)
            row["measurements"][name] = {"moves": len(word), "seconds": elapsed}
        rows.append(row)
        if (index + 1) % 25 == 0:
            print(f"{mode}: {index + 1}/{count}", file=sys.stderr, flush=True)
    summary = {}
    for name in methods:
        moves = [row["measurements"][name]["moves"] for row in rows]
        timings = [row["measurements"][name]["seconds"] for row in rows]
        improvements = [row["measurements"]["recommended"]["moves"] - value
                        for row, value in zip(rows, moves)]
        error = (1.96 * statistics.stdev(improvements) / count ** 0.5
                 if count > 1 else 0)
        summary[name] = {"mean_moves": statistics.mean(moves),
                         "p95_moves": quantile(moves, .95), "max_moves": max(moves),
                         "first_case_seconds": timings[0],
                         "warm_median_seconds": statistics.median(timings[1:] or timings),
                         "p95_seconds": quantile(timings, .95), "max_seconds": max(timings),
                         "mean_improvement": statistics.mean(improvements),
                         "improvement_ci95_normal": [statistics.mean(improvements) - error,
                                                     statistics.mean(improvements) + error],
                         "wins": sum(value > 0 for value in improvements),
                         "ties": sum(value == 0 for value in improvements),
                         "losses": sum(value < 0 for value in improvements)}
    process_hwm = None
    status_path = Path("/proc/self/status")
    if status_path.exists():
        process_hwm = next((int(line.split()[1])
                            for line in status_path.read_text().splitlines()
                            if line.startswith("VmHWM:")), None)
    return {"mode": mode, "n": 52, "seed": seed, "count": count,
            "timing_scope": "Planning including first-use loading; excludes replay; shared-process tables",
            "max_rss_kib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
            "process_vm_hwm_kib": process_hwm,
            "summary": summary, "targets": rows}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("leaves", "development", "holdout"))
    parser.add_argument("--count", type=int, default=100)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = (leaves() if args.mode == "leaves" else
              benchmark(args.count, 20261003 if args.mode == "development" else 2026100401,
                        args.mode))
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    if "summary" in result:
        print(json.dumps(result["summary"], indent=2))


if __name__ == "__main__":
    main()
