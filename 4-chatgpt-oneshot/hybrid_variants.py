from functools import lru_cache

from exact_solver import optimal
from hybrid_merge import _combine, _validate, hybrid_merge


def hybrid_window(initial, target, window=1, leaf_limit=8, max_n=64):
    if window < 0:
        raise ValueError('window must be nonnegative')
    baseline = hybrid_merge(initial, target, leaf_limit)
    if len(initial) > max_n:
        return baseline
    values = _validate(initial, target)

    @lru_cache(None)
    def solve(start, end):
        segment = values[start:end]
        ordered = sorted(segment)
        if segment == ordered:
            return (), tuple(ordered)
        if len(segment) <= leaf_limit:
            return tuple(optimal(segment, ordered)), tuple(ordered)
        middle = (start + end) // 2
        splits = range(max(start + 1, middle - window),
                       min(end, middle + window + 1))
        candidates = []
        for split in splits:
            first, left = solve(start, split)
            second, right = solve(split, end)
            candidates.append(_combine(first, second, left, right))
        return tuple(min(candidates, key=len)), tuple(ordered)

    candidate = list(solve(0, len(values))[0])
    solve.cache_clear()
    return candidate if len(candidate) < len(baseline) else baseline


def hybrid_aligned(initial, target, leaf_limit=8, max_n=64):
    baseline = hybrid_merge(initial, target, leaf_limit)
    if len(initial) > max_n:
        return baseline
    values = _validate(initial, target)

    @lru_cache(None)
    def solve(start, end):
        segment = values[start:end]
        ordered = sorted(segment)
        if segment == ordered:
            return (), tuple(ordered)
        if len(segment) <= leaf_limit:
            return tuple(optimal(segment, ordered)), tuple(ordered)
        middle = (end - start) // 2
        lower = middle // leaf_limit * leaf_limit
        upper = lower + leaf_limit
        offsets = {middle, lower, upper}
        candidates = []
        for offset in sorted(offsets):
            if not 0 < offset < end - start:
                continue
            split = start + offset
            first, left = solve(start, split)
            second, right = solve(split, end)
            candidates.append(_combine(first, second, left, right))
        return tuple(min(candidates, key=len)), tuple(ordered)

    candidate = list(solve(0, len(values))[0])
    solve.cache_clear()
    return candidate if len(candidate) < len(baseline) else baseline


def hybrid_root(initial, target, radius=10, leaf_limit=8, max_n=64):
    baseline = hybrid_merge(initial, target, leaf_limit)
    n = len(initial)
    if n <= leaf_limit or n > max_n:
        return baseline
    values = _validate(initial, target)
    best = baseline
    middle = n // 2
    for split in range(max(1, middle - radius), min(n, middle + radius + 1)):
        left = sorted(values[:split])
        right = sorted(values[split:])
        first = hybrid_merge(values[:split], left, leaf_limit)
        second = hybrid_merge(values[split:], right, leaf_limit)
        candidate = _combine(first, second, left, right)
        if len(candidate) < len(best):
            best = candidate
    return best


def hybrid_packed(initial, target, leaf_limit=8, max_n=64):
    baseline = hybrid_merge(initial, target, leaf_limit)
    if len(initial) > max_n:
        return baseline
    values = _validate(initial, target)

    def solve(segment, left_full):
        ordered = sorted(segment)
        if segment == ordered:
            return [], ordered
        if len(segment) <= leaf_limit:
            return optimal(segment, ordered), ordered
        leaves = (len(segment) + leaf_limit - 1) // leaf_limit
        left_leaves = leaves // 2
        split = (left_leaves * leaf_limit if left_full
                 else len(segment) - (leaves - left_leaves) * leaf_limit)
        first, left = solve(segment[:split], left_full)
        second, right = solve(segment[split:], left_full)
        return _combine(first, second, left, right), ordered

    candidates = [baseline, solve(values, True)[0], solve(values, False)[0]]
    return min(candidates, key=len)
