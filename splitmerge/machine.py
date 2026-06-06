"""The three-stack split-merge machine.

A *state* is a triple of tuples ``(D, A, B)`` where each tuple is a stack with
its TOP at the right (the last element).  ``D`` is the deck (hub + I/O); ``A``
and ``B`` are the two buffers.  There is no direct A<->B transfer.

Four moves, two involutive inverse pairs:

    SA : D -> A      MA : A -> D
    SB : D -> B      MB : B -> D

Cost is the number of moves.  Start: deck in ``D``, buffers empty.  Goal:
``D == (1, 2, ..., n)`` (sorted bottom-to-top), buffers empty.
"""

INV = {"SA": "MA", "MA": "SA", "SB": "MB", "MB": "SB"}


def GOAL(n):
    """The sorted goal state for ``n`` cards."""
    return (tuple(range(1, n + 1)), (), ())


def size(state):
    """Total number of cards in a state."""
    D, A, B = state
    return len(D) + len(A) + len(B)


def base_len(D):
    """Length of the committed sorted base: largest ``k`` with
    ``D[:k] == (1, ..., k)`` at the bottom of the deck."""
    k = 0
    while k < len(D) and D[k] == k + 1:
        k += 1
    return k


def succ(state):
    """Successor states as ``(next_state, move_name)`` pairs."""
    D, A, B = state
    out = []
    if D:
        out.append(((D[:-1], A + (D[-1],), B), "SA"))
        out.append(((D[:-1], A, B + (D[-1],)), "SB"))
    if A:
        out.append(((D + (A[-1],), A[:-1], B), "MA"))
    if B:
        out.append(((D + (B[-1],), A, B[:-1]), "MB"))
    return out


def neighbors(state):
    """Successor states only (the move graph is undirected, so these are also
    the predecessors)."""
    return [t for t, _ in succ(state)]


def apply_moves(state, moves):
    """Apply a sequence of move names (e.g. ``["SA", "MB", ...]``; any suffix
    after the two-letter code is ignored) and return the resulting state."""
    D, A, B = ([list(s) for s in state])
    for mv in moves:
        t = mv[:2]
        if t == "SA":
            A.append(D.pop())
        elif t == "SB":
            B.append(D.pop())
        elif t == "MA":
            D.append(A.pop())
        elif t == "MB":
            D.append(B.pop())
        else:
            raise ValueError(f"unknown move {mv!r}")
    return (tuple(D), tuple(A), tuple(B))


def comb_solution(n):
    """An explicit solution that sorts the *reversed* deck ``(n, ..., 1)`` in
    exactly ``4(n-1)`` moves -- the constructive witness for the upper bound
    ``opt(reversed deck) <= 4(n-1)`` (docs/NOTES.md, §I.5).

    Returns the list of move names.
    """
    s = []
    for j in range(1, n):              # phase 1: cards 1..n-1 -> B          (n-1 moves)
        s.append(f"SB{j}")
    s.append(f"SA{n}")                 # phase 2: card n -> A                (1 move)
    for j in range(n - 1, 0, -1):      # phase 3: pour B back, re-stash      (n-1 + n-2 moves)
        s.append(f"MB{j}")
        if j >= 2:
            s.append(f"SA{j}")
    for j in range(2, n + 1):          # phase 4: pour A                     (n-1 moves)
        s.append(f"MA{j}")
    return s


def reversed_deck(n):
    """The hardest known start state: ``((n, n-1, ..., 1), (), ())``."""
    return (tuple(range(n, 0, -1)), (), ())
