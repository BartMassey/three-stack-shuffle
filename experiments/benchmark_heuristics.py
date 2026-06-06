"""Head-to-head: does h_joint's tighter bound pay for its heavier per-node
cost?  Counts node expansions and wall-clock for h_best vs h_joint on the same
random decks.

Run:  python -m experiments.benchmark_heuristics [n] [num_decks]
"""
import random
import sys
import time

from splitmerge.search import ida_star
from splitmerge.heuristics import h_best, h_joint


def run(n=10, num_decks=80, seed=11, node_cap=600_000):
    rng = random.Random(seed)
    decks = []
    for _ in range(num_decks):
        p = list(range(1, n + 1))
        rng.shuffle(p)
        decks.append((tuple(p), (), ()))

    results = {}
    for name, h in (("h_best", h_best), ("h_joint", h_joint)):
        t0 = time.time()
        nodes, costs = [], []
        for st in decks:
            c, nd = ida_star(st, h, node_cap=node_cap)
            if c is not None:
                nodes.append(nd)
                costs.append(c)
        results[name] = (sorted(nodes), costs, time.time() - t0)

    print(f"n={n}, {num_decks} random decks")
    for name in ("h_best", "h_joint"):
        nodes, _, dt = results[name]
        med = nodes[len(nodes) // 2]
        print(f"  {name:8s}: median={med:6d} max={max(nodes):7d} "
              f"total={sum(nodes):8d} time={dt:6.1f}s")
    nb = sum(results["h_best"][0])
    nj = sum(results["h_joint"][0])
    print(f"  node reduction: {100 * (1 - nj / nb):.1f}%   "
          f"costs identical: {results['h_best'][1] == results['h_joint'][1]}")


if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 10
    num = int(sys.argv[2]) if len(sys.argv) > 2 else 80
    run(n, num)
