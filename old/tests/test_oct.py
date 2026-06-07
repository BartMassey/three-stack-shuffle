"""Exact OCT: it must equal the brute-force oracle on every real soft graph, and
solve cliques (the reversal case) in the right value without blowing up."""
import random

from splitmerge.search import bfs_dist
from splitmerge.heuristics import _soft_conflict_graph, _oct_pre_bruteforce
from splitmerge.oct import exact_oct


def test_oct_cliques():
    # K_m needs exactly m-2 deletions; must stay fast (no 3^k blowup)
    for m in (3, 5, 10, 20, 52):
        vals = list(range(1, m + 1))
        adj = {v: set(vals) - {v} for v in vals}
        assert exact_oct(vals, adj, {})[0] == m - 2


def test_oct_edge_cases():
    assert exact_oct([], {}, {})[0] == 0
    assert exact_oct([1, 2, 3], {1: set(), 2: set(), 3: set()}, {})[0] == 0  # no edges


def test_oct_matches_oracle_n6():
    dist = bfs_dist(6)
    for s in dist:
        vals, adj, pre, _ = _soft_conflict_graph(s)
        if vals:
            assert exact_oct(vals, adj, pre)[0] == _oct_pre_bruteforce(vals, adj, pre)


def test_oct_matches_oracle_n7_sample():
    dist = bfs_dist(7)
    rng = random.Random(0)
    states = rng.sample(list(dist), 3000)
    for s in states:
        vals, adj, pre, _ = _soft_conflict_graph(s)
        if vals:
            assert exact_oct(vals, adj, pre)[0] == _oct_pre_bruteforce(vals, adj, pre)


def test_oct_budget_never_hangs_and_is_admissible():
    # a large, far-from-clique soft graph: must return promptly, and (exact or
    # fallback) the value is a lower bound, i.e. between the chain bound and |V|.
    import time
    p = list(range(1, 31))
    random.Random(9).shuffle(p)
    vals, adj, pre, _ = _soft_conflict_graph((tuple(p), (), ()))
    t = time.time()
    value, exact = exact_oct(vals, adj, pre)
    assert time.time() - t < 10.0          # the un-budgeted version took ~295s
    # chain bound (clique - 2) is the floor; |V| the trivial ceiling
    order = sorted(vals)
    chain = {}
    longest = 0
    for v in order:
        b = 1
        for w in adj[v]:
            if w < v and w in chain and chain[w] + 1 > b:
                b = chain[w] + 1
        chain[v] = b
        longest = max(longest, b)
    assert max(0, longest - 2) <= value <= len(vals)
