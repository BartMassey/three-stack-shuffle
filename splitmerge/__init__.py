"""splitmerge -- the three-stack split-merge card-sorting machine.

Two cost models of one machine (kept separate; see ``docs/NOTES.md``):

* **operation count** -- ``machine``, ``heuristics``, ``oct``, ``search`` (exact
  lower bounds + IDA*), and ``sorters`` (constructive merge sorters).
* **whole cycle** -- ``cycle`` (permutation distance over 123-avoiding cycles).
"""
from .machine import (
    GOAL,
    INV,
    apply_moves,
    base_len,
    comb_solution,
    neighbors,
    reversed_deck,
    size,
    succ,
)
from .search import bfs_dist, ida_star
from .heuristics import h0, h_best, h_joint
from .sorters import (
    ascending_runs,
    hutucker_cost,
    hutucker_sort,
    natural_cost,
    natural_sort,
    topdown_cost,
    topdown_sort,
)
from .cycle import (
    cycle_diameter,
    cycle_distances,
    f,
    generators,
    lis,
    one_cycle_ok,
    rel,
)
from .planner import (
    completion,
    greedy_solution,
    local_search,
    rollout,
    rollout_merge,
)

__all__ = [
    # machine
    "GOAL", "INV", "apply_moves", "base_len", "comb_solution", "neighbors",
    "reversed_deck", "size", "succ",
    # search
    "bfs_dist", "ida_star",
    # heuristics (admissible lower bounds)
    "h0", "h_best", "h_joint",
    # constructive merge sorters (operation-count model)
    "ascending_runs", "natural_sort", "topdown_sort", "hutucker_sort",
    "natural_cost", "topdown_cost", "hutucker_cost",
    # whole-cycle model
    "cycle_diameter", "cycle_distances", "f", "generators", "lis",
    "one_cycle_ok", "rel",
    # inadmissible local-search planner
    "rollout", "rollout_merge", "completion", "greedy_solution", "local_search",
]
