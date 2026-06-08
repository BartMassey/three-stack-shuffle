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
- **The reversed deck needs exactly `4(n-1)` moves** (proven: the `comb_solution`
  construction gives the upper bound, the analytic LIS / two-buffer charge gives
  the matching lower bound; IDA* independently certifies it). For n=52 that is
  **204**. This is the operation diameter for n ≤ 8 (full BFS in this repo; n=9
  was reported from a larger BFS not committed here) and through n ≈ 11 (search)
  — but that is a *small-n coincidence*: `4(n-1)` is **not** the asymptotic
  diameter (it is linear; the diameter is Θ(n log n)). An earlier "M(n) = 4(n-1)"
  conjecture is **retracted**; see `docs/NOTES.md`.
- **Heuristics:** `h0 <= h_best <= h_joint`, all admissible lower bounds on the
  optimal move count (the test suite asserts this by full BFS for n = 6, 7, 8 —
  ~1.8M states at n = 8, 0 violations). `h_joint` is the current best; in exact
  IDA* search it expands ~90% fewer nodes than `h_best` at n=10 (median
  1122 → 97). The per-node OCT cost leaves it marginally slower in wall-clock at
  n=10, but the node advantage widens with n (the wall-clock gap is near-even by
  n=12).
- **Complexity** of optimal sorting (P vs NP-hard) is open, as is the exact
  constant in Θ(n log n) and the exact diameter at finite n. At n=52 the diameter
  lies in **[204, 600]**: lower bound the reversed deck's exact optimum (204,
  itself strengthened up from the counting bound ⌈log₃ 52!⌉ = 143); upper bound
  the Hu–Tucker sorter's proven worst case (600). A typical shuffled deck costs
  ~484 with that sorter.

The code is **Rust** (a `std`-only crate, no dependencies). The original Python
is kept under `old/` as a validated reference oracle.

## Layout

    src/
      machine.rs      states, moves, successors, base, comb_solution
      search.rs       exact BFS (small n), IDA* (generic over a heuristic)
      heuristics.rs   h0, h_best, h_joint  (+ the bounce / soft-conflict graph)
      oct.rs          exact constrained OCT (odd-cycle branch-and-bound)
      sorters.rs      constructive merge sorters: natural / top-down / Hu-Tucker
      cycle.rs        whole-cycle model: one-cycle reachability, f, diameter
      planner.rs      inadmissible rollout estimate + anytime local search
      util.rs         std-only FxHash-style hasher + SplitMix64 RNG
      bin/sm.rs       experiment runner (heuristics / sorters / planner / frontier ...)
    tests/            cross-validation vs the Python oracle (admissibility n<=8,
                      OCT vs brute force, IDA*==BFS, reversed-52 = 204/204, ...)
    old/              the reference Python implementation (run with pytest)
    docs/
      CURRENT.md            active working context (READ FIRST when resuming work)
      OVERVIEW.md           this file: orientation + run instructions
      NOTES.md              the full technical reference (start here for the science)
      structure.md          the live lower-bound / cascade theory (CURRENT points in)
      old/                  the original write-ups, superseded by NOTES.md (archive)

`docs/NOTES.md` is the single coherent account of the whole project (both cost
models, all proven results, dead ends, and open problems, with the proofs folded
in). The superseded originals are frozen under `docs/old/` for their longer
proofs and history.

`docs/CURRENT.md` is volatile *saved context* for the active research thread — a
terse, restart-ready snapshot of what is being worked on right now. Workflow:
`NOTES.md` is updated from `CURRENT.md` **only when something is removed from
`CURRENT.md`** (a settled item migrates into the permanent record). When picking
up the work, read `CURRENT.md` first.

## Usage

```rust
use splitmerge::machine::State;
use splitmerge::search::ida_star;
use splitmerge::heuristics::h_joint;
use splitmerge::sorters::hutucker_sort;

let (cost, nodes) = ida_star(&State::reversed_deck(52), &h_joint, 2_000_000);
assert_eq!((cost, nodes), (Some(204), 204));

// constructive sorter for any deck (replayable on the machine; <= 600 moves at n=52)
let deck = vec![5, 3, 1, 2, 4];
let moves = hutucker_sort(&deck);
assert_eq!(State::from_deck(deck).applied(&moves), State::goal(5));
```

```sh
cargo test --release            # all proven facts, validated vs the Python oracle
cargo run --release --bin sm -- frontier      # opt vs merge vs ILS across n
cargo run --release --bin sm -- planner 52    # planner at n=52
cargo run --release --bin sm -- heuristics 10 # h_best vs h_joint node counts
(cd old && pytest)              # the reference implementation
```

## The OCT computation

`h_joint` reduces to an exact min-OCT (odd cycle transversal) with pre-coloured
vertices on a small soft-conflict graph (`splitmerge/oct.py`). It is computed
**exactly** by odd-cycle branch-and-bound: the soft graph is a comparability
graph, so it is bipartite iff triangle-free, and the chain bound `clique - 2 <=
OCT` prunes the clique case instantly (this is what avoids the `3^k` blowup).
Validated against a brute-force oracle on every state at n <= 6 (0 mismatches),
with a 3000-state sample at n = 7 in the Python reference; the reversed-deck
size-52 clique solves in ~0.2s.

A search budget makes it fall back to the admissible chain lower bound on large
*far-from-clique* graphs — in practice essentially **every** scrambled `n >= ~30`
start (measured: 20/20 such starts at n=30,40,52 fall back; the reversed deck is
a clique and is always solved exactly, so the n=52 → 204 result is unaffected).
The heuristic stays admissible everywhere and never hangs, but on a *scrambled*
large deck its tightness degrades to roughly `h_best`. Removing the budget via a
polynomial comparability-graph 2-antichain (Greene-Kleitman) computation with
pre-colouring is the open engineering step.
