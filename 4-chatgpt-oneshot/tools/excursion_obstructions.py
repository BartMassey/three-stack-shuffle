import argparse
from collections import Counter
from functools import lru_cache
import itertools
import json
from pathlib import Path
import sys
from time import monotonic

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from tools.campaign_structure import (
    conditional_bound, conditional_obstructions, distances, feasible,
    graph, twice_feasible, witness,
)
from tools.structural_bound import common_suffix, instance_bound, shape


ROOT = Path(__file__).resolve().parents[1]


def increasing_twice(target, selected):
    projected = [card for card in target if card in selected]
    return sum(shape(projected)[:2]) == len(projected)


def projected_rules(target, rules, deadline=None):
    if deadline is not None and monotonic() > deadline:
        raise TimeoutError("group preprocessing deadline")
    positions = {card: position for position, card in enumerate(target)}
    by_pattern = {}
    for rule in rules:
        by_pattern.setdefault(tuple(rule["target"]), []).append(rule)
    occurrences = []
    examined = 0
    for size in sorted({len(pattern) for pattern in by_pattern}):
        for cards in itertools.combinations(range(len(target)), size):
            examined += 1
            if (deadline is not None and examined % 4096 == 0
                    and monotonic() > deadline):
                raise TimeoutError("group preprocessing deadline")
            pattern = tuple(sorted(range(size),
                                   key=lambda rank: positions[cards[rank]]))
            for rule in by_pattern.get(pattern, ()):
                trigger = sum(1 << cards[rank]
                              for rank in rule["twice_cards"])
                complement = sum(1 << card for card in cards) ^ trigger
                occurrences.append((trigger, complement))
    return occurrences


def catalog(through=6, seconds=50):
    started = monotonic()
    deadline = started + seconds
    output = {"through": through, "checks": [], "rules": []}
    try:
        for n in range(2, through + 1):
            states, ids, edges = graph(n, deadline)
            checked = inherited = rejected = oracle_visits = 0
            new = []
            exact = json.loads((ROOT / "results" /
                                f"exact-n{n}.json").read_text())
            for target in itertools.permutations(range(n)):
                goal = ids[((), target, ())]
                distance = distances(edges, goal)
                rank = 0
                available = list(range(n))
                for card in target:
                    offset = available.index(card)
                    rank = rank * len(available) + offset
                    available.pop(offset)
                assert distance[0] == exact["target_distances"][rank]
                smaller = projected_rules(target, output["rules"])
                minimal = []
                for size in range(n + 1):
                    for selected in itertools.combinations(range(n), size):
                        mask = sum(1 << card for card in selected)
                        if not increasing_twice(target, selected):
                            continue
                        if any(old & mask == old for old in minimal):
                            continue
                        if any(trigger & mask == trigger
                               for trigger, _ in smaller):
                            inherited += 1
                            minimal.append(mask)
                            continue
                        caps = [2 if mask & (1 << card) else 4
                                for card in range(n)]
                        plan, visits = feasible(edges, goal, distance, caps,
                                                sum(caps), deadline)
                        checked += 1
                        oracle_visits += visits
                        if plan is None:
                            rejected += 1
                            minimal.append(mask)
                            rule = {"target": list(target),
                                    "twice_cards": list(selected),
                                    "infeasible_budgets": caps,
                                    "visited_states": visits,
                                    "exact_distance": distance[0]}
                            new.append(rule)
                if monotonic() > deadline:
                    raise TimeoutError("catalog deadline")
            output["rules"].extend(new)
            output["checks"].append({
                "n": n, "physical_states": len(states),
                "oracle_checks": checked,
                "inherited_minimal_failures": inherited,
                "new_deletion_minimal_rules": rejected,
                "new_rule_sizes": dict(Counter(
                    len(rule["twice_cards"]) for rule in new)),
                "oracle_visits": oracle_visits,
            })
    except TimeoutError as error:
        output["halted"] = str(error)
    output["elapsed_seconds"] = monotonic() - started
    return output


def disjoint_groups(groups, six, eight, selected, cap=None):
    remaining = set()
    for complement in groups:
        choices = complement & ~selected
        if choices == 0:
            return None
        if choices.bit_count() == 1:
            six |= choices
        remaining.add(choices)
    remaining = {choices for choices in remaining if not choices & six}
    used = 0
    packed = []
    baseline = six.bit_count() + eight.bit_count()
    for choices in sorted(remaining, key=lambda mask: (mask.bit_count(), mask)):
        if not choices & used:
            used |= choices
            packed.append(choices)
            if cap is not None and baseline + len(packed) >= cap:
                break
    return baseline + len(packed), packed


def group_bound(target, rules, deficit=2, seconds=50):
    started = monotonic()
    deadline = started + seconds
    original = list(target)
    suffix = common_suffix(original)
    target = original[:len(original) - suffix] if suffix else original[:]
    n = len(target)
    if sorted(target) != list(range(n)):
        raise ValueError("target must be a permutation of 0..n-1")
    try:
        groups = projected_rules(target, rules, deadline)
    except TimeoutError as error:
        lower = 4 * n - 2 * sum(shape(target)[:2])
        return {"target": original, "active_target": target,
                "structural_bound": lower, "certified_lower_bound": lower,
                "group_occurrences": 0, "subset_checks": [],
                "halted": str(error),
                "elapsed_seconds": monotonic() - started}
    by_trigger = {}
    for trigger, complement in groups:
        by_trigger.setdefault(trigger, set()).add(complement)
    trigger_sizes = sorted({trigger.bit_count() for trigger in by_trigger})
    forced = conditional_obstructions(target)

    def include(value, low, high):
        if value > high:
            return low, value
        if value > low:
            return value, high
        return None

    @lru_cache(None)
    def best(position, low, high):
        if position == n:
            return 0
        result = best(position + 1, low, high)
        following = include(target[position], low, high)
        if following is not None:
            result = max(result, 1 + best(position + 1, *following))
        return result

    i2 = best(0, -1, -1)
    certified = 4 * n - 2 * i2
    checks = []
    examined = 0
    halted = None
    for omitted in range(min(deficit + 1, i2) + 1):
        desired = i2 - omitted
        cap = deficit + 1 - omitted
        if cap <= 0:
            break
        min_extra = cap
        count = pruned = 0
        minimizing = None

        def visit(position, low, high, cards, selected, six, eight, active_groups):
            nonlocal count, min_extra, examined, minimizing, pruned
            examined += 1
            if examined % 1024 == 0 and monotonic() > deadline:
                raise TimeoutError("group enumeration deadline")
            value = disjoint_groups(active_groups, six, eight, selected, cap)
            if value is None or value[0] >= cap:
                pruned += 1
                return
            needed = desired - len(cards)
            if needed == 0:
                count += 1
                extra = value[0]
                if extra < min_extra:
                    min_extra = extra
                    minimizing = list(cards)
                return
            if position == n or best(position, low, high) < needed:
                return
            following = include(target[position], low, high)
            if following is not None:
                card = target[position]
                next_six, next_eight = six, eight
                for triple in itertools.combinations(cards, 3):
                    key = (1 << card) | sum(1 << x for x in triple)
                    fsix, feight = forced.get(key, (0, 0))
                    next_six |= fsix
                    next_eight |= feight
                next_groups = active_groups.copy()
                for size in trigger_sizes:
                    for rest in itertools.combinations(cards, size - 1):
                        key = (1 << card) | sum(1 << x for x in rest)
                        next_groups.update(by_trigger.get(key, ()))
                visit(position + 1, *following, cards + (card,),
                      selected | (1 << card), next_six, next_eight, next_groups)
            visit(position + 1, low, high, cards, selected, six, eight, active_groups)

        try:
            visit(0, -1, -1, (), 0, 0, 0, set())
        except TimeoutError as error:
            halted = str(error)
            checks.append({"twice_set_size": desired, "exhausted": False,
                           "sets_examined": count})
            break
        checks.append({"twice_set_size": desired, "exhausted": True,
                       "sets_examined": count,
                       "minimum_extra_lower_bound": min_extra,
                       "excess_capped_at": cap,
                       "pruned_branches": pruned,
                       "minimizing_twice_set": minimizing})
        objectives = [4 * n - 2 * row["twice_set_size"]
                      + 2 * row["minimum_extra_lower_bound"]
                      for row in checks]
        certified = min(*objectives, 4 * n - 2 * (desired - 1))
        if min_extra == 0:
            break
    return {"target": original, "active_target": target,
            "structural_bound": 4 * n - 2 * i2,
            "certified_lower_bound": certified,
            "group_occurrences": len(groups), "subset_checks": checks,
            "halted": halted, "elapsed_seconds": monotonic() - started}


def conditional_costs(rules, seconds=50):
    started = monotonic()
    deadline = started + seconds
    output = {"cases": []}
    by_size = {}
    for rule in rules:
        by_size.setdefault(len(rule["target"]), []).append(rule)
    try:
        for n, selected_rules in sorted(by_size.items()):
            _, ids, edges = graph(n, deadline)
            for rule in selected_rules:
                target = tuple(rule["target"])
                selected = tuple(rule["twice_cards"])
                goal = ids[((), target, ())]
                plan, visits = twice_feasible(edges, goal, selected, deadline)
                row = dict(rule)
                row["minimum_with_selected_twice"] = (
                    None if plan is None else len(plan))
                row["unrestricted_oracle_states"] = visits
                row["witness"] = witness(target, plan)
                output["cases"].append(row)
    except TimeoutError as error:
        output["halted"] = str(error)
    output["elapsed_seconds"] = monotonic() - started
    return output


def brute_group_bound(target, rules):
    suffix = common_suffix(target)
    target = list(target[:len(target) - suffix] if suffix else target)
    n = len(target)
    occurrences = projected_rules(target, rules)
    forced = conditional_obstructions(target)

    @lru_cache(None)
    def hitting(groups):
        if not groups:
            return 0
        chosen = min(groups, key=int.bit_count)
        return 1 + min(hitting(tuple(group for group in groups
                                    if not group & (1 << card)))
                       for card in range(n) if chosen & (1 << card))

    lower = 4 * n
    for selected in range(1 << n):
        cards = [card for card in range(n) if selected & (1 << card)]
        if not increasing_twice(target, cards):
            continue
        six = eight = 0
        for trigger, (fsix, feight) in forced.items():
            if trigger & selected == trigger:
                six |= fsix
                eight |= feight
        groups = set()
        for trigger, complement in occurrences:
            if trigger & selected == trigger:
                choices = complement & ~selected
                if not choices:
                    break
                if not choices & six:
                    groups.add(choices)
        else:
            extra = six.bit_count() + eight.bit_count() + hitting(tuple(sorted(groups)))
            lower = min(lower, 4 * n - 2 * len(cards) + 2 * extra)
    return lower


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=("catalog", "costs", "validate", "pilot"))
    parser.add_argument("--through", type=int, default=6)
    parser.add_argument("--rules", type=Path)
    parser.add_argument("--seconds", type=float, default=50)
    parser.add_argument("--deficit", type=int, default=2)
    parser.add_argument("--samples", type=int, default=5)
    parser.add_argument("--max-pattern", type=int, default=6)
    parser.add_argument("--indices")
    parser.add_argument("--independent", action="store_true")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.mode == "catalog":
        output = catalog(args.through, args.seconds)
    else:
        rules = json.loads(args.rules.read_text())["rules"]
        rules = [rule for rule in rules
                 if len(rule["target"]) <= args.max_pattern]
        if args.mode == "costs":
            output = conditional_costs(rules, args.seconds)
        elif args.mode == "validate":
            started = monotonic()
            output = {"checks": [], "rule_count": len(rules),
                      "independent_hitting_validation": args.independent,
                      "deficit": args.deficit}
            for n in range(1, args.through + 1):
                exact = json.loads((ROOT / "results" /
                                    f"exact-n{n}.json").read_text())
                improved = stronger = matched = total = 0
                for target, distance in zip(itertools.permutations(range(n)),
                                            exact["target_distances"]):
                    result = group_bound(target, rules, args.deficit,
                                         args.seconds)
                    old = conditional_bound(target, args.deficit, args.seconds)
                    lower = result["certified_lower_bound"]
                    assert lower <= distance, (target, result, distance)
                    if args.independent:
                        brute = brute_group_bound(target, rules)
                        assert lower <= min(brute, instance_bound(target)
                                            + 2 * (args.deficit + 1)), (
                            target, lower, brute)
                    improved += lower > instance_bound(target)
                    stronger += lower > old["certified_lower_bound"]
                    matched += lower == distance
                    total += lower
                output["checks"].append({
                    "n": n, "targets": len(exact["target_distances"]),
                    "improved_structural": improved, "improved_old": stronger,
                    "matched_exact": matched, "bound_sum": total,
                    "exact_distance_sum": exact["distance_sum"],
                })
                args.output.write_text(json.dumps(output, indent=2) + "\n")
            output["elapsed_seconds"] = monotonic() - started
        else:
            stored = json.loads((ROOT / "results" /
                "campaign-structure-52-paired.json").read_text())
            output = {"source": "campaign-structure-52-paired.json",
                      "rule_count": len(rules), "deficit": args.deficit,
                      "seconds_per_target": args.seconds, "cases": []}
            chosen = (set(map(int, args.indices.split(",")))
                      if args.indices else None)
            cases = ([row for row in stored["cases"]
                      if row["index"] in chosen] if chosen is not None
                     else stored["cases"][:args.samples])
            for previous in cases:
                row = group_bound(previous["target"], rules,
                                  args.deficit, args.seconds)
                row["index"] = previous["index"]
                row["previous_bound"] = previous["certified_lower_bound"]
                row["combined_bound"] = max(row["certified_lower_bound"],
                                            row["previous_bound"])
                output["cases"].append(row)
                args.output.write_text(json.dumps(output, indent=2) + "\n")
                print(json.dumps({key: row[key] for key in (
                    "index", "certified_lower_bound", "previous_bound",
                    "elapsed_seconds", "halted")}), flush=True)
    args.output.write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps({key: value for key, value in output.items()
                      if key not in ("rules", "cases")}), flush=True)


if __name__ == "__main__":
    main()
