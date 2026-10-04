from collections.abc import Sequence
from functools import lru_cache, partial

from exact_solver import optimal
from hybrid_merge import hybrid_merge, hybrid_dp
from hybrid_variants import hybrid_window
from merge_candidates import natural_merge, patience_merge, optimal_partition_merge
from parked_leaf import parked_bounds, parked_recommended
from oriented_merge import oriented_bounds, oriented_merge
from radix_candidates import (
    radix, radix_gap, runs_radix, runs_radix_gap,
    runs_radix_gap_flexible,
)
from three_stack import cancel, invert, reversal, selection, validate_problem


LEAF_PARKING_COST = (0, 0, 2, 4, 8, 12, 16, 20, 24)


@lru_cache(maxsize=None)
def _bounds(n):
    if n < 0:
        raise ValueError("negative deck size")
    if n <= 8:
        return 4 * max(0, n - 1), LEAF_PARKING_COST[n]
    bound = 2 * n + _bounds(n // 2)[1] + _bounds(n - n // 2)[1]
    return bound, max(0, bound - 2)


def move_bound(n, algorithm="recommended"):
    if algorithm in ("oriented", "oriented_window"):
        return oriented_bounds(n)[0]
    if algorithm == "parked" and n <= 64:
        return parked_bounds(n)[0]
    return _bounds(n)[0]


def portfolio(initial: Sequence[int], target: Sequence[int]) -> list[str]:
    validate_problem(initial, target)
    if list(initial) == list(target):
        return []
    if list(initial) == list(reversed(target)):
        return reversal(len(initial))
    candidates = []
    for algorithm in (hybrid_merge, patience_merge, runs_radix_gap_flexible):
        candidates.append(cancel(algorithm(initial, target)))
        candidates.append(cancel(invert(algorithm(target, initial))))
    if len(initial) <= 16:
        candidates.append(selection(initial, target))
        candidates.append(invert(selection(target, initial)))
    return min(candidates, key=len)


def _special_case(initial, target):
    validate_problem(initial, target)
    if list(initial) == list(target):
        return []
    if len(initial) <= 8:
        return optimal(initial, target)
    if list(initial) == list(reversed(target)):
        return reversal(len(initial))
    return None


def _two_run_plan(initial, target):
    ranks = {card: index for index, card in enumerate(target)}
    sides = {"A": [], "B": []}
    operations = []
    for card in initial:
        value = ranks[card]
        eligible = [side for side in sides
                    if not sides[side] or sides[side][-1] < value]
        if not eligible:
            return None
        side = max(eligible, key=lambda item: sides[item][-1]
                   if sides[item] else -1)
        sides[side].append(value)
        operations.append("D" + side)
    while sides["A"] or sides["B"]:
        side = max((side for side in sides if sides[side]),
                   key=lambda item: sides[item][-1])
        sides[side].pop()
        operations.append(side + "D")
    return cancel(operations)


def _choose_with_two_runs(initial, target, candidates):
    for source, destination, reverse in ((initial, target, False),
                                         (target, initial, True)):
        candidate = _two_run_plan(source, destination)
        if candidate is not None:
            candidates.append(invert(candidate) if reverse else candidate)
    return min(candidates, key=len)


def fast(initial: Sequence[int], target: Sequence[int]) -> list[str]:
    special = _special_case(initial, target)
    if special is not None:
        return special
    return _choose_with_two_runs(initial, target,
                                [hybrid_merge(initial, target),
                                 invert(hybrid_merge(target, initial))])


def recommended(initial: Sequence[int], target: Sequence[int]) -> list[str]:
    special = _special_case(initial, target)
    if special is not None:
        return special
    return _choose_with_two_runs(initial, target,
                                [hybrid_window(initial, target, window=4),
                                 invert(hybrid_window(target, initial, window=4))])


def thorough(initial: Sequence[int], target: Sequence[int]) -> list[str]:
    best = recommended(initial, target)
    if 8 < len(initial) <= 64:
        for source, destination, reverse in ((initial, target, False),
                                             (target, initial, True)):
            candidate = hybrid_dp(source, destination)
            if reverse:
                candidate = invert(candidate)
            if len(candidate) < len(best):
                best = candidate
    return best


def parked(initial: Sequence[int], target: Sequence[int]) -> list[str]:
    special = _special_case(initial, target)
    if special is not None:
        return special
    return parked_recommended(initial, target)


def oriented(initial: Sequence[int], target: Sequence[int], window=0) -> list[str]:
    special = _special_case(initial, target)
    if special is not None:
        return special
    return _choose_with_two_runs(initial, target,
                                [oriented_merge(initial, target, window=window),
                                 invert(oriented_merge(target, initial, window=window))])


ALGORITHMS = {
    "radix": radix,
    "radix_gap": radix_gap,
    "runs_radix": runs_radix,
    "runs_radix_gap": runs_radix_gap,
    "runs_radix_flexible": runs_radix_gap_flexible,
    "merge": natural_merge,
    "patience": patience_merge,
    "partition_dp": optimal_partition_merge,
    "hybrid": hybrid_merge,
    "hybrid_dp": hybrid_dp,
    "window2": partial(hybrid_window, window=2),
    "window4": partial(hybrid_window, window=4),
    "selection": selection,
    "portfolio": portfolio,
    "recommended": recommended,
    "fast": fast,
    "thorough": thorough,
    "parked": parked,
    "oriented": oriented,
    "oriented_window": partial(oriented, window=4),
}
