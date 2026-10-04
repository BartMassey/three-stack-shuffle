import argparse
import json
from pathlib import Path
import random
import statistics
import sys
from time import perf_counter

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from endpoint_portfolio import endpoint_candidates, endpoint_portfolio
from hybrid_merge import _join, _swap
from oriented_merge import _central_merge
from solver import ALGORITHMS
from three_stack import Machine


def verify_guarded(initial, target, word, endpoint):
    machine = Machine(initial)
    guards = {"A": -101, "B": -102, "D": -103}
    for side, guard in guards.items():
        machine.stacks[side].insert(0, guard)
    for operation in word:
        if machine.stacks[operation[0]][-1] in guards.values():
            raise AssertionError("protected base was moved")
        machine.move(operation)
    for side in "ADB":
        expected = list(target) if side == endpoint else []
        if endpoint == "D":
            expected.reverse()
        if machine.stacks[side] != [guards[side]] + expected:
            raise AssertionError("incorrect guarded endpoint")


def witness():
    initial = [5, 2, 4, 3, 8, 6, 7, 1, 0]
    target = list(range(9))
    left = endpoint_candidates(initial[:4], sorted(initial[:4]), 4, "A")
    right = endpoint_candidates(initial[4:], sorted(initial[4:]), 4, "A")
    merge = _central_merge(sorted(initial[:4]), sorted(initial[4:]), False)
    verify_guarded(initial[:4], sorted(initial[:4]), left[0], "A")
    cases = []
    for second in right[:2]:
        reflected = _swap(second)
        raw = left[0] + reflected + merge
        parent = _join((left[0], reflected, merge))
        verify_guarded(initial[4:], sorted(initial[4:]), second, "A")
        verify_guarded(initial[4:], sorted(initial[4:]), reflected, "B")
        verify_guarded(initial, target, raw, "D")
        verify_guarded(initial, target, parent, "D")
        cases.append({"right_parked_word": second, "reflected_right_word": reflected,
                      "right_moves": len(second), "raw_parent_word": raw,
                      "parent_word": parent, "unreduced_moves": len(raw),
                      "parent_moves": len(parent), "cancelled_pairs": (len(raw) - len(parent)) // 2})
    return {"initial": initial, "target": target, "split": 4,
            "left_parked_word": left[0], "merge_word": merge, "cases": cases,
            "all_guarded_replays_passed": True}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--witness", action="store_true")
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(args.output)
    if args.witness:
        args.output.write_text(json.dumps(witness(), indent=2) + "\n")
        return
    rng = random.Random(2026100405)
    targets = [rng.sample(range(52), 52) for _ in range(20)]
    started = perf_counter()
    rows = []
    for index, target in enumerate(targets):
        initial = list(range(52))
        before = perf_counter()
        baseline = ALGORITHMS["oriented_window"](initial, target)
        times = {"oriented_window": perf_counter() - before}
        words = {"oriented_window": baseline}
        details = {}
        for width in (2, 4):
            name = f"width{width}"
            remaining = max(0, 280 - (perf_counter() - started))
            metadata = {}
            words[name] = endpoint_portfolio(initial, target, width=width,
                                              seconds=min(6.0, remaining), details=metadata)
            times[name] = metadata["seconds"]
            details[name] = metadata
            assert len(words[name]) <= len(baseline) <= 352
        for word in words.values():
            machine = Machine(initial)
            machine.run(word)
            machine.verify(target)
        rows.append({"target": target, "moves": {name: len(word) for name, word in words.items()},
                     "seconds": times, "details": details, "words": words})
        print(f"{index + 1}/20: " + str(rows[-1]["moves"]), flush=True)
    summary = {}
    for name in ("oriented_window", "width2", "width4"):
        costs = [row["moves"][name] for row in rows]
        elapsed = [row["seconds"][name] for row in rows]
        gains = [row["moves"]["oriented_window"] - row["moves"][name] for row in rows]
        summary[name] = {"mean_moves": statistics.mean(costs), "max_moves": max(costs),
                         "mean_gain": statistics.mean(gains), "wins": sum(gain > 0 for gain in gains),
                         "median_seconds": statistics.median(elapsed), "total_seconds": sum(elapsed),
                         "timeouts": sum(row["details"].get(name, {}).get("timed_out", False) for row in rows)}
    result = {"seed": 2026100405, "n": 52, "count": 20, "case_seconds": 6.0,
              "search_total_seconds": 280, "elapsed_seconds": perf_counter() - started,
              "selection": "shortest representative per first/last-six signature, then shortest width representatives",
              "summary": summary, "targets": rows}
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
