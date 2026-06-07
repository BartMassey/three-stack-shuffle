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
OCT, IDA*/BFS search, the constructive merge sorters, and the whole-cycle model
are all in `splitmerge/` and exercised by `tests/` (49 tests). Where a claim is
reproducible, the relevant module/test is named inline.

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
`k = 2, 3` its complexity is **[OPEN]** in the literature too. *(These citations
were not re-verified against the primary sources; treat the attributions as
provisional.)*

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
- *Bidirectional merge* **[NOT VERIFIED]**: free reversal-by-pour would let
  *monotone* (not just ascending) runs count — a big win on descending-structured
  input (reversed `624 → 156 = 3n`, a clean hand calculation) but **no
  typical-case win** (monotone runs are only ~0.83× ascending, rarely crossing a
  power-of-two pass boundary). It **assumes a modified machine** with a direct
  `A↔B` edge, so it cannot run on this machine at all; not implemented here.
- *Patience / LIS sorting* **[DEAD END]**: `LIS ≈ 12 < 16` at `n = 52` would save
  a pass, but forming the ~12 non-contiguous piles needs random access (binary
  search over pile tops) the machine lacks; formation costs more than the pass
  saved.
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
**[VERIFIED by full BFS n ≤ 7 in tests; n = 8 separately, 0 violations across
1.8M states]**. Implemented in `splitmerge/heuristics.py` and `splitmerge/oct.py`.

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
Validated against a brute-force oracle on all `n ≤ 7` states (0 mismatches) and
~33k larger random graphs; the reversed-deck size-52 clique solves in ~0.2 s.
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

## I.5 Exact search, and the exact value of the reversal

IDA* on `f = g + h` (pathmax, parent-move pruning, on-path cycle detection) finds
exact optima (`splitmerge/search.py`).

- **Validated [VERIFIED n ≤ 8]:** matches BFS optima on all tested decks;
  `h_best` and `h_joint` return identical optimal costs.
- **`h_joint` pays for itself [VERIFIED n = 10]:** vs `h_best`, ~84% fewer node
  expansions (median 810 → 111) and faster wall-clock despite the heavier
  per-node cost (`experiments/benchmark_heuristics.py`).
- **Residual gap.** On random start decks at `n = 10`, `cost − h_joint ≈ 1.1`
  (robust across seeds); the all-states mean at `n = 7` is `1.42`. (An earlier
  "≈ 2.4 at n = 10" figure was an unreliable carry-over and does not reproduce.)
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
   (pour the buffers back, then Hu–Tucker); the planner steers by the cheaper of
   the two and runs perturbed greedy descents within a time budget
   (`experiments/planner_search.py`). **Measured at `n = 52`:**
   - First complete solution: **`204` on the reversed deck** — the settle rollout
     is *exact* there, reconstructing the `comb` optimum — and **`~480` on random
     decks** (the merge rollout, on par with Hu–Tucker), each in **~0.5–1.6 ms**.
   - The two rollouts are mirror images: settle is exact on the reversal but
     `~850` on random; merge is `~480` on random but `600` on the reversal — so
     their min is never worse than either.
   - **Negative result:** the greedy local search **does not improve** on the
     first solution within seconds (0 improvement over 6 s on the reversal and on
     random decks, ~10–18 restarts). The rollout is a good *completion* but a poor
     *steering gradient* — the single-move landscape is flat — so the
     believed-achievable `~300` is **not** reached; the planner sits at the
     merge-sort frontier.

   **The "cascading charge" idea, measured [REFUTED].** A project note proposed a
   tighter inadmissible heuristic: simulate the forced bounces greedily ("every
   card bounces as soon as it would bury a smaller one") and count moves, hoping
   it beats `h_joint`'s static pairwise OCT. By the identity
   `|σ| = h0 + 2·(bounces)` this *cascading charge* equals the settle-rollout's
   length (`planner.cascade_charge` = `h0 + 2·cascade_bounces`), so it is exact on
   the reversal (`n−2` bounces). But measured against the exact optimum on random
   start decks (`experiments/cascade_eval.py`), it **overshoots**: cascade − opt
   ≈ 4, 8, 13, 13 at `n = 8, 9, 10, 11` and growing, overshooting on ~26–39 of 40
   decks, whereas `h_joint` sits **within ~1–2 below** opt. The greedy cascade
   *overcounts* bounces relative to the optimal interleaving, so it is much
   *looser*, not tighter, than the static bound — and steering the search by it
   instead of by the combined completion gives the same plateau (~470–504).

   Tightening the settle rollout's blocker-routing (the main quality lever) or
   replacing greedy descent with a non-greedy search is the open lead. *(This
   supersedes the fabricated `n = 52` planner figures flagged in
   `docs/old/HANDOFF.md` §2 — those were never run; these are reproducible.)*

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
  have `LIS = 3`; at `n = 8`, distance-3 splits `LIS 3: 4537, LIS 4: 1225` —
  asserted in `tests/test_cycle.py`). A genuine second-order correction lives on
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

---

# Part IV — Status summary

**Proven / verified.**
- Reversibility ⇒ undirected metric `g`; sorting ≡ producing; costs even. **[PROVEN]**
- Operation diameter is `Θ(n log n)` (counting LB `≥ 143` + merge-sort UB; `k = 3`
  star). **[PROVEN]**
- Natural merge sort: `2n⌈log₂ r⌉` ops (closed form proven for all `n`; replay-
  verified to `n = 400`). **[PROVEN/VERIFIED]**
- `h₀ ≤ h_best ≤ h_joint`, all admissible; `h_joint` dominates `h_best`.
  **[PROVEN; VERIFIED n ≤ 7 in tests, n = 8 separately]**
- Exact `OCT_pre` matches the brute-force oracle (`n ≤ 7` + ~33k random graphs);
  admissible chain-bound fallback on large far-from-clique graphs. **[VERIFIED]**
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

| module | backs |
|--------|-------|
| `splitmerge/machine.py` | §0, I.1, `comb_solution` = the I.5 reversal witness |
| `splitmerge/search.py` | `bfs_dist` (verification), `ida_star` (I.5) |
| `splitmerge/heuristics.py` | `h0`, `h_best`, `h_joint` (I.4) |
| `splitmerge/oct.py` | exact constrained `OCT_pre` (I.4) |
| `splitmerge/sorters.py` | `natural` / `top-down` / `Hu–Tucker` sorters (I.3) |
| `splitmerge/cycle.py` | one-cycle reachability, `f`, diameter (Part II) |
| `splitmerge/planner.py` | inadmissible rollout estimate + anytime local search (I.6 #5) |
| `tests/` | admissibility, reversal, IDA*=BFS, OCT oracle, sorter replay, cycle, planner |
| `experiments/` | `benchmark_heuristics`, `conjecture_Mn`, `benchmark_sorters`, `planner_search` |

**Provenance.** Earlier drafts (now in `docs/old/`) cited implementing code and
"*N* checks pass" counts that had never been committed. The merge sorters and the
whole-cycle model are now restored and verified; the bogus counts were replaced
by what the tests actually check; recursive-thirds and bidirectional merge remain
**[NOT VERIFIED]**. The `n = 52` local-search planner — whose fabricated figures
were the original void result — is now **built and measured** (I.6 #5), with the
honest outcome that it reaches the merge-sort frontier (`~480` random) but not
the hoped-for `~300`. The archived documents are kept only for their longer
proofs and history.
