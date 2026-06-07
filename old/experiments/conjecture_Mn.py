"""Conjecture test  M(n) = 4(n-1):  is the reversed deck the worst case?

Random sampling plus adversarial hill-climbing from the reversed deck, checking
whether any start state needs more than 4(n-1) moves.  Through n=11 nothing
beats it (random tops out below 4(n-1); hill-climb stays pinned at it).

Run:  python -m experiments.conjecture_Mn [n]
"""
import random
import sys
import time

from splitmerge.machine import reversed_deck
from splitmerge.search import ida_star
from splitmerge.heuristics import h_joint


def random_deck(n, rng):
    p = list(range(1, n + 1))
    rng.shuffle(p)
    return (tuple(p), (), ())


def run(n, num_random=150, climb_steps=200, seed=5, node_cap=300_000):
    rng = random.Random(seed)
    target = 4 * (n - 1)
    t0 = time.time()

    best_random, over, capped = 0, 0, 0
    for _ in range(num_random):
        c, _ = ida_star(random_deck(n, rng), h_joint, node_cap=node_cap)
        if c is None:
            capped += 1
            continue
        best_random = max(best_random, c)
        if c > target:
            over += 1

    cur = list(range(n, 0, -1))
    cur_cost, _ = ida_star(reversed_deck(n), h_joint)
    for _ in range(climb_steps):
        i, j = rng.randrange(n), rng.randrange(n)
        cand = cur[:]
        cand[i], cand[j] = cand[j], cand[i]
        c, _ = ida_star((tuple(cand), (), ()), h_joint, node_cap=node_cap)
        if c is not None and c >= cur_cost:
            cur, cur_cost = cand, c

    print(f"n={n}: target 4(n-1)={target}")
    print(f"  random:    max={best_random} over={over}/{num_random} capped={capped}")
    print(f"  hillclimb: best={cur_cost}")
    print(f"  OVERALL MAX = {max(best_random, cur_cost)}  [{time.time() - t0:.0f}s]")


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 10
    run(n)
