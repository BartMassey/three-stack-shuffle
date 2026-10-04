from functools import lru_cache
import json
from pathlib import Path

from three_stack import validate_problem


@lru_cache(maxsize=8)
def _table(n):
    path = Path(__file__).parent / "results" / f"plans-n{n}.json"
    data = json.loads(path.read_text())
    return data["plans"], data["move_codes"]


def optimal(initial, target):
    validate_problem(initial, target)
    n = len(initial)
    if n > 8:
        raise ValueError("exact lookup supports at most eight cards")
    if n < 2:
        return []
    positions = {card: index for index, card in enumerate(initial)}
    remaining = list(range(n))
    rank = 0
    for card in target:
        index = remaining.index(positions[card])
        rank = rank * len(remaining) + index
        remaining.pop(index)
    plans, codes = _table(n)
    packed = plans[rank]
    operations = []
    while packed != 1:
        operations.append(codes[packed & 3])
        packed >>= 2
    return operations[::-1]
