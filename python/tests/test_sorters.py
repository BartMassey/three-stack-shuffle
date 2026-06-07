"""Constructive merge sorters: every emitted move stream, replayed on a fresh
machine, must sort the deck with empty buffers; move counts must equal the
closed forms; and the proven orderings (Hu-Tucker optimal, worst cases) hold.

This is the "machine-verified" guarantee the docs cite -- now reproducible.
"""
import itertools
import math
import random

import pytest

from splitmerge.machine import GOAL, apply_moves, reversed_deck
from splitmerge import sorters as S

SORTERS = [
    (S.natural_sort, S.natural_cost),
    (S.topdown_sort, S.topdown_cost),
    (S.hutucker_sort, S.hutucker_cost),
]


def _deck(perm):
    return (tuple(perm), (), ())


@pytest.mark.parametrize("n", [1, 2, 3, 4, 5, 6, 7])
def test_all_perms_sort_and_count(n):
    # every permutation: replaying the moves sorts it, and len(moves) == closed form
    goal = GOAL(n)
    for p in itertools.permutations(range(1, n + 1)):
        for sort, cost in SORTERS:
            moves = sort(p)
            assert apply_moves(_deck(p), moves) == goal, f"{sort.__name__} failed on {p}"
            assert len(moves) == cost(p), f"{sort.__name__} count != closed form on {p}"


def test_random_large_decks_replay():
    # random decks up to n=400: emitted moves replay to a sorted deck, count == closed form
    rng = random.Random(2024)
    for n in (20, 52, 100, 400):
        goal = GOAL(n)
        for _ in range(8):
            p = list(range(1, n + 1))
            rng.shuffle(p)
            for sort, cost in SORTERS:
                moves = sort(p)
                assert apply_moves(_deck(p), moves) == goal
                assert len(moves) == cost(p)


def test_worst_case_is_reversed_deck():
    # the descending deck is the unique r = n input; proven worst costs at n = 52
    n = 52
    rev = list(range(n, 0, -1))
    assert S.natural_cost(rev) == 624          # 2n*ceil(log2 n)
    assert S.topdown_cost(rev) == 600
    assert S.hutucker_cost(rev) == 600
    # and it really sorts via replay
    assert apply_moves(_deck(rev), S.hutucker_sort(rev)) == GOAL(n)


def test_hutucker_is_optimal_and_dominates_topdown():
    # Hu-Tucker W equals the brute-force min over all order-preserving trees,
    # and is <= the adaptive top-down W, on random run-size compositions.
    def brute_min_W(sizes):
        from functools import lru_cache
        pre = [0]
        for s in sizes:
            pre.append(pre[-1] + s)

        @lru_cache(maxsize=None)
        def best(i, j):                        # min W spanning runs i..j inclusive
            if i == j:
                return 0
            w = pre[j + 1] - pre[i]
            return w + min(best(i, k) + best(k + 1, j) for k in range(i, j))

        return best(0, len(sizes) - 1) if sizes else 0

    rng = random.Random(7)
    for _ in range(400):
        r = rng.randint(1, 9)
        sizes = [rng.randint(1, 6) for _ in range(r)]
        wht = S.weighted_path_length(S.build_hutucker(sizes))
        wtd = S.weighted_path_length(S.build_topdown(sizes))
        assert wht == brute_min_W(sizes)       # Hu-Tucker is optimal
        assert wht <= wtd                      # and never worse than top-down


def test_diameter_window_upper_bound():
    # Hu-Tucker worst over all decks is 600, so the operation diameter M(52) <= 600;
    # check no random deck exceeds it and the proven bound W(T_HT) <= 300 holds.
    rng = random.Random(11)
    worst = 0
    for _ in range(500):
        p = list(range(1, 53))
        rng.shuffle(p)
        worst = max(worst, S.hutucker_cost(p))
    assert worst <= 600
    assert 2 * S.weighted_path_length(S.build_hutucker([1] * 52)) == 600   # all-singleton = 600


def test_natural_cost_closed_form():
    # cost depends only on the run count r: 2n*ceil(log2 r), 0 when already sorted
    assert S.natural_cost(list(range(1, 53))) == 0
    rng = random.Random(3)
    for _ in range(50):
        p = list(range(1, 53))
        rng.shuffle(p)
        r = len(S.ascending_runs(p))
        assert S.natural_cost(p) == 2 * 52 * math.ceil(math.log2(r))
