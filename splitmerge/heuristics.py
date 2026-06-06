"""Admissible heuristics for the split-merge machine.

All heuristics have the form ``h = h0 + 2 * (forced bounces)``, where a *bounce*
is a card forced to enter the deck more than once.  Each move handles one card,
so the number of moves is the base cost plus two per forced bounce
(docs/NOTES.md, §I.1 and §I.4).  Three heuristics, increasing strength:

    h0      base charge only: 2*(above-base deck) + |A| + |B|
    h_best  + 2*max((LIS-2)+ buried,  clique-2)      -- the "max" version
    h_joint + 2*(buried + OCT_pre)                   -- the joint version

``h_joint`` dominates ``h_best`` dominates ``h0`` everywhere, and all three are
admissible (verified by full BFS for n <= 7 in the test suite).
"""
import bisect
import itertools
from collections import deque
from functools import lru_cache

from .machine import base_len
from .oct import exact_oct


def _lis_len(seq):
    """Length of the longest strictly-increasing subsequence."""
    tails = []
    for x in seq:
        i = bisect.bisect_left(tails, x)
        if i == len(tails):
            tails.append(x)
        else:
            tails[i] = x
    return len(tails)


def _buried_set(A, B):
    """Cards that are *buried*: a smaller card sits below them in the same
    buffer, so they must lift off before it settles -> they must bounce
    (docs/NOTES.md, §I.4).
    """
    out = set()
    for buf in (A, B):
        for i, x in enumerate(buf):
            if any(y < x for y in buf[:i]):
                out.add(x)
    return out


def h0(state):
    """Base charge: each above-base deck card costs >= 2 (evacuate + return),
    each buffer card >= 1 (return).  Provably admissible; exact on rotations.
    """
    D, A, B = state
    k = base_len(D)
    return 2 * (len(D) - k) + len(A) + len(B)


def _clique_chain(D, A, B, k):
    """Size of the largest mutual-conflict clique, computed as the longest
    chain of the conflict comparability graph (edges orient smaller-value ->
    larger).  A clique of size c forces c-2 bounces (only 2 buffers).
    """
    # node = (value, kind, position); kind 0 = above-base deck (in pi order),
    # 1 = buffer A, 2 = buffer B.
    nodes = []
    pi = list(reversed(D[k:]))
    for pos, v in enumerate(pi):
        nodes.append((v, 0, pos))
    for d, v in enumerate(A):
        nodes.append((v, 1, d))
    for d, v in enumerate(B):
        nodes.append((v, 2, d))
    N = len(nodes)
    if N == 0:
        return 0

    def edge(i, j):
        vi, ki, xi = nodes[i]
        vj, kj, xj = nodes[j]
        if ki == 0 and kj == 0:                    # deck-deck: increasing in pi
            return (vi < vj) if xi < xj else (vj < vi)
        if ki == 0 and kj >= 1:                    # deck vs buffer: buffer smaller
            return vj < vi
        if kj == 0 and ki >= 1:
            return vi < vj
        if ki >= 1 and kj >= 1:                    # same buffer: deeper-smaller
            if ki != kj:
                return False
            return (vi < vj) if xi < xj else (vj < vi)
        return False

    order = sorted(range(N), key=lambda i: nodes[i][0])
    best = [1] * N
    ans = 0
    for a in range(N):
        i = order[a]
        for b in range(a):
            j = order[b]
            if edge(i, j) and best[j] + 1 > best[i]:
                best[i] = best[j] + 1
        ans = max(ans, best[i])
    return ans


@lru_cache(maxsize=None)
def h_best(state):
    """The "max" heuristic: ``h0 + 2*max((LIS-2)+buried, clique-2)``
    (docs/NOTES.md, §I.4)."""
    D, A, B = state
    k = base_len(D)
    pi = tuple(reversed(D[k:]))
    disjoint = max(0, _lis_len(pi) - 2) + len(_buried_set(A, B))
    clique = _clique_chain(D, A, B, k)
    return h0(state) + 2 * max(disjoint, max(0, clique - 2))


def _two_colourable(verts, adj, pre):
    """Is the graph ``(verts, adj)`` properly 2-colourable while respecting the
    fixed colours in ``pre``?  BFS propagation, seeding from the pre-coloured
    vertices first (so a free vertex never grabs a colour its pre-coloured
    neighbour will contradict)."""
    colour = {v: pre[v] for v in pre if v in set(verts)}
    dq = deque(colour.keys())
    while dq:                                  # propagate forced colours first
        u = dq.popleft()
        for w in adj[u]:
            if w in colour:
                if colour[w] == colour[u]:
                    return False
            else:
                colour[w] = 1 - colour[u]
                dq.append(w)
    for v in verts:                            # then free components
        if v in colour:
            continue
        colour[v] = 0
        dq = deque([v])
        while dq:
            u = dq.popleft()
            for w in adj[u]:
                if w in colour:
                    if colour[w] == colour[u]:
                        return False
                else:
                    colour[w] = 1 - colour[u]
                    dq.append(w)
    return True


def _soft_conflict_graph(state):
    """Build the soft-conflict graph (docs/NOTES.md, §I.4).

    Vertices: non-buried buffer cards (pre-coloured by their buffer) and all
    above-base deck cards (free).  A *soft* edge joins two cards that could go
    to different buffers but cannot share one if both are single-arrival:
    deck-deck increasing in pi, and deck ``d`` vs non-buried buffer card
    ``q < d``.  (Same-buffer pairs are handled separately by the buried count.)

    Returns ``(values, adjacency, pre_colour, n_buried)``.
    """
    D, A, B = state
    k = base_len(D)
    buried = _buried_set(A, B)
    pi = list(reversed(D[k:]))
    nodes = []                                 # (value, kind); kind 0=A, 1=B, 2=deck
    pre = {}
    for v in A:
        if v not in buried:
            nodes.append((v, 0))
            pre[v] = 0
    for v in B:
        if v not in buried:
            nodes.append((v, 1))
            pre[v] = 1
    for pos, v in enumerate(pi):
        nodes.append((v, 2))
    vals = [v for v, _ in nodes]
    adj = {v: set() for v in vals}
    # deck cards are appended in pi order; recover their pi position by index
    deck_pos = {}
    pos = 0
    for v, kind in nodes:
        if kind == 2:
            deck_pos[v] = pos
            pos += 1
    for i in range(len(nodes)):
        for j in range(i + 1, len(nodes)):
            vi, ki = nodes[i]
            vj, kj = nodes[j]
            if ki == 2 and kj == 2:
                e = (vi < vj) if deck_pos[vi] < deck_pos[vj] else (vj < vi)
            elif ki == 2 and kj < 2:
                e = vj < vi
            elif kj == 2 and ki < 2:
                e = vi < vj
            else:
                e = False
            if e:
                adj[vi].add(vj)
                adj[vj].add(vi)
    return vals, adj, pre, len(buried)


def _oct_pre(vals, adj, pre):
    """Minimum vertices to delete so the graph 2-colours respecting ``pre`` --
    the exact constrained OCT (see ``oct.exact_oct``), via odd-cycle
    branch-and-bound with the comparability chain bound (clique - 2 <= OCT)
    pruning the clique case.  Exact within a search budget; on a large
    far-from-clique graph it degrades gracefully to the admissible chain lower
    bound.  Validated against ``_oct_pre_bruteforce`` on all n <= 7 states.
    """
    value, _exact = exact_oct(vals, adj, pre)
    return value


def _oct_pre_bruteforce(vals, adj, pre):
    """Reference OCT by exhaustive deletion search -- the test oracle.  Correct
    but exponential; not used in production (see ``_oct_pre``)."""
    for d in range(len(vals) + 1):
        for drop in itertools.combinations(vals, d):
            dropped = set(drop)
            kept = [v for v in vals if v not in dropped]
            sub = {v: {w for w in adj[v] if w not in dropped} for v in kept}
            pre_kept = {v: pre[v] for v in kept if v in pre}
            if _two_colourable(kept, sub, pre_kept):
                return d
    return len(vals)


@lru_cache(maxsize=None)
def h_joint(state):
    """The joint heuristic: ``h0 + 2*(buried + OCT_pre)``
    (docs/NOTES.md, §I.4).  Eliminates the ``max`` of ``h_best`` by
    bounding the single-arrival set directly, and dominates ``h_best``.
    """
    vals, adj, pre, n_buried = _soft_conflict_graph(state)
    return h0(state) + 2 * (n_buried + _oct_pre(vals, adj, pre))
