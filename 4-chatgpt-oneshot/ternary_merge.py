from functools import lru_cache

from hybrid_merge import _join, _swap, _validate
from oriented_merge import _central_merge, _side_merge
from oriented_merge import oriented_bounds, oriented_merge


@lru_cache(None)
def ternary_bounds(n):
    if n < 0:
        raise ValueError("negative deck size")
    if n <= 64:
        return oriented_bounds(n)[:2]
    half = n // 2
    third = n // 3
    central = ternary_bounds(half)[1] + ternary_bounds(n - half)[1] + n
    parked = ternary_bounds(third)[1] + ternary_bounds(n - third)[0] + n + third
    return central, parked


def ternary_merge(initial, target, window=0, endpoint="D"):
    values = _validate(initial, target)
    if endpoint not in ("A", "D", "B"):
        raise ValueError("endpoint must be A, D, or B")
    if window < 0:
        raise ValueError("window must be nonnegative")

    def solve(start, end, reverse, parked):
        length = end - start
        ordered = tuple(sorted(values[start:end], reverse=reverse))
        if length <= 64:
            word = oriented_merge(values[start:end], ordered,
                                  window=window, endpoint="A" if parked else "D")
            return word, ordered
        if not parked:
            split = start + length // 2
            left, left_order = solve(start, split, reverse, True)
            right, right_order = solve(split, end, reverse, True)
            merging = _central_merge(left_order, right_order, reverse)
            return _join((left, _swap(right), merging)), ordered
        split = start + length // 3
        left, left_order = solve(start, split, not reverse, True)
        right, right_order = solve(split, end, reverse, False)
        merging = _side_merge(tuple(reversed(left_order)), right_order, reverse)
        word = min((_join((_swap(left), right, merging)),
                    _join((_swap(left), _swap(right), merging))), key=len)
        return word, ordered

    word, _ = solve(0, len(values), False, endpoint != "D")
    return _swap(word) if endpoint == "B" else word
