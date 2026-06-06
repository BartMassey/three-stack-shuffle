# split-merge

Admissible heuristics and lower bounds for a three-stack LIFO card-sorting
machine. The machine has a deck `D` and two buffers `A`, `B`; four moves
(`SA`, `SB`, `MA`, `MB`) shuffle one card at a time between the deck and a
buffer. The goal is to sort a shuffled deck into `(1, 2, ..., n)` in the fewest
moves. The headline target is `n = 52`.

## What's known

- **Diameter is Θ(n log n)** (proven): a counting bound gives Ω(n log n), and a
  natural merge sort gives O(n log n). No linear-operation sorter exists. The
  machine is a 3-stack *star* (reusable hub), the k=3 regime in the
  stack-sorting literature.
- **The reversed deck needs exactly `4(n-1)` moves** (proven three ways: the
  `comb_solution` construction, the `h_joint` lower bound, and IDA* search). For
  n=52 that is **204**. This is the operation diameter for n ≤ 9 (BFS) and
  through n ≈ 11 (search) — but that is a *small-n coincidence*: `4(n-1)` is
  **not** the asymptotic diameter (it is linear; the diameter is Θ(n log n)).
  An earlier "M(n) = 4(n-1)" conjecture is **retracted**; see `docs/PAPER.md`.
- **Heuristics:** `h0 <= h_best <= h_joint`, all admissible lower bounds on the
  optimal move count (verified by full BFS for n <= 7). `h_joint` is the current
  best; in exact IDA* search it expands ~84% fewer nodes than `h_best` at n=10.
- **Complexity** of optimal sorting (P vs NP-hard) is open, as is the exact
  constant in Θ(n log n) and the exact diameter at finite n. At n=52 the diameter
  lies in **[204, 600]**: lower bound the reversed deck's exact optimum (204,
  itself strengthened up from the counting bound ⌈log₃ 52!⌉ = 143); upper bound
  the Hu–Tucker sorter's proven worst case (600). A typical shuffled deck costs
  ~484 with that sorter.

## Layout

    splitmerge/
      machine.py      states, moves, successors, base, comb_solution
      search.py       exact BFS (small n), IDA* (parametrized by heuristic)
      heuristics.py   h0, h_best, h_joint  (+ the bounce / OCT internals)
    tests/            admissibility (full BFS n<=7), dominance, reversal, IDA*==BFS
    experiments/      heuristic benchmark, M(n) conjecture search
    docs/
      PAPER.md              the consolidated narrative / paper basis (start here)
      HANDOFF.md            continuation note for Claude Code (void results + plan)
      sources/              the working write-ups PAPER.md synthesizes
        operation-count-theory.md   structural theory + merge sorters + literature
        SORTING-BOUNDS.md           merge-family algorithms and bounds
        HEURISTIC-BOUNDS.md         the heuristic / IDA* program
        cycle-model-theory.md       whole-cycle permutation distance
        original-notes.md           the seed notes

`docs/PAPER.md` is the coherent account of the whole project (both cost models,
all proven results, dead ends, and open problems); the `sources/` files hold the
detailed proofs and tables it cites.

## Usage

```python
from splitmerge import reversed_deck, ida_star, h_joint, comb_solution

cost, nodes = ida_star(reversed_deck(52), h_joint)   # -> (204, 204)
moves = comb_solution(52)                             # explicit 204-move solution
```

```sh
pip install -e .[test]
pytest                                   # all proven facts, ~10s
python -m experiments.benchmark_heuristics 10
python -m experiments.conjecture_Mn 10
```

## The OCT computation

`h_joint` reduces to an exact min-OCT (odd cycle transversal) with pre-coloured
vertices on a small soft-conflict graph (`splitmerge/oct.py`). It is computed
**exactly** by odd-cycle branch-and-bound: the soft graph is a comparability
graph, so it is bipartite iff triangle-free, and the chain bound `clique - 2 <=
OCT` prunes the clique case instantly (this is what avoids the `3^k` blowup).
Validated against a brute-force oracle on all n <= 7 states (0 mismatches); the
reversed-deck size-52 clique solves in ~0.2s.

A search budget makes it fall back to the admissible chain lower bound on the
rare large *far-from-clique* graph (e.g. an n >= ~30 scrambled start), so the
heuristic stays admissible everywhere and never hangs, at the cost of tightness
on those states. Removing the budget via a polynomial comparability-graph
2-antichain (Greene-Kleitman) computation with pre-colouring is the open
engineering step.
