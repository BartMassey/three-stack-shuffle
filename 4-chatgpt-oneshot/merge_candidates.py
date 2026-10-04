from collections.abc import Sequence


def _ascending_merge(initial: Sequence, target: Sequence) -> list[str]:
    if len(initial) != len(target) or set(initial) != set(target):
        raise ValueError("initial and target must contain the same unique labels")
    if len(set(target)) != len(target):
        raise ValueError("labels must be unique")
    rank = {label: index for index, label in enumerate(target)}
    stacks = {"A": [], "D": [rank[label] for label in reversed(initial)], "B": []}
    operations = []

    def move(source, destination):
        stacks[destination].append(stacks[source].pop())
        operation = source + destination
        if operations and operations[-1] == operation[::-1]:
            operations.pop()
        else:
            operations.append(operation)

    n = len(initial)
    while True:
        fixed = 0
        while fixed < n and stacks["D"][fixed] == n - 1 - fixed:
            fixed += 1
        pending = list(reversed(stacks["D"][fixed:]))
        if not pending:
            return operations
        lengths = []
        for index, value in enumerate(pending):
            if not index or value < pending[index - 1]:
                lengths.append(1)
            else:
                lengths[-1] += 1
        if len(lengths) == 1:
            return operations
        for index, length in enumerate(lengths):
            destination = "A" if index % 2 == 0 else "B"
            for _ in range(length):
                move("D", destination)
        paired = len(lengths)
        if paired % 2:
            for _ in range(lengths[-1]):
                move("A", "D")
            paired -= 1
        for index in range(paired - 2, -1, -2):
            remaining_a, remaining_b = lengths[index:index + 2]
            while remaining_a or remaining_b:
                if remaining_a and (not remaining_b or stacks["A"][-1] > stacks["B"][-1]):
                    move("A", "D")
                    remaining_a -= 1
                else:
                    move("B", "D")
                    remaining_b -= 1


def natural_merge(initial: Sequence, target: Sequence) -> list[str]:
    direct = _ascending_merge(initial, target)
    n = len(initial)
    if n < 2 or len(direct) <= 4 * n - 4:
        return direct
    reversed_operations = ["DA"] * n
    for _ in range(n):
        reversed_operations.extend(("AD", "DB"))
    reversed_operations.extend(["BD"] * n)
    reversed_operations.extend(_ascending_merge(list(reversed(initial)), target))
    reduced = []
    for operation in reversed_operations:
        if reduced and reduced[-1] == operation[::-1]:
            reduced.pop()
        else:
            reduced.append(operation)
    return reduced if len(reduced) < len(direct) else direct


def _optimal_partition(pending):
    if not pending:
        return []
    states = {("A", -1): (1, 1, 0)}
    history = []
    for index in range(1, len(pending)):
        following = {}
        parents = {}
        for (side, other), (cost, runs_a, runs_b) in states.items():
            for destination in (side, "B" if side == "A" else "A"):
                switched = destination != side
                previous = other if switched else index - 1
                increment = int(previous < 0 or pending[index] < pending[previous])
                next_a = runs_a + increment * (destination == "A")
                next_b = runs_b + increment * (destination == "B")
                key = (destination, index - 1 if switched else other)
                proposal = (cost + increment, next_a, next_b)
                existing = following.get(key)
                if existing is None or (proposal[0], abs(next_a - next_b)) < (
                        existing[0], abs(existing[1] - existing[2])):
                    following[key] = proposal
                    parents[key] = (side, other)
        history.append(parents)
        states = following
    key = min(states, key=lambda item: (states[item][0],
              abs(states[item][1] - states[item][2])))
    assignments = [key[0]]
    for parents in reversed(history):
        key = parents[key]
        assignments.append(key[0])
    return list(reversed(assignments))


def _patience_merge(initial: Sequence, target: Sequence, restart_high: bool | None) -> list[str]:
    rank = {label: index for index, label in enumerate(target)}
    labels = list(target)
    stacks = {"A": [], "D": [rank[label] for label in reversed(initial)], "B": []}
    operations = []

    def move(source, destination):
        stacks[destination].append(stacks[source].pop())
        operation = source + destination
        if operations and operations[-1] == operation[::-1]:
            operations.pop()
        else:
            operations.append(operation)

    n = len(initial)
    budget = (n - 1).bit_length() if n else 0
    while True:
        fixed = 0
        while fixed < n and stacks["D"][fixed] == n - 1 - fixed:
            fixed += 1
        pending = list(reversed(stacks["D"][fixed:]))
        before = 1 + sum(a > b for a, b in zip(pending, pending[1:]))
        if before == 1:
            return operations
        lengths = {"A": [], "B": []}
        assignments = _optimal_partition(pending) if restart_high is None else None
        for index, value in enumerate(pending):
            if assignments is not None:
                side = assignments[index]
                if not lengths[side] or stacks[side][-1] > value:
                    lengths[side].append(1)
                else:
                    lengths[side][-1] += 1
                move("D", side)
                continue
            eligible = [side for side in ("A", "B")
                        if not stacks[side] or stacks[side][-1] < value]
            if eligible:
                side = max(eligible, key=lambda name: stacks[name][-1]
                           if stacks[name] else -1)
                if not lengths[side]:
                    lengths[side].append(0)
                lengths[side][-1] += 1
            else:
                side = (max if restart_high else min)(
                    ("A", "B"), key=lambda name: stacks[name][-1])
                lengths[side].append(1)
            move("D", side)
        while lengths["A"] or lengths["B"]:
            remaining_a = lengths["A"].pop() if lengths["A"] else 0
            remaining_b = lengths["B"].pop() if lengths["B"] else 0
            while remaining_a or remaining_b:
                if remaining_a and (not remaining_b or stacks["A"][-1] > stacks["B"][-1]):
                    move("A", "D")
                    remaining_a -= 1
                else:
                    move("B", "D")
                    remaining_b -= 1
        current = list(reversed(stacks["D"]))
        after = 1 + sum(a > b for a, b in zip(current, current[1:]))
        budget -= 1
        if after >= before or not budget:
            for operation in _ascending_merge([labels[value] for value in current], target):
                move(operation[0], operation[1])
            return operations


def patience_merge(initial: Sequence, target: Sequence) -> list[str]:
    direct = _ascending_merge(initial, target)
    if len(initial) < 3:
        return direct
    candidate = _patience_merge(initial, target, True)
    return candidate if len(candidate) < len(direct) else direct


def optimal_partition_merge(initial: Sequence, target: Sequence) -> list[str]:
    direct = _ascending_merge(initial, target)
    if len(initial) < 3 or len(initial) > 64:
        return direct
    candidate = _patience_merge(initial, target, None)
    return candidate if len(candidate) < len(direct) else direct
