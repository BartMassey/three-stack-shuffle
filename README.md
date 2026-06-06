# split-merge

Admissible heuristics and lower bounds for a three-stack LIFO card-sorting
machine. The machine has a deck `D` and two buffers `A`, `B`; four moves
(`SA`, `SB`, `MA`, `MB`) shuffle one card at a time between the deck and a
buffer. The goal is to sort a shuffled deck into `(1, 2, ..., n)` in the fewest
moves. The headline target is `n = 52`.

## What's known

- **Lower bound:** every deck needs at least `4(n-1)` moves in the worst case,
  and the reversed deck achieves exactly `4(n-1)` (proven three ways: the
  heuristic, the `comb_solution` construction, and IDA* search). For `n = 52`
  that pins the worst case at **204**.
- **Conjecture** `M(n) = 4(n-1)`: the reversed deck is the unique worst case.
  Proven by full BFS for `n <= 8`; supported by exhaustive-enough search
  through `n = 11`. The open piece is a matching universal upper bound.
- **Heuristics:** `h0 <= h_best <= h_joint`, all admissible (verified by full
  BFS for `n <= 7`). `h_joint` is the current best; in search it expands ~84%
  fewer nodes than `h_best` at `n = 10`.

## Layout

    splitmerge/
      machine.py      states, moves, successors, base, comb_solution
      search.py       exact BFS (small n), IDA* (parametrized by heuristic)
      heuristics.py   h0, h_best, h_joint  (+ the bounce / OCT internals)
    tests/            admissibility (full BFS n<=7), dominance, reversal, IDA*==BFS
    experiments/      heuristic benchmark, M(n) conjecture search
    docs/
      HEURISTIC-BOUNDS.md   the heuristic framework, proofs, search results
      SORTING-BOUNDS.md     the merge-style algorithms and the counting bound

The code mirrors the docs: `h0` is HEURISTIC-BOUNDS section 3, the buried charge
section 4, the LIS charge section 5, `h_best` (the "max") section 11, `h_joint`
section 12, the consistency analysis section 13, IDA* section 14.

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

## Known approximation point

`h_joint` reduces to an exact min-OCT (odd cycle transversal) computation on a
small soft-conflict graph. For graphs larger than `OCT_BRUTE_FORCE_LIMIT`
vertices (only the reversed-deck-style giant cliques hit this) it falls back to
a polynomial *lower* bound, which stays admissible but loses tightness. The
planned replacement is exact Reed-Smith-Vetta iterative compression (fast
because the OCT is small) -- see HEURISTIC-BOUNDS.md sections 12 and 12b. This
is the one place in the heuristic that is approximate by construction.
