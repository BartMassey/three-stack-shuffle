# The Split–Merge Machine: Sorting Cost and Permutation Distance

*A consolidated narrative and basis for a technical paper. This document is the
entry point; it states results, sketches the key arguments, reconciles the
threads, and labels every dead end and retraction. Full proofs and empirical
tables live in the `sources/` documents, cited inline. To continue the work
(and for a proposed anytime local-search planner that is **not yet built**), see
`docs/HANDOFF.md`.*

> **Caveat (void local-search results).** An exploratory session reported
> `n = 52` local-search results that were never actually run and are void (see
> `docs/HANDOFF.md` §2). Nothing about that planner appears as a result here; it
> is listed only as a proposed direction.
>
> **Provenance (resolved).** Earlier drafts of this document and the `sources/`
> notes cited implementing code and inflated "*N* checks pass" counts for the
> merge sorters and the whole-cycle model that had **never been committed**.
> That code is now in the repo and verified by the test suite
> (`splitmerge/sorters.py`, `splitmerge/cycle.py`; `tests/test_sorters.py`,
> `tests/test_cycle.py`); the worst cases (`624`/`600`), averages
> (`~520`/`~487`/`~484`), and cycle diameters (`1,1,2,2,2,3`) all reproduce, and
> the bogus check-counts have been replaced with what the tests actually verify.
> **Two constructions remain `[NOT VERIFIED]`:** recursive-thirds (tried; a
> measured dead-end, never implemented here) and bidirectional merge (needs a
> *modified* machine with a direct `A↔B` edge, so it cannot run on this machine).

**Status legend.** Every claim is tagged:
**[PROVEN]** (complete argument), **[VERIFIED n ≤ N]** (exhaustive computation),
**[CONJECTURE]**, **[OPEN]**, **[RETRACTED]** (believed earlier, now known
false), **[DEAD END]** (a route that does not work, kept so it is not retried).

---

## 0. The machine

Three LIFO stacks, each with an accessible top: a hub **deck** `D` and two
**buffers** `A`, `B`. Four operations, each pops one stack's top and pushes it
onto another:

| op | effect | | op | effect |
|----|--------|-|----|--------|
| `SA` | `D → A` | | `MA` | `A → D` |
| `SB` | `D → B` | | `MB` | `B → D` |

`A` and `B` never exchange directly; `D` is the hub, so a cross-buffer move
costs 2 (`A → D → B`). Each operation is the inverse of another
(`SA ↔ MA`, `SB ↔ MB`), so the configuration graph is **undirected**
**[PROVEN]**. Cards are distinct and identified with a permutation by rank.

Two cost measures have been studied, and they must not be conflated:

- the **operation-count model** (Part I): operations interleave freely, cost =
  number of operations. This is the main object of the project.
- the **whole-cycle model** (Part II): a *cycle* drains the entire deck into the
  buffers (`n` splits) then merges everything back (`n` merges); cost = number
  of cycles. This was the project's origin.

A cycle is a restricted operation schedule of length `2n`, so the cycle distance
`f` and the operation distance `g` satisfy `g(π) ≤ 2n·f(π)` — but free
interleaving usually does far better, and the two measures live on different
scales (for the reversal, `g = 4(n−1)` while `2n·f` grows super-linearly). Treat
them as two separate questions about one machine.

> **Idealized chronology.** The clean story runs Part I → Part II: the
> operation-count model is where the strongest, most complete results sit, so it
> leads; the whole-cycle model follows as a complementary lens and the project's
> historical seed. The *actual* order of discovery was the reverse and messier;
> Part III records the wrong turns.

---

# Part I — The operation-count model

## I.1 Problem and basic structure

**SPLIT-MERGE-OPS.** Given a target permutation `π`, transform the deck from the
identity to `π`, starting and ending with `A = B = ∅`, in the fewest operations.
Write `g(π)` for that minimum.

- **Sorting equivalence [PROVEN].** Every operation is reversible, so a sequence
  taking `S → T` reverses (swap `SA↔MA`, `SB↔MB`) into one taking `T → S` of
  equal length. Producing `π` from the identity costs exactly what sorting `π`
  to the identity costs; a sorter is all we need. `g` is a genuine metric.
- **Costs are even [PROVEN].** Each op changes `|A| + |B|` by ±1, and a solution
  starts and ends at 0.
- **Active-block reduction [VERIFIED n ≤ 6].** Strip the longest correct bottom
  run `1, …, f`; only the `m = n − f` remaining cards matter.
- **Floor [PROVEN].** `g(π) ≥ 2m`: every active card must leave `D` at least once
  and return. Writing `g = 2m + 2R`, the **relocation count** `R` counts cards
  forced to enter `D` more than once — the central difficulty. (In the heuristic
  language of Part I.4, these relocations are *bounces*.)

Full treatment: `sources/operation-count-theory.md` §1–3.

## I.2 The diameter is Θ(n log n)

This is the headline structural result, and it settles the asymptotics.

- **Counting lower bound [PROVEN].** Out-degree is ≤ 4, so the ball of radius `L`
  holds < `4^{L+1}` configurations. Reaching all `n!` permutations needs
  `4^{L+1} ≥ n!`, hence `max_π g(π) ≥ ½ log₂(n!) − O(1) = Ω(n log n)`.
- **Matching upper bound [PROVEN; replay-verified to n = 400].** The natural merge
  sort (I.3) sorts any deck in `2n⌈log₂ r⌉ ≤ 2n⌈log₂ n⌉ = O(n log n)` operations.
- **Therefore the diameter is `Θ(n log n)`.** No linear-operation sorter exists.

**Literature placement [PROVEN].** The machine is a 3-stack *star* (reusable hub)
in Tarjan's networks-of-stacks framework — the `k = 3` regime. Felsner & Pergel
(ESA 2008) give `O(n log_{k−1} n)`, i.e. `O(n log n)` for `k = 3`, matching the
merge sort; their counting lower bound matches ours. The `k = 2` `Ω(n²)` results
(Mihalák & Pont) do **not** apply, because the hub is a reusable third stack.
The exact optimum is NP-hard for general `k ≥ 4` networks (König & Lübbecke);
for `k = 2, 3` its complexity is **[OPEN]** in the literature too.

> **[RETRACTED] `4(n−1)` is not the diameter.** Exhaustive BFS gives diameter
> `4(n−1)` for `n = 2..8` (`4, 8, …, 28`; reproducible here — `n = 9 → 32` was
> reported from an uncommitted larger BFS), which fits a linear law exactly and
> twice tempted a linear-diameter conjecture (once early, once again during the
> later heuristic work of I.4–I.5). It is a **small-`n` artifact**: the counting
> bound is loose for small `n` (`≥ 143 = ⌈log₃ 52!⌉` at `n = 52`, below the
> reversal's `204`) and only provably exceeds `4(n−1)` near `n ≈ 213–690`. The
> diameter is `Θ(n log n)`, so the reversal is
> **not** the asymptotic worst case. See I.5 for what *is* true about the
> reversal, and Part III for the precise reconciliation.

Full treatment: `sources/operation-count-theory.md` §3, §7, §8.

## I.3 Constructive sorters and the constant

The asymptotics are settled, so the live constructive question is the **constant**
in `c · n log₂ n`. The measured **mean** optimum over start decks (exhaustive
BFS) is `c ≈ 0.9` and drifting down (`1.01, 0.91, 0.85` at `n = 4, 6, 8`); the
best construction sits near `1.75`. (The *diameter* constant is higher — `≈ 1.1`
at the largest BFS-computed `n` — and is a separate quantity.)

- **Natural merge sort [PROVEN].** Buffers `A`, `B` are the two input runs of a
  balanced two-way merge; `D` accumulates merged runs. Reversals are harmless
  (each run flips once out and once back). Cost exactly `2n⌈log₂ r⌉`, `r` =
  ascending runs; `0` on sorted input; `≤ 624` worst at `n = 52`, `~520` typical.
  The simplest sorter, recommended when implementation simplicity matters.
- **Adaptive top-down merge [PROVEN/VERIFIED].** Worst case `600`, average `~487`
  at `n = 52` — beats plain merge on both.
- **Hu–Tucker optimal merge tree [PROVEN].** The cost identity `cost = 2·W(tree)`
  plus the Hu–Tucker optimal alphabetic tree gives the **best no-reversal merge
  sorter**: `W(T) ≤ 300` for *every* input, so `cost ≤ 600` for every input, with
  `600` attained only by the descending deck; average `~484`. Since the optimal
  cost never exceeds any sorter's, this **proves the diameter upper bound
  `M(52) ≤ 600`** (not the plain merge's looser `624`).

  *(All three above are implemented in `splitmerge/sorters.py` and verified by
  replay on the machine in `tests/test_sorters.py` — every emitted move stream
  sorts the deck, and counts match the closed forms.)*
- **Bidirectional merge [NOT VERIFIED].** The idea: free reversal-by-pour turns
  descending runs ascending, so monotone (not just ascending) runs would count —
  a big win on descending-structured input (reversed `624 → 156 = 3n`) but **no
  typical-case win** (monotone runs are only ~0.83× ascending runs). This
  **assumes a modified machine with a direct `A↔B` edge**, so it cannot run on
  the machine studied here; it is **not implemented or verified** in this repo,
  and its quoted figures (incl. `smart_sort`) are from absent code.

The merge family's worst case bottoms out at `600` (Hu–Tucker, provably optimal
*among no-reversal merge sorters*). Beating `600` worst-case would require a
non-merge construction — the open `~0.9`-constant direction below.

**[DEAD END] Constructions that do not beat merge sort:**
- *Recursive-thirds* (Felsner–Pergel `k = 3`) **[NOT VERIFIED]**: on the **star**,
  `A↔B` costs 2, so the three stacks give a third *storage* area but not a third
  *merge stream* — structurally binary, so it cannot beat the binary merge family
  and the `~1.0` constant needs a true triangle (direct `A↔B`). We tried it; the
  earlier "measured constant ~2.2 (worse than merge)" came from code that is
  **not in this repo and was not reproduced** — treat as an untested dead end.
- *Patience / LIS sorting*: `LIS ≈ 12 < 16` at `n = 52` would save a pass, but
  forming the ~12 non-contiguous piles needs random access (binary search over
  pile tops) the machine lacks; formation costs more than the pass saved.
- *Block-selection* (extract smallest `k`, iterate): `Θ(n²/k)` re-handling, no
  random access to buried cards; best `k` ≈ 576 > 520.
- *Two clean passes* (`g ≤ 4n`): the one-pass set `{g ≤ 2n}` does not cover `Sₙ`
  under self-composition (`4815/5040` at `n = 7`); a constant number of passes is
  forbidden by counting (I.2).

**The recurring obstruction.** Every sub-merge idea dies on the same rock: **no
random access** (selection can't reach a buried card; patience can't reach the
right pile). Merge survives because it only touches accessible ends. Whether a
constructive sorter with constant `< 1.75` exists is **[OPEN]** (I.6).

Full treatment: `sources/operation-count-theory.md` §4–11; `sources/SORTING-BOUNDS.md`.

## I.4 Lower-bounding the relocations: an admissible-heuristic program

Beating the merge sort needs a handle on the *exact* optimum, i.e. on `R`. The
program below builds admissible lower bounds on `R` (equivalently on `g`), which
drive exact search (I.5). All heuristics have the form `h = h₀ + 2·(lower bound
on bounces)`; all are admissible **[PROVEN]** and **[VERIFIED by full BFS n ≤ 7]**.

- **Per-card / bounce decomposition.** Each move handles one card, so
  `g = Σ_c (moves of c)`; a bounce is a forced extra entry into `D`. A
  *checkpoint invariant* (just before card `i` settles, `D = (1, …, i−1)`
  exactly) makes the per-card floors solution-independent. **[PROVEN]**
- **`h₀`** = `2·(above-base deck) + |A| + |B|` (the `2m` floor, refined for
  buffer occupancy). **[PROVEN]**; exact on rotations.
- **Buried charge.** A buffer card with a smaller card below it must lift off
  before that card settles → it bounces. `#buried` is a directional, exact-per-
  buffer floor. **[PROVEN]**
- **LIS / two-buffer charge.** Among above-base deck cards, an increasing
  subsequence of the departure order forces `LIS − 2` bounces (only two buffers).
  **[PROVEN]**, via Dilworth.
- **Composition theorem.** Disjoint card groups' floors add without
  double-counting (contention only raises move counts). **[PROVEN]**
- **`h_best`** = `h₀ + 2·max((LIS−2)⁺ + #buried, clique − 2)`, the conflict-clique
  being computed as a longest chain in the conflict comparability graph.
  **[PROVEN admissible]**.
- **`h_joint`** = `h₀ + 2·(#buried + OCT_pre)` — the **joint bound**, which
  eliminates the `max`. It bounds the largest set of cards that can simultaneously
  avoid bouncing via two necessary conditions: (N1) no buried card avoids
  bouncing; (N2) cards that cannot share a buffer must take different buffers, so
  the *soft-conflict graph* (deck–deck increasing; deck vs smaller non-buried
  buffer card), with buffer cards pre-coloured by their buffer, must be 2-colour-
  able. `OCT_pre` is the minimum vertex deletion achieving that. **[PROVEN
  admissible; VERIFIED full BFS n ≤ 7]**; **dominates `h_best` at every state**;
  average residual gap drops ~35–45 %.

**Consistency [PROVEN].** `h₀` is consistent (`Δ = ±1`). `h_best` and `h_joint`
are admissible but **not** consistent (a single move can drop them by 3, by
splitting a conflicting pair across buffers). Search therefore uses **pathmax**
and cycle/duplicate detection rather than `g`-pruning.

**Computing `OCT_pre`.** Exactly, this is Odd Cycle Transversal — NP-hard in
general, but the founding problem of *iterative compression* and **fixed-
parameter tractable in the deletion count `k`** (Reed–Smith–Vetta `O(3^k·mn)`;
Hüffner's engineered solver). The implementation (`splitmerge/oct.py`) computes
it **exactly** by odd-cycle branch-and-bound: since the soft graph is a
comparability graph (perfect), an induced subgraph is bipartite iff triangle-
free, so the chain bound `clique − 2 ≤ OCT` prunes the otherwise-fatal clique
case immediately (at a clique it equals the incumbent, certifying optimality on
the first descent — this is what defeats the `3^k` blowup that killed the early
attempts). Pre-colouring is encoded with two undeletable terminal vertices, which
only *prunes* the branching. It is **validated against a brute-force oracle on
all `n ≤ 7` states (0 mismatches)** and solves the reversal's size-52 clique in
~0.2 s. A search budget makes it degrade gracefully to the admissible chain
lower bound on the rare large *far-from-clique* graph (e.g. an `n ≥ ~30`
scrambled start), so `h_joint` stays admissible everywhere and never hangs.

> **Still open (engineering):** on those large far-from-clique graphs the budget
> falls back rather than finishing exactly. The comparability structure should
> give a genuinely *polynomial* exact `OCT_pre` — max union of two antichains
> (Greene–Kleitman) with the buffer pre-colouring folded into a min-cut — which
> would remove the budget entirely. Not yet implemented.

> **[DEAD END] Greedy/Chaitin colouring for `OCT_pre`.** A prioritized graph-
> colouring (Chaitin–Briggs, buffers as priority colours) finds *a* deletion set
> fast, but a *feasible* set is an **upper bound** on the minimum, and an
> admissible heuristic needs a **lower** bound. Plugging it in overshoots and
> breaks admissibility (4226 violations at `n = 6`). Colouring algorithms are
> structurally the wrong tool here.

> **[DEAD END, instructive] Two false starts on the joint bound.** (1) A direct
> "max single-arrival set" with the rule *no deck card above any smaller buffer
> card* was too strict — a deck card may sit above a smaller buffer card if that
> card itself bounces away first — giving 584 admissibility violations.
> (2) A coloring routine that seeded free vertices before propagating the fixed
> buffer colours rejected valid colourings (2481 violations). The fix is to (N1)
> handle the directional bury separately and (N2) seed the 2-colouring from the
> pre-coloured vertices first.

> **[DEAD END] Per-card *upper* bounds.** The optimal per-card move count grows
> like `2⌊n/3⌋`, so there is no "≤ `c` moves per card" upper-bound argument. The
> per-card decomposition is a *lower-bound* tool only. (And greedy descent on `h`
> is a poor sorter — it cascades and often fails to terminate — because `h` is a
> good floor but does not price the relocations a move creates downstream.)

Full treatment: `sources/HEURISTIC-BOUNDS.md` §1–13.

## I.5 Exact search, and the exact value of the reversal

IDA* on `f = g + h` (pathmax, parent-move pruning, on-path cycle detection)
finds exact optima.

- **Validated [VERIFIED n ≤ 8]:** matches BFS optima on all tested decks.
- **`h_joint` pays for itself [VERIFIED n = 10]:** vs `h_best`, ~84 % fewer node
  expansions (median 810 → 111) and faster wall-clock despite the heavier per-node
  cost; identical optimal costs.
- **Exact value of the reversal [PROVEN]:** `opt(reversed deck) = 4(n−1)` for all
  `n`. Upper bound: the explicit `comb_solution` (`machine.comb_solution`) sorts
  the reversed deck in exactly `4(n−1)` moves. Lower bound: `h_joint` (admissible)
  evaluates to `4(n−1)` on the reversal. The two meet, so the value is exact; and
  `h_joint` is *exact along the whole optimal reversal path*, so IDA* solves the
  reversed `52`-deck in exactly `204` node expansions, independently certifying
  `opt = 204`.

> **Reconciliation (read with I.2).** `opt(reversal) = 4(n−1)` is an *exact value
> for one family of inputs* and is solid. It coincides with the **diameter** only
> for `n ≤ 8` (full BFS reproducible here; `n = 9` reported from an uncommitted
> BFS) and is supported as the diameter through `n ≈ 11` by heuristic search — but
> this is the small-`n` artifact of I.2, **not** an asymptotic fact.
> At `n = 52` the diameter `M(52)` is pinned to `[204, 600]`: the **lower bound
> `204`** is the reversal's exact optimum (itself a strengthening of the counting
> bound `≥ 143 = ⌈log₃ 52!⌉` — see I.2/SORTING-BOUNDS §4); the **upper bound
> `600`** is the proven worst case of the Hu–Tucker sorter (I.3), since the
> optimum never exceeds any constructive sorter. The gap is wide because our best
> *constructive* sorter spends `600` on the reversal whose true optimum is only
> `204` — the merge family's suboptimality on hard inputs (the I.3 constant gap).
> A typical shuffled deck costs `~484` (Hu–Tucker). **Do not state
> `M(n) = 4(n−1)` as the diameter, and do not cite `624` as the upper bound —
> `600` is proven and tighter.**

Full treatment: `sources/HEURISTIC-BOUNDS.md` §14.

## I.6 Open problems (operation-count model)

1. **Complexity.** Is SPLIT-MERGE-OPS in P or NP-hard? `R` is not a function of
   any natural statistic tried (inversions, cycle structure, `LIS`-family). **[OPEN]**
2. **The constant.** Determine `lim sup g(π)/(n log₂ n) ∈ [½, 2]`. Is there a
   constructive sorter below `1.75` (e.g. merging Dilworth chains, exploiting free
   interleaving), approaching the measured `~0.9` optimum? **[OPEN]**
3. **Exact diameter at finite `n`** (e.g. is it `204` or larger at `n = 52`?). **[OPEN]**
4. **Polynomial `OCT_pre` with pre-colouring** — exact OCT is implemented
   (odd-cycle branch-and-bound, exact within a budget); making it polynomial via
   the comparability-graph 2-antichain (Greene–Kleitman) with pre-colouring would
   remove the budget and run `h_joint` search at full `n = 52`. **[OPEN, engineering]**

---

# Part II — The whole-cycle model (permutation distance)

The project's origin: measure cost in whole drain-and-refill **cycles**.

- **What one cycle computes [PROVEN].** A cycle sends the deck to a 2-colouring
  (the split), each colour class is reversed by its stack, and the two reversed
  classes are freely interleaved (the merge). So the decks reachable in one cycle
  are exactly the interleavings of two reversed subsequences.
- **One-cycle reachability [PROVEN, VERIFIED n ≤ 6].** `e` is reachable from `d`
  in one cycle iff `LIS(d⁻¹e) ≤ 2` (the relative permutation is 123-avoiding).
  *Proof idea:* a valid cycle is a proper 2-colouring of the agreement graph,
  which is a permutation comparability graph (perfect), so it is 2-colourable iff
  its clique number `LIS ≤ 2`. The relation is symmetric, so the graph is
  undirected; sortable-in-one-cycle iff `LIS(d) ≤ 2` (count = Catalan `Cₙ`).
- **Cayley / factorization form [PROVEN].** `f(π)` = min cycles from identity =
  word length of `π` over the generating set `C = {σ : LIS(σ) ≤ 2}` of the
  123-avoiding permutations. `D(n) = max_π f(π)` is the eccentricity of the
  identity (vertex-transitive).
- **Membership in NP [PROVEN].** Any one card can be removed and reinserted
  anywhere in exactly two cycles, so `f(π) ≤ 2(n−1) = O(n)`; a YES-certificate is
  a short list of decks, each consecutive pair `LIS`-checkable. So the decision
  problem is in **NP**.
- **Diameter data [VERIFIED n ≤ 8; `n = 9` per uncommitted BFS].** `D(n) = 1, 1,
  2, 2, 2, 3, 3` for `n = 2..8` (reproduced here); `D(9) = 3` was computed by an
  uncommitted compiled BFS. **[CONJECTURE]** triangular `D(n) = (least c with
  c(c+1)/2 ≥ n) − 1` (≈ `√(2n)`); the reported `D(9) = 3` refutes a `⌈log₂ n⌉`
  law. The asymptotic growth (`Θ(√n)` vs `Θ(log n)`) is **[OPEN]** — `n ≤ 9`
  cannot separate them; `D(10)`, `D(11)` would.
- **`f` is not a function of `LIS` [VERIFIED n ≤ 8].** `LIS ∈ {3, 4}` occurs at
  both distance 2 and 3; a genuine second-order correction lives there.

**Open (cycle model):** P vs NP-hard (settle `k = 2` first — `f(π) ≤ 2` iff some
123-avoiding `g` has `g⁻¹π` 123-avoiding; needs an invariant finer than `LIS`);
the asymptotic diameter; relaxing "exhaust the deck each split."

Full treatment: `sources/cycle-model-theory.md`; `sources/original-notes.md`.

---

# Part III — Dead ends and retractions (consolidated)

Kept deliberately, clearly labeled, so they are not revisited.

- **[RETRACTED] `4(n−1)` as the operation diameter.** Linear fit to BFS `n ≤ 8`
  (`n = 9` per uncommitted BFS); refuted by the counting bound (I.2). Conjectured twice (early; and again in the
  recent heuristic work as "`M(n) = 4(n−1)`, missing only a universal upper
  bound"). What survives is the *exact value* `opt(reversal) = 4(n−1)` (I.5).
- **[DEAD END] Greedy/Chaitin colouring for `OCT_pre`** — upper bound, wrong
  direction for admissibility (I.4).
- **[DEAD END] Two early joint-bound formulations** — too-strict single-arrival
  rule, and a colouring-seed bug (I.4).
- **[DEAD END] Per-card upper bounds; greedy-`h` sorter** (I.4).
- **[DEAD END] Recursive-thirds, patience/LIS, block-selection, two clean
  passes** — all defeated by the star topology and lack of random access (I.3).
- **[INVALID] `f ≥ log₂ LIS(π)`** (cycle model): `LIS` is not submultiplicative
  under composition (16/4000 random pairs at `n = 6` violate it).
- **[WRONG MACHINE] "4 cycles for 52 cards."** That figure (and the monotone-cover
  / `⌈log₂(cover)⌉` theory) belongs to a *stronger flip-enabled* machine, not
  this one. Also distinguish the **FIFO-reload** variant — a classic riffle
  shuffle with clean polynomial (`⌈log₂(rising sequences)⌉`, Bayer–Diaconis)
  theory — from the **LIFO** machine studied here. The three are genuinely
  different; small-`n` reachable-set sizes separate them.

---

# Part IV — Status summary

**Proven / verified.**
- Reversibility ⇒ undirected metric `g`; sorting ≡ producing; costs even. **[PROVEN]**
- Diameter of the operation model is `Θ(n log n)` (counting LB + merge sort UB;
  `k = 3` star). **[PROVEN]**
- Natural merge sort: `2n⌈log₂ r⌉` ops (closed form proven for all `n`; verified
  by machine replay to `n = 400` in `tests/`). **[PROVEN/VERIFIED]**
- `h₀`, `h_best`, `h_joint` admissible; `h_joint` dominates `h_best`. **[PROVEN;
  VERIFIED n ≤ 7 in tests, n = 8 separately]**
- `opt(reversed deck) = 4(n−1)` exactly, all `n`. **[PROVEN]**
- IDA* exact-search, `h_joint` ~84 % fewer nodes than `h_best`. **[VERIFIED n ≤ 11]**
- Cycle model: one cycle = two reversed subsequences interleaved; reachable iff
  `LIS(d⁻¹e) ≤ 2`; `f ≤ 2(n−1)`; decision ∈ NP. **[PROVEN]**
- Operation diameter `= 4(n−1)` for `n ≤ 8` (full BFS reproducible in this repo;
  `n = 9` was reported from a larger BFS not committed here); cycle diameter
  `1,1,2,2,2,3,3` for `n ≤ 8` (independently reproduced; `n = 9 → 3` from the
  uncommitted compiled BFS). **[VERIFIED n ≤ 8; n = 9 per absent code]**

**Open.**
- P vs NP-hard for both SPLIT-MERGE-OPS and the cycle problem.
- The constant in `Θ(n log n)` and whether a sub-`1.75` constructive sorter exists.
- Exact operation diameter at finite `n` (e.g. `n = 52`: in `[204, 600]` —
  lower bound the reversal's exact optimum, upper bound Hu–Tucker's worst case).
- Asymptotic cycle diameter (`Θ(√n)` vs `Θ(log n)`).
- Polynomial exact `OCT_pre` with pre-colouring (the comparability 2-antichain),
  to remove the budget fallback on large far-from-clique graphs. (Exact OCT is
  implemented; it is exact within a budget and admissible beyond it.)
- **Proposed, not yet built:** an anytime local-search *planner* on a tight
  inadmissible estimate (greedy descent + restarts, returning the best complete
  sort within a time budget), for practical sorting at `n = 52` without optimality
  guarantees. Spec and measurement protocol in `docs/HANDOFF.md` §4.

**The one-line state.** We can sort a shuffled 52-card deck in `~484` moves
(Hu–Tucker; `≤ 600` worst case) and believe `~300` is achievable; the asymptotics
are settled at `Θ(n log n)`; the difficulty throughout is that the machine has no
random access.

---

## Appendix — repository map

| code | backs |
|------|-------|
| `splitmerge/machine.py` | §0, I.1, `comb_solution` = the I.5 reversal witness |
| `splitmerge/search.py` | `bfs_dist` (verification), `ida_star` (I.5) |
| `splitmerge/heuristics.py` | `h0`, `h_best`, `h_joint` (I.4) |
| `splitmerge/oct.py` | exact constrained `OCT_pre` (I.4), branch-and-bound + budget |
| `tests/` | admissibility (I.4), reversal exactness (I.5), comb (I.5), IDA*=BFS, OCT oracle |
| `experiments/` | `benchmark_heuristics` (I.5), `conjecture_Mn` (I.5/III) |
| `docs/HANDOFF.md` | continuation note: void-results, the proposed planner spec, protocol |

| source doc | model | role |
|------------|-------|------|
| `sources/operation-count-theory.md` | operation | fullest Part I treatment + literature |
| `sources/SORTING-BOUNDS.md` | operation | merge-family algorithms and bounds |
| `sources/HEURISTIC-BOUNDS.md` | operation | the I.4–I.5 heuristic/search program |
| `sources/cycle-model-theory.md` | cycle | full Part II treatment |
| `sources/original-notes.md` | cycle | the project's seed notes |

*Provenance note:* `sources/` are the working documents this narrative
synthesizes. Where they disagree with this file, this file is current — in
particular on the `4(n−1)` diameter question (Part III).
