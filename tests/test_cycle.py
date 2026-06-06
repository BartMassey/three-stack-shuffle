"""Whole-cycle model: the algebraic one-cycle test must equal the brute-force
construction, neighbour counts must be Catalan, and the Cayley-graph diameters
must match the recorded sequence -- the numbers the cycle-model write-up cites.
"""
import itertools
import random
from collections import Counter

import pytest

from splitmerge import cycle as C

CATALAN = [1, 1, 2, 5, 14, 42, 132, 429, 1430]


@pytest.mark.parametrize("n", [2, 3, 4, 5])
def test_bruteforce_equals_lis_test_full(n):
    perms = list(itertools.permutations(range(n)))
    for d in perms:
        bf = C.one_cycle_neighbors_bruteforce(d)
        alg = {e for e in perms if C.one_cycle_ok(d, e)}
        assert bf == alg, f"one-cycle reachability mismatch from {d}"


def test_bruteforce_equals_lis_test_n6_sample():
    n = 6
    perms = list(itertools.permutations(range(n)))
    rng = random.Random(0)
    for d in rng.sample(perms, 40):
        bf = C.one_cycle_neighbors_bruteforce(d)
        alg = {e for e in perms if C.one_cycle_ok(d, e)}
        assert bf == alg


@pytest.mark.parametrize("n", [2, 3, 4, 5, 6])
def test_identity_neighbors_are_catalan(n):
    # decks reachable from the identity in one cycle == 123-avoiding perms (Catalan)
    nbrs = C.one_cycle_neighbors_bruteforce(tuple(range(n)))
    assert len(nbrs) == CATALAN[n]
    assert len(C.generators(n)) == CATALAN[n]


def test_reachability_is_symmetric():
    n = 6
    perms = list(itertools.permutations(range(n)))
    rng = random.Random(1)
    for _ in range(400):
        d, e = rng.choice(perms), rng.choice(perms)
        assert C.one_cycle_ok(d, e) == C.one_cycle_ok(e, d)


def test_diameter_sequence():
    # D(n) = 1,1,2,2,2,3 for n = 2..7 (n=8,9 -> 3,3 are slower / per the write-up)
    assert [C.cycle_diameter(n) for n in range(2, 8)] == [1, 1, 2, 2, 2, 3]


def test_bfs_layers_and_lis_breakdown_n7():
    # full Cayley BFS at n=7: layer sizes sum to 7!, and every distance-3
    # permutation has LIS exactly 3 (the "225 states" the write-up records)
    dist = C.cycle_distances(7)
    cnt = Counter(dist.values())
    assert sum(cnt.values()) == 5040
    assert cnt[3] == 225
    far = Counter(C.lis(p) for p, d in dist.items() if d == 3)
    assert dict(far) == {3: 225}


def test_f_word_length():
    # f counts cycles from the identity; identity is 0, a 123-avoiding perm is 1
    assert C.f((0, 1, 2, 3)) == 0
    assert C.f((1, 0, 3, 2)) == 1          # LIS = 2 -> one cycle
    assert C.f((2, 3, 0, 1)) == 1          # LIS = 2 (two decreasing classes)
