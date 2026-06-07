"""Exact minimum odd-cycle transversal (OCT) with pre-coloured vertices.

`exact_oct(vals, adj, pre)` returns the minimum number of vertices to delete so
that the graph 2-colours respecting the fixed colours in `pre` (buffer cards
fixed to their buffer's side; deck cards free). This is the exact `OCT_pre` that
`heuristics.h_joint` needs.

Why this is both exact and fast here:

* The soft-conflict graph is a *comparability graph* (edges orient by value and
  are transitive), hence perfect, so an induced subgraph is bipartite iff it is
  triangle-free, and its max clique equals its longest value-chain.
* OCT is NP-hard in general and FPT in the deletion count k -- but a pure FPT
  branch would blow up on the reversal's size-n clique (k = n-2). The fix is to
  prune with the chain lower bound `clique - 2 <= OCT`: at a clique it equals the
  incumbent immediately, so the first DFS path certifies optimality and every
  sibling is pruned. No `3^k` blowup.

Pre-colouring is encoded with two undeletable terminal vertices `T0`, `T1`
joined by an edge: a side-0 vertex is joined to `T1` (forcing it opposite), a
side-1 vertex to `T0`. Min OCT of the augmented graph (never deleting the
terminals) is exactly the constrained OCT.
"""
from collections import deque

T0, T1 = -1, -2          # terminal sentinels (real vertices are card values >= 1)


def _clique(remaining, adj):
    """Max clique = longest value-chain of the comparability graph induced on
    `remaining` (a set). O(|remaining|^2)."""
    if not remaining:
        return 0
    best = {}
    ans = 0
    for v in sorted(remaining):          # ascending value: all w < v already done
        b = 1
        for w in adj[v]:
            if w in remaining and w < v and best[w] + 1 > b:
                b = best[w] + 1
        best[v] = b
        if b > ans:
            ans = b
    return ans


def _bipartite_or_odd_cycle(adj):
    """2-colour `adj` (dict vertex -> set). Return None if bipartite, else a list
    of vertices forming an odd closed walk (its vertex set meets every
    bipartization)."""
    colour = {}
    parent = {}
    for src in adj:
        if src in colour:
            continue
        colour[src] = 0
        parent[src] = None
        dq = deque([src])
        while dq:
            u = dq.popleft()
            for w in adj[u]:
                if w not in colour:
                    colour[w] = 1 - colour[u]
                    parent[w] = u
                    dq.append(w)
                elif colour[w] == colour[u]:
                    # odd cycle: climb to common ancestor
                    au = []
                    x = u
                    seen = set()
                    while x is not None:
                        au.append(x)
                        seen.add(x)
                        x = parent[x]
                    cyc = []
                    x = w
                    while x is not None and x not in seen:
                        cyc.append(x)
                        x = parent[x]
                    lca = x
                    path_u = []
                    for a in au:
                        path_u.append(a)
                        if a == lca:
                            break
                    return path_u + cyc
    return None


def exact_oct(vals, adj, pre, budget=40_000):
    """Minimum vertices to delete so the graph 2-colours respecting `pre`.

    Exact whenever the branch-and-bound finishes within `budget` node
    expansions (covers cliques and all small/moderate graphs); if the budget is
    exhausted on a large, far-from-clique graph it returns the chain lower bound
    `clique - 2 <= OCT`, which keeps the result an admissible *lower* bound.  The
    boolean is False in that fallback case.

    Returns ``(value, exact)``.
    """
    vals = list(vals)
    if not vals:
        return 0, True
    # augmented adjacency with pre-colour terminals
    aug = {v: set(adj[v]) for v in vals}
    aug[T0] = set()
    aug[T1] = set()

    def link(a, b):
        aug[a].add(b)
        aug[b].add(a)

    link(T0, T1)
    for v in vals:
        c = pre.get(v)
        if c == 0:
            link(v, T1)
        elif c == 1:
            link(v, T0)

    best = [len(vals)]           # trivial feasible solution: delete every card
    calls = [0]
    aborted = [False]

    def bb(cur, deleted, remaining):
        if aborted[0] or deleted >= best[0]:
            return
        calls[0] += 1
        if calls[0] > budget:
            aborted[0] = True
            return
        lb = deleted + max(0, _clique(remaining, adj) - 2)
        if lb >= best[0]:
            return
        oc = _bipartite_or_odd_cycle(cur)
        if oc is None:
            best[0] = deleted
            return
        cands = [v for v in oc if v != T0 and v != T1]
        if not cands:
            return
        for v in cands:
            sub = {x: (cur[x] - {v}) for x in cur if x != v}
            bb(sub, deleted + 1, remaining - {v})

    bb(aug, 0, set(vals))
    if aborted[0]:
        return max(0, _clique(set(vals), adj) - 2), False
    return best[0], True
