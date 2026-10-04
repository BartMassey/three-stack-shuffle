import argparse
import json
from pathlib import Path
import random
import statistics
import sys
from time import perf_counter

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from endpoint_portfolio import endpoint_portfolio
from solver import ALGORITHMS
from three_stack import Machine


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(args.output)
    rng = random.Random(2026100406)
    targets = [rng.sample(range(52), 52) for _ in range(10)]
    initial = list(range(52))
    started = perf_counter()
    rows = []
    for index, target in enumerate(targets):
        before = perf_counter()
        words = {"oriented_window": ALGORITHMS["oriented_window"](initial, target)}
        times = {"oriented_window": perf_counter() - before}
        details = {}
        for policy in ("shortest", "boundary"):
            name = f"width4_{policy}"
            metadata = {}
            remaining = max(0, 150 - (perf_counter() - started))
            allowance = min(6.0, remaining)
            words[name] = endpoint_portfolio(initial, target, width=4,
                                              seconds=allowance, details=metadata,
                                              policy=policy)
            metadata["allowance_seconds"] = allowance
            times[name] = metadata["seconds"]
            details[name] = metadata
            assert len(words[name]) <= len(words["oriented_window"]) <= 352
        for word in words.values():
            machine = Machine(initial)
            machine.run(word)
            machine.verify(target)
        rows.append({"target": target, "moves": {name: len(word) for name, word in words.items()},
                     "seconds": times, "details": details, "words": words})
        print(f"{index + 1}/10: {rows[-1]['moves']}", flush=True)
    summary = {}
    for name in ("oriented_window", "width4_shortest", "width4_boundary"):
        costs = [row["moves"][name] for row in rows]
        gains = [row["moves"]["oriented_window"] - row["moves"][name] for row in rows]
        summary[name] = {"mean_moves": statistics.mean(costs), "max_moves": max(costs),
                         "mean_gain": statistics.mean(gains), "wins": sum(gain > 0 for gain in gains),
                         "median_seconds": statistics.median(row["seconds"][name] for row in rows),
                         "total_seconds": sum(row["seconds"][name] for row in rows),
                         "timeouts": sum(row["details"].get(name, {}).get("timed_out", False) for row in rows)}
    differences = [row["moves"]["width4_shortest"] - row["moves"]["width4_boundary"] for row in rows]
    result = {"seed": 2026100406, "n": 52, "count": 10,
              "initial": initial, "case_seconds": 6.0, "search_total_seconds": 150,
              "elapsed_seconds": perf_counter() - started,
              "selection": "keep shortest signature representative first, then rank remaining by length minus twice maximum initial/final same-move run; ties by length/word",
              "all_replays_passed": True, "summary": summary,
              "boundary_vs_shortest": {"wins": sum(d > 0 for d in differences),
                                       "ties": sum(d == 0 for d in differences),
                                       "losses": sum(d < 0 for d in differences),
                                       "mean_gain": statistics.mean(differences)},
              "targets": rows}
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({"summary": summary, "boundary_vs_shortest": result["boundary_vs_shortest"]}, indent=2))


if __name__ == "__main__":
    main()
