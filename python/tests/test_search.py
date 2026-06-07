"""IDA* correctness: it must return the exact optimum (matches BFS) and walk
the reversed deck with zero backtracking (nodes == cost)."""
import random

import pytest

from splitmerge.machine import reversed_deck
from splitmerge.search import bfs_dist, ida_star
from splitmerge.heuristics import h_best, h_joint


@pytest.mark.parametrize("h", [h_best, h_joint])
def test_ida_matches_bfs_n7(h):
    dist = bfs_dist(7)
    rng = random.Random(7)
    decks = []
    for _ in range(12):
        p = list(range(1, 8))
        rng.shuffle(p)
        decks.append((tuple(p), (), ()))
    for st in decks:
        cost, _ = ida_star(st, h)
        assert cost == dist[st]


@pytest.mark.parametrize("h", [h_best, h_joint])
def test_ida_reversal_optimum(h):
    for n in (8, 10, 12):
        cost, _ = ida_star(reversed_deck(n), h)
        assert cost == 4 * (n - 1)


def test_reversal_exact_path_nodes():
    # h_best is exact along the optimal reversal path => nodes == cost.
    for n in (10, 12, 20):
        cost, nodes = ida_star(reversed_deck(n), h_best)
        assert cost == 4 * (n - 1)
        assert nodes == 4 * (n - 1), f"expected exact path at n={n}, got {nodes} nodes"
