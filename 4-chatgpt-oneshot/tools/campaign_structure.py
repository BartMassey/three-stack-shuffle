import argparse
from collections import Counter, deque
from functools import lru_cache
import itertools
import json
from pathlib import Path
import random
import sys
from time import monotonic

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from three_stack import Machine
from tools.structural_bound import common_suffix, instance_bound, shape


MOVES = ((0, 1, "AD"), (1, 0, "DA"),
         (1, 2, "DB"), (2, 1, "BD"))


def successors(state):
    for source, destination, code in MOVES:
        if state[source]:
            card = state[source][0]
            next_state = list(state)
            next_state[source] = state[source][1:]
            next_state[destination] = (card,) + state[destination]
            yield tuple(next_state), card, code


def graph(n, deadline):
    initial = ((), tuple(range(n)), ())
    states = [initial]
    ids = {initial: 0}
    edges = []
    for state in states:
        links = []
        for next_state, card, code in successors(state):
            if next_state not in ids:
                ids[next_state] = len(states)
                states.append(next_state)
            links.append((ids[next_state], card, code))
        edges.append(links)
        if len(edges) % 1000 == 0 and monotonic() > deadline:
            raise TimeoutError("graph deadline")
    return states, ids, edges


def distances(edges, goal):
    result = [-1] * len(edges)
    result[goal] = 0
    queue = deque([goal])
    while queue:
        current = queue.popleft()
        for following, _, _ in edges[current]:
            if result[following] < 0:
                result[following] = result[current] + 1
                queue.append(following)
    return result


def feasible(edges, goal, distance, budgets, max_cost, deadline):
    visits = 0

    @lru_cache(None)
    def search(state, remaining, cost):
        nonlocal visits
        visits += 1
        if visits % 4096 == 0 and monotonic() > deadline:
            raise TimeoutError("budget oracle deadline")
        if state == goal:
            return ()
        if distance[state] > cost or distance[state] > sum(remaining):
            return None
        for following, card, code in sorted(
                edges[state], key=lambda edge: distance[edge[0]]):
            if remaining[card] and distance[following] < cost:
                rest = list(remaining)
                rest[card] -= 1
                suffix = search(following, tuple(rest), cost - 1)
                if suffix is not None:
                    return (code,) + suffix
        return None

    plan = search(0, tuple(budgets), max_cost)
    return plan, visits


def twice_feasible(edges, goal, selected, deadline):
    index = {card: position for position, card in enumerate(selected)}
    initial = (0, (0,) * len(selected))
    previous = {initial: None}
    queue = deque([initial])
    while queue:
        current, counts = key = queue.popleft()
        if current == goal:
            plan = []
            while previous[key] is not None:
                key, code = previous[key]
                plan.append(code)
            return tuple(reversed(plan)), len(previous)
        for following, card, code in edges[current]:
            next_counts = counts
            if card in index:
                position = index[card]
                if counts[position] == 2:
                    continue
                temp = list(counts)
                temp[position] += 1
                next_counts = tuple(temp)
            next_key = following, next_counts
            if next_key not in previous:
                previous[next_key] = key, code
                queue.append(next_key)
        if len(previous) % 4096 == 0 and monotonic() > deadline:
            raise TimeoutError("unlimited-other-card oracle deadline")
    return None, len(previous)


def witness(target, plan):
    if plan is None:
        return None
    machine = Machine(list(range(len(target))))
    counts = [0] * len(target)
    for code in plan:
        counts[machine.stacks[code[0]][-1]] += 1
        machine.move(code)
    machine.verify(target)
    return {"moves": list(plan), "counts": counts, "cost": len(plan)}


def run(args):
    started = monotonic()
    deadline = started + args.seconds
    n = args.n
    source = Path(__file__).resolve().parents[1] / "results"
    exact = json.loads((source / f"exact-n{n}.json").read_text())
    cases = [(p, d, instance_bound(p)) for p, d in zip(
        itertools.permutations(range(n)), exact["target_distances"])
        if d > instance_bound(p) and common_suffix(p) == 0]
    cases.sort(key=lambda row: (row[2] - row[1], row[0]))
    states, ids, edges = graph(n, deadline)
    output = {"n": n, "physical_states": len(states),
              "eligible_targets": len(cases), "cases": []}
    try:
        for target, optimal, lower in cases[:args.limit]:
            goal = ids[((), target, ())]
            distance = distances(edges, goal)
            assert distance[0] == optimal
            plan, visits = feasible(edges, goal, distance, [4] * n,
                                    4 * n, deadline)
            optimal_plan, optimal_visits = feasible(
                edges, goal, distance, [4] * n, optimal, deadline)
            row = {"target": target, "distance": optimal,
                   "structural_bound": lower,
                   "all_at_most_four": witness(target, plan),
                   "optimal_all_at_most_four": witness(target, optimal_plan),
                   "oracle_states": visits + optimal_visits,
                   "maximum_twice_sets": []}
            i2 = sum(shape(target)[:2])
            for selected in itertools.combinations(range(n), i2):
                projected = tuple(card for card in target if card in selected)
                if sum(shape(projected)[:2]) < i2:
                    continue
                budgets = [2 if card in selected else 4 for card in range(n)]
                candidate, count = feasible(edges, goal, distance, budgets,
                                            sum(budgets), deadline)
                assert candidate is None
                if args.unlimited:
                    candidate, unbounded_count = twice_feasible(
                        edges, goal, selected, deadline)
                else:
                    candidate, unbounded_count = feasible(
                        edges, goal, distance,
                        [2 if card in selected else optimal for card in range(n)],
                        optimal, deadline)
                item = {
                    "cards": selected, "at_B_infeasible": True,
                    "other_cards_unlimited": args.unlimited,
                    "feasible_witness": witness(target, candidate),
                    "oracle_states": count + unbounded_count}
                if args.unlimited and candidate is not None:
                    item["minimum_total_with_selected_twice"] = len(candidate)
                    if len(selected) == n - 1:
                        exceptional = next(card for card in range(n) if card not in selected)
                        minimum = item["feasible_witness"]["counts"][exceptional]
                        caps = [2] * n
                        caps[exceptional] = minimum - 2
                        failed, checked = feasible(edges, goal, distance, caps,
                                                   sum(caps), deadline)
                        assert failed is None
                        item["independent_lower_budget_check"] = {
                            "exceptional_card": exceptional,
                            "infeasible_budgets": caps,
                            "visited_states": checked}
                row["maximum_twice_sets"].append(item)
            output["cases"].append(row)
            Path(args.output).write_text(json.dumps(output, indent=2) + "\n")
            print(json.dumps({"target": target, "gap": optimal-lower,
                              "at_most_four": plan is not None,
                              "optimal_at_most_four": optimal_plan is not None,
                              "twice_sets": len(row["maximum_twice_sets"]),
                              "twice_feasible": sum(item["feasible_witness"] is not None
                                  for item in row["maximum_twice_sets"])}), flush=True)
    except TimeoutError as error:
        output["timeout"] = str(error)
    output["elapsed_seconds"] = monotonic() - started
    output["completed_cases"] = len(output["cases"])
    Path(args.output).write_text(json.dumps(output, indent=2) + "\n")
    return output


def relaxation(args):
    import numpy as np
    from scipy.optimize import Bounds, LinearConstraint, milp
    from scipy.sparse import coo_matrix

    started = monotonic()
    target = list(range(args.n))
    random.Random(args.seed).shuffle(target)
    suffix = common_suffix(target)
    target = [card for card in target if card < args.n - suffix]
    n = len(target)
    positions = [target.index(card) for card in range(n)]
    patterns = {(2, 1, 4, 3, 0): 0, (2, 4, 1, 3, 0): 0,
                (4, 1, 0, 3, 2): 4, (4, 2, 0, 3, 1): 4}
    rows, columns, values, bounds = [], [], [], []

    def constraint(entries, upper):
        for column, value in entries:
            rows.append(len(bounds))
            columns.append(column)
            values.append(value)
        bounds.append(upper)

    i2 = sum(shape(target)[:2])
    constraint([(card, 1) for card in range(n)], i2)
    for card in range(n):
        constraint([(card, 1), (n + card, 1)], 1)
    triples = 0
    for cards in itertools.combinations(range(n), 3):
        if positions[cards[0]] > positions[cards[1]] > positions[cards[2]]:
            constraint([(card, 1) for card in cards], 2)
            triples += 1
    obstructions = 0
    for cards in itertools.combinations(range(n), 5):
        pattern = tuple(sorted(range(5), key=lambda index: positions[cards[index]]))
        if pattern in patterns:
            exceptional = cards[patterns[pattern]]
            constraint([(card, 1) for card in cards if card != exceptional]
                       + [(n + exceptional, -1)], 3)
            obstructions += 1
    matrix = coo_matrix((values, (rows, columns)),
                        shape=(len(bounds), 2 * n)).tocsc()
    construction_seconds = monotonic() - started
    result = milp(np.array([-1.] * n + [1.] * n),
                  integrality=np.ones(2 * n),
                  bounds=Bounds(np.zeros(2 * n), np.ones(2 * n)),
                  constraints=LinearConstraint(matrix, -np.inf, bounds),
                  options={"time_limit": max(1, args.seconds - construction_seconds),
                           "mip_rel_gap": 0})
    output = {"target": target, "seed": args.seed,
              "structural_bound": 4 * n - 2 * i2,
              "decreasing_triples": triples,
              "conditional_obstructions": obstructions,
              "construction_seconds": construction_seconds,
              "elapsed_seconds": monotonic() - started,
              "solver_status": int(result.status),
              "solver_message": result.message,
              "warning": "Floating MILP bounds are exploratory, not certified.",
              "objective": None if result.fun is None else float(result.fun),
              "dual_bound": float(getattr(result, "mip_dual_bound", -np.inf))}
    if result.x is not None:
        chosen = np.rint(result.x).astype(int)
        assert np.max(matrix @ chosen - np.asarray(bounds)) <= 0
        output["twice_cards"] = [i for i in range(n) if chosen[i]]
        output["six_or_more_cards"] = [i for i in range(n) if chosen[n + i]]
        output["relaxation_feasible_objective"] = int(4 * n + 2 * (
            sum(chosen[n:]) - sum(chosen[:n])))
    Path(args.output).write_text(json.dumps(output, indent=2) + "\n")
    print(json.dumps(output))
    return output


def conditional_obstructions(target):
    n = len(target)
    positions = [0] * n
    for position, card in enumerate(target):
        positions[card] = position
    low_after = [[0] * n for _ in range(n)]
    high_before = [[0] * n for _ in range(n)]
    for value in range(n):
        for position in range(n):
            low_after[value][position] = sum(1 << card for card in range(value)
                                             if positions[card] > position)
            high_before[value][position] = sum(1 << card for card in range(value + 1, n)
                                               if positions[card] < position)
    forced = {}
    for a, b, c, d in itertools.combinations(range(n), 4):
        pa, pb, pc, pd = positions[a], positions[b], positions[c], positions[d]
        six = eight = 0
        if pb < pa < pd < pc:
            six = low_after[a][pc] | high_before[d][pb]
        elif pb < pd < pa < pc:
            six = eight = low_after[a][pc]
        elif pc < pa < pd < pb:
            six = eight = high_before[d][pc]
        if six:
            forced[(1 << a) | (1 << b) | (1 << c) | (1 << d)] = six, eight
    return forced


def conditional_bound(target, deficit=2, seconds=55):
    started = monotonic()
    deadline = started + seconds
    target = list(target)
    original_target = target.copy()
    original_n = len(target)
    if sorted(target) != list(range(original_n)):
        raise ValueError("target must be a permutation of 0..n-1")
    suffix = common_suffix(target)
    target = [card for card in target if card < original_n - suffix]
    n = len(target)
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
    assert i2 == sum(shape(target)[:2])
    checks = []
    certified = 4 * n - 2 * i2
    examined = 0
    halted = None
    for omitted in range(deficit + 1):
        desired = i2 - omitted
        count = 0
        cap = deficit + 1 - omitted
        min_extra = cap
        pruned = 0
        witness_cards = None

        def visit(position, low, high, cards, mask, six, eight):
            nonlocal count, min_extra, examined, witness_cards, pruned
            examined += 1
            if examined % 4096 == 0 and monotonic() > deadline:
                raise TimeoutError("subset enumeration deadline")
            if six.bit_count() + eight.bit_count() >= cap:
                pruned += 1
                return
            needed = desired - len(cards)
            if needed == 0:
                count += 1
                extra = six.bit_count() + eight.bit_count()
                if extra < min_extra:
                    min_extra = extra
                    witness_cards = cards
                return
            if position == n or best(position, low, high) < needed:
                return
            following = include(target[position], low, high)
            if following is not None:
                value = target[position]
                next_six, next_eight = six, eight
                for triple in itertools.combinations(cards, 3):
                    key = (1 << value) | sum(1 << card for card in triple)
                    fsix, feight = forced.get(key, (0, 0))
                    next_six |= fsix
                    next_eight |= feight
                visit(position + 1, *following, cards + (value,),
                      mask | (1 << value), next_six, next_eight)
            visit(position + 1, low, high, cards, mask, six, eight)

        try:
            visit(0, -1, -1, (), 0, 0, 0)
        except TimeoutError as error:
            halted = str(error)
            checks.append({"twice_set_size": desired, "exhausted": False,
                           "sets_examined": count})
            break
        checks.append({"twice_set_size": desired, "exhausted": True,
                       "sets_examined": count, "minimum_forced_excess": min_extra,
                       "excess_capped_at": cap, "certified_pruned_branches": pruned,
                       "minimizing_twice_set": witness_cards})
        objectives = [4 * n - 2 * item["twice_set_size"]
                      + 2 * item["minimum_forced_excess"] for item in checks]
        certified = min(*objectives, 4 * n - 2 * (desired - 1))
        if min_extra == 0:
            break
    output = {"n": original_n, "target": original_target,
              "active_target": target, "common_suffix": suffix,
              "structural_bound": 4 * n - 2 * i2,
              "certified_lower_bound": certified,
              "conditional_quartets": len(forced),
              "subset_checks": checks, "halted": halted,
              "elapsed_seconds": monotonic() - started}
    return output


def enumerate_bound(args, target_override=None, emit=True):
    target = list(range(args.n)) if target_override is None else list(target_override)
    if target_override is None:
        random.Random(args.seed).shuffle(target)
    output = conditional_bound(target, deficit=args.deficit, seconds=args.seconds)
    output["seed"] = args.seed
    if emit:
        Path(args.output).write_text(json.dumps(output, indent=2) + "\n")
        print(json.dumps(output))
    return output


def check_small(args):
    started = monotonic()
    checks = []
    directory = Path(__file__).resolve().parents[1] / "results"
    patterns = {(2, 1, 4, 3, 0): (0, 1), (2, 4, 1, 3, 0): (0, 2),
                (4, 1, 0, 3, 2): (4, 1), (4, 2, 0, 3, 1): (4, 2)}
    for n in range(1, args.check_through + 1):
        exact = json.loads((directory / f"exact-n{n}.json").read_text())
        improved = 0
        bound_sum = 0
        for target, distance in zip(itertools.permutations(range(n)),
                                    exact["target_distances"]):
            args.n = n
            result = enumerate_bound(args, target_override=target, emit=False)
            active = tuple(result["active_target"])
            m = len(active)
            obstructions = []
            for cards in itertools.combinations(range(m), 5):
                projected = tuple(cards.index(card) for card in active if card in cards)
                if projected in patterns:
                    exceptional, weight = patterns[projected]
                    obstructions.append((set(cards) - {cards[exceptional]},
                                         cards[exceptional], weight))
            brute = 4 * m
            for bits in range(1 << m):
                selected = {card for card in range(m) if bits & (1 << card)}
                projected = tuple(card for card in active if card in selected)
                if sum(shape(projected)[:2]) != len(selected):
                    continue
                extra = {}
                for quartet, exceptional, weight in obstructions:
                    if quartet <= selected:
                        assert exceptional not in selected
                        extra[exceptional] = max(extra.get(exceptional, 0), weight)
                brute = min(brute, 4 * m - 2 * len(selected) + 2 * sum(extra.values()))
            expected = min(brute, instance_bound(target) + 2 * (args.deficit + 1))
            assert result["certified_lower_bound"] == expected, (target, result, brute)
            assert expected <= distance, (target, expected, distance)
            improved += expected > instance_bound(target)
            bound_sum += expected
            if monotonic() - started > args.seconds:
                raise TimeoutError("small exact validation deadline")
        checks.append({"n": n, "targets": len(exact["target_distances"]),
                       "improved_targets": improved, "bound_sum": bound_sum,
                       "exact_distance_sum": exact["distance_sum"]})
    result = {"checks": checks, "elapsed_seconds": monotonic() - started,
              "validated": "Pruned DP equals brute-force subset relaxation; bound <= exact distance."}
    Path(args.output).write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--n", type=int, default=5)
    parser.add_argument("--limit", type=int, default=100)
    parser.add_argument("--seconds", type=float, default=55)
    parser.add_argument("--unlimited", action="store_true")
    parser.add_argument("--relaxation", action="store_true")
    parser.add_argument("--enumerate-bound", action="store_true")
    parser.add_argument("--deficit", type=int, default=0)
    parser.add_argument("--samples", type=int, default=1)
    parser.add_argument("--check-through", type=int, default=0)
    parser.add_argument("--paired-batch", action="store_true")
    parser.add_argument("--seed", type=int, default=20261004)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.check_through:
        check_small(args)
        return
    if args.paired_batch:
        started = monotonic()
        rng = random.Random(args.seed)
        output = {"n": args.n, "seed": args.seed,
                  "sampling": "One Random(seed); fresh range(n), shuffle for each target",
                  "requested_samples": args.samples, "deficit": args.deficit,
                  "cases": []}
        for index in range(args.samples):
            target = list(range(args.n))
            rng.shuffle(target)
            result = conditional_bound(target, deficit=args.deficit, seconds=args.seconds)
            result["index"] = index
            output["cases"].append(result)
            output["elapsed_seconds"] = monotonic() - started
            Path(args.output).write_text(json.dumps(output, indent=2) + "\n")
            print(json.dumps({"index": index, "B": result["structural_bound"],
                              "bound": result["certified_lower_bound"],
                              "seconds": result["elapsed_seconds"]}), flush=True)
        return
    if args.enumerate_bound:
        if args.samples == 1:
            enumerate_bound(args)
        else:
            started = monotonic()
            deadline = started + args.seconds
            results = []
            for _ in range(args.samples):
                args.seconds = deadline - monotonic()
                if args.seconds <= 2:
                    break
                results.append(enumerate_bound(args))
                args.seed += 1
            output = {"samples": len(results), "cases": results,
                      "elapsed_seconds": monotonic() - started}
            Path(args.output).write_text(json.dumps(output, indent=2) + "\n")
        return
    if args.relaxation:
        relaxation(args)
        return
    result = run(args)
    print(json.dumps({key: value for key, value in result.items() if key != "cases"}))


if __name__ == "__main__":
    main()
