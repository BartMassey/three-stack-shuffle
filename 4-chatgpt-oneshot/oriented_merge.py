from functools import lru_cache

from exact_solver import optimal
from hybrid_merge import _join, _swap, _validate
from parked_leaf import PARKED_LEAF_MAXIMUM
from parked_leaf import parked_optimal, parked_recommended
from three_stack import invert


@lru_cache(None)
def oriented_bounds(n):
    if n < 0:
        raise ValueError("negative deck size")
    if n <= 8:
        return 4 * max(0, n - 1), PARKED_LEAF_MAXIMUM[n], None, None
    if n > 64:
        left, right = n // 2, n - n // 2
        central = n + oriented_bounds(left)[1] + oriented_bounds(right)[1]
        parked = oriented_bounds(left)[1] + oriented_bounds(right)[0] + n + left
        return central, min(parked, central + n - 2), left, left
    central, central_split = min(
        (n + oriented_bounds(k)[1] + oriented_bounds(n - k)[1], k)
        for k in range(1, n))
    parked, parked_split = min(
        (oriented_bounds(k)[1] + oriented_bounds(n - k)[0] + n + k, k)
        for k in range(1, n))
    if central + n - 2 < parked:
        parked, parked_split = central + n - 2, None
    return central, max(n, parked), central_split, parked_split


def _central_merge(left, right, reverse):
    i, j = len(left) - 1, len(right) - 1
    sign = -1 if reverse else 1
    word = []
    while i >= 0 or j >= 0:
        if i >= 0 and (j < 0 or sign * left[i] > sign * right[j]):
            word.append("AD")
            i -= 1
        else:
            word.append("BD")
            j -= 1
    return word


def _side_merge(left, right, reverse):
    i = j = 0
    sign = -1 if reverse else 1
    word = []
    while i < len(left) or j < len(right):
        if i < len(left) and (j == len(right) or sign * left[i] < sign * right[j]):
            word.extend(("BD", "DA"))
            i += 1
        else:
            word.append("DA")
            j += 1
    return word


def oriented_merge(initial, target, window=0, endpoint="D", max_n=64):
    values = _validate(initial, target)
    if endpoint not in ("A", "D", "B"):
        raise ValueError("endpoint must be A, D, or B")
    if window < 0:
        raise ValueError("window must be nonnegative")
    if max_n < 8:
        raise ValueError("search cutoff must be at least eight")

    def splits(start, end, fixed):
        middle = (start + end) // 2
        if end - start > max_n:
            return [middle]
        choices = set(range(max(start + 1, middle - window),
                            min(end, middle + window + 1)))
        if fixed is not None:
            choices.add(start + fixed)
        return sorted(choices)

    @lru_cache(None)
    def solve(start, end, reverse):
        segment = values[start:end]
        ordered = tuple(sorted(segment, reverse=reverse))
        length = end - start
        if length <= 8:
            central = optimal(segment, ordered)
            parked = parked_optimal(segment, ordered)
            return tuple(central), tuple(parked), ordered
        bounds = oriented_bounds(length)
        central = [] if tuple(segment) == ordered else None
        for split in splits(start, end, bounds[2]):
            left = solve(start, split, reverse)
            right = solve(split, end, reverse)
            merge = _central_merge(left[2], right[2], reverse)
            candidate = _join((left[1], _swap(right[1]), merge))
            if central is None or len(candidate) < len(central):
                central = candidate
        parked = min((_join((central, ["DA"] * length)),
                      _join((_swap(central), ["DA"] * length))), key=len)
        for split in splits(start, end, bounds[3]):
            left = solve(start, split, not reverse)
            right = solve(split, end, reverse)
            left_ordered = tuple(reversed(left[2]))
            merge = _side_merge(left_ordered, right[2], reverse)
            for right_word in (right[0], _swap(right[0])):
                candidate = _join((_swap(left[1]), right_word, merge))
                if len(candidate) < len(parked):
                    parked = candidate
        return tuple(central), tuple(parked), ordered

    central, parked, _ = solve(0, len(values), False)
    solve.cache_clear()
    if endpoint == "D":
        return list(central)
    return list(parked) if endpoint == "A" else _swap(parked)


def oriented_recommended(initial, target, window=0):
    candidates = [parked_recommended(initial, target),
                  oriented_merge(initial, target, window=window),
                  invert(oriented_merge(target, initial, window=window))]
    return min(candidates, key=len)
