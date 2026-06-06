"""Search over the move graph: exact BFS (small ``n``) and IDA* driven by an
admissible heuristic.
"""
import sys
from collections import deque

from .machine import GOAL, succ, neighbors, size, INV

sys.setrecursionlimit(100_000)


def bfs_dist(n):
    """Exact distance-to-goal for every reachable state with ``n`` cards, by
    BFS backward from the sorted goal.  Returns ``{state: distance}``.

    Only feasible for small ``n`` (state count grows ~ ``3^n``); used for
    verification, not production search.
    """
    goal = GOAL(n)
    dist = {goal: 0}
    q = deque([goal])
    while q:
        s = q.popleft()
        for t in neighbors(s):
            if t not in dist:
                dist[t] = dist[s] + 1
                q.append(t)
    return dist


def ida_star(start, heuristic, node_cap=2_000_000):
    """IDA* from ``start`` to the sorted goal, using ``heuristic`` (must be
    admissible for the returned cost to be optimal).

    Uses parent-move pruning, on-path cycle detection, and pathmax -- the
    latter because the strong heuristics here are admissible but *not*
    consistent (HEURISTIC-BOUNDS.md, section 13), and pathmax repairs
    consistency along each search path.

    Returns ``(cost, nodes_expanded)``; ``cost`` is ``None`` if the node cap
    was hit before a solution was found.
    """
    n = size(start)
    goal = GOAL(n)
    nodes = [0]
    on_path = {start}

    def dfs(s, g, bound, last_move, h_s):
        f = g + h_s
        if f > bound:
            return f
        if s == goal:
            return -1                      # found
        nodes[0] += 1
        if nodes[0] > node_cap:
            return -2                      # gave up
        best = float("inf")
        for t, mv in succ(s):
            if last_move and mv == INV[last_move]:
                continue                   # don't undo the previous move
            if t in on_path:
                continue                   # no cycles on the current path
            h_t = heuristic(t)
            ph = h_t if h_t > h_s - 1 else h_s - 1     # pathmax
            if g + 1 + ph > bound:
                best = min(best, g + 1 + ph)
                continue
            on_path.add(t)
            r = dfs(t, g + 1, bound, mv, ph)
            on_path.discard(t)
            if r == -1:
                return -1
            if r == -2:
                return -2
            best = min(best, r)
        return best

    bound = heuristic(start)
    while True:
        r = dfs(start, 0, bound, None, bound)
        if r == -1:
            return bound, nodes[0]
        if r in (-2, float("inf")):
            return None, nodes[0]
        bound = r
