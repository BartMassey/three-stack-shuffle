import argparse
from collections import Counter
from fractions import Fraction
import itertools
import json
import math
from pathlib import Path
import sys
from time import monotonic

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.structural_bound import instance_bound, partitions, shape, tableau_count


ROOT = Path(__file__).resolve().parents[1]


def record(value):
    return {"numerator": str(value.numerator),
            "denominator": str(value.denominator), "decimal": float(value)}


def binomial_tail_numerator(n, k, numerator, denominator):
    return sum(math.comb(n, j) * numerator ** j
               * (denominator - numerator) ** (n - j)
               for j in range(k, n + 1))


def exact_lower_probability(n, k, alpha, denominator=10**6):
    if k == 0:
        return Fraction(0), Fraction(0)
    low, high = 0, denominator
    while low < high:
        mid = (low + high + 1) // 2
        tail = binomial_tail_numerator(n, k, mid, denominator)
        if tail * alpha.denominator <= alpha.numerator * denominator ** n:
            low = mid
        else:
            high = mid - 1
    tail = Fraction(binomial_tail_numerator(n, k, low, denominator),
                    denominator ** n)
    assert tail <= alpha
    return Fraction(low, denominator), tail


def confidence(alpha=Fraction(1, 100)):
    source = ROOT / "results/campaign-structure-52-paired.json"
    sample = json.loads(source.read_text())
    baseline = json.loads((ROOT / "results/structural-lower-bound-52.json").read_text())
    mean_b = Fraction(int(baseline["mean_lower_bound_numerator"]),
                      int(baseline["mean_lower_bound_denominator"]))
    gains = []
    for row in sample["cases"]:
        assert row["structural_bound"] == instance_bound(row["target"])
        gain = row["certified_lower_bound"] - row["structural_bound"]
        assert gain in (0, 2, 4, 6)
        gains.append(gain)
    thresholds = []
    lower_gain = Fraction(0)
    for threshold in (2, 4, 6):
        successes = sum(gain >= threshold for gain in gains)
        probability, tail = exact_lower_probability(
            len(gains), successes, alpha / 3)
        lower_gain += 2 * probability
        thresholds.append({"gain_at_least": threshold,
                           "successes": successes,
                           "probability_lower": record(probability),
                           "binomial_tail_at_lower": record(tail)})
    mean_gain = Fraction(sum(gains), len(gains))
    radius = 6 * math.sqrt(math.log(1 / float(alpha)) / (2 * len(gains)))
    return {"source": str(source.relative_to(ROOT)),
            "sampling_model_required": "Independent ideal uniform permutations; stored seeded PRNG output is not a proof of this premise",
            "deterministic_mean_theorem_improved": False,
            "sample_count": len(gains), "gain_counts": dict(Counter(gains)),
            "mean_gain": record(mean_gain), "failure_probability": record(alpha),
            "thresholds": thresholds, "mean_gain_lower_confidence": record(lower_gain),
            "optimal_mean_lower_confidence": record(mean_b + lower_gain),
            "hoeffding_optimal_mean_lower_confidence": float(mean_b + mean_gain) - radius}


def lp_population(n, seconds):
    started = monotonic()
    previous = Counter({0: 1})
    stuck = 1
    remaining_gain_ceiling = 0
    stages = []
    for m in range(1, n + 1):
        current = Counter()
        factorial = math.factorial(m)
        for partition in partitions(m):
            current[sum(partition[:2])] += tableau_count(partition, factorial) ** 2
        assert sum(current.values()) == factorial
        active_stuck = sum(count - previous[i2 - 1]
                           for i2, count in current.items() if 2 * i2 <= m)
        assert active_stuck >= 0
        stuck += active_stuck
        remaining_gain_ceiling += sum(
            (2 * i2 - 4) * (count - previous[i2 - 1])
            for i2, count in current.items() if 2 * i2 > m)
        stages.append({"active_cards": m, "stuck_count": str(active_stuck)})
        previous = current
        if monotonic() - started > seconds:
            raise TimeoutError("partition population deadline")
    return {"n": n, "total_targets": str(math.factorial(n)),
            "certified_stuck_targets": str(stuck),
            "certified_stuck_fraction": record(Fraction(stuck, math.factorial(n))),
            "mean_improvement_ceiling": record(Fraction(
                remaining_gain_ceiling, math.factorial(n))),
            "condition": "identity, or active I2 <= m/2",
            "stages": stages, "elapsed_seconds": monotonic() - started}


def counterexamples():
    q = (2, 1, 4, 3, 0)
    exact5 = json.loads((ROOT / "results/exact-n5.json").read_text())["target_distances"]
    exact6 = json.loads((ROOT / "results/exact-n6.json").read_text())["target_distances"]
    by_shape = {}
    same_shape = None
    for p, d in zip(itertools.permutations(range(5)), exact5):
        key = shape(p)
        gap = d - instance_bound(p)
        if key in by_shape and by_shape[key][2] != gap:
            same_shape = {"shape": key, "first": by_shape[key],
                          "second": (p, d, gap)}
            break
        by_shape[key] = (p, d, gap)
    disappeared = None
    strata = {}
    for p, d in zip(itertools.permutations(range(6)), exact6):
        i2 = sum(shape(p)[:2])
        event = tuple(card for card in p if card < 5) == q
        row = strata.setdefault(i2, [0, 0])
        row[0] += 1
        row[1] += event
        if disappeared is None and d == instance_bound(p):
            for omit in range(6):
                projected = tuple(card - (card > omit) for card in p if card != omit)
                if projected == q:
                    disappeared = {"smaller_target": q, "smaller_distance": 14,
                                   "smaller_bound": 12, "larger_target": p,
                                   "larger_distance": d, "larger_bound": instance_bound(p),
                                   "deleted_card": omit}
                    break
    return {"gap_not_pattern_monotone": disappeared,
            "shape_does_not_determine_gap": same_shape,
            "fixed_five_rank_pattern_I2_strata": strata,
            "unconditional_fixed_pattern_probability": record(Fraction(1, 120)),
            "expected_two_chain_candidate_counts": {
                k: record(Fraction(math.comb(52, k) * math.comb(2 * k, k),
                                   (k + 1) * math.factorial(k)))
                for k in (16, 18, 20, 22, 24, 26, 28, 30)},
            "effective_word_growth_required_at_274": math.exp(math.lgamma(53) / 274)}


def chain_pilot(seconds):
    import numpy as np
    from scipy.optimize import linprog
    from scipy.sparse import coo_matrix
    from tools.excursion_obstructions import projected_rules

    started = monotonic()
    source = json.loads((ROOT / "results/campaign-structure-52-paired.json").read_text())
    target = source["cases"][0]["target"]
    n = len(target)
    i2 = sum(shape(target)[:2])
    rules = json.loads((ROOT / "results/excursion-catalog-n6.json").read_text())["rules"]
    rules = [rule for rule in rules if len(rule["target"]) == 5]
    triggers = {trigger for trigger, _ in projected_rules(
        target, rules, deadline=started + seconds)}
    constraints = [(tuple(i for i in range(n) if mask & (1 << i)),
                    mask.bit_count() - 1) for mask in triggers]
    constraints.append((tuple(range(n)), i2))
    cuts = set()
    iterations = 0
    while monotonic() - started < seconds:
        rows, columns = [], []
        for row, (cards, _) in enumerate(constraints):
            rows.extend([row] * len(cards))
            columns.extend(cards)
        matrix = coo_matrix((np.ones(len(rows)), (rows, columns)),
                            shape=(len(constraints), n)).tocsr()
        result = linprog(-np.ones(n), A_ub=matrix,
                         b_ub=[bound for _, bound in constraints],
                         bounds=(0, 1), method="highs",
                         options={"time_limit": max(0.01, seconds - (monotonic() - started))})
        iterations += 1
        if not result.success:
            return {"status": "solver_stopped", "message": result.message}
        weights, paths = [], []
        added = 0
        for position, card in enumerate(target):
            predecessors = [j for j in range(position) if target[j] > card]
            predecessor = max(predecessors, key=lambda j: weights[j], default=None)
            weights.append(result.x[card] + (weights[predecessor] if predecessor is not None else 0))
            paths.append((paths[predecessor] if predecessor is not None else ()) + (card,))
            if weights[-1] > 2 + 1e-8 and paths[-1] not in cuts:
                cuts.add(paths[-1])
                constraints.append((paths[-1], 2))
                added += 1
        if not added:
            fractions = [Fraction(float(value)).limit_denominator(10**6) for value in result.x]
            loads = []
            for position, card in enumerate(target):
                loads.append(fractions[card] + max(
                    (loads[j] for j in range(position) if target[j] > card), default=0))
            verified = all(0 <= value <= 1 for value in fractions)
            verified &= all(sum(fractions[i] for i in cards) <= bound
                            for cards, bound in constraints)
            verified &= max(loads) <= 2
            verified &= sum(fractions) == i2
            return {"target": target, "I2": i2,
                    "zero_excess_triggers": len(triggers), "chain_cuts": len(cuts),
                    "iterations": iterations, "floating_objective": -result.fun,
                    "chain_cut_cards": list(cuts),
                    "positive_trigger_duals": [list(cards) for (cards, _), dual in
                        zip(constraints[:len(triggers)], result.ineqlin.marginals)
                        if dual < -1e-8],
                    "rational_size_I_assignment_verified": bool(verified),
                    "assignment": [record(value) for value in fractions],
                    "elapsed_seconds": monotonic() - started}
    return {"status": "deadline", "iterations": iterations, "chain_cuts": len(cuts)}


def extra_pilot(seconds):
    import numpy as np
    from scipy.optimize import linprog
    from scipy.sparse import coo_matrix
    from tools.excursion_obstructions import projected_rules

    started = monotonic()
    pilot = json.loads((ROOT / "results/third-ensemble-chains.json").read_text())
    target = pilot["target"]
    n = len(target)
    selected = {sum(1 << card for card in cards) for cards in pilot["positive_trigger_duals"]}
    catalog = json.loads((ROOT / "results/excursion-catalog-n6.json").read_text())["rules"]
    catalog = [rule for rule in catalog if len(rule["target"]) == 5]
    occurrences = set((trigger, complement) for trigger, complement in projected_rules(
        target, catalog, deadline=started + seconds) if trigger in selected)
    constraints = []
    for trigger, complement in sorted(occurrences):
        coefficients = {i: 1 for i in range(n) if trigger & (1 << i)}
        coefficients.update({n + i: -1 for i in range(n) if complement & (1 << i)})
        constraints.append((coefficients, trigger.bit_count() - 1))
    for cards in pilot["chain_cut_cards"]:
        constraints.append(({i: 1 for i in cards}, 2))
    constraints.append(({i: 1 for i in range(n)}, pilot["I2"]))
    singleton_start = len(constraints)
    constraints.extend(({i: 1}, 1) for i in range(n))
    rows, columns, values = [], [], []
    for row, (coefficients, _) in enumerate(constraints):
        for column, value in coefficients.items():
            rows.append(row)
            columns.append(column)
            values.append(value)
    matrix = coo_matrix((values, (rows, columns)), shape=(len(constraints), 2 * n)).tocsr()
    result = linprog(np.array([-1] * n + [1] * n), A_ub=matrix,
                     b_ub=[bound for _, bound in constraints], bounds=(0, None),
                     method="highs", options={"time_limit": max(
                         0.01, seconds - (monotonic() - started))})
    if not result.success:
        return {"status": "solver_stopped", "message": result.message}
    dual = [max(Fraction(0), Fraction(float(-value)).limit_denominator(10**6))
            for value in result.ineqlin.marginals]
    loads = [Fraction(0)] * (2 * n)
    for weight, (coefficients, _) in zip(dual, constraints):
        for column, value in coefficients.items():
            loads[column] += weight * value
    scale = max([Fraction(1)] + [-value for value in loads[n:]])
    dual = [weight / scale for weight in dual]
    loads = [value / scale for value in loads]
    for card in range(n):
        residual = max(Fraction(0), 1 - loads[card])
        dual[singleton_start + card] += residual
        loads[card] += residual
    assert all(value >= 1 for value in loads[:n])
    assert all(value >= -1 for value in loads[n:])
    upper = sum(weight * bound for weight, (_, bound) in zip(dual, constraints))
    lower = 4 * n - 2 * upper
    even_lower = 2 * math.ceil(lower / 2)
    return {"target": target, "I2": pilot["I2"],
            "structural_bound": instance_bound(target),
            "selected_triggers": len(selected), "group_constraints": len(occurrences),
            "chain_constraints": len(pilot["chain_cut_cards"]),
            "floating_lower_bound": 4 * n + 2 * result.fun,
            "verified_lower_bound": record(lower), "even_lower_bound": even_lower,
            "dual": [{"coefficients": coefficients, "rhs": bound, "weight": record(weight)}
                     for weight, (coefficients, bound) in zip(dual, constraints) if weight],
            "elapsed_seconds": monotonic() - started}


def verify_extra_certificate(certificate):
    def require(condition, message):
        if not condition:
            raise ValueError(message)

    def integer(value):
        if type(value) is int:
            return value
        require(type(value) is str, "expected integer or canonical integer string")
        try:
            result = int(value)
        except ValueError as error:
            raise ValueError("invalid integer string") from error
        require(str(result) == value, "noncanonical integer string")
        return result

    def fraction(value):
        require(type(value) is dict and "numerator" in value
                and "denominator" in value, "malformed fraction")
        numerator = integer(value["numerator"])
        denominator = integer(value["denominator"])
        require(denominator > 0, "nonpositive denominator")
        result = Fraction(numerator, denominator)
        if "decimal" in value:
            require(type(value["decimal"]) in (int, float)
                    and value["decimal"] == float(result), "incorrect decimal metadata")
        return result

    require(type(certificate) is dict, "certificate must be an object")
    target = certificate.get("target")
    require(type(target) is list and all(type(card) is int for card in target),
            "target must be an integer list")
    n = len(target)
    require(n > 1 and sorted(target) == list(range(n)), "target must be a full permutation")
    require(target[-1] != n - 1, "verifier requires no fixed bottom suffix")
    positions = {card: position for position, card in enumerate(target)}
    i2 = sum(shape(target)[:2])
    catalog = json.loads((ROOT / "results/excursion-catalog-n6.json").read_text())["rules"]
    allowed = {(tuple(rule["target"]), tuple(rule["twice_cards"])) for rule in catalog}
    loads = [Fraction(0)] * (2 * n)
    upper = Fraction(0)
    require(type(certificate.get("dual")) is list, "dual must be a list")
    for row in certificate["dual"]:
        require(type(row) is dict and type(row.get("coefficients")) is dict
                and "rhs" in row and "weight" in row, "malformed dual row")
        coefficients = {}
        for column, value in row["coefficients"].items():
            column = integer(column)
            require(0 <= column < 2 * n and column not in coefficients, "invalid column")
            require(type(value) is int and value in (-1, 1), "coefficient must be +1 or -1")
            coefficients[column] = value
        require(coefficients, "empty row")
        positive = {column for column, value in coefficients.items() if value == 1}
        negative = {column - n for column, value in coefficients.items() if value == -1}
        require(all(0 <= card < n for card in positive | negative), "invalid variable sign")
        rhs = row["rhs"]
        require(type(rhs) is int, "right side must be an integer")
        if negative:
            require(not positive & negative, "trigger intersects its group")
            cards = sorted(positive | negative)
            ranks = {card: rank for rank, card in enumerate(cards)}
            pattern = tuple(ranks[card] for card in sorted(cards, key=positions.__getitem__))
            trigger = tuple(ranks[card] for card in sorted(positive))
            require((pattern, trigger) in allowed, "uncertified group pattern")
            require(rhs == len(positive) - 1, "incorrect group right side")
        elif len(positive) == n and rhs == i2:
            pass
        elif len(positive) == 1 and rhs == 1:
            pass
        else:
            require(rhs == 2, "incorrect chain right side")
            chain = sorted(positive, key=positions.__getitem__)
            require(all(first > second for first, second in zip(chain, chain[1:])),
                    "row is not a decreasing chain")
        weight = fraction(row["weight"])
        require(weight >= 0, "negative dual weight")
        upper += rhs * weight
        for column, value in coefficients.items():
            loads[column] += value * weight
    require(all(value >= 1 for value in loads[:n]), "insufficient twice-card coverage")
    require(all(value >= -1 for value in loads[n:]), "extra-excursion capacity exceeded")
    lower = 4 * n - 2 * upper
    require(fraction(certificate.get("verified_lower_bound")) == lower,
            "incorrect claimed rational bound")
    require(type(certificate.get("even_lower_bound")) is int
            and 2 * math.ceil(lower / 2) == certificate["even_lower_bound"],
            "incorrect claimed even bound")
    return certificate["even_lower_bound"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("confidence", "population", "counterexamples", "chains", "extras"))
    parser.add_argument("--n", type=int, default=52)
    parser.add_argument("--seconds", type=float, default=55)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "confidence":
        result = confidence()
    elif args.mode == "population":
        result = lp_population(args.n, args.seconds)
    elif args.mode == "chains":
        result = chain_pilot(args.seconds)
    elif args.mode == "extras":
        result = extra_pilot(args.seconds)
        if "dual" in result:
            verify_extra_certificate(result)
    else:
        result = counterexamples()
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
