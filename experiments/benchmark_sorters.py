"""Average and worst-case move counts for the three no-reversal merge sorters,
on random decks -- reproduces the figures in ``docs/NOTES.md`` §I.3
(at n=52: average ~520 / ~487 / ~484, worst 624 / 600 / 600).

Costs are taken from the closed forms (``*_cost``), which the test suite proves
equal to the actual replayed move counts.

Run:  python -m experiments.benchmark_sorters [n] [num_decks]
"""
import random
import statistics
import sys

from splitmerge import sorters as S


def run(n=52, num_decks=2000, seed=2024):
    rng = random.Random(seed)
    cols = (("natural", S.natural_cost), ("topdown", S.topdown_cost),
            ("hutucker", S.hutucker_cost))
    data = {name: [] for name, _ in cols}
    td_ge_ht = True
    for _ in range(num_decks):
        p = list(range(1, n + 1))
        rng.shuffle(p)
        for name, cost in cols:
            data[name].append(cost(p))
        if S.topdown_cost(p) < S.hutucker_cost(p):
            td_ge_ht = False

    rev = list(range(n, 0, -1))
    print(f"n={n}, {num_decks} random decks (seed {seed})")
    for name, cost in cols:
        v = data[name]
        print(f"  {name:8s}: avg={statistics.mean(v):6.1f}  min={min(v)}  "
              f"max={max(v)}  worst(reversed)={cost(rev)}")
    print(f"  topdown >= hutucker on every deck: {td_ge_ht}")
    print(f"  diameter window at n={n}: [reversal opt 4(n-1)={4*(n-1)}, "
          f"Hu-Tucker worst {S.hutucker_cost(rev)}]")


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 52
    num = int(sys.argv[2]) if len(sys.argv) > 2 else 2000
    run(n, num)
