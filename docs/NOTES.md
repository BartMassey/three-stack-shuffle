# The Split–Merge Machine — Notes

The single technical reference for this project: the machine, both cost models,
every proven result, the dead ends, and the open problems. It folds together the
documents now archived under `docs/old/` (the consolidated narrative `PAPER.md`
and the working write-ups in `sources/`) into one organized account, with the
fabricated/uncommitted material corrected or removed. For a short orientation and
run instructions see `docs/OVERVIEW.md`.

**Status legend.** Every claim is tagged:
**[PROVEN]** (complete argument), **[VERIFIED n ≤ N]** (exhaustive computation),
**[CONJECTURE]**, **[OPEN]**, **[RETRACTED]** (believed earlier, now known
false), **[DEAD END]** (a route that does not work, kept so it is not retried),
**[NOT VERIFIED]** (argued but never implemented or reproduced here).

**What the committed code backs.** The operation-count machine, heuristics, exact
OCT, IDA*/BFS search, the constructive merge sorters, the whole-cycle model, and
the inadmissible planner are all in the Rust crate (`src/`) and exercised by
`cargo test` (validated against the reference Python implementation under
`old/`). Where a claim is reproducible, the relevant module is named inline.

> **Code references.** Inline names below use the module/function names shared by
> the Rust crate (`src/<module>.rs`, canonical) and the reference Python
> (`old/splitmerge/<module>.py`) — e.g. `h_joint`, `hutucker_sort`,
> `iterated_local_search`. Experiments named `experiments/foo.py` are ported to
> the `sm` CLI subcommands (`cargo run --release --bin sm -- foo`).

---

## 0. The machine

Three LIFO stacks, each with an accessible top: a hub **deck** `D` (also the I/O
port) and two **buffers** `A`, `B`. Four operations, each pops one stack's top
and pushes it onto another:

| op | effect | | op | effect |
|----|--------|-|----|--------|
| `SA` | `D → A` | | `MA` | `A → D` |
| `SB` | `D → B` | | `MB` | `B → D` |

`A` and `B` never exchange directly; `D` is the hub, so a cross-buffer move costs
2 (`A → D → B`). Each operation is the inverse of another (`SA ↔ MA`,
`SB ↔ MB`), so the configuration graph is **undirected** **[PROVEN]**. Cards are
distinct and identified with a permutation by rank; a state is `(D, A, B)` with
the top of each stack at the right (`splitmerge/machine.py`).

Two cost measures, which must not be conflated:

- the **operation-count model** (Part I): operations interleave freely, cost =
  number of operations. The main object of the project.
- the **whole-cycle model** (Part II): a *cycle* drains the entire deck into the
  buffers (`n` splits) then merges everything back (`n` merges); cost = number of
  cycles. The project's origin.

A cycle is a restricted operation schedule of length `2n`, so the two measures
live on different scales (for the reversal, operation distance `g = 4(n−1)` while
`2n · f` grows super-linearly). Treat them as two questions about one machine.

---

# Part I — The operation-count model

## I.1 Problem and basic structure

**SPLIT-MERGE-OPS.** Given a target permutation `π`, transform the deck from the
identity to `π`, starting and ending with `A = B = ∅`, in the fewest operations.
Write `g(π)` for that minimum.

- **Sorting ≡ producing [PROVEN].** Every operation is reversible, so a sequence
  taking `S → T` reverses (swap `SA↔MA`, `SB↔MB`) into one taking `T → S` of
  equal length. Producing `π` from the identity costs exactly what sorting `π`
  costs; a sorter is all we need, and `g` is a genuine metric.
- **Costs are even [PROVEN].** Each op changes `|A| + |B|` by ±1, and a solution
  starts and ends at 0.
- **Active-block reduction [VERIFIED n ≤ 6].** Strip the longest correct bottom
  run `1, …, f`; only the `m = n − f` remaining cards matter. The committed base
  `k = base_len(D)` is this `f` (`machine.base_len`).
- **Floor [PROVEN].** `g(π) ≥ 2m`: every active card must leave `D` at least once
  and return. Writing `g = 2m + 2R`, the **relocation count** `R` counts cards
  forced to enter `D` more than once — the central difficulty. In the heuristic
  language of I.4 these relocations are *bounces*.

**Moves, arrivals, bounces [PROVEN].** For any solution, `arrivals − departures =`
net deck growth `= n − |D₀|`, and `|σ| = arrivals + departures`; eliminating
departures, `|σ| = 2·(arrivals at D) − (n − |D₀|)`. Each non-base card arrives ≥ 1
(its settle); an extra arrival is a **bounce**. With `B` = total forced bounces,
`|σ| ≥ h₀ + 2B`, where `h₀` is the base charge of I.4. *Lower-bounding moves ⇔
lower-bounding bounces*, which is the entire heuristic program.

## I.2 The diameter is Θ(n log n)

The headline structural result; it settles the asymptotics.

- **Counting lower bound [PROVEN].** On a shortest path no move is the inverse of
  the move just made (else they cancel), so after the first step each step has
  ≤ `4 − 1 = 3` choices; from the goal only `SA, SB` apply, so ≤ 2 first moves.
  Hence the ball of radius `L` has `|Ball(L)| ≤ 1 + Σ_{j≥1} 2·3^{j−1} = 3^L`.
  Reaching all `n!` permutations needs `3^M ≥ n!`, so
  `M ≥ log₃(n!) = Ω(n log n)`. At `n = 52`, `log₃(52!) = 142.33`, so **`M ≥ 143`**
  (the strongest counting bound; the cruder `log₄(n!) ≈ 113` ignores
  no-backtracking).
- **Matching upper bound [PROVEN; replay-verified to n = 400].** The natural
  merge sort (I.3) sorts any deck in `2n⌈log₂ r⌉ ≤ 2n⌈log₂ n⌉ = O(n log n)`
  operations.
- **Therefore the diameter is `Θ(n log n)`.** No linear-operation sorter exists.

**Literature placement [PROVEN].** The machine is a 3-stack *star* (reusable hub)
in Tarjan's networks-of-stacks framework — the `k = 3` regime. Felsner & Pergel
(ESA 2008) give `O(n log_{k−1} n)`, i.e. `O(n log n)` for `k = 3`, matching the
merge sort; their counting lower bound matches ours. The `k = 2` `Ω(n²)` results
(Mihalák & Pont) do **not** apply, because the hub is a reusable third stack. The
exact optimum is NP-hard for general `k ≥ 4` networks (König & Lübbecke); for
`k = 2, 3` its complexity is **[OPEN]** in the literature too. *(Citations
verified: Felsner & Pergel, ESA 2008, "The Complexity of Sorting with Networks of
Stacks and Queues"; Felix G. König & Marco E. Lübbecke, ISAAC 2008, "Sorting with
Complete Networks of Stacks" — `k ≥ 4` NP-hard via Min-`k`-Partition on circle
graphs; Matúš Mihalák & Marc Pont, ATMOS 2019, "On Sorting with a Network of Two
Stacks" — the `k = 2` MinUnCut/`Ω(n²)` result, for a two-stack machine with a
direct edge under the "midnight" constraint, which does not model our reusable
hub.)*

> **[RETRACTED] `4(n−1)` is not the diameter.** Exhaustive BFS gives diameter
> `4(n−1)` for `n = 2..8` (`4, 8, …, 28`; reproducible here via `bfs_dist` —
> `n = 9 → 32` was reported from an uncommitted larger BFS), which fits a linear
> law exactly and twice tempted a linear-diameter conjecture. It is a **small-`n`
> artifact**: the counting bound is loose for small `n` and only provably exceeds
> `4(n−1)` near `n ≈ 213–690`. The diameter is `Θ(n log n)`, so the reversal is
> **not** the asymptotic worst case. What survives is the *exact value*
> `opt(reversal) = 4(n−1)` (I.5).

## I.3 Constructive sorters and the constant

Asymptotics settled, the live constructive question is the **constant** in
`c · n log₂ n`. The measured **mean** optimum over start decks (exhaustive BFS)
is `c ≈ 0.9` and drifting down (`1.01, 0.91, 0.85` at `n = 4, 6, 8`); the best
construction sits near `1.75`. (The *diameter* constant is higher, `≈ 1.1` at the
largest BFS-computed `n`, and is a separate quantity.)

**The cost identity [PROVEN, machine-verified].** All the sorters below are
*adjacent-merge*: they only ever merge two adjacent sorted runs, so each is a
binary **merge tree** over the input's ascending runs (leaves = runs, in order).
Realized by recursion with **parking** — sort the upper child on top of `D`, pour
it onto `A` (it reverses to min-on-top); sort the lower child, pour onto `B`;
merge the two parked runs back to `D` *by exact count* (never by empty-stack
test, so a merge never digs into an inert run parked beneath) — the machine cost
is exactly `2·W(T)`, where `W(T) = Σ sᵢ·depth(leaf i)` is the weighted external
path length. Each node spends `2·(cards in node)` moves, summing to `2·W`. All
three sorters are implemented in `splitmerge/sorters.py` and verified by replay
in `tests/test_sorters.py` (every emitted move stream sorts the deck with empty
buffers, for all permutations `n ≤ 7` and random decks to `n = 400`, using
exactly the closed-form count). Figures below are for `n = 52`.

- **Natural merge sort (`natural_sort`) [PROVEN].** Bottom-up two-way merge in
  synchronous passes: distribute the runs alternately onto `A`/`B`, merge top
  pairs back to `D`; the run count halves each pass. Cost exactly `2n⌈log₂ r⌉`
  (`r` = ascending runs): `0` on sorted input, `≤ 624` worst (descending deck,
  `r = n`), `~520` typical (random `E[r] ≈ 26.5` ⇒ 5 passes). The stack
  reversals are harmless — each run flips once out and once back. The simplest
  sorter and the recommended one for the machine as specified.
- **Adaptive top-down merge (`topdown_sort`) [PROVEN/VERIFIED].** Build the tree
  by splitting the run sequence at the boundary nearest the card midpoint (long
  runs stay shallow). Worst `600`, average `~487` — beats plain merge on both.
- **Hu–Tucker optimal merge tree (`hutucker_sort`) [PROVEN].** The minimum-`W`
  order-preserving (alphabetic) merge tree, via an optimal-BST `O(r²)`–`O(r³)`
  DP; the **best no-reversal adjacent-merge sorter**. `W(T) ≤ 300` for *every*
  input (the all-singleton descending deck maximizes it: `40` leaves at depth 6,
  `12` at depth 5, `W = 300`), so `cost ≤ 600` for every input, `600` attained
  only by the descending deck; average `~484`. Since the optimum never exceeds
  any sorter's cost, this **proves the diameter upper bound `M(52) ≤ 600`**.

The merge family's worst case bottoms out at `600`. So at `n = 52` the operation
diameter is pinned to **`[204, 600]`**: lower bound the reversal's exact optimum
`4(n−1) = 204` (I.5; itself a strengthening of the counting bound `143`), upper
bound Hu–Tucker's proven worst case `600`. The gap is wide because our best
*constructive* sorter spends `600` on the reversal whose true optimum is only
`204`. (`experiments/benchmark_sorters.py` reproduces the averages and worsts.)

**[DEAD END] / [NOT VERIFIED] constructions that do not beat merge sort:**
- *Recursive-thirds* (Felsner–Pergel `k = 3`) **[NOT VERIFIED]**: on the star,
  `A↔B` costs 2, so the three stacks give a third *storage* area but not a third
  *merge stream* — structurally binary, so it cannot beat the merge family; the
  `~1.0` constant needs a true triangle (direct `A↔B`). We argued this but never
  implemented it here; the earlier "measured constant ~2.2" came from uncommitted
  code and is not reproduced.
- *Reversal / bidirectional merge* **[ANSWERED via cost-bracket, `src/bin/revsort.rs`]**.
  Reversal-by-pour is **native** to this machine (through `D`); the earlier "needs a
  modified `A↔B` machine" applies only to a *simultaneous* bidirectional merge, not to
  reversing a run. A reverse-aware adjacent-merge may use **descending** runs as leaves.
  Bracket its cost with an optimal-alphabetic DP charging a descending leaf `c` per
  card (`c=0` = free lower bound; `c=∞` = ascending-only baseline = Hu–Tucker, sanity
  `=hutucker_cost`): a size-`s` descending run reverses in `≈2s` moves (comb-calibrated —
  the whole reversed deck gives `2·2·52 = 208 ≈ opt 204`), so `c≈2` is realizable.
  Results (`n=52`, `2W`):
  - **random:** free LB `457.7` vs baseline `484.1` = 5.4% headroom, but at realizable
    `c=2` the gap is **0.0%** (41/3000 decks benefit at all). The headroom is an artifact
    of free reversal: a random deck's descending runs are too short — reversing costs
    `≈2s`, shred-and-merge costs `≈2s·log₂s`, so reversal wins only for `s ≳ 3`. This is
    the old "no typical-case win," now **rigorous** (a bracket, not a sample, and
    mechanistic) — and it corrects the old entry's `156=3n` (below the proven `204`) and
    "monotone ~0.83× ascending" (it is *more*: `E[mono]=(2n−1)/3 > E[asc]=(n+1)/2`).
  - **interleave / `obvious`:** gap **exactly 0 at every charge** — no descending
    structure; reversal is orthogonal to the class where merge is a `log` factor off
    (that gap is the *concatenation / two-open-piles* lever, not reversal).
  - **descending-structured:** wins big and realizably — reversed `600 → 208` (≈ comb),
    two long embedded descending blocks `380 → 318`; benefit scales with run length.
  Conclusion: reversal helps **only** genuinely descending-structured decks (where it
  ~recovers the comb), and does nothing for random or the interleave class. Exact
  realizable numbers await a replay-verified bidirectional sorter; the `c≈2` charge is
  comb-calibrated, not yet move-emitted.
- *Patience / LIS sorting* **[DEAD END for general n; 2-pile specialist measured,
  `src/bin/psort.rs`]**: `LIS ≈ 12 < 16` at `n = 52` would save a pass, but forming
  the ~12 non-contiguous piles needs random access (binary search over pile tops) the
  machine lacks. The machine *can*, however, run **2-pile** patience (both tops are
  visible): place each departing card on the pile it fits under (top > card), one pass +
  one merge = `2n` moves. It sorts **iff `LDS(deck) ≤ 2`** (the pop-order needs ≤ 2
  decreasing piles) — exactly the `obvious`/interleave class, where it crushes merge
  (`n = 52`: **`104` vs Hu–Tucker `496`**, replay-verified, 4.8×). But `LDS ≤ 2` is a
  **vanishing fraction** (57% of decks at n=4, 3.5% at n=8, ~0% by n ≥ 14; a random deck
  has `LDS ≈ 2√n`), so 2-pile patience is a *specialist* — it never applies to random,
  and its flat `2n` even loses on low-run decks (`0` on already-sorted). It is
  **complementary to reversal**: patience wins on the interleave extreme (where reversal
  does nothing) and is stuck on the reversed deck (`LDS = n`, where the comb wins). The
  two stack freedoms each crack one structured extreme; **neither touches the random
  constant**, because two piles overflow at `LDS ≈ 2√n`. This is the §I.4a two-open-piles
  realizability wall, now measured. Beating merge on random needs > 2 open piles
  (impossible here) or the recursive/multi-pass resolution of the cascade (open).
- *Recursive patience* **[IDEALIZED win, realizability OPEN; `src/bin/recpat.rs`]**.
  Split the deck into two subsequences each with ~half the LDS (Dilworth cover into `LDS`
  increasing chains — count `= LDS`, verified — then 2-colour the chains), recurse, merge:
  depth `~log₂LDS`, total `~2n·log₂LDS`, i.e. the `log r → log LDS` granularity. Idealized
  cost (charges `2·len` per level and assumes the split is realizable in `len` moves — an
  optimistic bracket, cf. revsort's free reversal): random **n=52 `367.6` vs Hu–Tucker
  `484`, −24%, on 5000/5000 decks**, matching `2n·log₂LDS = 367` to the decimal, with the
  margin **growing in n** (16%→28% over n=16→120) — the first construction that would beat
  the merge constant on random (though still `~367` vs opt `~250`, so LDS-halving is not
  the whole story; the full RSK shape is, per §I.4a). **Realizability is the open crux:**
  merge sort is realizable because its splits are *positional* (contiguous blocks park as
  inert stacked runs under "merge by exact count"); the LDS-halving split is
  *non-positional* (interleaved subsequences), which cannot park that way — recursively
  sorting one half appears to require storing the other, i.e. a 4th stack (the same
  2-buffer wall). Open: a realizable *multi-pass* analog — a full-deck pass `D→{A,B}→D`
  that halves LDS — would make the 24% real. Not yet move-emitted.
- *Block-selection* **[DEAD END]**: `Θ(n²/k)` re-handling, no random access to
  buried cards; structurally cannot beat the merge family.
- *Two clean passes* (`g ≤ 4n`) **[RETRACTED]**: the one-pass set `{g ≤ 2n}` does
  not cover `Sₙ` under self-composition (`4815/5040` at `n = 7`); a constant
  number of passes is forbidden by counting (I.2).

**The recurring obstruction.** Every sub-merge idea dies on the same rock: **no
random access** (selection can't reach a buried card; patience can't reach the
right pile). Merge survives because it only touches accessible ends. Whether a
constructive sorter with constant `< 1.75` exists is **[OPEN]** (I.6).

## I.4 Lower-bounding the relocations: an admissible-heuristic program

Beating the merge sort needs a handle on the *exact* optimum, i.e. on `R`. The
program builds admissible lower bounds on `R` (equivalently `g`), all of the form
`h = h₀ + 2·(lower bound on bounces)`, all admissible **[PROVEN]** and
**[VERIFIED by full BFS at n = 6, 7, 8 in `tests/validation.rs`, 0 violations
(~1.8M states at n = 8)]**. Implemented in `splitmerge/heuristics.py` and `splitmerge/oct.py`.

**The lever — per-card decomposition.** Each move picks up exactly one card, so
`|σ| = Σ_c μ_σ(c)`. *Principle:* if for every solution `μ_σ(c) ≥ m(c)`, then
`h = Σ_c m(c)` is admissible. This localizes the estimate to "how few times can
each card move."

**The checkpoint invariant [PROVEN].** In every solution there are times
`τ₁ < … < τₙ` with the deck equal to `(1,…,i−1)` just before `τᵢ` settles card
`i`. Consequences: every above-base deck card departs before `τ_{k+1}`, and the
above-base deck cards make their **first departures in current top-to-bottom
order**. This constrains *every* solution; nothing below assumes a strategy.

**The charges (each a proven per-card / per-group floor):**

- **`h₀` (base charge).** A base card costs 0; a buffer card ≥ 1 (it must `MA/MB`
  home); an above-base deck card ≥ 2 (it departs ≥ once by the invariant, and
  starting/ending in `D` it arrives equally often). So
  `h₀ = 2·(|D| − k) + |A| + |B|`. Exact on rotations.
- **Bury charge (+2).** If buffer card `x` has a smaller `y` below it in the same
  buffer, then `y` settles first but cannot be reached until `x` is popped to the
  deck — so `x` is premature and must leave and return: `μ_σ(x) ≥ 3`. `#buried`
  is a directional, exact-per-buffer floor.
- **LIS / two-buffer charge.** Let `π` be the above-base deck values read
  top-to-bottom (their forced first-departure order). Take an increasing
  subsequence of `π`; its members settle in that order. Two single-arrival
  members sharing a buffer would force the deeper (earlier-departed, smaller) one
  to be popped *after* the one above it — impossible. So at most 2 single-arrival
  members fit in the 2 buffers; the other `≥ LIS − 2` bounce. Hence
  `Σ_{above-base} μ ≥ 2m + 2·(LIS − 2)⁺` — Dilworth meeting the two-buffer
  constraint.
- **Composition theorem [PROVEN].** Floors on *disjoint* groups of cards add
  without double-counting: buffer contention can only raise move counts, never
  lower them. So the above-base-deck floor and the buffer floors sum.

**`h_best`** = `h₀ + 2·max((LIS − 2)⁺ + #buried, clique − 2)`, where the second
term is a pairwise-conflict clique bound: two single-arrival cards *conflict* if
they must occupy different buffers (deck–deck increasing in `π`; deck `d` vs
buffer `q < d`; same-buffer deeper-smaller), and a mutual-conflict clique of size
`c` needs `c − 2` bounces (only 2 buffers). The clique is the longest chain of a
comparability graph, a polynomial longest-chain DP.

**`h_joint`** = `h₀ + 2·(#buried + OCT_pre)` — the **joint bound**, which
eliminates the `max`. It bounds the largest *single-arrival* (never-bouncing)
set `U` directly via two necessary conditions: **(N1)** `U` contains no buried
card; **(N2)** the **soft-conflict graph** on `U` — deck–deck increasing in `π`,
and deck `d` vs non-buried buffer `q < d`, with buffer cards pre-coloured by
their buffer — must be properly 2-coloured. `OCT_pre` is the minimum vertex
deletion achieving that, so `B ≥ #buried + OCT_pre`. It **dominates `h_best`**
(the deck cards form a soft clique of size `LIS`, so `OCT_pre ≥ (LIS−2)⁺`; any
conflict clique gives `OCT_pre ≥ clique − 2`; `#buried` adds on the disjoint
buffer side) and is verified to dominate at every state for `n ≤ 8`. Average
residual gap drops ~35–45% vs `h_best`.

**Computing `OCT_pre` exactly (`splitmerge/oct.py`).** This is constrained Odd
Cycle Transversal (NP-hard in general, FPT in the deletion count). The soft graph
is a *comparability graph* (perfect), so an induced subgraph is bipartite iff
triangle-free and its max clique equals its longest value-chain; the chain bound
`clique − 2 ≤ OCT` prunes the otherwise-fatal clique case immediately (at a
clique it equals the incumbent on the first descent — this defeats the `3^k`
blowup). Pre-colouring is encoded with two undeletable terminal vertices.
Validated against a brute-force oracle on every state at `n ≤ 6` (0 mismatches),
with a 3000-state sample at `n = 7`; the reversed-deck size-52 clique solves in ~0.2 s.
A search budget makes it fall back to the admissible chain bound `clique − 2` on
large *far-from-clique* graphs — in practice **essentially every scrambled
`n ≥ ~30` start** (measured 20/20 at n = 30, 40, 52). On those it stays
admissible (the fallback returns a valid *lower* bound, never the partial
incumbent) and fast (~1 s) but degrades to roughly `h_best` strength. The
reversed deck is a clique, always solved exactly, so the `n = 52 → 204` result is
unaffected.

**Consistency [PROVEN].** `h₀` is consistent (`|Δ| = 1` per move). `h_best` and
`h_joint` are admissible but **not** consistent: a single move can split a
conflicting pair across buffers and drop the bound by 3. Search therefore uses
**pathmax** and cycle/duplicate detection, never `g`-pruning by stored cost.

**[DEAD END] heuristic mistakes recorded.** Greedy/Chaitin graph-colouring for
`OCT_pre` gives a *feasible* deletion set = an **upper** bound (wrong direction;
4226 admissibility violations at `n = 6`). A too-strict "no deck card above any
smaller buffer card" single-arrival rule (584 violations). A 2-colouring that
seeds free vertices before the fixed buffer colours (2481 violations) — the fix
is to seed from the pre-coloured vertices first. Greedy descent on `h` is a poor
*sorter* (it cascades and often fails to terminate): `h` is a good floor but does
not price the relocations a move creates downstream.

## I.4a Structure of the bounce-minimization problem (tangle, buried, cascade)

I.1–I.4 give the floors and the admissible bound `h_joint`; this section is the
conceptual map of *where the remaining difficulty lives* — the live research
frontier. (Consolidated from the former `docs/structure.md`; it cross-references
the proofs in I.1/I.4/I.5 and Part III rather than repeat them. The volatile
day-to-day frontier notes are in `docs/CURRENT.md`, which points here.)

**Tangle and buried are one relation read in two places [PROVEN].** Let `σ` = the
deck top-to-bottom = the **departure order**. A set `U` of cards can all be
*single-arrival* (`e_c = 1`, no bounce) **iff `U` partitions into two
`σ`-decreasing subsequences** (each buffer pops in increasing order, so its cards
decrease in `σ`; two buffers, two runs). So the largest single-arrival set is
`a₂(σ)` = max union of two `σ`-decreasing subsequences, and `B ≥ m − a₂(σ)`. The
**tangle graph** `T` has an edge for each *increasing* pair of `σ` (the
comparability graph of the (position, value) dominance order — perfect); a
single-arrival set induces a bipartite subgraph, so the bound is an **odd-cycle
transversal** `B ≥ OCT(T) = m − a₂`, polynomial by Greene–Kleitman (`a₂` = sum of
the two longest RSK columns). This is exactly `h_joint`'s bounce term (I.4).
**buried** (a smaller card under a larger one in a *buffer*) is the *same
inversion* as a **tangle** (an increasing pair of `σ`, in the *deck*) — read in
placement order vs departure order. A clean start deck is all tangle; buried is
what tangle turns into when a card is placed on a smaller one.

**The static bound is an asymptotically vanishing fraction of the work.** For a
uniform-random deck `a₂(σ) = Θ(√n)` (Ulam–Hammersley: longest decreasing
subsequence `~2√n`), so `B_static = m − a₂ = Θ(n)`. But `B_opt = Θ(n log n)` for
**almost every** deck (the I.2 counting bound holds for all but a vanishing
fraction; merge gives the match). So `B_static / B_opt = Θ(n)/Θ(n log n) → 0` —
tangle/buried prove only that *almost every card bounces ≳ once*, and are blind to
the average card bouncing `Θ(log n)` times. The decomposition:

```
B_opt  =  OCT(T)              +     cascade(π)
          static, poly, Θ(n)        dynamic, Θ(n log n), the open core
```

The **cascade** is the dynamic cost of *LIFO-scheduling* the 2-colouring:
evacuating a buried card re-enters it into `D`, where re-placing it can create new
buried cards, recursively — `T` has no edges for "bounces created by handling
other bounces." This is why `h_joint` is tight at small `n` and a vanishing
fraction asymptotically, and why the merge sorter (which is *entirely* cascade —
every card re-handled `⌈log₂ r⌉` times) is within a constant of optimal while the
static bound is a `log n` factor below it.

**Do not presume the optimum is pass/recursion-structured.** The merge sorter's
uniform `⌈log₂ r⌉`-deep re-handling is exactly its `~1.75` waste; the optimum may
spread the `Θ(log n)`-average re-entries *unevenly* (most cards once, a few many
times), cards settling opportunistically. `opt` is uncomputable past `n ≈ 14`, so
clean global structure there is unfalsifiable and not to be assumed. (Direct
evidence it is scheduling, not a graph parameter: the I.5 double-transfer atom's
*hot-potato* mechanism is a pure buffer-**capacity** forcing — card 1, the global
minimum, has no value-conflict at all yet is forced to transfer twice.)

**The missing keystone: an instance-sensitive `Ω(n log n)` lower bound.** `OCT =
Θ(n)` is the only instance-sensitive bound and is a `log n` factor short; the
counting `Θ(n log n)` is uniform (same for every deck), so it cannot see that
reversed is `Θ(n)` while random is `Θ(n log n)`. Pursue this bound **before** any
steering potential `Φ` (an ungrounded `Φ` was the prior rollout's failure, and the
lower-bound potential and the steering potential are plausibly the same object).
Tellingly the cascade is **not monotone in `a₂`**: it is `0` at both extremes —
reversed (`a₂ = 2`, fully tangled but *regular*, the comb pays no cascade) and
sorted (`a₂ = n`) — and `Θ(n log n)` in the middle for generic permutations. So the
right bound is **entropy / incompressibility-flavoured** (a random `π` carries
`Θ(n log n)` bits to pay off; structured families are cheap), instance-sensitive,
and *unrelated to OCT*. Candidate tools (unmined): buffer occupancy `|A|+|B|` over
departure time is a **cut/separation profile** (cutwidth / minimum-linear-
arrangement / pathwidth is the natural home for an `Ω(n log n)` bound); and the
**full Greene–Kleitman hierarchy** `a₁,a₂,…` (the whole RSK shape), not just `a₂`
(the "2" is the two buffers; the missing `log` should come from iterating the
chain/antichain structure). The 2-stack-sorting literature does **not** transfer
(I.2 citations: different machines — complete networks, or a direct `A↔B` edge).

**The transfer reduction, and why single cuts fail.** By the checkpoint invariant
(I.4), when card `i` settles `D = (1,…,i−1)` exactly and all `n−i` unsettled cards
are in the two buffers — `D` is an I/O port, not storage. So (modulo parking)
every non-settling deck entry is a **transfer** `A→D→B` of a stack top, and the
problem reads as *sort two stacks by transfers, minimise transfers*. **[LOSSY for
the optimum — see I.5 (parking) and Part III; the reduction survives only as the
source of `OCT ≤ B`.]** Why a single value-cut proves nothing: project values to
`{≤k, >k}` and the instance needs **zero** transfers (one class per stack), so
`B ≥ 0` — vacuous. Bounces need a **3-way** distinction (three values can have
`LIS = 3 >` two buffers); the cost lives in the *fine, multi-scale* order, not any
single cut, which is why no single cutwidth-type quantity captures it. The natural
fix — a dyadic-refinement sum of 3-way OCTs — is **[REFUTED, Part III, `phitest`]**:
value-coarsened `OCT^(ℓ)` is monotone in scale, so scales share transfers and the
telescoped sum collapses to the base `OCT^(0) = Θ(n)`. **No static value-partition
statistic of `σ` reaches `Θ(n log n)` admissibly** — the bound must be amortised
over the schedule.

**Runs over-count; the right granularity is the increasing-subsequence cover.** The
merge sorter pays `2·W(T)` over `r` ascending runs, but ascending runs are a
*positional* artifact. The minimum increasing-subsequence cover is `LDS` (Dilworth
dual: longest *decreasing* subsequence), and `LDS ≤ r` always, often `≪`
(interleave: `LDS=2` vs `r=n/2`). **"Leave a run for later" = keep an increasing
subsequence intact** — within one, no card sits above a smaller one, so it pours
out sorted for free; the cost is *cross-subsequence* blocking. Swapping
`log r → log(LDS)` moves the constant from `~1.75` toward `log(2√n) =
½log₂n+O(1)` — into the neighbourhood of the measured optimum `~0.8`. Two caveats:
(1) `LDS` over-counts the *dual* way (reversed has `LDS=n` yet `opt=Θ(n)` — one
decreasing run, sorted by reversal-by-pour); the true governor is the full RSK
shape, not `LDS` alone. (2) Realisation: two buffers keep only **two** increasing
subsequences open at once, but a random deck has `≈2√n` of them — whether the rest
merge in `~log(LDS)` rounds *without* patience's random access is open.

**The safe/forced boundary, and why resolution is global [boundary PROVEN; (a)
answered].** In the transfer view, placing the departing `v` keeps a pile sorted
**iff `v < top(A)` or `v < top(B)`** (it becomes a smaller new top); **forced iff
`v >` both tops** (the apex of an increasing triple — it must bury one pile). So
"both piles stay sorted, zero transfers" is possible **iff `LIS(σ) ≤ 2`** (the
`B=0` class); the first forced transfer is the first time the live arrival stream's
`LIS` reaches 3. Deferral freedom is *narrow* — a departing card is the deck top,
placed now — so all real decisions concentrate at the forced events: which pile to
break, and where the displaced card goes. At a forced event, **peel-to-fit =
insertion sort = `O(n²)`** (bad); **bury = `O(1)` deferred**, and exactly optimal on
reversed (the comb, each card once). But `min transfers ≠ #forced events`:
`#forced = OCT = n − a₂ = Θ(n)`, yet `opt = Θ(n log n)`. **(a) answered: min
transfers is strictly more than `#forced` whenever the cascade is nonzero** (already
true at small `n`: `opt − h_joint ≈ 1` at `n ≤ 14`). The gap is the deferred
transfer re-burying on the other pile when it executes; its resolution is **global**
(the comb reverses a whole pile at once), not per-event greedy. So the open object
is the **resolution schedule** of the deferred transfers. Remaining sub-questions:
**(b)** the optimal pile-to-break and displaced-card destination at a forced event;
**(c)** magnitude — does the running-`LIS`-excess of `σ` total `Θ(n log n)`?

> **Observability caveat (governs every large-`n` claim here).** Exact `opt` is
> computable only to `n ≈ 14` (IDA*); `OCT = n − a₂` is poly at any `n`. So the
> cascade `= opt − OCT` is *measurable* only where it is `~1` (`n ≤ 14`) and
> *unmeasured* where it dominates (`n ≳ 20`). All large-`n` cascade / crossover
> figures are **extrapolation** from proven asymptotics (`opt = Θ(n log n)` by
> counting-LB + merge-UB; `OCT = Θ(n)` by `a₂ = Θ(√n)`), not computed — and no `n`
> is both solvable and cascade-rich, so the cascade cannot be reverse-engineered
> from optimal traces. This is a theory problem.

## I.5 Exact search, and the exact value of the reversal

IDA* on `f = g + h` (pathmax, parent-move pruning, on-path cycle detection) finds
exact optima (`splitmerge/search.py`).

- **Validated [VERIFIED n ≤ 8]:** matches BFS optima on all tested decks;
  `h_best` and `h_joint` return identical optimal costs.
- **`h_joint` pays for itself [VERIFIED n = 10]:** vs `h_best`, ~90% fewer node
  expansions (median 1122 → 97; the saving widens with `n` — ~93% at `n = 12`).
  In the Rust crate node expansion is cheap enough that the per-node OCT cost
  leaves `h_joint` marginally *slower* in wall-clock at `n = 10`, but the gap
  closes as the node advantage grows (near-even by `n = 12`) (`sm heuristics`).
- **Residual gap.** On random start decks at `n = 10`, `cost − h_joint ≈ 1.1`
  (robust across seeds); the all-states mean at `n = 7` is `1.42`. (An earlier
  "≈ 2.4 at n = 10" figure was an unreliable carry-over and does not reproduce.)
- **Double-transfer atom — TWO mechanisms [VERIFIED, `src/bin/dbx.rs`,
  `src/bin/dbx9.rs` (exhaustive n=9), `src/bin/dbxshow.rs`].** A card *transfers
  twice* (enters D ≥ 3 times) by one of two distinct forcings — the earlier "only
  by re-burial" claim is **refuted**:
  - **re-burial:** evacuated from one buffer the card lands on a *smaller* unsettled
    card in the other, so it is buried again and must move once more. (232 of the
    287 forced n=9 decks, classified on a representative optimum.)
  - **hot-potato (capacity/scheduling):** a card that must stay top-accessible —
    it settles *before* everything currently staged, so anything stacked on it
    would have to settle first, which it can't — *freezes* its buffer. With only
    two buffers, when each must be loaded in turn the card is shuttled across to
    free them. It **lands only on larger cards**, so it is never re-buried. (55 of
    287; in particular any deck whose doubler is card 1 — the global minimum, which
    *cannot* land on a smaller card.)
  Strictly stronger than cascade (which begins as merely *more cards bouncing once*
  than `OCT`). Exhaustively (real, parking-capable machine): **no optimum forces a
  double for n ≤ 8**; at **n = 9 exactly 287 decks force one** (all base-free),
  cheapest at **opt = 22** (12 decks). Genuine smallest/lex-first witness:
  **`[3,5,2,4,7,9,6,8,1]`** (opt 22), a hot-potato on card 1 — and *no* optimum of
  it spares card 1 (`dbxshow hotpotato`), so its forced double is *necessarily*
  hot-potato, not re-burial. (The previously recorded `[6,2,3,5,8,9,1,7,4]`, opt 24,
  card 4, was a re-burial example found by sampling, **not** minimal.) Verdict:
  "forced" = no optimal-length solution has all per-card D-arrivals ≤ 2. Depends on
  `h_joint` admissibility (used for both the opt and the pruning).
- **Parking is necessary — "B = inter-buffer transfers" is LOSSY [VERIFIED n ≤ 8,
  `src/bin/park.rs`].** Treating D as a one-card transit slot (the *transit-only
  transfer reduction* — **not** the six-action machine, which represents parking
  fine as a bare `MA`/`MB` onto a non-base D) is **not** WLOG-optimal: some optima
  must *park* a card in D — arrive it onto a deck that still holds unsettled cards,
  using D as a third LIFO. First forced at **n = 6** (`[1,3,5,6,4,2]`, opt = 12: transit-only needs
  `B = 2`, parking achieves `B = 1`); necessary in >50% of random decks by n = 9–10.
  So the true minimum bounce count can be *below* the transit-only transfer minimum —
  the reduction over-counts the optimum. (`OCT ≤ B` is unaffected; the admissible
  lower bound stands.) This corrects the transfer-reduction claim in §I.4a.
- **Exact value of the reversal [PROVEN]:** `opt(reversed deck) = 4(n−1)` for all
  `n`. *Upper bound:* the explicit `comb_solution` (`machine.comb_solution`)
  sorts the reversed deck in exactly `4(n−1)` moves (per-card profile
  `2,4,…,4,2`). *Lower bound:* the analytic LIS/two-buffer charge of I.4 already
  gives `h*(reversal) = 2n + 2(n−2) = 4(n−1)` for all `n` (`π = (1,…,n)`,
  `LIS = n`, base 0) — this needs only the proven §I.4 floors, **not** the full
  OCT machinery or any BFS. The two meet, so the value is exact. Moreover
  `h_joint` is exact along the whole optimal reversal path, so IDA* solves the
  reversed `52`-deck in exactly **`204` node expansions**, independently
  certifying `opt = 204`.

> **Reconciliation (read with I.2).** `opt(reversal) = 4(n−1)` is an exact value
> for one family of inputs and is solid; `M(n) ≥ 4(n−1)` follows, so `M(52) ≥ 204`.
> It coincides with the **diameter** only for `n ≤ 8` (full BFS reproducible here;
> `n = 9` from an uncommitted BFS) and is supported as the diameter through
> `n ≈ 11` by heuristic search (`experiments/conjecture_Mn.py`: random sampling +
> adversarial hill-climbing find nothing above `4(n−1)`). But this is the
> small-`n` artifact of I.2, **not** an asymptotic fact. **Do not state
> `M(n) = 4(n−1)` as the diameter; do not cite `624` as the upper bound — `600`
> is proven and tighter.**

## I.6 Open problems (operation-count model)

1. **Complexity.** Is SPLIT-MERGE-OPS in P or NP-hard? `R` is not a function of
   any natural statistic tried (inversions, cycle structure, `LIS`-family). **[OPEN]**
2. **The constant.** Determine `lim sup g(π)/(n log₂ n) ∈ [½, 2]`. Is there a
   constructive sorter below `1.75` (e.g. merging Dilworth chains, exploiting free
   interleaving), approaching the measured `~0.9` mean optimum? **[OPEN]**
3. **Exact diameter at finite `n`** (e.g. is it `204` or larger at `n = 52`?). **[OPEN]**
4. **Polynomial exact `OCT_pre` with pre-colouring** — via the comparability-graph
   2-antichain (Greene–Kleitman) folded into a min-cut, to remove the
   branch-and-bound budget on large far-from-clique graphs (validate any
   replacement against `heuristics._oct_pre_bruteforce` on all `n ≤ 7` states).
   **[OPEN, engineering]**
5. **An anytime local-search planner [BUILT — `splitmerge/planner.py`].** A
   practical (non-optimal) sorter steered by a tight *inadmissible* estimate.
   Two deterministic completions supply the estimate: `rollout` (settle the next
   card, draining the above-base deck into two patience piles) and `rollout_merge`
   (pour the buffers back, then Hu–Tucker); the first solution is the cheaper of
   the two, then a path-kick search perturbs and re-completes within a time budget
   (`experiments/planner_search.py`). **Measured at `n = 52`:**
   - First complete solution: **`204` on the reversed deck** — the settle rollout
     is *exact* there, reconstructing the `comb` optimum — and **`~480` on random
     decks** (the merge rollout, on par with Hu–Tucker), each in **~0.5–1.6 ms**.
   - The two rollouts are mirror images: settle is exact on the reversal but
     `~850` on random; merge is `~480` on random but `600` on the reversal — so
     their min is never worse than either.
   - **Search on top — two regimes.** Greedy *stepping* (descend move-by-move on
     the estimate; `local_search`'s epsilon-restarts) **does not help and even
     drifts** — a step + re-completion is usually worse than completing
     immediately, so the best it does is the plain completion. The rollout is a
     good *completion* but a poor *steering gradient* (flat single-move
     landscape). **Path-kick iterated local search** (`iterated_local_search`:
     back up to a random state on the best path, take a few random legal moves,
     re-complete; keep the best) **does** make small, real gains — `~1–3 %` below
     the merge frontier (e.g. random `n = 52`: `452 → 438` over 30 s, ~10⁵
     kicks), and exact `204` on the reversal. But the believed-achievable `~300`
     is **not** reached: the perturbation polishes within the merge basin but
     cannot find a structurally cheaper sort. **The bottleneck is completion
     quality (the `~480` merge frontier), not the search strategy.**

   **The "cascading charge" idea, measured [REFUTED].** A project note proposed a
   tighter inadmissible heuristic: simulate the forced bounces greedily ("every
   card bounces as soon as it would bury a smaller one") and count moves, hoping
   it beats `h_joint`'s static pairwise OCT. By the identity
   `|σ| = h0 + 2·(bounces)` this *cascading charge* equals the settle-rollout's
   length (`planner.cascade_charge` = `h0 + 2·cascade_bounces`), so it is exact on
   the reversal (`n−2` bounces). But measured against the exact optimum on random
   start decks (`sm cascade`), it **overshoots**: cascade − opt
   ≈ 5, 9, 13, 16 at `n = 8, 9, 10, 11` and growing, overshooting on ~32–38 of 40
   decks, whereas `h_joint` sits **within ~1–2 below** opt. The greedy cascade
   *overcounts* bounces relative to the optimal interleaving, so it is much
   *looser*, not tighter, than the static bound — and steering the search by it
   instead of by the combined completion gives the same plateau (~470–504).

   Path-kick search already wins what little there is to win above the merge
   completion; the real open lead is a **better completion** — a sub-`1.75`
   constructive sorter (#2) — since that, not the search, is the binding
   constraint. *(All of this supersedes the fabricated `n = 52` planner figures
   flagged in `docs/old/HANDOFF.md` §2 — those were never run; these are
   reproducible via `experiments/planner_search.py` and `cascade_eval.py`.)*

---

# Part II — The whole-cycle model (permutation distance)

The project's origin: measure cost in whole drain-and-refill **cycles**.
Implemented in `splitmerge/cycle.py`, verified by `tests/test_cycle.py`.

- **What one cycle computes [PROVEN].** A cycle sends the deck to a 2-colouring
  (the split), each colour class is reversed by its stack, and the two reversed
  classes are freely interleaved (the merge). So the decks reachable in one cycle
  from `d` are exactly the interleavings of two reversed subsequences of `d`
  (`cycle.one_cycle_neighbors_bruteforce` constructs this set directly).
- **One-cycle reachability [PROVEN; VERIFIED n ≤ 6 vs the brute-force oracle].**
  `e` is reachable from `d` in one cycle iff `LIS(d⁻¹e) ≤ 2` (the relative
  permutation is 123-avoiding). *Proof idea:* a valid cycle is a proper
  2-colouring of the agreement graph, which is a permutation comparability graph
  (perfect), 2-colourable iff its clique number `LIS ≤ 2`. The relation is
  symmetric (`LIS(σ) = LIS(σ⁻¹)`), so the graph is undirected. Sortable in one
  cycle iff `LIS(d) ≤ 2`; the decks reachable from the identity in one cycle are
  the 123-avoiding permutations, counted by the Catalan number `Cₙ` (checked
  exactly for `n ≤ 6`: `2, 5, 14, 42, 132`).
- **Cayley / factorization form [PROVEN].** Two decks are one cycle apart iff
  `g⁻¹g′ ∈ C`, where `C = {σ : LIS(σ) ≤ 2}` (the generating set, `|C| = Cₙ`). So
  the reachability graph is the undirected Cayley graph `Cay(Sₙ, C)`, vertex-
  transitive, and `f(π)` = min cycles from the identity = the word length of `π`
  over `C`. SPLIT-MERGE PERMUTATION is the shortest-factorization / Cayley-
  distance problem with the Catalan-many 123-avoiding generators. `D(n) = max_π
  f(π)` is the eccentricity of the identity.
- **Membership in NP [PROVEN].** Any one card can be removed and reinserted
  anywhere in exactly two cycles, so `f(π) ≤ 2(n−1) = O(n)`; a YES-certificate is
  a short list of decks, each consecutive pair `LIS`-checkable. So the decision
  problem is in **NP**.
- **Diameter data [VERIFIED n ≤ 8; `n = 9` per uncommitted BFS].** `D(n) =
  1,1,2,2,2,3,3` for `n = 2..8` (reproduced by `cycle.cycle_diameter`); `D(9) = 3`
  was computed by an uncommitted compiled BFS (layer sizes `1, 4862, 261807,
  96210`, summing to `9!`), consistent with the pattern. **[CONJECTURE]**
  triangular `D(n) = (least c with c(c+1)/2 ≥ n) − 1` (≈ `√(2n)`); the reported
  `D(9) = 3` refutes a `⌈log₂ n⌉` law. The asymptotic growth (`Θ(√n)` vs
  `Θ(log n)`) is **[OPEN]** — `n ≤ 9` cannot separate them; `D(10)`, `D(11)`
  would.
- **`f` is not a function of `LIS` [VERIFIED n ≤ 8].** `LIS ∈ {3, 4}` occurs at
  both distance 2 and 3 (e.g. at `n = 7`, the 225 distance-3 permutations all
  have `LIS = 3`; at `n = 8`, distance-3 splits `LIS 3: 4537, LIS 4: 1225`). The
  `n = 7` case is asserted in `tests/test_cycle.py`; the `n = 8` split is computed
  by `cycle_distances`, not asserted. A genuine second-order correction lives on
  `LIS ∈ {3,4}`.
- **[INVALID] `f ≥ log₂ LIS(π)`.** `LIS` is not submultiplicative under
  composition (16/4000 random pairs at `n = 6` violate `LIS(ab) ≤ LIS(a)·LIS(b)`),
  so this tempting lower bound must not be used.

**Open (cycle model):** P vs NP-hard (settle `k = 2` first — `f(π) ≤ 2` iff some
123-avoiding `g` has `g⁻¹π` 123-avoiding; needs an invariant finer than `LIS`);
the asymptotic diameter; relaxing "exhaust the deck each split."

---

# Part III — Dead ends and retractions (consolidated)

Kept deliberately, clearly labeled, so they are not revisited.

- **[RETRACTED] `4(n−1)` as the operation diameter.** Linear fit to BFS `n ≤ 8`
  (`n = 9` per uncommitted BFS); refuted by the counting bound (I.2). Conjectured
  twice. What survives is the *exact value* `opt(reversal) = 4(n−1)` (I.5).
- **[NOT VERIFIED] Recursive-thirds and bidirectional merge** (I.3) — argued but
  never implemented here; bidirectional needs a modified `A↔B` machine. Their
  earlier benchmark figures came from uncommitted code.
- **[DEAD END] Patience/LIS, block-selection, two clean passes** (I.3) — defeated
  by the star topology and lack of random access.
- **[DEAD END] Greedy/Chaitin colouring and two early joint-bound formulations**
  for `OCT_pre` (I.4) — wrong direction / too strict / colouring-seed bug.
- **[DEAD END] Per-card upper bounds; greedy-`h` sorter** (I.4) — the per-card
  decomposition is a lower-bound tool only.
- **[INVALID] `f ≥ log₂ LIS(π)`** (cycle model) — `LIS` not submultiplicative.
- **[WRONG MACHINE] "4 cycles for 52 cards" and the monotone-cover /
  `⌈log₂(cover)⌉` theory** belong to a *stronger flip-enabled* machine, not this
  one. Distinguish also the **FIFO-reload** variant — a classic riffle shuffle
  with clean polynomial (`⌈log₂(rising sequences)⌉`, Bayer–Diaconis) theory —
  from the **LIFO** machine studied here. The three are genuinely different;
  small-`n` reachable-set sizes separate them.
- **[DEAD END] Multi-scale value-coarsening OCT sum** as an instance-sensitive
  `Ω(n log n)` lower bound (the §I.4a multi-scale proposal). Summing the static
  `OCT^(ℓ) = m − a₂` of the departure order σ coarsened to value-blocks of size
  `2^ℓ`, over all scales `ℓ`, **over-counts**: `OCT^(ℓ)` is monotone
  non-increasing in `ℓ` (coarsening only merges adjacent values, which can only
  enlarge the two-decreasing cover), so coarse-scale conflicts are a *nested
  subset* of fine ones and a single transfer is charged at every scale it
  survives. Measured (`src/bin/phitest.rs`, n ≤ 11): the sum overshoots `B_opt` by
  ~2× on the reversed deck at every n, and on 39/40 random decks by n = 11 (gap
  growing). The telescoping repair `Σ(OCT^(ℓ) − OCT^(ℓ+1))` collapses to the base
  `OCT^(0) = Θ(n)`. Since `OCT^(0)` already exhausts σ's static conflicts, **no
  value-partition hierarchy reaches `Θ(n log n)` admissibly** — the missing bulk
  is the dynamic cascade, invisible to any static function of σ.

---

# Part IV — Status summary

**Proven / verified.**
- Reversibility ⇒ undirected metric `g`; sorting ≡ producing; costs even. **[PROVEN]**
- Operation diameter is `Θ(n log n)` (counting LB `≥ 143` + merge-sort UB; `k = 3`
  star). **[PROVEN]**
- Natural merge sort: `2n⌈log₂ r⌉` ops (closed form proven for all `n`; replay-
  verified to `n = 400`). **[PROVEN/VERIFIED]**
- `h₀ ≤ h_best ≤ h_joint`, all admissible; `h_joint` dominates `h_best`.
  **[PROVEN; VERIFIED by full BFS at n = 6, 7, 8 in tests]**
- Exact `OCT_pre` matches the brute-force oracle (every state at `n ≤ 6`, sampled
  at `n = 7`); admissible chain-bound fallback on large far-from-clique graphs. **[VERIFIED]**
- `opt(reversed deck) = 4(n−1)` exactly, all `n`; IDA* certifies `204` at `n = 52`
  in `204` nodes. **[PROVEN]**
- Operation diameter `= 4(n−1)` for `n ≤ 8` (reproducible here; `n = 9` per absent
  BFS). **[VERIFIED n ≤ 8]**
- Three merge sorters emit machine-verified moves; Hu–Tucker optimal; worst cases
  `624 / 600 / 600`, averages `~520 / ~487 / ~484`. **[PROVEN/VERIFIED]**
- Cycle model: one cycle = two reversed subsequences interleaved; reachable iff
  `LIS(d⁻¹e) ≤ 2`; `f ≤ 2(n−1)`; decision ∈ NP. **[PROVEN]**
- Cycle diameter `1,1,2,2,2,3,3` for `n ≤ 8` (reproduced; `n = 9 → 3` per absent
  BFS). **[VERIFIED n ≤ 8]**

**Open.**
- P vs NP-hard for both SPLIT-MERGE-OPS and the cycle problem.
- The constant in `Θ(n log n)` and whether a sub-`1.75` constructive sorter exists.
- Exact operation diameter at finite `n` (`n = 52`: in `[204, 600]`).
- Asymptotic cycle diameter (`Θ(√n)` vs `Θ(log n)`).
- Polynomial exact `OCT_pre` with pre-colouring (remove the budget fallback).
- A practical sorter below the merge-sort frontier: the inadmissible planner
  (I.6 #5) is built but only reaches `~480` on random `n = 52` (the believed
  `~300` is not achieved); a better steering signal or search is open.

**The one-line state.** We can sort a shuffled 52-card deck in `~484` moves
(Hu–Tucker; `≤ 600` worst) and believe `~300` is achievable; the asymptotics are
settled at `Θ(n log n)`; the difficulty throughout is that the machine has no
random access.

---

## Appendix — code map

Rust crate (`src/`); the reference Python mirrors it module-for-module under
`old/splitmerge/`.

| module | backs |
|--------|-------|
| `src/machine.rs` | §0, I.1, `comb_solution` = the I.5 reversal witness |
| `src/search.rs` | `bfs_dist` (verification), `ida_star` (I.5) |
| `src/heuristics.rs` | `h0`, `h_best`, `h_joint` (I.4) |
| `src/oct.rs` | exact constrained `OCT_pre` (I.4) |
| `src/sorters.rs` | `natural` / `top-down` / `Hu–Tucker` sorters (I.3) |
| `src/cycle.rs` | one-cycle reachability, `f`, diameter (Part II) |
| `src/planner.rs` | inadmissible rollout estimate + anytime local search (I.6 #5) |
| `tests/validation.rs` | admissibility n≤8, OCT oracle, IDA*=BFS, reversed-52 = 204/204 |
| `src/bin/sm.rs` | experiment runner: `heuristics`, `sorters`, `conjecture`, `planner`, `cascade`, `frontier` |

**Provenance.** Earlier drafts (now in `docs/old/`) cited implementing code and
"*N* checks pass" counts that had never been committed. The merge sorters and the
whole-cycle model are now restored and verified; the bogus counts were replaced
by what the tests actually check; recursive-thirds and bidirectional merge remain
**[NOT VERIFIED]**. The `n = 52` local-search planner — whose fabricated figures
were the original void result — is now **built and measured** (I.6 #5), with the
honest outcome that it reaches the merge-sort frontier (`~480` random) but not
the hoped-for `~300`. The archived documents are kept only for their longer
proofs and history.
