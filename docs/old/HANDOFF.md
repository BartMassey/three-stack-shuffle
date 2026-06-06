# Handoff — continuing in Claude Code

This file is the "pick up here" note. It records (1) what in this repo is real and
verified, (2) an explicit void-results notice, and (3) a concrete, honest spec for
the anytime local-search planner that was *discussed but never actually built*, so
it can be implemented and measured locally.

---

## 1. What is real (verified, tested)

Everything reachable from `pytest` (49 tests, all passing):

- `splitmerge/machine.py` — the 3-stack machine, moves, `comb_solution` (the
  exact `4(n−1)` reversal witness).
- `splitmerge/search.py` — exact `bfs_dist` (small `n`), `ida_star`.
- `splitmerge/heuristics.py` — `h0`, `h_best`, `h_joint` (admissible; full-BFS
  verified `n ≤ 7` in tests, separately confirmed exhaustively at `n = 8`).
- `splitmerge/oct.py` — exact constrained OCT for `h_joint` (odd-cycle
  branch-and-bound; validated vs brute force on all `n ≤ 7`; budget fallback to
  the admissible chain bound on large far-from-clique graphs).
- `splitmerge/sorters.py` — constructive `natural` / `top-down` / `Hu-Tucker`
  merge sorters; every emitted move stream replays to a sorted deck (all perms
  `n ≤ 7`, random to `n = 400`), counts match the closed forms.
- `splitmerge/cycle.py` — the whole-cycle model: one-cycle reachability, the
  brute-force interleaving oracle, and the Cayley-graph BFS for `f` / `D(n)`.
- `docs/PAPER.md` — the consolidated narrative; numbers there come from real
  BFS/IDA* runs (`n ≤ 11`) or proofs.

The solid quantitative facts: diameter is `Θ(n log n)`; `opt(reversed deck) =
4(n−1)` exactly (so `204` at `n = 52`); diameter window at `n = 52` is
`[204, 600]`; `h_joint` mean residual gap (`cost − h_joint`) at `n = 10` is
**≈ 1.1 on random start decks** (re-measured; robust across seeds). The earlier
"≈ 2.4" figure was an unreliable carry-over and does not reproduce: the
all-states mean at `n = 7` is `1.42` (matching `HEURISTIC-BOUNDS.md` §12),
growing only slowly with `n`.

---

## 2. VOID RESULTS — do not trust, regenerate

An exploratory chat session sketched an "anytime greedy local-search planner"
and reported results that **were never actually run**. The module was never
committed; the experiments did not execute; the numbers were fabricated. In
particular, disregard entirely:

- any "`local_search.py`" / "cascade heuristic" / "greedy + restarts" results;
- the claimed `n = 52` figures (e.g. reversal "solved to 204 in 0.32 s", random
  decks "first solution 237–244", and the anytime improvement curves);
- any gap reported against the counting bound `143`.

Treat the plan in §4 as **unimplemented**. Every number must be produced locally
and reproducibly. (The `n = 10` residual-gap figure was re-confirmed: see §1.)

**Now resolved (a separate matter).** The docs also described merge sorters and
a whole-cycle model whose *code* was likewise never committed — but, unlike the
planner, their reported results were real. That code has since been **restored
and verified** (`splitmerge/sorters.py`, `splitmerge/cycle.py`, with
`tests/test_sorters.py`, `tests/test_cycle.py`); the figures reproduce. The
recursive-thirds and bidirectional-merge variants remain **NOT VERIFIED** (not
implemented here; bidirectional needs a modified machine). Only the §4 planner
is still genuinely unbuilt.

---

## 3. Measure quality against `h_joint`, not the counting bound

For a *specific* deck, the admissible lower bound is `h_joint(deck)`. The
counting bound (`⌈log₃ 52!⌉ = 143`) is the weakest bound we have and should not
be used to report quality. The worst-case (diameter) lower bound is the
reversal's exact `4(n−1)`.

Report planner quality as `cost − h_joint(deck)` and `cost / h_joint(deck)`, and,
where exact search is feasible (`n ≤ ~11`), as `cost − opt`. Use the reversed
deck as the calibration case (known optimum `4(n−1)`).

---

## 4. Proposed: anytime local-search planner (UNIMPLEMENTED)

**Motivation.** `h_joint` is an admissible *lower bound* but a poor *steering*
signal: it bottoms out on cascading bounces (a card bounces, lands on an occupied
buffer, buries itself, cascades again). For a planner — where we want a good
solution, not a bound — it is worth giving up admissibility for a tighter
estimate of *actual* cost.

**Anytime property (the point).** Local search returns the best complete solution
found so far at any interruption, so it can be run for a fixed wall-clock budget
(e.g. 30 s) and yield a usable sort. This also suits the physical machine.

### 4a. Value function to guide greedy descent

Start from a baseline and then experiment:

- **Baseline (admissible, already available):** `v(s) = g(s) + h_joint(s)`. Known
  to be a weak steering signal; use it to get the harness working.
- **Inadmissible candidate — greedy-completion (rollout) cost:** `v(s) = g(s) +
  rollout(s)`, where `rollout(s)` runs a fixed deterministic "settle the next
  card" policy from `s` to a sorted deck and returns its move count. This is a
  real achievable cost (an upper bound on `g(s)`), so it is tight when the policy
  is good — and it is exactly the kind of inadmissible estimate that steers well.

A concrete rollout policy to start from (then iterate on it):

```
def rollout(s):
    moves = 0
    while not sorted(s):
        k = base_len(D)            # base = (1..k) settled
        target = k + 1             # next card to settle
        locate target in D, A, or B
        # 1. if target is buried under j cards, evacuate those j cards
        #    to whichever buffer keeps them in pour-sorted order if possible,
        #    else to the buffer that minimizes future buries (this choice is
        #    the lever to experiment with);
        # 2. settle target into D;
        # 3. pour back any buffer cards that are now next-in-order.
        moves += (the moves performed)
    return moves
```

The blocker-routing choice in step 1 is where most of the quality lives; treat
it as the main experimental knob.

### 4b. The search

- **Greedy descent:** from `s`, take the successor minimizing `v`; repeat to a
  sorted deck. Record the path cost.
- **Restarts / perturbation:** greedy alone plateaus at local minima. On
  plateau, perturb (reverse a random block of the *start* deck, or back up `k`
  moves and force a different branch) and restart; keep the global best.
- **Anytime loop:** keep restarting until the time budget elapses; return the
  best complete solution and its cost.

### 4c. Measurement protocol (do for real)

For `n ∈ {8, 10, 12, 15, 20, 30, 52}`, on several random decks and the reversed
deck, record: time-to-first-solution; best cost at `t ∈ {1, 5, 10, 30, 60}` s;
`cost − h_joint(deck)`; and (for `n ≤ 11`) `cost − opt`. The honest open question
is whether quality at `n = 52` within a ~30 s budget is good enough to be useful.

---

## 5. Other directions discussed

- **Tighter inadmissible heuristics:** besides rollout cost, a cascade-counting
  charge (simulate forced bounces with sequencing, not just pairwise conflicts),
  or a learned value model trained on `(state → opt)` pairs from `n ≤ 8` BFS.
- **Hybrid exact search:** use a tight inadmissible estimate to *order* IDA*
  branching while keeping `h_joint` for the admissible `f`-bound, so optimality
  is still certified. Lets the tight estimate help without losing soundness.
- **Dilworth / LDS sorter — NOT constructible here.** Decomposing into
  `LDS`-many decreasing chains and merging would save a pass, but forming the
  chains needs random access (scanning pile tops), which this sequential-access
  machine lacks. It is a lower-bound idea, not a constructive algorithm; see
  `docs/sources/operation-count-theory.md` §11.

---

## 6. Open engineering carried over

- **Polynomial exact `OCT_pre`** via the comparability-graph 2-antichain
  (Greene–Kleitman) with pre-colouring folded into a min-cut, to remove the
  branch-and-bound budget in `oct.py` on large far-from-clique graphs. The
  branch-and-bound version is exact within budget and admissible beyond it; this
  would make it exact *and* fast everywhere. Validate any replacement against
  `heuristics._oct_pre_bruteforce` on all `n ≤ 7` states (the existing oracle).
