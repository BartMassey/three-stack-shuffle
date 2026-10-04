def _check(initial, target):
    if len(initial) != len(target) or len(set(target)) != len(target) or set(initial) != set(target):
        raise ValueError('initial and target must contain the same distinct cards')


def _plan(initial, target, codes, flexible=False):
    _check(initial, target)
    if not initial:
        return []
    code_for = dict(zip(target, codes))
    stacks = {'A': [], 'D': list(reversed(initial)), 'B': []}
    moves = []
    for bit in range(max(codes).bit_length()):
        values = [(code_for[card] >> bit) & 1 for card in reversed(stacks['D'])]
        if all(left <= right for left, right in zip(values, values[1:])):
            continue
        count = len(values)
        while count and values[count - 1]:
            count -= 1
        zero_side, one_side = 'A', 'B'
        if flexible and moves:
            previous_side = moves[-1][0]
            other_side = 'B' if previous_side == 'A' else 'A'
            zero_side, one_side = (previous_side, other_side) if values[0] == 0 else (other_side, previous_side)
        for _ in range(count):
            card = stacks['D'].pop()
            destination = one_side if (code_for[card] >> bit) & 1 else zero_side
            stacks[destination].append(card)
            moves.append('D' + destination)
        for source in (one_side, zero_side):
            while stacks[source]:
                stacks['D'].append(stacks[source].pop())
                moves.append(source + 'D')
    if list(reversed(stacks['D'])) != list(target):
        raise AssertionError('radix plan did not realize target')
    return moves


def _gap_codes(count, split=None):
    if count <= 1:
        return [0] * count
    capacity = 1 << (count - 1).bit_length()
    half = capacity // 2
    if split is None:
        split = count - half
    if not count - half <= split <= half:
        raise ValueError('gap split must leave at most half the codes on each side')
    return list(range(split)) + list(range(split + capacity - count, capacity))


def _run_codes(initial, target):
    _check(initial, target)
    positions = {card: index for index, card in enumerate(initial)}
    group = 0
    codes = []
    for index, card in enumerate(target):
        if index and positions[target[index - 1]] > positions[card]:
            group += 1
        codes.append(group)
    return codes


def radix(initial, target):
    return _plan(initial, target, list(range(len(target))))


def radix_gap(initial, target):
    return _plan(initial, target, _gap_codes(len(target)))


def runs_radix(initial, target):
    return _plan(initial, target, _run_codes(initial, target))


def runs_radix_gap(initial, target):
    groups = _run_codes(initial, target)
    labels = _gap_codes(max(groups, default=-1) + 1)
    return _plan(initial, target, [labels[group] for group in groups])


def radix_flexible(initial, target):
    return _plan(initial, target, list(range(len(target))), flexible=True)


def radix_gap_flexible(initial, target):
    return _plan(initial, target, _gap_codes(len(target)), flexible=True)


def runs_radix_flexible(initial, target):
    return _plan(initial, target, _run_codes(initial, target), flexible=True)


def runs_radix_gap_flexible(initial, target):
    groups = _run_codes(initial, target)
    labels = _gap_codes(max(groups, default=-1) + 1)
    return _plan(initial, target, [labels[group] for group in groups], flexible=True)


def radix_gap_candidates(initial, target, runs=False, flexible=False):
    _check(initial, target)
    groups = _run_codes(initial, target) if runs else list(range(len(target)))
    count = max(groups, default=-1) + 1
    if count <= 1:
        return [[]]
    half = 1 << ((count - 1).bit_length() - 1)
    candidates = []
    for split in range(count - half, half + 1):
        labels = _gap_codes(count, split)
        candidates.append(_plan(initial, target, [labels[group] for group in groups], flexible=flexible))
    return candidates


def reversal(initial, target):
    _check(initial, target)
    if list(target) != list(reversed(initial)):
        raise ValueError('target must reverse initial')
    count = len(initial)
    if count <= 1:
        return []
    return ['DA'] * (count - 1) + ['DB'] + ['AD', 'DB'] * (count - 2) + ['AD'] + ['BD'] * (count - 1)
