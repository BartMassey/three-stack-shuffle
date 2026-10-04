import argparse
from collections import Counter
import itertools
import json
from pathlib import Path
import sys
from time import monotonic

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.campaign_structure import distances, graph, witness
from tools.excursion_obstructions import group_bound
from tools.structural_bound import common_suffix, instance_bound


ROOT = Path(__file__).resolve().parents[1]


def packed_feasible(edges, goal, distance, budgets, deadline,
                    max_states=300000):
    if any(value < 0 or value > 7 for value in budgets):
        raise ValueError("packed budgets must lie between zero and seven")
    if monotonic() >= deadline:
        raise TimeoutError("pair oracle deadline")
    width = 3 * len(budgets)
    remaining = sum(value << (3 * card)
                    for card, value in enumerate(budgets))
    failed = set()
    visits = 0
    path = []

    def search(state, counts, total):
        nonlocal visits
        visits += 1
        if visits % 1024 == 0 and monotonic() >= deadline:
            raise TimeoutError("pair oracle deadline")
        if state == goal:
            return True
        if distance[state] > total:
            return False
        key = (state << width) | counts
        if key in failed:
            return False
        for following, card, code in sorted(
                edges[state], key=lambda edge: distance[edge[0]]):
            unit = 1 << (3 * card)
            if counts & (7 * unit) and distance[following] < total:
                path.append(code)
                if search(following, counts - unit, total - 1):
                    return True
                path.pop()
        if len(failed) >= max_states:
            raise TimeoutError("pair oracle memo limit")
        failed.add(key)
        return False

    accepted = search(0, remaining, sum(budgets))
    return (tuple(path) if accepted else None), visits, len(failed)


def candidate_targets(n, mode, limit):
    exact = json.loads((ROOT / "results" / f"exact-n{n}.json").read_text())
    eligible = [(target, distance) for target, distance in zip(
        itertools.permutations(range(n)), exact["target_distances"])
        if common_suffix(target) == 0]
    if mode == "hard":
        eligible.sort(key=lambda row: (
            instance_bound(row[0]) - row[1], -row[1], row[0]))
    elif mode == "miss":
        rules = json.loads((ROOT / "results" /
                            "excursion-catalog-n6.json").read_text())["rules"]
        eligible = [(target, distance) for target, distance in eligible
                    if group_bound(target, rules, 2, 10)
                    ["certified_lower_bound"] < distance]
        eligible.sort(key=lambda row: (-row[1], row[0]))
    elif mode == "structured":
        eligible = [(target, distance) for target, distance in eligible
                    if sum(a < b for a, b in itertools.combinations(target, 2))
                    <= 3]
        eligible.sort(key=lambda row: (-row[1], row[0]))
    return eligible[:limit]


def run(n=7, mode="hard", limit=14, seconds=50,
        max_states=300000, output=None):
    if not 2 <= n <= 7:
        raise ValueError("this bounded campaign supports n=2 through n=7")
    started = monotonic()
    deadline = started + seconds
    targets = candidate_targets(n, mode, limit)
    result = {"n": n, "selection": mode, "requested_targets": len(targets),
              "seconds_limit": seconds, "memo_limit": max_states,
              "cases": []}
    try:
        states, ids, edges = graph(n, deadline)
        result["physical_states"] = len(states)
        for target, exact in targets:
            goal = ids[((), target, ())]
            distance = distances(edges, goal)
            if monotonic() >= deadline:
                raise TimeoutError("pair batch deadline")
            assert distance[0] == exact
            positions = {card: position for position, card in enumerate(target)}
            pairs = [(a, b) for a, b in itertools.combinations(range(n), 2)
                     if positions[a] < positions[b]]
            row = {"target": list(target), "distance": exact,
                   "structural_bound": instance_bound(target),
                   "increasing_pairs": len(pairs), "pairs": []}
            result["cases"].append(row)
            for selected in pairs:
                budgets = [2 if card in selected else 4 for card in range(n)]
                try:
                    plan, visits, memo = packed_feasible(
                        edges, goal, distance, budgets, deadline, max_states)
                except TimeoutError as error:
                    row["pairs"].append({"selected": list(selected),
                                         "status": "unknown",
                                         "halted": str(error)})
                    raise
                checked = witness(target, plan)
                if checked is not None:
                    assert all(count <= cap for count, cap in zip(
                        checked["counts"], budgets))
                row["pairs"].append({
                    "selected": list(selected), "budgets": budgets,
                    "status": "feasible" if plan is not None else "infeasible",
                    "oracle_visits": visits, "memoized_failures": memo,
                    "witness": checked})
            row["complete"] = True
            print(json.dumps({"target": target, "pairs": len(pairs),
                              "statuses": dict(Counter(
                                  pair["status"] for pair in row["pairs"]))}),
                  flush=True)
    except TimeoutError as error:
        result["halted"] = str(error)
    result["statuses"] = dict(Counter(
        pair["status"] for row in result["cases"] for pair in row["pairs"]))
    result["completed_targets"] = sum(bool(row.get("complete"))
                                      for row in result["cases"])
    result["elapsed_seconds"] = monotonic() - started
    if output is not None:
        Path(output).write_text(json.dumps(result, indent=2) + "\n")
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=7)
    parser.add_argument("--mode", choices=("hard", "miss", "structured"),
                        default="hard")
    parser.add_argument("--limit", type=int, default=14)
    parser.add_argument("--seconds", type=float, default=50)
    parser.add_argument("--max-states", type=int, default=300000)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    result = run(args.n, args.mode, args.limit, args.seconds,
                 args.max_states, args.output)
    print(json.dumps({key: value for key, value in result.items()
                      if key != "cases"}))


if __name__ == "__main__":
    main()
