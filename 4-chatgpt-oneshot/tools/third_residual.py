import argparse
from collections import Counter
from itertools import combinations
import json
from pathlib import Path
import subprocess
from tempfile import TemporaryDirectory
from time import monotonic

from tools.residual_bounds import (
    PackedDatabase, active_state, full_distances, mandatory_charges,
    move_word, neighbors, projected_bound, ranked_state, residual_bound,
)


def fast_side_options(side):
    lengths = [1] * len(side)
    by_rank = {}
    for position in range(len(side) - 1, -1, -1):
        value = side[position]
        lengths[position] = 1 + max(
            (lengths[later] for later in range(position + 1, len(side))
             if side[later] < value), default=0)
        by_rank[value] = lengths[position]
    options = [(-1, 0)]
    maximum = 0
    for value in sorted(by_rank):
        if by_rank[value] > maximum:
            maximum = by_rank[value]
            options.append((value, maximum))
    return options


def fast_residual_bound(A, D, B, target):
    A, D, B = active_state(*ranked_state(A, D, B, target))
    baseline = len(A) + 2 * len(D) + len(B)
    states = {}
    for left, left_count in fast_side_options(A):
        for right, right_count in fast_side_options(B):
            key = tuple(sorted((left, right)))
            states[key] = max(states.get(key, -1), left_count + right_count)
    for value in D:
        for (left, right), count in list(states.items()):
            if value > left:
                key = tuple(sorted((value, right)))
                states[key] = max(states.get(key, -1), count + 1)
            if value > right:
                key = (left, value)
                states[key] = max(states.get(key, -1), count + 1)
    return baseline + 2 * (len(A) + len(D) + len(B) - max(states.values()))


def partition_bound(state, patterns, database):
    n = sum(map(len, state))
    best = 0
    for pattern in patterns:
        selected = set(pattern)
        rank = {card: index for index, card in enumerate(pattern)}
        projected = tuple(tuple(rank[card] for card in stack if card in selected)
                          for stack in state)
        complement = tuple(card for card in range(n) if card not in selected)
        omitted = tuple(tuple(card for card in stack if card not in selected)
                        for stack in state)
        best = max(best, database[projected]
                   + fast_residual_bound(*omitted, complement))
    return best


def selected_partition_bound(state, patterns, database):
    charges = mandatory_charges(*state)
    def score(pattern):
        rank = {card: index for index, card in enumerate(pattern)}
        projected = tuple(tuple(rank[card] for card in stack if card in rank)
                          for stack in state)
        return database[projected] + sum(charges.values()) - sum(
            charges[card] for card in pattern)
    pattern = max(patterns, key=score)
    return partition_bound(state, (pattern,), database)


def validate(through, binary):
    records = []
    databases = {n: full_distances(n) for n in range(1, through + 1)}
    with TemporaryDirectory(prefix="third-residual-") as directory:
        for n in range(1, through + 1):
            started = monotonic()
            distances = databases[n]
            patterns = tuple(sorted(combinations(range(n), min(4, n)),
                                    key=lambda pattern: sum(1 << card for card in pattern)))
            path = Path(directory) / f"bound-{n}.bin"
            subprocess.run([str(binary), "dump", str(n), str(path)],
                           check=True, timeout=30)
            cpp = PackedDatabase(path, n)
            partition_path = Path(directory) / f"partition-{n}.bin"
            subprocess.run([str(binary), "partition", str(n), str(partition_path)],
                           check=True, timeout=30)
            cpp_partition = PackedDatabase(partition_path, n)
            selected_path = Path(directory) / f"selected-{n}.bin"
            subprocess.run([str(binary), "selected", str(n), str(selected_path)],
                           check=True, timeout=30)
            cpp_selected = PackedDatabase(selected_path, n)
            values = {}
            partition_values = {}
            selected_values = {}
            totals = Counter()
            for state, distance in distances.items():
                reference = residual_bound(*state, tuple(range(n)))
                value = fast_residual_bound(*state, tuple(range(n)))
                partition = partition_bound(state, patterns, databases[min(4, n)])
                selected = selected_partition_bound(state, patterns,
                                                    databases[min(4, n)])
                pdb = projected_bound(state, patterns, databases[min(4, n)],
                                      mandatory_charges(*state))
                if value != reference or value != cpp[state] or value > distance:
                    raise AssertionError((n, state, distance, reference, value, cpp[state]))
                if partition > distance:
                    raise AssertionError((n, state, distance, partition))
                if cpp_partition[state] != max(value, pdb, partition):
                    raise AssertionError((n, state, value, pdb, partition,
                                          cpp_partition[state]))
                if cpp_selected[state] != max(value, pdb, selected):
                    raise AssertionError((n, state, value, pdb, selected,
                                          cpp_selected[state]))
                values[state] = value
                partition_values[state] = max(value, pdb, partition)
                selected_values[state] = max(value, pdb, selected)
                totals.update(states=1, joint=value, partition=partition,
                              old_hybrid=max(value, pdb),
                              new_hybrid=max(value, pdb, partition),
                              selected_hybrid=max(value, pdb, selected),
                              selected_gain=int(selected > max(value, pdb)),
                              partition_gain=int(partition > max(value, pdb)))
            edges = 0
            selected_violations = 0
            for state, value in values.items():
                for following in neighbors(state):
                    edges += 1
                    if value > values[following] + 1:
                        raise AssertionError((n, state, following, value, values[following]))
                    if partition_values[state] > partition_values[following] + 1:
                        raise AssertionError((n, state, following,
                                              partition_values[state],
                                              partition_values[following]))
                    if selected_values[state] > selected_values[following] + 1:
                        selected_violations += 1
            record = {
                "n": n, "states": len(distances), "directed_edges": edges,
                "formula_mismatches": 0, "cpp_mismatches": 0,
                "admissibility_violations": 0, "consistency_violations": 0,
                "partition_admissibility_violations": 0,
                "partition_consistency_violations": 0,
                "partition_cpp_mismatches": 0,
                "selected_cpp_mismatches": 0,
                "selected_consistency_violating_edges": selected_violations,
                "partition_improves_old_hybrid_states": totals["partition_gain"],
                "selected_improves_old_hybrid_states": totals["selected_gain"],
                "means": {key: totals[key] / len(distances)
                          for key in ("joint", "partition", "old_hybrid", "new_hybrid",
                                      "selected_hybrid")},
                "seconds": monotonic() - started,
            }
            records.append(record)
            print(json.dumps(record), flush=True)
    return records


def samples(pdb_path):
    database = PackedDatabase(pdb_path, 8)
    previous = json.loads(Path("results/residual-campaign-samples.json").read_text())
    records = []
    for prior in previous:
        started = monotonic()
        patterns = prior["patterns"]
        counts = Counter()
        gains = Counter()
        rows = []
        for row in prior["rows"]:
            state = tuple(tuple(stack) for stack in row["state"])
            partition = partition_bound(state, patterns, database)
            selected = selected_partition_bound(state, patterns, database)
            old_hybrid = max(row["joint"], row["pdb"])
            gain = max(0, partition - old_hybrid)
            upper = row["verified_remaining_upper"]
            if upper is not None and partition > upper:
                raise AssertionError((state, partition, upper))
            counts[row["kind"]] += 1
            gains[row["kind"]] += gain
            rows.append({"kind": row["kind"], "depth": row["depth"],
                         "partition": partition, "old_hybrid": old_hybrid,
                         "gain": gain, "selected_partition": selected,
                         "selected_gain": max(0, selected - old_hybrid)})
        record = {
            "n": prior["n"], "patterns": len(patterns), "states": len(rows),
            "partition_improves_old_hybrid_states": sum(row["gain"] > 0 for row in rows),
            "maximum_gain": max(row["gain"] for row in rows),
            "selected_improves_old_hybrid_states": sum(row["selected_gain"] > 0 for row in rows),
            "selected_mean_gain": {
                kind: sum(row["selected_gain"] for row in rows if row["kind"] == kind) / counts[kind]
                for kind in counts},
            "mean_gain": {kind: gains[kind] / counts[kind] for kind in counts},
            "seconds": monotonic() - started, "rows": rows,
        }
        records.append(record)
        print(json.dumps({key: value for key, value in record.items() if key != "rows"}),
              flush=True)
    return records


def pilot(binary, baseline, seconds, n, modes):
    source = json.loads(Path("results/campaign-exact-extended.json").read_text())
    prior = next(row for row in source if row["n"] == n)
    cases = [(f"optimized-mode-{mode}", binary, mode) for mode in modes]
    if baseline is not None:
        cases.insert(0, ("previous-mode-1", baseline, 1))
    runs = []
    for label, executable, mode in cases:
        process = subprocess.run(
            [str(executable.resolve()), ",".join(map(str, prior["target"])),
             str(prior["initial_upper"]), str(prior["structural_bound"]),
             str(seconds), "32", "1000000", str(mode)],
            capture_output=True, text=True, check=True, timeout=seconds + 10)
        result = json.loads(process.stdout)
        word = (result["word"] if result["search_found_plan"]
                else prior["verified_upper_word"])
        state = ranked_state((), tuple(range(n)), (), prior["target"])
        for move in word:
            state = move_word(state, move)
        if state != ((), tuple(range(n)), ()) or len(word) != result["upper"]:
            raise AssertionError("pilot witness does not replay")
        result["label"] = label
        result["time_limit_seconds"] = seconds
        result["verified_upper_word"] = word
        runs.append(result)
        print(json.dumps({key: value for key, value in result.items()
                          if key not in ("word", "verified_upper_word")}), flush=True)
    return {"n": n, "target": prior["target"], "runs": runs}


def regression(binary, baseline):
    records = []
    for n in range(1, 7):
        target = tuple(reversed(range(n)))
        for mode in (0, 1):
            runs = []
            for executable in (baseline, binary):
                process = subprocess.run(
                    [str(executable.resolve()), ",".join(map(str, target)),
                     str(4 * n + 2), "0", "5", "32", "1000000", str(mode)],
                    capture_output=True, text=True, check=True, timeout=10)
                runs.append(json.loads(process.stdout))
            compared = ("lower", "upper", "initial_heuristic", "nodes",
                        "transposition_hits", "tt_entries", "interrupted",
                        "search_found_plan", "exhausted_thresholds", "word")
            if any(runs[0][key] != runs[1][key] for key in compared):
                raise AssertionError((n, mode, runs))
            records.append({"n": n, "mode": mode, "target": target,
                            "matched_fields": compared, "runs": runs})
    return records


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("validate", "samples", "pilot", "regression"))
    parser.add_argument("--through", type=int, default=6)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--seconds", type=float, default=10)
    parser.add_argument("--n", type=int, choices=(14, 16, 18), default=18)
    parser.add_argument("--modes", nargs="+", type=int, choices=(0, 1, 2, 3),
                        default=(1, 2))
    parser.add_argument("--pdb", type=Path,
                        default=Path("/tmp/residual-bounds-pdb8.bin"))
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "validate":
        if not 1 <= args.through <= 6 or args.binary is None:
            parser.error("validation requires --binary and --through in 1..6")
        result = validate(args.through, args.binary.resolve())
    elif args.mode == "samples":
        result = samples(args.pdb)
    elif args.mode == "regression":
        if args.binary is None or args.baseline is None:
            parser.error("regression requires --binary and --baseline")
        result = regression(args.binary, args.baseline)
    else:
        if args.binary is None or not 0 < args.seconds <= 45:
            parser.error("pilot requires --binary and seconds in (0,45]")
        result = pilot(args.binary, args.baseline, args.seconds, args.n, args.modes)
    args.output.write_text(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
