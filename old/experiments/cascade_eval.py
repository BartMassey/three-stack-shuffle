"""Evaluate the inadmissible "cascading charge" (`planner.cascade_charge`) as a
heuristic -- the idea from the project notes that a greedy cascade simulation of
the forced bounces would be *tighter* than the static pairwise bound `h_joint`.

The charge equals the settle-rollout's length (`h0 + 2*cascade_bounces`). This
script measures, on random start decks where exact search is feasible (n <= 11):

  * `h_joint` gap = opt - h_joint    (>= 0; the admissible bound undershoots),
  * cascade overshoot = cascade - opt (> 0 means it overshoots opt, i.e. is
    inadmissible), and how often it overshoots.

Finding: the cascade *overshoots* opt by an amount that grows with n and does so
almost always, while `h_joint` sits within ~1-2 below opt -- so the cascade is
**much looser, not tighter**: the greedy cascade overcounts bounces relative to
the optimal interleaving. (It is also no better as a search steering signal; see
`planner.local_search(..., estimate=cascade_charge)`.)

Run:  python -m experiments.cascade_eval [max_n] [num_decks]
"""
import random
import statistics
import sys

from splitmerge.search import ida_star
from splitmerge.heuristics import h_joint
from splitmerge.planner import cascade_charge


def run(max_n=11, num=40, seed=4):
    rng = random.Random(seed)
    print(f"random start decks, {num} per n  (opt by IDA*):")
    print("  n :  h_joint_gap(opt-LB)   cascade_overshoot(cas-opt)   max   overshoots")
    for n in range(6, max_n + 1):
        hj_gap, over_gap, over = [], [], 0
        for _ in range(num):
            p = list(range(1, n + 1))
            rng.shuffle(p)
            st = (tuple(p), (), ())
            opt, _ = ida_star(st, h_joint)
            hj_gap.append(opt - h_joint(st))
            cas = cascade_charge(st)
            over_gap.append(cas - opt)
            over += cas > opt
        print(f"  {n:2d}:      {statistics.mean(hj_gap):5.2f}                "
              f"{statistics.mean(over_gap):6.2f}              {max(over_gap):3d}    "
              f"{over}/{num}")


if __name__ == "__main__":
    max_n = int(sys.argv[1]) if len(sys.argv) > 1 else 11
    num = int(sys.argv[2]) if len(sys.argv) > 2 else 40
    run(max_n, num)
