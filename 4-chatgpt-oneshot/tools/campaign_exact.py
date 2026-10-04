import argparse
import itertools
import json
from pathlib import Path
import random
import resource
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from solver import thorough
from tools.structural_bound import instance_bound


def replay(target, word):
    stacks = {"A": [], "D": list(reversed(range(len(target)))), "B": []}
    for move in word:
        source, destination = move
        assert move in ("AD", "DA", "DB", "BD")
        assert stacks[source]
        stacks[destination].append(stacks[source].pop())
    assert stacks["A"] == stacks["B"] == []
    assert stacks["D"] == list(reversed(target))


def limits():
    resource.setrlimit(resource.RLIMIT_AS, (1024 * 1024**2, 1024 * 1024**2))


def run(target, binary, seconds, patterns, known=None, pad=False,
        use_structural=True):
    witness = thorough(list(range(len(target))), list(target))
    if pad:
        witness = ["DA", "AD"] + witness
    replay(target, witness)
    structural = instance_bound(target)
    process = subprocess.run(
        [str(binary), ",".join(map(str, target)), str(len(witness)),
         str(structural if use_structural else 0), str(seconds),
         str(patterns), "1000000"],
        capture_output=True, text=True, check=True,
        timeout=seconds + 10, preexec_fn=limits)
    result = json.loads(process.stdout)
    result["target"] = target
    result["structural_bound"] = structural
    result["used_structural_seed"] = use_structural
    result["initial_upper"] = len(witness)
    result["time_limit_seconds"] = seconds
    result["memory_limit_mib"] = 1024
    if result["search_found_plan"]:
        witness = result["word"]
    assert len(witness) == result["upper"]
    replay(target, witness)
    result["verified_upper_word"] = witness
    if known is not None:
        result["known_optimum"] = known
        assert result["lower"] <= known <= result["upper"]
        if not result["interrupted"]:
            assert result["lower"] == result["upper"] == known
    print(json.dumps({key: value for key, value in result.items()
                      if key not in ("word", "verified_upper_word")}), flush=True)
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=Path("/tmp/campaign_exact"))
    parser.add_argument("--mode", choices=("validate", "pilot", "extended"),
                        default="validate")
    parser.add_argument("--seconds", type=float, default=20)
    parser.add_argument("--patterns", type=int, default=32)
    parser.add_argument("--output", type=Path,
                        default=Path("results/campaign-exact-validation.json"))
    args = parser.parse_args()
    rng = random.Random(20261004)
    records = []
    if args.mode == "validate":
        for n in (5, 8, 9):
            exact = json.loads((ROOT / "results" / f"exact-n{n}.json").read_text())
            distances = exact["target_distances"]
            ranks = {len(distances) - 1, *rng.sample(range(len(distances)), 3)}
            for rank, target in enumerate(itertools.permutations(range(n))):
                if rank in ranks:
                    records.append(run(target, args.binary, args.seconds,
                                       args.patterns, distances[rank], pad=True))
                    args.output.write_text(json.dumps(records, indent=2) + "\n")
        for target, known in (
                ([6, 8, 4, 7, 3, 1, 5, 2, 0], 30),
                ([8, 7, 3, 2, 0, 4, 1, 6, 5], 28),
                ([8, 5, 7, 2, 3, 0, 4, 6, 1], 28),
                ([8, 2, 6, 7, 3, 1, 5, 4, 0], 28)):
            records.append(run(target, args.binary, args.seconds,
                               args.patterns, known, pad=True))
            args.output.write_text(json.dumps(records, indent=2) + "\n")
        records.append(run(list(reversed(range(12))), args.binary, 0,
                           args.patterns, 44, pad=True, use_structural=False))
        args.output.write_text(json.dumps(records, indent=2) + "\n")
    elif args.mode == "pilot":
        for n in (10, 11, 12):
            candidates = []
            for _ in range(100):
                target = list(range(n))
                rng.shuffle(target)
                witness = thorough(list(range(n)), target)
                candidates.append((len(witness) - instance_bound(target),
                                   len(witness), target))
            candidates.sort(reverse=True)
            targets = [list(reversed(range(n))), candidates[0][2], candidates[1][2]]
            for target in targets:
                records.append(run(target, args.binary, args.seconds, args.patterns))
                args.output.write_text(json.dumps(records, indent=2) + "\n")
    else:
        for n in (14, 16, 18, 20):
            target = list(range(n - 1, -1, -2)) + list(range(n - 2, -1, -2))
            records.append(run(target, args.binary, args.seconds, args.patterns))
            args.output.write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
