from collections.abc import Sequence


def _reduce(operations):
    result = []
    for operation in operations:
        if result and result[-1] == operation[::-1]:
            result.pop()
        else:
            result.append(operation)
    return result


def _swap(operations):
    reflected = {'AD': 'BD', 'DA': 'DB', 'DB': 'DA', 'BD': 'AD'}
    return [reflected[operation] for operation in operations]


def _join(parts):
    result = []
    for part in parts:
        cancelled = 0
        while result and cancelled < len(part) and result[-1] == part[cancelled][::-1]:
            result.pop()
            cancelled += 1
        result.extend(part[cancelled:])
    return result


def _validate(initial, target):
    if len(initial) != len(target) or len(set(target)) != len(target) or set(initial) != set(target):
        raise ValueError('initial and target must contain the same distinct cards')
    ranks = {card: index for index, card in enumerate(target)}
    return [ranks[card] for card in initial]


def _merge(left, right):
    left_index, right_index = len(left) - 1, len(right) - 1
    operations = []
    while left_index >= 0 or right_index >= 0:
        if left_index >= 0 and (right_index < 0 or left[left_index] > right[right_index]):
            operations.append('AD')
            left_index -= 1
        else:
            operations.append('BD')
            right_index -= 1
    return operations


def _combine(left_plan, right_plan, left, right, left_swap=None, right_swap=None):
    middle = ['DA'] * len(left)
    parking = ['DB'] * len(right)
    merging = _merge(left, right)
    if left_swap is None:
        left_swap = _swap(left_plan)
    if right_swap is None:
        right_swap = _swap(right_plan)
    best = None
    for first in (left_plan, left_swap):
        for second in (right_plan, right_swap):
            candidate = _join((first, middle, second, parking, merging))
            if best is None or len(candidate) < len(best):
                best = candidate
    return best


def hybrid_merge(initial: Sequence, target: Sequence, leaf_limit: int = 8) -> list[str]:
    from exact_solver import optimal

    values = _validate(initial, target)
    if not 1 <= leaf_limit <= 8:
        raise ValueError('leaf_limit must be between one and eight')

    def solve(segment):
        ordered = sorted(segment)
        if segment == ordered:
            return [], ordered
        if len(segment) <= leaf_limit:
            return optimal(segment, ordered), ordered
        split = len(segment) // 2
        first_plan, left = solve(segment[:split])
        second_plan, right = solve(segment[split:])
        return _combine(first_plan, second_plan, left, right), ordered

    return solve(values)[0]


def hybrid_dp(initial: Sequence, target: Sequence, leaf_limit: int = 8, max_n: int = 64) -> list[str]:
    from exact_solver import optimal

    values = _validate(initial, target)
    if not 1 <= leaf_limit <= 8:
        raise ValueError('leaf_limit must be between one and eight')
    baseline = hybrid_merge(initial, target, leaf_limit)
    count = len(values)
    if count > max_n:
        return baseline
    plans = {}
    reflected = {}
    ordered = {}
    for length in range(1, count + 1):
        for start in range(count - length + 1):
            end = start + length
            segment = values[start:end]
            key = (start, end)
            ordered[key] = sorted(segment)
            if segment == ordered[key]:
                plans[key] = []
                reflected[key] = []
                continue
            if length <= leaf_limit:
                plans[key] = optimal(segment, ordered[key])
                reflected[key] = _swap(plans[key])
                continue
            best = None
            for split in range(start + 1, end):
                left_key, right_key = (start, split), (split, end)
                candidate = _combine(plans[left_key], plans[right_key], ordered[left_key], ordered[right_key],
                                     reflected[left_key], reflected[right_key])
                if best is None or len(candidate) < len(best):
                    best = candidate
            plans[key] = best
            reflected[key] = _swap(best)
    candidate = plans.get((0, count), [])
    return candidate if len(candidate) < len(baseline) else baseline
