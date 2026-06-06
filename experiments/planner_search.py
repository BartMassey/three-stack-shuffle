"""First-solution quality/timing and anytime behaviour of the inadmissible
local-search planner (`splitmerge.planner`).

For the reversed deck and several random decks it reports: the first complete
solution (the cheaper of the settle-next-card and merge rollouts) and the time
to produce it; the two rollout costs separately; the admissible lower bound
`h_joint`; and the best cost the anytime greedy search reaches within a budget.

Run:  python -m experiments.planner_search [n] [budget_seconds] [num_random]

Honest summary at n=52: the first solution is ~204 on the reversal (the settle
rollout is optimal there) and ~480 on random decks (the merge rollout, on par
with Hu-Tucker); the greedy local search does **not** improve on it within
seconds — the rollout estimate is a good completion but a poor steering gradient.
"""
import random
import sys
import time

from splitmerge.machine import GOAL, apply_moves, reversed_deck
from splitmerge.heuristics import h_joint
from splitmerge import planner as P


def _row(name, st, budget):
    n = len(st[0]) + len(st[1]) + len(st[2])
    t = time.time()
    first_moves, first = P.completion(st)
    ft = time.time() - t
    assert apply_moves(st, first_moves) == GOAL(n)
    settle, merge = P.rollout_cost(st), len(P.rollout_merge(st))
    lb = h_joint(st)
    bm, best, fc, _, nr = P.local_search(st, time_budget=budget, seed=1)
    assert apply_moves(st, bm) == GOAL(n)
    print(f"  {name:12s} first={first:5d} ({ft*1000:6.1f}ms)  "
          f"[settle={settle:5d} merge={merge:5d}]  h_joint(LB)={lb:4d}  "
          f"first/LB={first/lb:4.2f}  best@{budget:.0f}s={best:5d} (x{nr})")


def run(n=52, budget=6.0, num_random=5, seed=7):
    print(f"n={n}, anytime budget {budget:.0f}s")
    _row("reversed", reversed_deck(n), budget)
    rng = random.Random(seed)
    for i in range(num_random):
        p = list(range(1, n + 1))
        rng.shuffle(p)
        _row(f"random#{i}", (tuple(p), (), ()), budget)


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 52
    budget = float(sys.argv[2]) if len(sys.argv) > 2 else 6.0
    num = int(sys.argv[3]) if len(sys.argv) > 3 else 5
    run(n, budget, num)
