from collections.abc import Iterable, Sequence
from dataclasses import dataclass, field


MOVES = {"AD": ("A", "D"), "DA": ("D", "A"),
         "DB": ("D", "B"), "BD": ("B", "D")}


def validate_problem(initial: Sequence[int], target: Sequence[int]) -> None:
    if len(set(initial)) != len(initial):
        raise ValueError("initial cards must be distinct")
    if len(target) != len(initial) or set(initial) != set(target):
        raise ValueError("target must contain exactly the initial cards")


@dataclass
class Machine:
    initial: Sequence[int]
    stacks: dict[str, list[int]] = field(init=False)
    operations: int = field(default=0, init=False)

    def __post_init__(self) -> None:
        validate_problem(self.initial, self.initial)
        self.stacks = {"A": [], "D": list(reversed(self.initial)), "B": []}

    def move(self, operation: str) -> None:
        if operation not in MOVES:
            raise ValueError(f"illegal move: {operation!r}")
        source, destination = MOVES[operation]
        if not self.stacks[source]:
            raise ValueError(f"empty source stack: {operation}")
        self.stacks[destination].append(self.stacks[source].pop())
        self.operations += 1

    def run(self, operations: Iterable[str]) -> tuple[int, ...]:
        for operation in operations:
            self.move(operation)
        return tuple(reversed(self.stacks["D"]))

    def verify(self, target: Sequence[int]) -> None:
        if self.stacks["A"] or self.stacks["B"]:
            raise AssertionError("side stacks are not empty")
        if tuple(reversed(self.stacks["D"])) != tuple(target):
            raise AssertionError("incorrect final deck")


def cancel(operations: Iterable[str]) -> list[str]:
    result: list[str] = []
    for operation in operations:
        if operation not in MOVES:
            raise ValueError(f"illegal move: {operation!r}")
        if result and result[-1] == operation[::-1]:
            result.pop()
        else:
            result.append(operation)
    return result


def invert(operations: Sequence[str]) -> list[str]:
    return [operation[::-1] for operation in reversed(operations)]


def reversal(n: int) -> list[str]:
    if n < 0:
        raise ValueError("negative deck size")
    if n < 2:
        return []
    return (["DA"] * (n - 1) + ["DB"]
            + ["AD", "DB"] * (n - 2) + ["AD"]
            + ["BD"] * (n - 1))


def selection(initial: Sequence[int], target: Sequence[int]) -> list[str]:
    validate_problem(initial, target)
    machine = Machine(initial)
    result: list[str] = []

    def move(operation: str) -> None:
        machine.move(operation)
        result.append(operation)

    for card in target:
        while machine.stacks["D"][-1] != card:
            move("DA")
        move("DB")
        while machine.stacks["A"]:
            move("AD")
    while machine.stacks["B"]:
        move("BD")
    return cancel(result)
