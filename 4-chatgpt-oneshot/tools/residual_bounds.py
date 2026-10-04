import argparse
from bisect import bisect_left
from collections import Counter, deque
from itertools import combinations
import json
from math import factorial
from pathlib import Path
import random
import subprocess
from tempfile import TemporaryDirectory
from time import monotonic


def decreasing_length(sequence):
    tails = []
    for value in sequence:
        position = bisect_left(tails, -value)
        if position == len(tails):
            tails.append(-value)
        else:
            tails[position] = -value
    return len(tails)


def two_increasing_score(sequence, states):
    for value in sequence:
        following = states.copy()
        for (left, right), count in states.items():
            if value > left:
                key = (value, right)
                following[key] = max(following.get(key, 0), count + 1)
            if value > right:
                key = (left, value)
                following[key] = max(following.get(key, 0), count + 1)
        states = following
    return max(states.values())


def two_increasing_length(sequence, first=-1, second=-1):
    return two_increasing_score(sequence, {(first, second): 0})


def ranked_state(A, D, B, target):
    ranks = {card: position for position, card in enumerate(target)}
    if len(ranks) != len(target):
        raise ValueError("target cards must be distinct")
    cards = tuple(A) + tuple(D) + tuple(B)
    if len(cards) != len(target) or set(cards) != set(target):
        raise ValueError("state and target must contain the same cards once")
    return tuple(tuple(ranks[card] for card in stack) for stack in (A, D, B))


def active_state(A, D, B):
    n = len(A) + len(D) + len(B)
    suffix = 0
    while suffix < len(D) and D[-1 - suffix] == n - 1 - suffix:
        suffix += 1
    return A, D[:len(D) - suffix], B


def mandatory_charges(A, D, B):
    active_A, active_D, active_B = active_state(A, D, B)
    charges = {card: 1 for card in active_A + active_B}
    charges.update({card: 2 for card in active_D})
    charges.update({card: 0 for card in D[len(active_D):]})
    return charges


def independent_bound(A, D, B, target):
    A, D, B = active_state(*ranked_state(A, D, B, target))
    baseline = len(A) + 2 * len(D) + len(B)
    ordinary = (decreasing_length(A) + two_increasing_length(D)
                + decreasing_length(B))
    return baseline + 2 * (len(A) + len(D) + len(B) - ordinary)


def side_options(side):
    return [(-1, 0)] + [
        (threshold, decreasing_length(value for value in side if value <= threshold))
        for threshold in sorted(side)]


def residual_bound(A, D, B, target):
    A, D, B = active_state(*ranked_state(A, D, B, target))
    baseline = len(A) + 2 * len(D) + len(B)
    initial = {
        (left, right): left_count + right_count
        for left, left_count in side_options(A)
        for right, right_count in side_options(B)}
    ordinary = two_increasing_score(D, initial)
    return baseline + 2 * (len(A) + len(D) + len(B) - ordinary)


def neighbors(state):
    A, D, B = state
    if A:
        yield (A[1:], (A[0],) + D, B)
    if D:
        yield ((D[0],) + A, D[1:], B)
        yield (A, D[1:], (D[0],) + B)
    if B:
        yield (A, (B[0],) + D, B[1:])


def full_distances(n):
    goal = ((), tuple(range(n)), ())
    distances = {goal: 0}
    queue = deque([goal])
    while queue:
        state = queue.popleft()
        for following in neighbors(state):
            if following not in distances:
                distances[following] = distances[state] + 1
                queue.append(following)
    return distances


def projected_bound(state, patterns, database, charges):
    baseline = sum(charges.values())
    best = baseline
    for pattern in patterns:
        rank = {card: position for position, card in enumerate(pattern)}
        projected = tuple(tuple(rank[card] for card in stack if card in rank)
                          for stack in state)
        value = database[projected] + baseline - sum(charges[card] for card in pattern)
        best = max(best, value)
    return best


class PackedDatabase:
    def __init__(self, path, n):
        self.n = n
        self.distances = path.read_bytes()
        self.factorials = tuple(factorial(index) for index in range(n + 1))
        self.cut_count = (n + 1) * (n + 2) // 2
        if len(self.distances) != factorial(n) * self.cut_count:
            raise ValueError("packed PDB has unexpected length")

    def __getitem__(self, state):
        A, D, B = state
        remaining = (1 << self.n) - 1
        rank = 0
        for position, card in enumerate(A + D + B):
            rank += (remaining & ((1 << card) - 1)).bit_count() * self.factorials[self.n - 1 - position]
            remaining &= ~(1 << card)
        cut = len(A) * (self.n + 1) - len(A) * (len(A) - 1) // 2 + len(D)
        return self.distances[rank * self.cut_count + cut]


def move_word(state, move):
    A, D, B = state
    if move == "AD" and A:
        return (A[1:], (A[0],) + D, B)
    if move == "DA" and D:
        return ((D[0],) + A, D[1:], B)
    if move == "DB" and D:
        return (A, D[1:], (D[0],) + B)
    if move == "BD" and B:
        return (A, (B[0],) + D, B[1:])
    raise ValueError("illegal move")


def sample_campaign(pdb_path, seconds):
    database = PackedDatabase(pdb_path, 8)
    source = json.loads(Path("results/campaign-exact-extended.json").read_text())
    records = []
    rng = random.Random(20261004)
    for prior in source:
        started = monotonic()
        n = prior["n"]
        target = tuple(prior["target"])
        initial = ranked_state((), tuple(range(n)), (), target)
        charges = mandatory_charges(*initial)
        scored = []
        for pattern in combinations(range(n), 8):
            score = projected_bound(initial, [pattern], database, charges)
            mask = sum(1 << card for card in pattern)
            scored.append((-score, mask, pattern))
        scored.sort()
        patterns = [row[2] for row in scored[:32]]
        states = []
        state = initial
        word = prior["verified_upper_word"]
        states.append(("witness", 0, len(word), state))
        for depth, move in enumerate(word, 1):
            state = move_word(state, move)
            states.append(("witness", depth, len(word) - depth, state))
        if state != ((), tuple(range(n)), ()):
            raise AssertionError("campaign word does not reach goal")
        for index in range(128):
            state = initial
            previous = None
            for _ in range(1 + index % (2 * n)):
                candidates = [candidate for candidate in neighbors(state) if candidate != previous]
                previous, state = state, rng.choice(candidates)
            states.append(("random_walk", index, None, state))
        rows = []
        for kind, depth, remaining, state in states:
            charges = mandatory_charges(*state)
            then = monotonic()
            independent = independent_bound(*state, tuple(range(n)))
            independent_seconds = monotonic() - then
            then = monotonic()
            joint = residual_bound(*state, tuple(range(n)))
            joint_seconds = monotonic() - then
            then = monotonic()
            pdb = projected_bound(state, patterns, database, charges)
            pdb_seconds = monotonic() - then
            if remaining is not None and max(joint, pdb) > remaining:
                raise AssertionError((n, state, remaining, joint, pdb))
            rows.append({"kind": kind, "depth": depth,
                         "verified_remaining_upper": remaining, "state": state,
                         "mandatory": sum(charges.values()),
                         "independent": independent, "joint": joint, "pdb": pdb,
                         "independent_seconds": independent_seconds,
                         "joint_seconds": joint_seconds, "pdb_seconds": pdb_seconds})
            if monotonic() - started > seconds:
                raise TimeoutError(f"n={n} sample exceeded {seconds}s")
        summary = {}
        for kind in ("witness", "random_walk"):
            selected = [row for row in rows if row["kind"] == kind]
            summary[kind] = {
                "states": len(selected),
                "means": {name: sum(row[name] for row in selected) / len(selected)
                          for name in ("mandatory", "independent", "joint", "pdb")},
                "joint_beats_pdb_states": sum(row["joint"] > row["pdb"] for row in selected),
                "independent_beats_pdb_states": sum(row["independent"] > row["pdb"] for row in selected),
                "maximum_joint_gain_over_pdb": max(row["joint"] - row["pdb"] for row in selected),
                "mean_hybrid_gain": sum(max(0, row["joint"] - row["pdb"]) for row in selected) / len(selected),
                "mean_independent_microseconds": 1e6 * sum(row["independent_seconds"] for row in selected) / len(selected),
                "mean_joint_microseconds": 1e6 * sum(row["joint_seconds"] for row in selected) / len(selected),
                "mean_pdb_microseconds": 1e6 * sum(row["pdb_seconds"] for row in selected) / len(selected)}
        record = {"n": n, "target": target, "patterns": patterns,
                  "initial_pdb": rows[0]["pdb"], "initial_joint": rows[0]["joint"],
                  "summary": summary, "rows": rows, "seconds": monotonic() - started}
        records.append(record)
        print(json.dumps({key: value for key, value in record.items()
                          if key not in ("rows", "patterns")}), flush=True)
    return records


def verify_cpp(binary, through):
    records = []
    with TemporaryDirectory(prefix="residual-bounds-") as directory:
        for n in range(1, through + 1):
            started = monotonic()
            path = Path(directory) / f"residual-{n}.bin"
            subprocess.run([str(binary), str(path), str(n), "residual"],
                           check=True, timeout=60)
            cpp = PackedDatabase(path, n)
            distances = full_distances(n)
            goal = tuple(range(n))
            for state, distance in distances.items():
                expected = residual_bound(*state, goal)
                if cpp[state] != expected or cpp[state] > distance:
                    raise AssertionError((n, state, distance, expected, cpp[state]))
            record = {"n": n, "states": len(distances),
                      "formula_mismatches": 0, "admissibility_violations": 0,
                      "seconds": monotonic() - started}
            records.append(record)
            print(json.dumps(record), flush=True)
    return records


def search_pilot(binary, seconds, only_n=None):
    source = json.loads(Path("results/campaign-exact-extended.json").read_text())
    records = []
    for prior in source:
        if only_n is not None and prior["n"] != only_n:
            continue
        pair = {"n": prior["n"], "target": prior["target"],
                "time_limit_seconds": seconds, "runs": []}
        for enabled in ((0, 1) if only_n is None else (1,)):
            process = subprocess.run(
                [str(binary), ",".join(map(str, prior["target"])),
                 str(prior["initial_upper"]), str(prior["structural_bound"]),
                 str(seconds), "32", "1000000", str(enabled)],
                check=True, capture_output=True, text=True, timeout=seconds + 10)
            result = json.loads(process.stdout)
            witness = result["word"] if result["search_found_plan"] else prior["verified_upper_word"]
            state = ranked_state((), tuple(range(prior["n"])), (), prior["target"])
            for move in witness:
                state = move_word(state, move)
            if state != ((), tuple(range(prior["n"])), ()) or len(witness) != result["upper"]:
                raise AssertionError("pilot upper word does not replay")
            result["verified_upper_word"] = witness
            pair["runs"].append(result)
            print(json.dumps({key: value for key, value in result.items()
                              if key not in ("word", "verified_upper_word")}), flush=True)
        records.append(pair)
    return records


def validate(through, pattern_size, seconds):
    records = []
    databases = {}
    for n in range(1, through + 1):
        started = monotonic()
        distances = full_distances(n)
        databases[n] = distances
        k = min(pattern_size, n)
        patterns = list(combinations(range(n), k))
        target = tuple(range(n))
        totals = Counter()
        gain_histogram = Counter()
        pdb_gain_histogram = Counter()
        gaps = Counter()
        examples = []
        violations = 0
        for index, (state, distance) in enumerate(distances.items()):
            charges = mandatory_charges(*state)
            mandatory = sum(charges.values())
            independent = independent_bound(*state, target)
            joint = residual_bound(*state, target)
            pdb = projected_bound(state, patterns, databases[k], charges)
            if not mandatory <= independent <= joint <= distance:
                raise AssertionError((n, state, distance, mandatory, independent, joint))
            totals.update(states=1, exact_distance=distance,
                          mandatory=mandatory, independent=independent,
                          joint=joint, pdb=pdb, hybrid=max(joint, pdb),
                          joint_exact=int(joint == distance),
                          joint_beats_independent=int(joint > independent),
                          joint_beats_pdb=int(joint > pdb))
            gain_histogram[joint - independent] += 1
            pdb_gain_histogram[joint - pdb] += 1
            gaps[distance - joint] += 1
            if joint > pdb and len(examples) < 5:
                examples.append({"state": state, "distance": distance,
                                 "mandatory": mandatory, "independent": independent,
                                 "joint": joint, "pdb": pdb})
            if index % 512 == 0 and monotonic() - started > seconds:
                raise TimeoutError(f"n={n} validation exceeded {seconds}s")
        for state in distances:
            value = residual_bound(*state, target)
            if any(value > 1 + residual_bound(*following, target)
                   for following in neighbors(state)):
                violations += 1
        count = len(distances)
        record = {"n": n, "states": count, "pattern_size": k,
                  "patterns": len(patterns), "admissibility_violations": 0,
                  "consistency_violating_states": violations,
                  "means": {name: totals[name] / count for name in
                            ("exact_distance", "mandatory", "independent",
                             "joint", "pdb", "hybrid")},
                  "joint_exact_states": totals["joint_exact"],
                  "joint_beats_independent_states": totals["joint_beats_independent"],
                  "joint_beats_pdb_states": totals["joint_beats_pdb"],
                  "joint_minus_independent_histogram": dict(sorted(gain_histogram.items())),
                  "joint_minus_pdb_histogram": dict(sorted(pdb_gain_histogram.items())),
                  "joint_gap_histogram": dict(sorted(gaps.items())),
                  "examples": examples, "seconds": monotonic() - started}
        records.append(record)
        print(json.dumps(record), flush=True)
    return records


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--through", type=int, default=6)
    parser.add_argument("--pattern-size", type=int, default=4)
    parser.add_argument("--seconds", type=float, default=60)
    parser.add_argument("--pdb-bytes", type=Path)
    parser.add_argument("--cpp-bound-binary", type=Path)
    parser.add_argument("--search-binary", type=Path)
    parser.add_argument("--search-n", type=int, choices=(14, 16, 18))
    parser.add_argument("--output", type=Path,
                        default=Path("results/residual-validation.json"))
    args = parser.parse_args()
    if not 1 <= args.through <= 6:
        parser.error("--through must be between 1 and 6")
    if not 1 <= args.pattern_size <= args.through:
        parser.error("invalid --pattern-size")
    if args.cpp_bound_binary:
        records = verify_cpp(args.cpp_bound_binary, args.through)
    elif args.search_binary:
        records = search_pilot(args.search_binary, args.seconds, args.search_n)
    elif args.pdb_bytes:
        records = sample_campaign(args.pdb_bytes, args.seconds)
    else:
        records = validate(args.through, args.pattern_size, args.seconds)
    args.output.write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
