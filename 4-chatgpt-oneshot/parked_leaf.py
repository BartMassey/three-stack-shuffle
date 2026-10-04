from functools import lru_cache
import json
from pathlib import Path

from exact_solver import optimal
from hybrid_merge import _join, _merge, _swap, _validate, hybrid_merge
from hybrid_variants import hybrid_window
from three_stack import invert


PARKED_LEAF_MAXIMUM = (0, 1, 4, 7, 12, 15, 20, 23, 28)


@lru_cache(maxsize=None)
def parked_bounds(n):
    if n < 0:
        raise ValueError("negative deck size")
    if n <= 8:
        return 4 * max(0, n - 1), PARKED_LEAF_MAXIMUM[n], None
    split = min(range(1, n),
                key=lambda k: parked_bounds(k)[1] + parked_bounds(n - k)[1])
    bound = n + parked_bounds(split)[1] + parked_bounds(n - split)[1]
    return bound, max(n, bound + n - 2), split


@lru_cache(maxsize=16)
def _table(n, prefer_tail=False):
    label = "tail-plans" if prefer_tail else "plans"
    path = Path(__file__).parent / "results" / f"campaign-execution-{label}-n{n}.json"
    data = json.loads(path.read_text())
    return data["plans"], data["move_codes"]


def parked_optimal(initial, target, side="A", prefer_tail=False):
    values = _validate(initial, target)
    n = len(values)
    if side not in ("A", "B"):
        raise ValueError("parking side must be A or B")
    if n > 8:
        raise ValueError("parked lookup supports at most eight cards")
    if not n:
        return []
    remaining = list(range(n))
    rank = 0
    for value in values:
        index = remaining.index(value)
        rank = rank * len(remaining) + index
        remaining.pop(index)
    plans, codes = _table(n, prefer_tail)
    word = [codes[int(code)] for code in plans[rank]]
    return word if side == "A" else _swap(word)


def parked_hybrid(initial, target, window=0, leaf_limit=8,
                  max_n=64, retain_baseline=True, bound_splits=False,
                  tail_alternative=False):
    values = _validate(initial, target)
    if not 1 <= leaf_limit <= 8:
        raise ValueError("leaf_limit must be between one and eight")
    if window < 0:
        raise ValueError("window must be nonnegative")
    baseline = None
    if retain_baseline or len(values) > max_n:
        baseline = (hybrid_window(initial, target, window=window,
                              leaf_limit=leaf_limit, max_n=max_n)
                if window else hybrid_merge(initial, target, leaf_limit))
    if len(values) > max_n:
        return baseline

    @lru_cache(None)
    def solve(start, end):
        segment = values[start:end]
        ordered = sorted(segment)
        length = end - start
        if segment == ordered:
            best = []
        elif length <= leaf_limit:
            best = optimal(segment, ordered)
        else:
            best = None
            middle = (start + end) // 2
            splits = list(range(max(start + 1, middle - window),
                                min(end, middle + window + 1)))
            if bound_splits and length > 8:
                offset = parked_bounds(length)[2]
                splits.extend([start + offset, end - offset])
            for split in dict.fromkeys(splits):
                left = solve(start, split)
                right = solve(split, end)
                merging = _merge(left[1], right[1])
                for first in left[2]:
                    for second in right[3]:
                        word = _join((first, second, merging))
                        if best is None or len(word) < len(best):
                            best = word
        reflected = _swap(best)
        parked = []
        for side in ("A", "B"):
            parking = ["D" + side] * length
            candidates = [_join((best, parking)),
                          _join((reflected, parking))]
            if length <= leaf_limit:
                candidates.append(parked_optimal(segment, ordered, side))
                if tail_alternative:
                    candidates.append(parked_optimal(segment, ordered, side, True))
            parked.append(tuple(dict.fromkeys(tuple(word) for word in candidates)))
        return tuple(best), tuple(ordered), parked[0], parked[1]

    candidate = list(solve(0, len(values))[0])
    solve.cache_clear()
    return min((baseline, candidate), key=len) if retain_baseline else candidate


def parked_recommended(initial, target, window=4):
    from solver import recommended

    candidates = [recommended(initial, target),
                  parked_hybrid(initial, target, window=window,
                                retain_baseline=False, bound_splits=True),
                  invert(parked_hybrid(target, initial, window=window,
                                       retain_baseline=False, bound_splits=True))]
    return min(candidates, key=len)
