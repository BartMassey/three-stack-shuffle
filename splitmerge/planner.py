"""An anytime, *inadmissible* local-search planner (the I.6 #5 direction).

Unlike the heuristics in `heuristics.py` (admissible *lower* bounds, good for
proving optimality but weak for steering), this module estimates the *actual*
completion cost with a deterministic rollout policy and uses it to steer a
greedy local search. The estimate is an achievable upper bound on `g`, so it is
tight when the policy is good — exactly the kind of inadmissible signal that
steers well — but it must **not** be used as a lower bound.

`rollout(state)` is a complete deterministic sorter from *any* state (not just a
clean deck), so its length is a real solution cost and the planner always has a
solution in hand (the anytime property). It emits real machine moves, verified
by replay in `tests/test_planner.py`.

Rollout policy ("settle the next card"): repeatedly settle `k+1` where `k` is the
committed base. Drain the above-base deck into the two buffers by a 2-pile
patience rule (place each card on the buffer whose top is the smallest value
still above it, else open the empty pile, else accept a bury on the closer top);
expose `k+1` (move the cards burying it in its buffer across to the other buffer,
via the deck); settle it; then pour back every buffer card that is now
next-in-order. Each pass settles at least one card, so it terminates in ≤ n
passes. The blocker-routing choice is the main quality lever (see HANDOFF §4a).
"""
import random
import time

from .machine import GOAL, base_len, size, succ
from .sorters import hutucker_sort


def rollout(state):
    """A deterministic completion: return the list of moves that sorts ``state``.
    Works from any ``(D, A, B)``; its length is an achievable cost for ``g``."""
    D, A, B = ([list(s) for s in state])
    n = len(D) + len(A) + len(B)
    moves = []

    def SA():
        A.append(D.pop()); moves.append("SA")

    def SB():
        B.append(D.pop()); moves.append("SB")

    def MA():
        D.append(A.pop()); moves.append("MA")

    def MB():
        D.append(B.pop()); moves.append("MB")

    def pour_back():
        while True:
            nxt = base_len(D) + 1
            if nxt > n:
                break
            if A and A[-1] == nxt:
                MA()
            elif B and B[-1] == nxt:
                MB()
            else:
                break

    def route_top():
        """Pour D's top card onto a buffer by the 2-pile patience rule."""
        c = D[-1]
        ta = A[-1] if A else None
        tb = B[-1] if B else None
        a_ok = ta is not None and ta > c
        b_ok = tb is not None and tb > c
        if a_ok and b_ok:
            SA() if ta <= tb else SB()          # smallest top still above c
        elif a_ok:
            SA()
        elif b_ok:
            SB()
        elif not A:
            SA()                                # open the empty pile
        elif not B:
            SB()
        else:
            SA() if ta >= tb else SB()          # forced bury: the closer (larger) top

    pour_back()
    while base_len(D) < n:
        k = base_len(D)
        target = k + 1
        # 1. drain the above-base deck into the buffers (no-op after the first pass)
        while len(D) > k:
            route_top()
        # 2. expose target: it is now in a buffer; lift the cards burying it across
        #    to the other buffer, routing each via the deck (buffers don't touch).
        src, dst, m_src, s_dst = (A, B, MA, SB) if target in A else (B, A, MB, SA)
        while src[-1] != target:
            m_src()          # blocker: buffer -> deck (lands above the base)
            s_dst()          # deck -> other buffer
        # 3. settle target, then pour back the run it unlocks
        m_src()
        pour_back()
    return moves


def rollout_merge(state):
    """A second deterministic completion: pour both buffers back to the deck, then
    sort with the optimal Hu-Tucker merge sorter. Mirror image of ``rollout`` — it
    is good on shuffled decks (where merge sort is near-optimal) and bad on the
    reversal (where merge sort spends ``3n``), whereas ``rollout`` is the
    opposite. Returns the move list."""
    D, A, B = ([list(s) for s in state])
    moves = []
    while A:
        D.append(A.pop()); moves.append("MA")
    while B:
        D.append(B.pop()); moves.append("MB")
    return moves + hutucker_sort(D)


def rollout_cost(state):
    """Length of the settle-next-card completion (an upper bound on ``g``)."""
    return len(rollout(state))


def completion(state):
    """The cheaper of the two deterministic completions — the tight inadmissible
    estimate the planner steers by. Returns ``(moves, cost)``; ``moves`` always
    sorts ``state``."""
    a = rollout(state)
    b = rollout_merge(state)
    return (a, len(a)) if len(a) <= len(b) else (b, len(b))


def completion_cost(state):
    return completion(state)[1]


def greedy_solution(start, rng=None, epsilon=0.0, step_cap=None):
    """One local-search descent: from ``start`` repeatedly step to the successor
    minimizing the rollout estimate (with probability ``epsilon`` take the
    runner-up, for restart diversity); finish from wherever it stalls by running
    the rollout, so the returned move list always sorts ``start``.

    Returns ``(moves, cost)``."""
    n = size(start)
    goal = GOAL(n)
    if step_cap is None:
        step_cap = 8 * n
    s = start
    moves = []
    visited = {start}
    steps = 0
    while s != goal and steps < step_cap:
        cands = []
        for t, mv in succ(s):
            if t in visited:
                continue
            cands.append((completion_cost(t), t, mv))
        if not cands:
            break
        cands.sort(key=lambda x: x[0])
        pick = cands[0]
        if rng is not None and epsilon and len(cands) > 1 and rng.random() < epsilon:
            pick = cands[1]
        _, s, mv = pick
        moves.append(mv)
        visited.add(s)
        steps += 1
    moves += completion(s)[0]                    # guaranteed completion (cheaper of the two)
    return moves, len(moves)


def local_search(start, time_budget=10.0, seed=0):
    """Anytime planner: the first solution is the plain rollout; then keep running
    perturbed greedy descents and keep the best complete solution, until the wall-
    clock budget elapses. Returns
    ``(best_moves, best_cost, first_cost, first_time, n_restarts)``."""
    rng = random.Random(seed)
    t0 = time.time()
    first_moves, first_cost = completion(start)
    first_time = time.time() - t0
    best_moves, best_cost = first_moves, first_cost
    restarts = 0
    while time.time() - t0 < time_budget:
        eps = 0.0 if restarts == 0 else 0.3      # first descent is pure greedy
        moves, cost = greedy_solution(start, rng=rng, epsilon=eps)
        restarts += 1
        if cost < best_cost:
            best_moves, best_cost = moves, cost
    return best_moves, best_cost, first_cost, first_time, restarts
