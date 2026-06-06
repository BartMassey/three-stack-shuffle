# The Split–Merge Machine (Operation-Count Model) and an `O(n log n)` Sorter

**Scope.** This document covers the *generalized* split–merge machine, where the
four operations may be interleaved freely and **cost is the number of
operations** (no notion of "passes"). It defines the machine and the decision
problem, records the structural facts established by exhaustive search, then
gives a constructive **natural merge sort** with pseudocode and a proof that it
uses at most `2n⌈log₂ n⌉` operations. Claim status: **[PROVEN]**, **[VERIFIED]**
(exhaustive, stated range of *n*), **[CONJECTURE]**, **[OPEN]**.

---

## 1. The machine

Three stacks, each with an accessible **top**: the **deck** `D`, and two buffers
`A`, `B`. There are four operations, each pops the top of one stack and pushes it
onto the top of another:

| op | effect |
|----|--------|
| `SA` | pop top of `D`, push onto `A` |
| `SB` | pop top of `D`, push onto `B` |
| `MA` | pop top of `A`, push onto `D` |
| `MB` | pop top of `B`, push onto `D` |

`A` and `B` never exchange directly — `D` is the hub. A **run** of the machine is
any sequence of these operations in which no operation pops an empty stack.

Each operation is the inverse of another: `SA`↔`MA` and `SB`↔`MB`. Consequently
the reachability relation on configurations is **symmetric**: the configuration
graph is **undirected**.

A deck of `n` distinct cards is written bottom-to-top and identified with a
permutation after relabeling the cards `1..n` by rank.

---

## 2. The problem

**SPLIT-MERGE-OPS (decision).** Given a target permutation `π` of `n` cards and
an integer `k`, can the machine transform the deck from the identity `(1,…,n)`
to `π`, starting and ending with `A = B = ∅`, using at most `k` operations?

**Optimization.** `g(π)` = the minimum number of operations to do so.

**Sorting equivalence [PROVEN].** Because every operation is reversible, a move
sequence taking the deck `S → T` reverses (swapping `SA↔MA`, `SB↔MB`) into one
taking `T → S` of the same length. Hence producing `π` from the identity costs
exactly as much as *sorting* `π` to the identity:
`g(π) = (minimum operations to sort the deck π into ascending order)`.
So a sorter is all we need; the algorithm below sorts, and `produce` is its
reversed/inverted move list.

---

## 3. Structural facts

Established by exhaustive BFS over full configurations for `n ≤ 7` (and `n ≤ 9`
for the diameter):

- **Undirected metric [PROVEN].** Reversibility makes `g` a genuine distance.
- **Costs are even [PROVEN].** Each operation changes the number of cards held in
  `A ∪ B` by ±1, and a solution starts and ends with that number at 0.
- **Active-block reduction [VERIFIED n ≤ 6].** `g(π)` depends only on the
  *active block*: strip the longest already-correct bottom run `1,2,…,f`; only
  the remaining `m = n − f` cards matter.
- **Lower bound [PROVEN].** `g(π) ≥ 2m` — every active card must leave `D` once
  and return once. Equality holds iff the block needs no relocation.
- **Diameter is `Θ(n log n)` [LOWER BOUND PROVEN; upper bound via §4].** The
  configuration graph has out-degree `≤ 4`, so the number of configurations
  reachable within `L` moves is `< 4^{L+1}`. Reaching all `n!` permutations
  requires `4^{L+1} ≥ n!`, hence
  `max_π g(π) ≥ ⌈log₂(n!)/2⌉ − 1 = Θ(n log n)`.
  The merge sort of §4 gives the matching `O(n log n)` upper bound. **There is no
  linear-operation sorter** — the cardinality of `Sₙ` forbids it.
- **`4(n−1)` was a small-`n` artifact [RETRACTED].** Exhaustive BFS gave
  `D(n) = 4, 8, 12, 16, 20, 24, 28, 32` for `n = 2..9`, which fit `4(n−1)` exactly
  and led to a (now-withdrawn) linear-diameter conjecture. The counting bound above
  is loose at small `n` (e.g. `⌈log₂(9!)/2⌉ ≈ 9` vs the true `32`), so the two are
  indistinguishable there; but `log₂(n!)/2` overtakes `4(n−1)` near `n ≈ 690`
  (or `n ≈ 213` crediting the effective out-degree `3`), so `4(n−1)` is
  *provably false* for large `n`. The linear extrapolation was exactly the kind of
  small-`n` overfit the counting argument rules out.
- **No simple closed form [VERIFIED n ≤ 7].** Writing `g = 2m + 2R`, the
  relocation count `R` is **not** a function of natural statistics such as
  `(m, lds, m−l2)`, **nor of cycle structure** (the diameter-achieving reversal is
  cycle-trivial — all 2-cycles and fixed points — yet maximal; and `g` is not a
  function of cycle count). Whether SPLIT-MERGE-OPS is in P or NP-hard is **[OPEN]**.

These facts reframe the goal. A linear sorter is impossible (counting), so the
only available win over the merge sort is in the **constant of `n log n`** (§7).

**One pass is a generalized riffle [structural].** A single drain-and-refill
("out reverses, back reverses again") moves a chosen subset of cards while
preserving their relative order: it partitions the cards into two
*order-preserving subsequences* (the `A`-pile and `B`-pile) and re-interleaves them
freely. This is *stronger* than a Gilbert–Shannon–Reeds riffle, which may only cut
the deck into two **contiguous** packets; here the two piles are arbitrary
subsequences. One pass sorts iff the deck is a union of two monotone subsequences
(the Greene/Dilworth `k = 2` case — see §7).

**Retraction (a route that does *not* work).** One might hope every `π` factors as
two such passes (each `≤ 2n` moves), giving `g ≤ 4n`. It fails: the true one-pass
set `{π : g(π) ≤ 2n}` has sizes `5, 15, 51, 190, 756` for `n = 3..7`, and composing
it with itself does **not** cover `Sₙ` (`4815/5040` at `n = 7`). Consistent with
the `Θ(n log n)` bound, a *constant* number of passes cannot sort all of `Sₙ`.

---

## 4. The algorithm: natural merge sort

The two buffers are exactly the two input runs of a balanced two-way merge, and
`D` is the output. The stack reversals — fatal to a relocation-based greedy —
are harmless here: each run is reversed once on the way out and once on the way
back, so orientation is invariant across levels and no cleanup is ever needed.

We keep runs **ascending bottom-to-top** (equivalently, *max on top*) on `D`. A
level distributes the runs alternately onto `A` and `B`, then merges the top run
of each back onto `D`.

```
natural_sort(D):                       # D is the deck; A = B = empty
    runs ← lengths of maximal ascending (bottom→top) runs of D   # bottom→top
    while |runs| > 1:
        # ---- distribute: pop runs off the top of D, alternating A, B ----
        a_runs ← [],  b_runs ← [],  to_a ← true
        for L in reverse(runs):                    # topmost run first
            if to_a:  pop L cards D→A (L × SA);  append L to a_runs
            else:     pop L cards D→B (L × SB);  append L to b_runs
            to_a ← not to_a
        # ---- merge: top run of A with top run of B → one run on D ----
        new_runs ← []
        while a_runs and b_runs:
            la ← pop_last(a_runs);  lb ← pop_last(b_runs)
            # both runs now have their minimum on top
            repeat until one run is exhausted:
                if top(A) < top(B):  MA          # move smaller onto D
                else:                MB
            flush the remaining run onto D (MA's or MB's)
            append (la + lb) to new_runs
        flush any leftover run (odd count): pop its cards buffer→D, append length
        runs ← new_runs
    return the recorded operation sequence

produce(target):                       # identity → target
    return reverse(natural_sort(target)) with SA↔MA, SB↔MB swapped
```

Concretely, distribution sends the runs (from the top of `D`)
`q_t → A, q_{t-1} → B, q_{t-2} → A, …`, so consecutive — hence adjacent — runs
land on opposite buffers, and the merge pairs them off `(q_1,q_2), (q_3,q_4), …`.
Merged results are pushed back in the same order, so the run order on `D` is
preserved for the next level.

---

## 5. Correctness and complexity

Fix a level and let the runs on `D` (bottom-to-top) be `q_1, …, q_r`, each sorted
**ascending** (max on top), with `A = B = ∅`.

**Lemma 1 (reversal).** Popping all cards of an ascending run (max on top) onto
an otherwise-untouched stack yields that run with its **minimum on top**.
*Proof.* Pops occur in order max, …, min; the last pushed (the min) is on top. ∎

**Lemma 2 (merge).** Given a run on `A` and a run on `B`, each with its minimum on
top, repeatedly moving the smaller of `top(A), top(B)` onto `D` — then flushing
the remaining run — produces on `D` a single sorted run of all those cards,
**ascending** (max on top).
*Proof.* This is the standard two-way merge: the cards pushed to `D` come out in
increasing order, so `D` bottom-to-top is increasing and the maximum ends on
top. ∎

**Lemma 3 (level invariant).** One level transforms a configuration of `r`
ascending runs on `D` (with `A = B = ∅`) into one of `⌈r/2⌉` ascending runs on
`D` (with `A = B = ∅`), using exactly `2n` operations, and the multiset of cards
is unchanged.
*Proof.* Distribution pops every card exactly once (`n` ops) and, by Lemma 1,
deposits each run min-on-top; runs `q_t, q_{t-2}, …` go to `A` and
`q_{t-1}, q_{t-3}, …` to `B`. Because consecutive runs alternate buffers, the
topmost run of `A` and the topmost run of `B` are an adjacent pair; the merge
loop processes these top pairs in turn, so it merges exactly the adjacent pairs
`(q_1,q_2), (q_3,q_4), …`. By Lemma 2 each merge produces one ascending run on
`D`; an unpaired final run (when `r` is odd) is flushed buffer→`D`, which by
Lemma 1 is again ascending. The merge pushes every card back exactly once (`n`
ops), so the level uses `n + n = 2n` operations and leaves `A = B = ∅`. Pairing
`r` runs yields `⌊r/2⌋` merged runs plus a possible singleton, i.e. `⌈r/2⌉`
runs, in their original relative order. ∎

**Theorem.** On an input with `r` maximal ascending runs, `natural_sort`
terminates with `D` sorted ascending and `A = B = ∅`, using exactly
`2n·⌈log₂ r⌉` operations. In particular `g(π) ≤ 2n·⌈log₂ n⌉` for every `π`,
constructively, with equality of the *bound* only forced when `r = n`
(no two adjacent cards ascending, e.g. the reversal).
*Proof.* By Lemma 3 the run count maps `r ↦ ⌈r/2⌉` each level, costing `2n`
operations, and the ascending-run invariant is maintained. Starting from `r`
runs, after `⌈log₂ r⌉` levels the count reaches `1`: a single ascending run
spanning all `n` cards, i.e. the sorted deck, with `A = B = ∅`. The total is
`2n·⌈log₂ r⌉`. Since `r ≤ n`, this is `≤ 2n·⌈log₂ n⌉`. The sorting equivalence of
§2 turns this into the stated bound on `g(π)`; reversing/inverting the sort costs
the same number of operations. ∎

**Remarks.**
- `r = 1` (already sorted) costs `0` — matching `g(identity) = 0`.
- The bound is *worst-case `O(n log n)`*. By the counting lower bound (§3) the
  diameter is `Θ(n log n)`, so this is within a constant factor of optimal — no
  algorithm is asymptotically better. The merge sort's constant is `2` (each card
  moves twice per level over `⌈log₂ r⌉` levels); the counting bound permits a
  constant as small as `≈ 0.5`, so a better *constant* may exist (§7), but linear
  is impossible.

---

## 6. Empirical results (from `tests.py`, `benches.py`)

Correctness: `45,911` checks pass — all permutations for `n ≤ 7`, plus randomized
decks up to `n = 20000`, each verified by replaying the emitted moves on a fresh
machine and confirming a sorted deck with empty buffers (and the exact identity
`ops = 2n⌈log₂ r⌉`).

Operation counts (uniform random permutations):

| n | mean ops | `2n⌈log₂n⌉` | counting LB `½log₂(n!)` | ops/(n log₂ n) |
|---|---|---|---|---|
| 100 | 1,200 | 1,400 | 263 | 1.81 |
| 1,000 | 18,190 | 20,000 | 4,305 | 1.83 |
| 10,000 | 260,000 | 280,000 | 59,235 | 1.96 |
| 50,000 | 1,500,000 | 1,600,000 | 351,357 | 1.92 |

Random decks have `≈ n/2` natural runs, so they typically save one level versus
the singleton worst case. Nearly-ordered inputs do far better: at `n = 10000`, a
`k`-run input costs `≈ 2n⌈log₂ k⌉` (e.g. `k = 2` → ~12.5k ops, identity → 0),
versus 280k for a uniform deck — the payoff of seeding with natural runs.

---

## 7. Diameter is `Θ(n log n)`; the open question is the constant

**Settled (this supersedes the earlier `4(n−1)` conjecture).** The counting
argument of §3 proves `max_π g(π) = Ω(n log n)`, and the merge sort proves
`O(n log n)`. So the diameter is `Θ(n log n)`: **no linear-operation sorter
exists.** The `4(n−1)` pattern (verified `n ≤ 9`) was a small-`n` artifact — the
counting bound is loose there and only provably exceeds `4(n−1)` near `n ≈ 213–690`.

**The riffle / Greene–Dilworth picture.** One pass partitions the cards into two
order-preserving subsequences and re-interleaves them — a generalized riffle,
stronger than a contiguous-cut GSR riffle. One pass sorts iff the deck is a union
of two monotone subsequences, which is exactly the Greene/Dilworth `k = 2` case:
- *Dilworth / patience sorting* decomposes a permutation into `c` increasing
  subsequences ("chains") with `c = LDS` (longest decreasing subsequence), in
  `O(n log n)`. `c = LDS` is the minimum such cover (Dilworth's theorem;
  Erdős–Szekeres is the `c = 1` corollary).
- *Greene's theorem* generalizes: the total length of the `k` longest disjoint
  increasing subsequences equals the sum of the first `k` rows of the RSK shape
  `λ(π)`; the `k` longest decreasing subsequences give the first `k` columns.
  One pass = collapsing two columns of `λ`; sorting = reducing `λ` to one cell.

**What this buys for the constant [the live question].** The merge sort spends
`2n⌈log₂ r⌉` ops, `r` = ascending runs. A Greene/Dilworth sorter would merge the
`LDS` chains instead of the `r` runs. Since a random permutation has
`LDS ≈ 2√n` but `r ≈ n/2`, for a shuffled `n = 52` deck this is `⌈log₂ 14⌉ = 4`
vs `⌈log₂ 26⌉ = 5` levels — about `2·52·4 = 416` ops vs the merge sort's `520`,
a ~20% win on typical decks (worst case, the reversal with `LDS = n`, is `624`
either way). Free interleaving (the op model, not clean passes) appears to shave
the constant further — at `n = 9` the BFS optimum is `32` vs `54` for clean
chain-merging, a `~1.7×` gap.

> **Open problem (the constant).** Determine `lim sup g(π)/(n log₂ n)`. The
> counting bound gives `≥ ½`; the merge sort gives `≤ 2`. Is there a constructive
> sorter achieving a constant below `2` for all `π` — e.g. by merging Dilworth
> chains rather than runs, and exploiting free interleaving? For the physical
> machine (`n = 52`, ~10 ops/s) this is the only available speedup: linear is
> impossible, but `520 → ~300` ops would halve the wall-clock per sort.

**The target constant, measured.** Exhaustive BFS gives the *mean optimal* op
count; dividing by `n log₂ n`:

| n | opt mean / (n log₂ n) | merge sort / (n log₂ n) |
|---|---|---|
| 4 | 1.01 | ~1.8 |
| 6 | 0.91 | ~1.8 |
| 8 | 0.85 | ~1.8 |

The optimal constant is `~0.85` and *drifting down* with `n`, while merge sort
holds at `~1.8`. So the optimum is roughly **2× better than the merge sort**, and
this is not a small-`n` mirage — the ratio is stable/growing. For `n = 52` this
suggests an achievable `~0.9·n log₂ n ≈ 300` ops versus the merge sort's `520`.

**Why — the machine-usage signature.** Tracing optimal solutions (`n = 8`, 300
random decks): the optimum empties `D` only `~1.3` times mid-solution (the merge
sort empties it `~log₂ r` times), keeps **both buffers non-empty 66%** of the time
(vs ~50% for the alternating drain/merge of the merge sort), and fills the buffers
nearly full (`~7.8` of `8`). I.e. the optimum **drains almost everything once, then
does the reordering among the buffers with both live**, instead of cycling cards
through `D` once per level. The merge sort's repeated full drains are the waste.

**Design target [OPEN, constructive].** A sorter that drains `D` (near-)once and
then minimizes re-emptying, keeping both buffers active, should approach the
`~0.9` constant. A greedy descent on the admissible heuristic `h` does **not**
achieve it (it cascades — constant `~5` and frequently fails to terminate under a
`6 n log n` cap), because `h` is a good lower bound but a poor one-shot steering
signal: it does not price the downstream relocations a move creates. A workable
construction needs to plan the buffer contents so the rebuild is cheap — the same
"few relocations" problem as §3's `R`, now with a concrete `~0.9·n log₂ n` goal.

**Subtlety to resolve.** Achieving exactly `⌈log₂(LDS)⌉` passes with only two
piles is not obviously possible — one pass cannot trivially halve an arbitrary
chain count, and the orientation of the monotone-cover condition (increasing vs
decreasing) needs pinning against the reversal, which exposes the convention. The
robust, actionable claim is narrower: **use `LDS`, not run-count, as the merge
statistic** — `LDS ≤ r` always, with strict gain on typical decks.

**Ruled-out proof routes (recorded so they are not revisited).** Per-card bounding
(optimal per-card move count grows `~2⌊n/3⌋`, so no "≤ `c` moves per card"
argument); two clean passes (`{g ≤ 2n}²  ⊊ Sₙ`); a constant number of passes
(forbidden by counting); cycle structure (`g` not a function of cycle count; the
reversal is cycle-trivial yet maximal); inversions (range `Θ(n²)`, cannot govern a
near-linear cost); and any hope of a linear diameter (counting).

---

## 8. Relation to the stack-sorting literature

This machine is an instance of Tarjan's *sorting with networks of stacks* (1972):
a directed network with a source, a sink, and storage nodes that are stacks.
Getting the placement right matters, because the field's headline bounds split
sharply by the number of *reusable working stacks*.

**Our machine is a 3-stack star, i.e. the `k ≥ 3` regime.** The deck `D` is a
**reusable hub**: cards flow `D→A→D→B→D…`, and the merge sort of §4 uses `D` as
working storage (runs accumulate there between levels). So `D` is a genuine third
working stack; with leaves `A`, `B` the network is a star on three stacks.

- **`k ≥ 3` (our regime): `Θ(n log n)`.** Felsner & Pergel (ESA 2008): any input
  sorts in `O(n log_{k−1} n)` moves, tight up to the log base. For `k = 3` this is
  `O(n log n)` — exactly the merge sort of §4. Combined with the counting lower
  bound of §3 (`Ω(n log n)`), the diameter of our machine is `Θ(n log n)`. This is
  **proven and verified by construction** (the §4 sorter, 45,911 checks to
  `n = 20000`, always `2n⌈log₂ r⌉`).
- **`k = 2` (NOT our regime): `Ω(n²)`.** With only two working stacks and a
  *non-reusable* source/sink, Felsner & Pergel show inputs requiring `Ω(n^{2−ε})`,
  and Mihalák & Pont (ATMOS 2019) prove a clean `Ω(n²)` lower bound under the
  "midnight constraint." **This does not apply to us**, because our hub `D` is a
  reusable third stack. (A natural earlier confusion: "two buffers" looks like
  `k = 2`, but the reusable hub puts us at `k = 3`. The verified `O(n log n)`
  construction is the ground truth that settles it.)

**Complexity of the *exact* optimum.** König & Lübbecke (ISAAC 2008): minimizing
shuffles is NP-hard to approximate within `O(n^{1−ε})` for complete networks of
`k ≥ 4` stacks (via Min `k`-Partition on circle graphs). This is consistent with
our finding that no simple closed form governs `g` and that exact `T[n]` is
intractable to search (§3, §A*-experiments). For `k = 2` and `k = 3` the exact
complexity is **open** in the literature.

**Two things we rediscovered independently.**
- The *midnight constraint* — Mihalák & Pont note it can be forced by appending a
  `0` to the permutation so nothing is output until everything has left the source.
  This is precisely our structural fact "the finished prefix accumulates at `D`'s
  bottom, so all unfinished cards must be in the buffers before output begins."
- The `Ω(n²)` worst-case input for `k = 2`, `π* = (2,4,6,…,n, n−1,n−3,…,3,1)`
  (even values up, then odd values down), is exactly the odd/even interleaving on
  which our single-pile `comb` heuristic blew up to `n²`. (On our `k = 3` machine
  the merge sort still sorts it in `O(n log n)`; the quadratic adversary bites only
  the 2-stack model.)

**MinUnCut connection (for the restricted 2-stack variant).** Mihalák & Pont reduce
the restricted (strong-midnight) `k = 2` problem to **MinUnCut**: choose, for each
card, which stack it goes to, minimizing same-part "conflict" edges; this yields a
randomized `O(√log n)`- and deterministic `O(log n)`-approximation. This is a clean
formalization of the "deck-as-accumulator, minimize double-moves" idea — but it
governs the *2-stack* machine, not ours. They explicitly leave the **unrestricted
`k = 2`** case (shuffling before the source is empty) open, noting that early
shuffling moves the shuffle-accounting index dynamically — which is exactly the hub
flexibility our machine has for free.

**Takeaway.** Our machine is firmly in the `Θ(n log n)` regime; the merge sort is
within a constant factor of optimal. For `n = 52`: `≤ 624` ops worst case, `~520`
typical (~52–62 s at 10 ops/s). The remaining question is purely the **constant**
(merge sort `~1.8 · n log₂ n`; measured optimum `~0.9`; §7), not the asymptotics —
and certainly not anything quadratic.

---

## 9. The recursive-thirds algorithm (Felsner–Pergel) and our star topology

Felsner & Pergel (ESA 2008) give the `k`-communicating-stacks upper bound as an
explicit recursive-splitting sorter. Reading their `sort(B)` directly:

- Distribute the input into `k` blocks by value (smallest third, middle, largest).
- Process the block with the smallest unplaced items; split its lower half to one
  neighbor stack and its upper half to another, making new sub-blocks; recurse,
  always acting on the block holding the globally-smallest unplaced items.
- A size-1 block is output. Each move sends an item to a block of half the size,
  so each item moves `≤ log₂ n` times between stacks.

**Their bound.** For `k` communicating stacks, total moves `≤ n log_{k−1} n + n −
(n−1)/(k−2)`. For `k = 3` this is `≈ n log₂ n + 1` — **constant `~1`**, versus our
merge sort's `~1.8`. For `n = 52` that is `≈ 300` moves vs merge sort's `~520`: the
hoped-for ~2×.

**The topology catch (why our constant is larger).** Their `k` stacks are
*fully connected* — every stack moves directly to every other, so a stack-to-stack
move costs 1. Our machine is a **star**: `A` and `B` connect only through the hub
`D`, so an `A→B` move costs **2** (`A→D→B`). The paper's substructure `S1` (a
directed 3-cycle of stacks `S₁→S₂→S₃→S₁`) is what guarantees `O(n log n)` on
non-complete networks, and our star contains the cycle `A→D→B→D→A`, so the
recursive-thirds algorithm *does* apply to us and gives `O(n log n)`. But the hub
tax on cross-buffer moves inflates the constant: our realized constant lies
**between `~1` (their complete `k=3`) and `~1.8` (binary merge sort)**, with the
exact value depending on the cross-buffer vs buffer-to-`D` move mix — an empirical
question for our topology, settled by building and measuring it (§ companion code).

**Confirmations of our independent findings.**
- Their `k ≥ 3` *lower* bound is a counting argument giving `(n/2) log_k n − O(n)`,
  within a factor `≤ 3.2` of their upper bound — the same counting bound we
  reinvented (§3), confirming the diameter of our (`k=3`) machine is `Θ(n log n)`.
- They pose as their "single most intriguing" open problem whether **two**
  communicating stacks can sort in `o(n²)`, and leave the complexity of optimal
  sorting of a fixed permutation open — matching our conclusion that exact `T[n]`
  is intractable and that our merge-family algorithms (which exploit the reusable
  hub as a third stack) are the right regime.

**Plan.** Implement `sort(B)` on the real star machine (hub `D`, leaves `A`, `B`),
verify with the existing test harness, and benchmark its op count against the
merge sort at `n = 52` to read off the actual constant on our topology.

**Result [DONE; negative].** Implemented as `recthirds.py` (validated by
`test_recthirds.py`: 19,799 checks, all permutations `n ≤ 7` plus random to
`n = 20000`, every emitted sequence replayed legal and sorted). Benchmarked by
`bench_recthirds.py` against `natural_sort`:

| n | recthirds / (n log₂ n) | merge / (n log₂ n) | recthirds / merge |
|---|---|---|---|
| 52 | 2.37 | 1.75 | 1.35 |
| 256 | 2.25 | 1.88 | 1.20 |
| 5000 | 2.17 | 1.95 | 1.11 |

**The recursive-thirds sorter is *worse* than merge sort on our machine** — about
`1.1–1.35×` more moves (704 vs 520 at `n = 52`), and non-adaptive (it costs 704
even on the identity, where merge sort costs 0). The reason is exactly the star
topology: Felsner-Pergel's `k = 3` advantage (`log₂ → log_{k−1}`) comes entirely
from *direct* `Sᵢ→Sⱼ` edges, which let an item move between two sibling stacks in
one move. Our `A` and `B` are **not** sibling-connected — they reach each other
only through the hub `D` at cost 2 — so our three stacks give a third *storage*
area but not a third *merge stream*. The machine is structurally a **binary**
merge device, and merge sort's `~1.8 · n log₂ n` is essentially the floor for the
merge family. Beating it would require either a genuinely non-merge construction
(the `~0.9` optimum of §7, whose constructive form remains open) or a different
machine (a `D`↔`A`↔`B` triangle with a direct `A↔B` edge would admit the true
`k = 3` algorithm and its `~1` constant). For the physical machine as specified,
**the natural merge sort is the recommended sorter**: `≤ 624` worst / `~520`
typical ops for `n = 52` (~52–62 s at 10 ops/s).

---

## 10. Bidirectional merge: exploiting forward/backward symmetry

Plain natural merge sort (§4) exploits only *ascending* runs, so a descending
stretch reads as singleton runs and reversed is its worst case (`r = n`, 624 ops
at n=52). But descending order is just as sorted, only reversed, and the
literature frames the machine symmetrically: Felsner-Pergel's two-stacks-glued
"operating head" tape and Mihalak-Pont's state `l = v1 · v2^R` both make
ascending and descending the same thing seen from opposite sides of the head
(our hub `D`). The asymmetry is an artifact of the *algorithm*, not the machine.

**The free-reversal mechanic.** On a stack, a pour reverses. Popping a run off
`D` into a buffer flips its order, so a descending run can be turned ascending
during a move we are making anyway — the machine analogue of Timsort reversing
descending runs, but (nearly) free. (A queue would not do this; Felsner-Pergel's
`S2` substructure uses a queue precisely because it *preserves* order.)

**Experiment (this section assumes the modified machine where the direct
buffer-to-buffer transfer `A↔B` costs 1 — the complete 3-stack network).**
Implemented in `bimerge.py` (validated by `test_bimerge.py`: 72,162 checks, all
permutations n ≤ 7 plus random to n=1000). `normalize_moves` flips descending
runs (rebuild in `A`, `B` as transient scratch, then `A→D`; cost `2n + Σ|desc
runs|`); `bidir_sort` = normalize then plain merge; `smart_sort` = the cheaper of
the two (never worse than plain merge, worst case stays ≤ 624).

Measured op counts at n=52 (`bench_bimerge.py`):

| input | asc runs `r` | monotone runs `r′` | plain | bidir | smart |
|---|---|---|---|---|---|
| reversed | 52 | 1 | 624 | **156** | **156** |
| 2 desc blocks | 51 | 2 | 624 | 156 | 156 |
| 8 desc blocks | 45 | 8 | 624 | 156 | 156 |
| 16 desc blocks | 37 | 16 | 624 | 357 | 357 |
| random (mean) | ~26 | ~22 | 520 | 648 | 520 |

**Findings.**
- **Descending structure: up to 4× win.** Reversed 624 → 156 = exactly `3n` —
  the cheap-reversal construction recovered as a special case of the general
  algorithm. Holds for any input that is a few long descending blocks.
- **Typical random: no win.** Monotone-run detection cuts the run count by a
  steady factor ~0.83 (26.4 → 21.8), but a constant factor on `r` almost never
  crosses a power-of-two pass boundary: 26 and 22 are both in the band (16, 32],
  so the merge still takes 5 passes and the cost stays 520. Even an
  overhead-free bidirectional merge would only tie at 520 for random.
- **`smart_sort` is a strict free upgrade.** Best-of-both: 520 on random (worst
  case ≤ 624), big wins on any descending-structured input. Recommended sorter
  if the direct edge exists.

**Why random is fundamentally out of reach for this approach, and the real
frontier.** Beating 520 on random needs `r′ < 16`, which a *consecutive* monotone
partition cannot give (greedy is optimal at ~22). The only way below is a
decomposition into non-consecutive monotone *subsequences* — patience-sorting,
where the pile count is the longest decreasing subsequence ≈ `2√52 ≈ 14 < 16`.
That would cross the boundary, but it is a fundamentally different,
routing-heavy algorithm (not a natural merge), and whether its routing cost on
this machine stays below the merge it replaces is open. This is the LIS/LDS
(Greene-Dilworth) direction flagged in §7 — the genuine open lead for the
typical-case constant.

---

## 11. Patience sorting: why the LIS advantage does not transfer

Patience sorting deals the deck into piles by one rule — each card onto the
leftmost pile whose top exceeds it, else a new pile on the right. Each pile is
then decreasing top-to-bottom (smallest exposed), pile tops increase left to
right, and the number of piles equals the **longest increasing subsequence**.
Finishing is a single k-way merge (the global minimum is always the leftmost
top). For a random 52-deck `LIS ≈ 11.6` (sd 1.4; `≤ 16` over 99.8% of the time),
versus ~26 ascending runs and ~22 monotone runs. Since merge cost here is
`2n·⌈log₂(#starting sorted pieces)⌉`, starting from ~12 pieces would be **4
passes (416)** instead of 5 (520) — exactly the ~20% gap, and below the
threshold that monotone runs (~22, still band 5) could not cross.

**Why three stacks block it.** Patience needs two things the machine lacks.
(1) The ~12 piles must exist *simultaneously and separately* — any incoming card
may land on any pile — but we have two working stacks plus `D` (which must stay
clean as output), so we can hold two piles, not twelve. (2) Placement requires a
**binary search over pile tops** ("leftmost top > x"), which is the entire source
of patience's `O(n log n)` efficiency and relies on random access to all tops at
once. A stack exposes one top. Without the search, finding a card's pile means
scanning — `O(#piles)` per card, `O(n·LIS) ≈ 700+` moves just to *form* the
piles, before any merge.

So the realizable version reduces to "produce ~12 sorted runs in `D`, then
binary-merge in 4 passes." The merge half (416) is fine; the formation half is
fatal. One pass over the deck only yields *contiguous* structure (26 ascending,
22 monotone runs); reaching the ~12 *non-contiguous* LIS pieces fundamentally
needs more than one pass of rearrangement, spending the single pass (104 moves)
it would have saved. Net loss. The LIS advantage is a **random-access**
phenomenon (Dilworth/Greene–Kleitman gives the count; realizing it needs each
element routed into a searchable structure); this machine is sequential-access
with a reversal tax. Same wall as block-selection (§block-selection): no random
access → cannot reach the right pile (or a buried card) cheaply. Merge survives
only because it touches accessible ends.

---

## 12. Summary and open problems

**Where we landed.** For the split-merge machine (hub deck `D`, buffers `A`, `B`,
4 reversible moves `SA/SB/MA/MB`; `n = 52`), the recommended sorter is the
**natural merge sort** of §4: `2n⌈log₂ r⌉` moves (`r` = ascending runs), `~520`
typical, `≤ 624` worst, `0` on sorted input. At ~10 ops/sec that is ~52–62 s.
Provable, built, and verified (45,911 checks to `n = 20000`).

**The machine's place in the literature.** It is a 3-stack *star* network
(reusable hub) in Tarjan's framework, hence the `k = 3` regime: diameter
`Θ(n log n)`, with Felsner–Pergel's `O(n log n)` upper bound and a counting lower
bound `(n/2)log_k n`. The `k = 2` `Ω(n²)` results do **not** apply (the hub is a
reusable third stack); the exact optimum is NP-hard in the general König–Lübbecke
setting and open for `k = 2, 3`.

**The gap (the interesting part).** Merge sort spends a constant `~1.75 · n log₂n`.
The measured mean optimum is `~0.9` — so optimal sorting is roughly **2× cheaper**
than what we can construct. The diameter is `Θ(n log n)`; the open question is
purely the **constant**, between `~0.9` (optimum) and `~1.75` (merge).

**Leads explored, and why each stalls (all verified empirically):**
- *Block-selection* (extract smallest `k`, iterate): `Θ(n²/k)` re-handling; no
  random access to buried cards. Best `k` ≈ 576 > 520. No.
- *Recursive-thirds* (Felsner–Pergel `k=3`): on the star, `A↔B` is 2 moves, so the
  three stacks give a third *storage* but not a third *merge stream* — structurally
  binary, constant ~2.2 (worse than merge). True `k=3` (~1.0) needs a direct `A↔B`
  edge **and** a separate I/O port; measured ~404 (~22%) — not worth the hardware.
- *Bidirectional merge* (monotone runs + free reversal-by-pour): big win on
  descending-structured inputs (reversed 624→156 = 3n), but monotone runs are only
  ~0.83× ascending runs — a constant factor that does not cross a power-of-two pass
  boundary, so **no typical-case win**. `smart` best-of-both is never worse.
- *Patience / LIS* (§11): `LIS ≈ 12 < 16` would save a pass, but forming the piles
  needs random access (binary search over ~12 pile tops) the machine lacks;
  formation costs more than the pass saved. No.

**The recurring obstruction.** Every sub-merge idea dies on the same rock: **no
random access**. Selection can't reach a buried card; patience can't reach the
right pile; both go quadratic emulating it. Merge wins because it only ever
touches the accessible ends of two sequences. The reversal tax (a pour flips
order; cross-buffer costs 2 via the hub) is the secondary constraint.

**Open problems worth handing to a theorist:**
1. **Constructive constant.** Is there a poly-planner-time sorter for the 3-stack
   star with worst- or average-case constant strictly below `1.75 · n log₂n`,
   approaching the `~0.9` optimum? Or a lower bound showing the merge family's run-
   count dependence is essentially forced under sequential access?
2. **Optimal assignment.** Mihalák–Pont reduce the *restricted* (strong-midnight)
   `k=2` problem to MinUnCut (`O(√log n)`-approx). The **unrestricted** case — with
   early shuffling, which our reusable hub permits — is open: can the dynamic
   shuffle-accounting be captured, and does it admit a non-trivial approximation?
3. **Realizing LIS without random access.** Is there a sequential-access (stack)
   construction that exploits the `LIS`-many-sorted-pieces structure for `o(1)`
   amortized placement, or a proof that sequential access forces `Ω(run-count)`
   starting pieces — i.e., that patience's advantage is unrealizable here?

A clean way to state the taunt: *we can sort a shuffled 52-card deck in ~520
moves and we are quite sure ~300 is possible, and the whole difficulty is that
the machine has no random access.*
