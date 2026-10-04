from collections.abc import Sequence
from functools import lru_cache
from time import monotonic

from three_stack import MOVES, Machine


_EDGES = ((0, 1, "AD"), (1, 0, "DA"),
          (1, 2, "DB"), (2, 1, "BD"))


def _step(state, source, destination):
    result = list(state)
    result[source] = state[source][:-1]
    result[destination] = state[destination] + state[source][-1:]
    return tuple(result)


def _ball(initial, depth):
    paths = {initial: ()}
    frontier = [initial]
    for _ in range(depth):
        following = []
        for state in frontier:
            for source, destination, operation in _EDGES:
                if not state[source]:
                    continue
                target = _step(state, source, destination)
                if target not in paths:
                    paths[target] = paths[state] + (operation,)
                    following.append(target)
        frontier = following
    return paths


@lru_cache(maxsize=50000)
def _replacement(heights, operations):
    state = []
    label = 0
    for height in heights:
        state.append(tuple(range(label, label + height)))
        label += height
    initial = tuple(state)
    target = initial
    for operation in operations:
        source, destination = MOVES[operation]
        target = _step(target, "ADB".index(source),
                       "ADB".index(destination))
    depth = min(4, (len(operations) - 1) // 2)
    forward = _ball(initial, depth)
    backward = _ball(target, depth)
    best = operations
    for meeting in forward.keys() & backward.keys():
        candidate = (forward[meeting]
                     + tuple(move[::-1] for move in
                             reversed(backward[meeting])))
        if len(candidate) < len(best):
            best = candidate
    return best


def _remove_loops(initial, operations):
    machine = Machine(initial)
    initial_state = tuple(tuple(machine.stacks[key]) for key in "ADB")
    states = [initial_state]
    positions = {initial_state: 0}
    result = []
    for operation in operations:
        machine.move(operation)
        state = tuple(tuple(machine.stacks[key]) for key in "ADB")
        if state in positions:
            retain = positions[state]
            for obsolete in states[retain + 1:]:
                positions.pop(obsolete)
            del states[retain + 1:]
            del result[retain:]
        else:
            result.append(operation)
            states.append(state)
            positions[state] = len(result)
    return result, tuple(tuple(machine.stacks[key]) for key in "ADB")


def shorten(initial: Sequence[int], operations: Sequence[str],
            window: int = 12, *, time_limit: float = 0.25) -> list[str]:
    if window < 4 or window > 12:
        raise ValueError("window must be between 4 and 12")
    if time_limit < 0:
        raise ValueError("time_limit must be nonnegative")
    deadline = monotonic() + time_limit
    result, expected = _remove_loops(initial, operations)
    machine = Machine(initial)
    index = 0
    while index < len(result) and monotonic() < deadline:
        replaced = False
        for length in range(min(window, len(result) - index), 3, -1):
            if monotonic() >= deadline:
                break
            heights = tuple(min(len(machine.stacks[key]), length)
                            for key in "ADB")
            segment = tuple(result[index:index + length])
            replacement = _replacement(heights, segment)
            if len(replacement) < length:
                result[index:index + length] = replacement
                replaced = True
                break
        if not replaced and index < len(result):
            machine.move(result[index])
            index += 1
    verifier = Machine(initial)
    verifier.run(result)
    actual = tuple(tuple(verifier.stacks[key]) for key in "ADB")
    if actual != expected:
        raise AssertionError("shortcut changed full endpoint state")
    return result
