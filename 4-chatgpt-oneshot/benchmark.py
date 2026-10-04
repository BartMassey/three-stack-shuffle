import argparse
import itertools
import json
import math
from pathlib import Path
import platform
import random
import statistics
import time

from solver import ALGORITHMS
from three_stack import Machine, cancel


def benchmark(n, samples, seed, algorithms, exhaustive=False):
    initial = list(range(n))
    generator = random.Random(seed)
    targets = (list(itertools.permutations(initial)) if exhaustive else
               [generator.sample(initial, n) for _ in range(samples)])
    exact_path = Path(__file__).parent / "results" / f"exact-n{n}.json"
    exact = json.loads(exact_path.read_text()) if exact_path.exists() else None
    optimal = None
    if exact:
        optimal = []
        for target in targets:
            remaining = list(initial)
            rank = 0
            for card in target:
                index = remaining.index(card)
                rank = rank * len(remaining) + index
                remaining.pop(index)
            optimal.append(exact["target_distances"][rank])
    results = {}
    for name in algorithms:
        algorithm = ALGORITHMS[name]
        counts = []
        elapsed = 0.0
        for target in targets:
            started = time.perf_counter()
            operations = cancel(algorithm(initial, target))
            elapsed += time.perf_counter() - started
            machine = Machine(initial)
            machine.run(operations)
            machine.verify(target)
            counts.append(len(operations))
        entry = {
            "mean": statistics.mean(counts),
            "min": min(counts),
            "max": max(counts),
            "stdev": statistics.pstdev(counts),
            "mean_standard_error": statistics.pstdev(counts) / math.sqrt(len(counts)),
            "p50": sorted(counts)[(len(counts) - 1) // 2],
            "p95": sorted(counts)[math.ceil(len(counts) * .95) - 1],
            "planning_seconds": elapsed,
            "microseconds_per_plan": elapsed * 1e6 / len(targets),
            "hardest_sample": list(targets[counts.index(max(counts))]),
        }
        if exact:
            if any(count < optimum for count, optimum in zip(counts, optimal)):
                raise AssertionError("plan shorter than exact distance")
            entry["mean_optimal_gap"] = statistics.mean(
                count - optimum for count, optimum in zip(counts, optimal))
            entry["optimal_count"] = sum(
                count == optimum for count, optimum in zip(counts, optimal))
            entry["sample_optimal_mean"] = statistics.mean(optimal)
        results[name] = entry
    return {"n": n, "samples": len(targets), "seed": seed,
            "exhaustive": exhaustive, "validated": True,
            "python": platform.python_version(),
            "platform": platform.platform(),
            "timing_scope": "planning and cancellation, excluding simulation",
            "algorithms": results}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--samples", type=int, default=1000)
    parser.add_argument("--seed", type=int, default=20261003)
    parser.add_argument("--algorithms", nargs="+", choices=ALGORITHMS,
                        default=["radix", "runs_radix_flexible", "merge", "portfolio"])
    parser.add_argument("--exhaustive", action="store_true")
    parser.add_argument("--output", type=Path)
    arguments = parser.parse_args()
    if arguments.n < 0 or arguments.samples < 1:
        parser.error("n must be nonnegative and samples positive")
    if arguments.exhaustive and arguments.n > 8:
        parser.error("exhaustive benchmarks are capped at n=8")
    result = benchmark(arguments.n, arguments.samples, arguments.seed,
                       arguments.algorithms, arguments.exhaustive)
    output = json.dumps(result, indent=2) + "\n"
    if arguments.output:
        arguments.output.parent.mkdir(parents=True, exist_ok=True)
        arguments.output.write_text(output)
    print(output)


if __name__ == "__main__":
    main()
