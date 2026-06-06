"""The bound guarantees -- the tests that must never regress.

Admissibility is checked exhaustively against exact BFS distances for n <= 7:
every reachable state must satisfy h(s) <= dist(s) for all three heuristics.
"""
import pytest

from splitmerge.machine import reversed_deck, GOAL
from splitmerge.search import bfs_dist
from splitmerge.heuristics import h0, h_best, h_joint


@pytest.fixture(scope="module")
def dist6():
    return bfs_dist(6)


@pytest.fixture(scope="module")
def dist7():
    return bfs_dist(7)


@pytest.mark.parametrize("h", [h0, h_best, h_joint])
def test_admissible_n6(dist6, h):
    bad = [s for s, d in dist6.items() if h(s) > d]
    assert not bad, f"{h.__name__} inadmissible at {len(bad)} states, e.g. {bad[0]}"


@pytest.mark.parametrize("h", [h0, h_best, h_joint])
def test_admissible_n7(dist7, h):
    bad = [s for s, d in dist7.items() if h(s) > d]
    assert not bad, f"{h.__name__} inadmissible at {len(bad)} states, e.g. {bad[0]}"


def test_dominance_ordering(dist6, dist7):
    # h_joint >= h_best >= h0 at every state
    for dist in (dist6, dist7):
        for s in dist:
            assert h0(s) <= h_best(s) <= h_joint(s)


def test_reversal_exact():
    # all three agree with the proven optimum 4(n-1) on the reversed deck;
    # h0 alone gives only 2n there.
    for n in range(3, 13):
        s = reversed_deck(n)
        assert h_best(s) == 4 * (n - 1)
        assert h_joint(s) == 4 * (n - 1)
        assert h0(s) == 2 * n


def test_known_examples():
    # compound state from the docs: opt = 17
    compound = ((6, 2, 5), (4, 1), (3,))
    assert h0(compound) == 9
    assert h_best(compound) == 11
    assert h_joint(compound) == 13          # joint is strictly tighter here
