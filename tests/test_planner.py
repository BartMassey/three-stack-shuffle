"""The inadmissible local-search planner: every completion it produces (the two
rollouts, their cheaper combination, a greedy descent, and the anytime search)
must be a real move stream that sorts the deck when replayed on the machine.

These are correctness guarantees only — quality numbers live in
`experiments/planner_search.py`.
"""
import itertools
import random

import pytest

from splitmerge.machine import GOAL, apply_moves, reversed_deck
from splitmerge import planner as P


def _deck(perm):
    return (tuple(perm), (), ())


def _rand_state(n, rng):
    p = list(range(1, n + 1))
    rng.shuffle(p)
    a = rng.randrange(0, n + 1)
    b = rng.randrange(0, n - a + 1)
    return (tuple(p[a + b:]), tuple(p[:a]), tuple(p[a:a + b]))


@pytest.mark.parametrize("n", [1, 2, 3, 4, 5, 6])
def test_rollouts_sort_all_perms(n):
    goal = GOAL(n)
    for p in itertools.permutations(range(1, n + 1)):
        st = _deck(p)
        for fn in (P.rollout, P.rollout_merge):
            assert apply_moves(st, fn(st)) == goal, f"{fn.__name__} failed on {p}"
        moves, cost = P.completion(st)
        assert apply_moves(st, moves) == goal
        assert cost == len(moves)


def test_rollouts_sort_arbitrary_states():
    # the search visits states with occupied buffers; both rollouts must handle them
    rng = random.Random(0)
    for n in range(2, 13):
        for _ in range(800):
            st = _rand_state(n, rng)
            for fn in (P.rollout, P.rollout_merge):
                assert apply_moves(st, fn(st)) == GOAL(n)


def test_settle_rollout_optimal_on_reversal():
    # the settle-next-card rollout is exact on the reversed deck (= 4(n-1)),
    # so the combined completion is too
    for n in (8, 10, 12, 20, 52):
        assert len(P.rollout(reversed_deck(n))) == 4 * (n - 1)
        assert P.completion(reversed_deck(n))[1] == 4 * (n - 1)


def test_cascade_charge_identity():
    # the cascade charge is the settle rollout read as h0 + 2*bounces
    from splitmerge.heuristics import h0
    rng = random.Random(5)
    for n in range(2, 12):
        for _ in range(50):
            st = _rand_state(n, rng)
            assert P.cascade_charge(st) == len(P.rollout(st))
            assert P.cascade_charge(st) == h0(st) + 2 * P.cascade_bounces(st)
    # exact on the reversal: 4(n-1) total => n-2 cascade bounces
    for n in (8, 12, 20, 52):
        assert P.cascade_bounces(reversed_deck(n)) == n - 2


def test_greedy_and_local_search_sort():
    rng = random.Random(1)
    for n in (6, 8, 10):
        for _ in range(4):
            p = list(range(1, n + 1)); rng.shuffle(p); st = _deck(p)
            gm, gc = P.greedy_solution(st)
            assert apply_moves(st, gm) == GOAL(n) and gc == len(gm)
            bm, bc, fc, ft, nr = P.local_search(st, time_budget=0.1, seed=2)
            assert apply_moves(st, bm) == GOAL(n)
            assert bc <= fc                          # anytime: best never worse than first
