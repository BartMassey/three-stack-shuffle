"""Constructive no-reversal merge sorters for the operation-count model.

Each sorter takes a deck (a permutation written bottom-to-top, the
``machine.py`` convention: index 0 is the bottom, index -1 the top) and returns
the list of move names (``"SA"``/``"SB"``/``"MA"``/``"MB"``) that sort it into
``(1, ..., n)`` with both buffers empty.  Replaying the returned moves on a
fresh machine (``machine.apply_moves``) reproduces the sort -- that replay is
what makes "machine-verified" true (see ``tests/test_sorters.py``).

Three sorters, all *adjacent-merge* (they only ever merge two adjacent sorted
runs), differing only in the binary **merge tree** they build over the runs:

    natural_sort   multi-pass natural merge      cost = 2n*ceil(log2 r)
    topdown_sort   adaptive top-down tree        cost = 2*W(tree)
    hutucker_sort  optimal alphabetic tree       cost = 2*W(tree), W minimal

where ``r`` = number of maximal ascending runs and ``W(T) = sum_i s_i*depth_i``
is the weighted external path length (leaf weights = run sizes).  See
``docs/sources/SORTING-BOUNDS.md`` and ``operation-count-theory.md`` (Part I.3).

The realizations keep runs **ascending bottom-to-top (max on top)** on the deck;
a pour onto a buffer reverses a run to min-on-top, and a count-bounded merge
pours two such runs back ascending.  Merges are by *exact count*, never by
empty-stack test, so a merge never digs into an inert run parked beneath it.
"""
import math


def ascending_runs(deck):
    """Sizes of the maximal ascending (bottom-to-top) runs of ``deck``."""
    deck = list(deck)
    if not deck:
        return []
    sizes = []
    cur = 1
    for i in range(1, len(deck)):
        if deck[i] > deck[i - 1]:
            cur += 1
        else:
            sizes.append(cur)
            cur = 1
    sizes.append(cur)
    return sizes


class _Emitter:
    """A tiny live machine: applies moves to ``(D, A, B)`` while recording them,
    so a sorter can decide each merge step from the real top-of-stack values."""

    def __init__(self, deck):
        self.D = list(deck)
        self.A = []
        self.B = []
        self.moves = []

    def SA(self):
        self.A.append(self.D.pop())
        self.moves.append("SA")

    def SB(self):
        self.B.append(self.D.pop())
        self.moves.append("SB")

    def MA(self):
        self.D.append(self.A.pop())
        self.moves.append("MA")

    def MB(self):
        self.D.append(self.B.pop())
        self.moves.append("MB")

    def _merge_AB(self, n_from_a, n_from_b):
        """Merge the top ``n_from_a`` cards of A with the top ``n_from_b`` of B
        back onto D.  Both runs have their minimum on top, so taking the smaller
        top first emits an ascending run (max on top) of ``n_from_a+n_from_b``
        cards, touching nothing parked deeper."""
        while n_from_a and n_from_b:
            if self.A[-1] < self.B[-1]:
                self.MA()
                n_from_a -= 1
            else:
                self.MB()
                n_from_b -= 1
        while n_from_a:
            self.MA()
            n_from_a -= 1
        while n_from_b:
            self.MB()
            n_from_b -= 1


# ---------------------------------------------------------------------------
# Algorithm 1: multi-pass natural merge.  cost = 2n*ceil(log2 r).
# ---------------------------------------------------------------------------

def natural_sort(deck):
    """Bottom-up two-way natural merge in synchronous passes.  Returns the move
    list; cost is exactly ``2n*ceil(log2 r)`` (``r`` = ascending runs)."""
    m = _Emitter(deck)
    runs = ascending_runs(m.D)
    while len(runs) > 1:
        # distribute: pop runs off the top of D, alternating A, B
        a_runs, b_runs, to_a = [], [], True
        for L in reversed(runs):              # topmost run first
            if to_a:
                for _ in range(L):
                    m.SA()
                a_runs.append(L)
            else:
                for _ in range(L):
                    m.SB()
                b_runs.append(L)
            to_a = not to_a
        # merge: pair the top run of A with the top run of B
        new_runs = []
        while a_runs and b_runs:
            la = a_runs.pop()
            lb = b_runs.pop()
            m._merge_AB(la, lb)
            new_runs.append(la + lb)
        for L in reversed(a_runs):            # odd leftover on A (only one side)
            for _ in range(L):
                m.MA()
            new_runs.append(L)
        for L in reversed(b_runs):
            for _ in range(L):
                m.MB()
            new_runs.append(L)
        runs = new_runs
    return m.moves


# ---------------------------------------------------------------------------
# Tree sorters: build a merge tree over the runs, then realize it (cost 2*W).
# A tree is ('leaf', size) or ('node', left, right, size).
# ---------------------------------------------------------------------------

def _tree_size(t):
    return t[1] if t[0] == "leaf" else t[3]


def weighted_path_length(tree):
    """``W(T) = sum_i size_i * depth_i`` (root at depth 0)."""
    def rec(t, depth):
        if t[0] == "leaf":
            return t[1] * depth
        return rec(t[1], depth + 1) + rec(t[2], depth + 1)
    return rec(tree, 0)


def build_topdown(run_sizes):
    """Adaptive top-down tree: split the run sequence at the boundary nearest
    the card midpoint (long runs stay shallow)."""
    pre = [0]
    for s in run_sizes:
        pre.append(pre[-1] + s)

    def build(i, j):                          # runs [i, j)
        if j - i == 1:
            return ("leaf", run_sizes[i])
        target = (pre[i] + pre[j]) / 2
        best_k, best_d = i + 1, None
        for k in range(i + 1, j):
            d = abs(pre[k] - target)
            if best_d is None or d < best_d:
                best_d, best_k = d, k
        left, right = build(i, best_k), build(best_k, j)
        return ("node", left, right, pre[j] - pre[i])

    if not run_sizes:
        return ("leaf", 0)
    return build(0, len(run_sizes))


def build_hutucker(run_sizes):
    """Optimal alphabetic (order-constrained) merge tree via the optimal-BST
    DP: ``C[i][j]`` = min weight spanning runs ``i..j``.  ``O(r^3)``; this is
    the minimum-``W`` tree, so ``hutucker_sort`` is the optimal no-reversal
    adjacent-merge sorter."""
    r = len(run_sizes)
    if r == 0:
        return ("leaf", 0)
    if r == 1:
        return ("leaf", run_sizes[0])
    pre = [0]
    for s in run_sizes:
        pre.append(pre[-1] + s)
    INF = float("inf")
    C = [[0] * r for _ in range(r)]
    K = [[i for _ in range(r)] for i in range(r)]
    for length in range(2, r + 1):
        for i in range(0, r - length + 1):
            j = i + length - 1
            w = pre[j + 1] - pre[i]
            best, bk = INF, i
            for k in range(i, j):
                v = C[i][k] + C[k + 1][j]
                if v < best:
                    best, bk = v, k
            C[i][j] = best + w
            K[i][j] = bk

    def build(i, j):                          # inclusive runs i..j
        if i == j:
            return ("leaf", run_sizes[i])
        k = K[i][j]
        left, right = build(i, k), build(k + 1, j)
        return ("node", left, right, pre[j + 1] - pre[i])

    return build(0, r - 1)


def realize_tree(deck, tree):
    """Emit the moves that sort ``deck`` by realizing ``tree`` (cost ``2*W``).

    On entry to a node its run-block occupies the top of D, with the right
    (upper-index) child on top.  Sort the right child, park it onto A; sort the
    left child, park it onto B; count-merge the two parked runs back to D.  Each
    node spends ``2 * (cards in node)`` moves, summing to ``2*W``."""
    m = _Emitter(deck)

    def rec(t):
        if t[0] == "leaf":
            return                            # a single ascending run already on D
        _, left, right, _ = t
        rec(right)                            # sort the upper (top-of-D) child
        for _ in range(_tree_size(right)):    # park it onto A (reverses)
            m.SA()
        rec(left)                             # sort the lower child (now on top)
        for _ in range(_tree_size(left)):     # park it onto B
            m.SB()
        m._merge_AB(_tree_size(right), _tree_size(left))

    rec(tree)
    return m.moves


def topdown_sort(deck):
    """Adaptive top-down merge sort; returns the move list (cost ``2*W``)."""
    return realize_tree(deck, build_topdown(ascending_runs(deck)))


def hutucker_sort(deck):
    """Optimal Hu-Tucker merge sort; returns the move list (cost ``2*W``, ``W``
    minimal over all order-preserving merge trees)."""
    return realize_tree(deck, build_hutucker(ascending_runs(deck)))


# ---------------------------------------------------------------------------
# Closed-form costs (no machine replay needed), for cross-checking.
# ---------------------------------------------------------------------------

def natural_cost(deck):
    """``2n*ceil(log2 r)`` -- the exact natural_sort move count."""
    n = len(deck)
    r = len(ascending_runs(deck))
    return 0 if r <= 1 else 2 * n * math.ceil(math.log2(r))


def topdown_cost(deck):
    return 2 * weighted_path_length(build_topdown(ascending_runs(deck)))


def hutucker_cost(deck):
    return 2 * weighted_path_length(build_hutucker(ascending_runs(deck)))
