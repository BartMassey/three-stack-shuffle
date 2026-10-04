from functools import lru_cache
from time import perf_counter

from exact_solver import optimal
from hybrid_merge import _join, _swap, _validate
from oriented_merge import _central_merge, _side_merge, oriented_bounds
from parked_leaf import parked_optimal
from three_stack import invert


def _select(words, width):
    representatives = {}
    for word in words:
        word = tuple(word)
        signature = word[:6], word[-6:]
        old = representatives.get(signature)
        if old is None or (len(word), word) < (len(old), old):
            representatives[signature] = word
    return tuple(sorted(representatives.values(), key=lambda word: (len(word), word))[:width])


def endpoint_candidates(initial, target, width=2, endpoint="D", deadline=None):
    values = _validate(initial, target)
    if width not in (2, 4):
        raise ValueError("width must be two or four")
    if endpoint not in ("A", "D", "B"):
        raise ValueError("endpoint must be A, D, or B")
    if len(values) > 64:
        raise ValueError("endpoint search supports at most 64 cards")

    def check_time():
        if deadline is not None and perf_counter() >= deadline:
            raise TimeoutError("endpoint portfolio deadline reached")

    def splits(start, end, fixed):
        middle = (start + end) // 2
        choices = set(range(max(start + 1, middle - 4), min(end, middle + 5)))
        if fixed is not None:
            choices.add(start + fixed)
        return sorted(choices)

    @lru_cache(None)
    def solve(start, end, reverse):
        check_time()
        segment = values[start:end]
        ordered = tuple(sorted(segment, reverse=reverse))
        length = end - start
        if length <= 8:
            central = optimal(segment, ordered)
            central = _select((central, _swap(central)), width)
            parked = [parked_optimal(segment, ordered)]
            if length:
                parked.append(parked_optimal(segment, ordered, prefer_tail=True))
            parked.extend(_join((word, ["DA"] * length)) for word in central)
            return central, _select(parked, width), ordered
        bounds = oriented_bounds(length)
        central = [()] if tuple(segment) == ordered else []
        for split in splits(start, end, bounds[2]):
            check_time()
            left = solve(start, split, reverse)
            right = solve(split, end, reverse)
            merge = _central_merge(left[2], right[2], reverse)
            for first in left[1]:
                for second in right[1]:
                    word = _join((first, _swap(second), merge))
                    central.extend((word, _swap(word)))
        central = _select(central, width)
        if endpoint == "D" and start == 0 and end == len(values):
            return central, (), ordered
        parked = [_join((word, ["DA"] * length)) for word in central]
        parked.extend(_join((_swap(word), ["DA"] * length)) for word in central)
        for split in splits(start, end, bounds[3]):
            check_time()
            left = solve(start, split, not reverse)
            right = solve(split, end, reverse)
            merge = _side_merge(tuple(reversed(left[2])), right[2], reverse)
            for first in left[1]:
                for second in right[0]:
                    for reflected in (second, _swap(second)):
                        parked.append(_join((_swap(first), reflected, merge)))
        return central, _select(parked, width), ordered

    try:
        central, parked, _ = solve(0, len(values), False)
        if endpoint == "D":
            return [list(word) for word in central]
        return [list(word) if endpoint == "A" else _swap(word) for word in parked]
    finally:
        solve.cache_clear()


def endpoint_portfolio(initial, target, width=2, seconds=8.0, details=None):
    from solver import ALGORITHMS

    if width not in (2, 4):
        raise ValueError("width must be two or four")
    if seconds < 0:
        raise ValueError("seconds must be nonnegative")
    started = perf_counter()
    best = ALGORITHMS["oriented_window"](initial, target)
    baseline_seconds = perf_counter() - started
    baseline_moves = len(best)
    completed = 0
    timed_out = False
    if len(initial) <= 64:
        deadline = started + seconds
        for source, destination, backward in ((initial, target, False),
                                               (target, initial, True)):
            try:
                words = endpoint_candidates(source, destination, width=width,
                                            deadline=deadline)
            except TimeoutError:
                timed_out = True
                break
            completed += 1
            for word in words:
                candidate = invert(word) if backward else word
                if len(candidate) < len(best):
                    best = candidate
    if details is not None:
        details.update(baseline_moves=baseline_moves, baseline_seconds=baseline_seconds,
                       seconds=perf_counter() - started, completed_directions=completed,
                       timed_out=timed_out)
    return best
