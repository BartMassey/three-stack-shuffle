"""The whole-cycle (permutation-distance) model.

A *cycle* drains the whole deck into the two buffers (``n`` splits) then merges
everything back (``n`` merges); cost is the number of cycles.  This is a
restricted operation schedule and a different cost measure from the
operation-count model in the rest of the package -- keep them separate (see
``docs/sources/cycle-model-theory.md``).

Core facts realized here:

* **One cycle = two reversed subsequences interleaved.**  Splitting 2-colours the
  deck; each colour class is reversed by its stack; the merge freely interleaves
  the two reversed streams.
* **Reachability test.**  ``e`` is reachable from ``d`` in one cycle iff
  ``LIS(d^{-1} e) <= 2`` (the relative permutation is 123-avoiding).  So the
  reachability graph is the undirected Cayley graph of ``S_n`` with the
  Catalan-many ``{sigma : LIS(sigma) <= 2}`` as generators, and ``f(pi)`` (cycles
  from the identity) is the word length there.

Permutations are 0-indexed tuples; the identity is ``(0, 1, ..., n-1)``.
"""
import bisect
import itertools
from collections import deque


def lis(seq):
    """Length of the longest strictly-increasing subsequence."""
    tails = []
    for x in seq:
        i = bisect.bisect_left(tails, x)
        if i == len(tails):
            tails.append(x)
        else:
            tails[i] = x
    return len(tails)


def rel(d, e):
    """The relative permutation ``d^{-1} e`` (relabel each card by its index in
    ``d``, then read ``e``)."""
    pos = {c: i for i, c in enumerate(d)}
    return tuple(pos[x] for x in e)


def one_cycle_ok(d, e):
    """Is ``e`` reachable from ``d`` in exactly one cycle?  (``LIS(d^{-1}e) <= 2``.)"""
    return lis(rel(d, e)) <= 2


def generators(n):
    """The generating set ``C = {sigma in S_n : LIS(sigma) <= 2}`` (the
    123-avoiding permutations; ``|C| = Catalan(n)``)."""
    return [p for p in itertools.permutations(range(n)) if lis(p) <= 2]


def _compose(g, c):
    """``(g . c)(i) = g[c[i]]`` -- the deck reached from ``g`` by the cycle ``c``."""
    return tuple(g[c[i]] for i in range(len(g)))


def one_cycle_neighbors_bruteforce(d):
    """The one-cycle reachable set of deck ``d``, by direct construction: over
    every 2-colouring (subset to buffer A, complement to B), reverse each colour
    class and enumerate all order-preserving interleavings.  The cross-check
    oracle for :func:`one_cycle_ok` (correct but exponential)."""
    n = len(d)
    reach = set()
    for bits in range(1 << n):
        a = [d[i] for i in range(n) if (bits >> i) & 1]
        b = [d[i] for i in range(n) if not (bits >> i) & 1]
        ra, rb = a[::-1], b[::-1]               # each stack reverses its class
        # all interleavings of ra and rb (choose positions of ra)
        for pos in itertools.combinations(range(n), len(ra)):
            out = [None] * n
            pset = set(pos)
            ia = ib = 0
            for k in range(n):
                if k in pset:
                    out[k] = ra[ia]
                    ia += 1
                else:
                    out[k] = rb[ib]
                    ib += 1
            reach.add(tuple(out))
    return reach


def cycle_distances(n):
    """``{permutation: f}`` for all of ``S_n`` by BFS in the Cayley graph
    (neighbours of ``g`` are ``g . c`` for each generator ``c``).  Returns the
    distance-from-identity dict."""
    C = generators(n)
    idt = tuple(range(n))
    dist = {idt: 0}
    q = deque([idt])
    while q:
        g = q.popleft()
        for c in C:
            h = _compose(g, c)
            if h not in dist:
                dist[h] = dist[g] + 1
                q.append(h)
    return dist


def f(pi):
    """Minimum cycles to reach ``pi`` from the identity (word length over the
    123-avoiding generators).  ``pi`` is a 0-indexed permutation tuple."""
    return cycle_distances(len(pi))[tuple(pi)]


def cycle_diameter(n):
    """``D(n) = max_pi f(pi)`` -- the eccentricity of the identity."""
    return max(cycle_distances(n).values())
