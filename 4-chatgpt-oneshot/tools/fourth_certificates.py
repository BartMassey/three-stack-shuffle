import argparse
from collections import Counter
from fractions import Fraction
import json
import math
from pathlib import Path
import random
import sys
from time import monotonic

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.campaign_structure import conditional_bound
from tools.excursion_obstructions import projected_rules
from tools.structural_bound import common_suffix, instance_bound, shape
from tools.third_ensemble import ROOT, record, verify_extra_certificate


def fresh_targets(n, count, seed):
    rng = random.Random(seed)
    targets = []
    for _ in range(count):
        target = list(range(n))
        rng.shuffle(target)
        targets.append(target)
    return targets


def decreasing_cuts(target, values):
    loads, paths, cuts = [], [], []
    for position, card in enumerate(target):
        previous = max((j for j in range(position) if target[j] > card),
                       key=lambda j: loads[j], default=None)
        loads.append(values[card] + (loads[previous] if previous is not None else 0))
        paths.append((paths[previous] if previous is not None else ()) + (card,))
        if loads[-1] > 2 + 1e-8:
            cuts.append(paths[-1])
    return cuts


def rational_certificate(target, constraints, marginals, singleton_start):
    n = len(target)
    weights = [max(Fraction(0), Fraction(float(-value)).limit_denominator(10**6))
               for value in marginals]
    loads = [Fraction(0)] * (2 * n)
    for weight, (coefficients, _) in zip(weights, constraints):
        for column, value in coefficients.items():
            loads[column] += weight * value
    scale = max([Fraction(1)] + [-value for value in loads[n:]])
    weights = [weight / scale for weight in weights]
    loads = [value / scale for value in loads]
    for card in range(n):
        weights[singleton_start + card] += max(Fraction(0), 1 - loads[card])
    upper = sum(weight * bound for weight, (_, bound) in zip(weights, constraints))
    lower = 4 * n - 2 * upper
    return {"target": list(target), "verified_lower_bound": record(lower),
            "even_lower_bound": 2 * math.ceil(lower / 2),
            "dual": [{"coefficients": coefficients, "rhs": bound,
                      "weight": record(weight)}
                     for weight, (coefficients, bound) in zip(weights, constraints)
                     if weight]}


def discover(target, seconds=10):
    import numpy as np
    from scipy.optimize import linprog
    from scipy.sparse import coo_matrix

    started = monotonic()
    deadline = started + seconds
    suffix = common_suffix(target)
    active = list(target[:len(target) - suffix] if suffix else target)
    n = len(active)
    result = {"status": "unknown", "active_target": active,
              "common_suffix": suffix, "structural_bound": instance_bound(target),
              "certified_lower_bound": instance_bound(target)}
    if n < 2:
        result.update(status="identity", discovery_seconds=monotonic() - started,
                      verification_seconds=0)
        return result
    i2 = sum(shape(active)[:2])
    catalog = json.loads((ROOT / "results/excursion-catalog-n6.json").read_text())["rules"]
    catalog = [rule for rule in catalog if len(rule["target"]) == 5]

    def solve(constraints, cost, until):
        remaining = until - monotonic()
        if remaining <= 0:
            raise TimeoutError("LP discovery deadline")
        rows, columns, values = [], [], []
        for row, (coefficients, _) in enumerate(constraints):
            for column, value in coefficients.items():
                rows.append(row)
                columns.append(column)
                values.append(value)
        matrix = coo_matrix((values, (rows, columns)),
                            shape=(len(constraints), len(cost))).tocsr()
        return linprog(cost, A_ub=matrix,
                       b_ub=[bound for _, bound in constraints],
                       bounds=(0, None), method="highs",
                       options={"time_limit": max(0.001, until - monotonic())})

    try:
        occurrences = sorted(set(projected_rules(active, catalog, deadline)))
        triggers = sorted({trigger for trigger, _ in occurrences})
        constraints = [({i: 1 for i in range(n) if trigger & (1 << i)},
                        trigger.bit_count() - 1) for trigger in triggers]
        constraints.append(({i: 1 for i in range(n)}, i2))
        constraints.extend(({i: 1}, 1) for i in range(n))
        chain_deadline = deadline - min(2.5, seconds / 3)
        cuts, selected = set(), set()
        iterations = 0
        chain_status = "deadline"
        while monotonic() < chain_deadline:
            solved = solve(constraints, -np.ones(n), chain_deadline)
            iterations += 1
            if not solved.success:
                chain_status = solved.message
                break
            selected = {trigger for trigger, dual in
                        zip(triggers, solved.ineqlin.marginals[:len(triggers)])
                        if dual < -1e-8}
            added = set(decreasing_cuts(active, solved.x)) - cuts
            if not added:
                chain_status = "separated"
                break
            cuts.update(added)
            constraints.extend(({i: 1 for i in chain}, 2) for chain in sorted(added))
        constraints = []
        for trigger, complement in occurrences:
            if trigger not in selected:
                continue
            coefficients = {i: 1 for i in range(n) if trigger & (1 << i)}
            coefficients.update({n + i: -1 for i in range(n)
                                 if complement & (1 << i)})
            constraints.append((coefficients, trigger.bit_count() - 1))
        group_count = len(constraints)
        constraints.extend(({i: 1 for i in chain}, 2) for chain in sorted(cuts))
        constraints.append(({i: 1 for i in range(n)}, i2))
        singleton_start = len(constraints)
        constraints.extend(({i: 1}, 1) for i in range(n))
        result.update(group_occurrences=len(occurrences), selected_triggers=len(selected),
                      group_constraints=group_count, chain_constraints=len(cuts),
                      chain_iterations=iterations, chain_status=chain_status)
        solved = solve(constraints, np.array([-1] * n + [1] * n), deadline)
        if not solved.success:
            result.update(status="solver_stopped", solver_message=solved.message)
        else:
            certificate = rational_certificate(active, constraints,
                                               solved.ineqlin.marginals, singleton_start)
            result["discovery_seconds"] = monotonic() - started
            verifying = monotonic()
            bound = verify_extra_certificate(certificate)
            result.update(status="verified", certificate=certificate,
                          floating_lower_bound=4 * n + 2 * solved.fun,
                          verification_seconds=monotonic() - verifying,
                          certified_lower_bound=max(instance_bound(target), bound))
    except TimeoutError as error:
        result.update(status="timeout", reason=str(error))
    result.setdefault("discovery_seconds", monotonic() - started)
    result.setdefault("verification_seconds", 0)
    return result


def batch(n=52, count=20, seed=2026100404, lp_seconds=10,
          integer_seconds=3, total_seconds=420):
    import scipy

    started = monotonic()
    targets = fresh_targets(n, count, seed)
    cases = []
    for index, target in enumerate(targets):
        if monotonic() - started + lp_seconds + integer_seconds > total_seconds:
            cases.extend({"index": i, "target": p, "status": "batch_deadline_unrun"}
                         for i, p in enumerate(targets[index:], index))
            break
        lp = discover(target, lp_seconds)
        integer = conditional_bound(target, deficit=2, seconds=integer_seconds)
        bound = max(lp["certified_lower_bound"], integer["certified_lower_bound"])
        row = {"index": index, "target": target, "structural_bound": instance_bound(target),
               "lp": lp, "integer": integer, "strongest_certified_bound": bound}
        cases.append(row)
        print(json.dumps({"index": index, "B": row["structural_bound"],
                          "lp": lp["certified_lower_bound"],
                          "integer": integer["certified_lower_bound"],
                          "lp_status": lp["status"]}), flush=True)
    completed = [row for row in cases if "lp" in row]
    return {"n": n, "count": count, "seed": seed,
            "scipy_version": scipy.__version__,
            "lp_seconds": lp_seconds, "integer_seconds": integer_seconds,
            "integer_deficit": 2, "integer_improvement_cap": 6,
            "total_seconds": total_seconds, "elapsed_seconds": monotonic() - started,
            "budget_semantics": "Cooperative monotonic deadlines; solver and enumeration checks may overshoot slightly",
            "suffix_policy": "Trim common fixed bottom suffix; verify the active permutation with the existing exact verifier",
            "lp_status_counts": dict(Counter(row["lp"]["status"] for row in completed)),
            "integer_timeout_count": sum(row["integer"]["halted"] is not None for row in completed),
            "lp_gain_counts": dict(Counter(row["lp"]["certified_lower_bound"] - row["structural_bound"]
                                           for row in completed)),
            "integer_gain_counts": dict(Counter(row["integer"]["certified_lower_bound"] - row["structural_bound"]
                                                for row in completed)),
            "cases": cases}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--verify", type=Path)
    mode.add_argument("--fairness", type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(args.output)
    if args.fairness:
        artifact = json.loads(args.fairness.read_text())
        started = monotonic()
        selected = [row for row in artifact["cases"] if "lp" in row
                    and row["lp"]["certified_lower_bound"]
                    > row["integer"]["certified_lower_bound"]][:5]
        cases = []
        unrun = []
        for row in selected:
            if (artifact["elapsed_seconds"] + monotonic() - started + 10
                    > artifact["total_seconds"]):
                unrun.append(row["index"])
                continue
            integer = conditional_bound(row["target"], deficit=5, seconds=10)
            cases.append({"index": row["index"], "target": row["target"],
                          "structural_bound": row["structural_bound"],
                          "lp_bound": row["lp"]["certified_lower_bound"],
                          "original_integer_bound": row["integer"]["certified_lower_bound"],
                          "integer": integer})
        result = {"source": str(args.fairness), "integer_deficit": 5,
                  "integer_improvement_cap": 12, "integer_seconds": 10,
                  "selection": "First at most five LP winners in the frozen batch",
                  "elapsed_seconds": monotonic() - started,
                  "unrun_due_to_shared_budget": unrun, "cases": cases}
    elif args.verify:
        artifact = json.loads(args.verify.read_text())
        started = monotonic()
        verified = []
        for row in artifact["cases"]:
            certificate = row.get("lp", {}).get("certificate")
            if certificate is not None:
                suffix = common_suffix(row["target"])
                active = row["target"][:len(row["target"]) - suffix] if suffix else row["target"]
                if certificate["target"] != active:
                    raise ValueError("certificate target does not match trimmed original")
                verified.append({"index": row["index"],
                                 "bound": verify_extra_certificate(certificate)})
        result = {"verified": verified, "elapsed_seconds": monotonic() - started,
                  "original_integer_deficit": 2, "original_integer_improvement_cap": 6}
    else:
        result = batch()
    with args.output.open("x") as stream:
        stream.write(json.dumps(result, indent=2) + "\n")


if __name__ == "__main__":
    main()
