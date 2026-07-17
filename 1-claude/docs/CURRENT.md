# CURRENT — active working context

**What this is.** Volatile saved context: the thread we are *actively* thinking
about, terse but enough to restart cold without re-reading everything. **Read this
first when resuming.**

**Workflow (important).** `docs/NOTES.md` is the **single** permanent record and is
updated from this file **only when something is removed here** — i.e. when a
question is settled, migrate its conclusion into NOTES (the bounce/cascade theory
lives in **NOTES §I.4a**) and delete it from CURRENT. This file summarizes the
frontier and points into NOTES; it does not replace it.

---

## Notation (so this file is standalone)

- **Machine.** Three LIFO stacks: hub **deck** `D` (also the I/O port) and **buffers**
  `A`, `B`. Four ops, each moves one top: `SA`/`SB` = `D→A`/`D→B`, `MA`/`MB` =
  `A→D`/`B→D`. No direct `A↔B` (a cross-buffer move costs 2, via `D`). Sort a start
  deck `π` (all in `D`, `A=B=∅`) to the identity `1..n`; cost = #ops. *Six-action
  view (for reasoning about transfers):* add cost-2 macros `TA = MB,SA` (`B→A`) and
  `TB = MA,SB` (`A→B`) — same machine, and **parking** is still allowed (a bare
  `MA`/`MB` landing on a non-base `D`).
- **settle / arrival / bounce.** A card *arrives* each time it enters `D` (`MA`/`MB`);
  it *settles* on the arrival that seats it permanently onto the growing base
  `1..i−1`. Each non-base card settles exactly once; any extra arrival is a
  **bounce**. `B` = total bounces (`g = 2m + 2B`); a **transfer** = a bounce realized
  as `A→D→B` (cost 2). A *double-transfer* card bounces twice (arrives ≥ 3×).
- `n` cards; `π` the deck (a permutation); `m` non-base cards.
- `σ` — **departure order** = the deck read top-to-bottom (the sequence sorted away).
- `a₂(σ)` — max #cards coverable by **two `σ`-decreasing subsequences**
  (Greene–Kleitman, polynomial). `OCT = m − a₂` is the **static bounce lower
  bound** (`m = n` for a clean base-free deck, the case here, so `OCT = n − a₂` below).
- `LIS` / `LDS` — longest increasing / decreasing subsequence of `σ` (`≈ 2√n` random).
- `r` — number of **ascending runs** (the merge sorter's granularity); `LDS ≤ r`.
- `OCT` is a bound on **bounces** `B`; `h_joint = h0 + 2·OCT` is the admissible
  bound on the full **move** count, so `opt − h_joint = 2·(opt_bounces − OCT) =
  2·cascade`. (`h_joint` is what the code/IDA* use; `OCT` is its bounce part.)
- **comb** — the reversed-deck-optimal construction: pile everything onto one
  buffer, then transfer the whole pile across (reversing it to sorted), each card
  once → `n−2` transfers. "comb-like" = a *global* one-sweep resolution.
- `Φ` — a hypothetical potential function (open Q1). `T` — the tangle graph
  (increasing-pair comparability graph of `σ`).

## The thread

Understand the operation-count optimum well enough to get **either** an
instance-sensitive lower bound **or** a sub-merge constructive algorithm. Current
focus: the **cascade** — the part of the bounce cost the static (tangle/OCT) bound
misses, which is where essentially all the difficulty is.

## Load-bearing facts (established; detail at `NOTES.md §I.4a`)

- Cost `g = 2m + 2B`; minimize **bounces** `B`. (I.1)
- **Settle-time lemma:** at each settle the deck holds only the base; *all*
  unsettled cards are in the two buffers. (I.4a)
- **Reduction (LOSSY — `park`):** "sort two stacks by transfers" treats D as a
  1-card transit slot, but that is **not** WLOG-optimal — *parking* a card in D
  (arriving it onto a deck still holding unsettled cards, using D as a 3rd LIFO)
  can lower `B`. First forced at **n=6** (`[1,3,5,6,4,2]`: transit-only needs
  `B=2`, parking gives `B=1`); necessary in >50% of random decks by n=9–10. So
  true-`B` can be *below* the transit-only transfer minimum — use the transfer
  view only for the (still valid) `OCT` *lower* bound; reason about the real
  4-move machine for opt. (I.4a)
- **Double-transfer atom — TWO mechanisms [`dbx`, `dbx9` exhaustive n=9, `dbxshow`]:**
  a card transfers ≥2× (enters D ≥3×) by **(a) re-burial** — evacuated, it lands on a
  *smaller* unsettled card in the other buffer (re-buried) — *or* **(b) hot-potato** —
  a card that settles *before* everything currently staged must stay top-accessible,
  freezing its buffer; with only two buffers it gets shuttled across to free each in
  turn, landing only on *larger* cards (never re-buried). The earlier "only by
  re-burial" claim is **refuted** (hot-potato is real: 55 of 287 n=9 forced decks,
  incl. every deck whose doubler is card 1, the global min). **Impossible for n≤8**;
  at **n=9 exactly 287 decks force a double** (all base-free), cheapest at **opt=22**.
  Smallest/lex-first = **`[3,5,2,4,7,9,6,8,1]`** (opt 22), a hot-potato on card 1 with
  *no* card-1-sparing optimum ⇒ necessarily hot-potato. (Old witness
  `[6,2,3,5,8,9,1,7,4]`, opt 24, was a sampled re-burial, **not** minimal.) Distinct
  from cascade (which starts smaller, as extra *single* bounces). Hot-potato is a
  buffer-**capacity** forcing, not a σ-value-structure one — see Q1.
- `σ` = departure order = deck top-to-bottom. **Single-arrival (bounce-free) ⟺ two
  `σ`-decreasing subsequences**; static bound `B ≥ OCT = m − a₂(σ)`, polynomial
  (Greene–Kleitman). (I.4a)
- **tangle** = increasing pair in `σ` (deck); **buried** = inversion in a buffer.
  Same inversion, two locations. (I.4a)
- **Where the complexity is:** `OCT = Θ(n)` (`a₂ = Θ(√n)`, Ulam–Hammersley) but
  `opt = Θ(n log n)` (counting LB for almost-all + merge UB). The static bound is an
  asymptotically vanishing fraction; the whole bulk is `cascade = opt − OCT`. (I.4a)
- **Safe/forced boundary:** placing departing `v` keeps a pile sorted iff
  `v < top(A)` or `v < top(B)`; **forced** iff `v >` both tops. All-defer (`B=0`) ⟺
  `LIS(σ) ≤ 2`. (I.4a)
- **At a forced event:** peel-to-fit = insertion sort = `O(n²)` (bad); **bury =
  O(1) deferred**, optimal on reversed (the comb). `#forced = OCT = Θ(n)` but
  `min-transfers > #forced` (the cascade) — deferred transfers re-bury, and the
  resolution schedule is **global** (comb reverses a whole pile), not per-event
  greedy. (I.4a)
- **Merge critique [granularity real, but NOT realizable as a sorter — `recreal`]:**
  ascending runs over-count; right granularity is the increasing-subsequence cover
  `= LDS ≤ r`. `log r → log(LDS)` moves the constant `1.75 → ~0.8` *in idealized counting*
  — but realizing the `log LDS` split on the single hub costs `~4·len`/level (the
  non-positional split tax), inflating it **past** merge (+37% on random); so the lever
  does not survive (Constructive thread / Q4). Also: two buffers hold only 2 open piles
  (`LDS ≈ 2√n`), and reversed shows `LDS` over-counts the *dual* way. (I.4a)

## Hard constraint — observability

Exact `opt` computable only to **n ≈ 14** (IDA*); `OCT = n − a₂` is poly at any
`n`. So the cascade is measurable only where it's `~1` bounce (n ≤ 14) and
**unmeasured where it dominates** (n ≥ 20). All large-`n` cascade/opt numbers are
**extrapolation**. ⇒ no `n` is both solvable and cascade-rich, so the cascade
**cannot** be reverse-engineered from optimal traces — this is a theory problem.

## Open (active) sub-questions

1. **Potential `Φ`** with `|ΔΦ| = O(1)`/move and `Φ(random) = Θ(n log n)` ⇒ the
   missing instance-sensitive lower bound. Single value-cuts are *vacuous*, and
   the multi-scale **value-coarsening OCT sum is now refuted** (`phitest`): any
   sum of value-scale OCTs over-counts (`OCT^(ℓ)` is monotone in scale ⇒ the
   scales share transfers) and the telescoping fix collapses to the base
   `OCT = Θ(n)`. ⇒ **no static function of σ's value structure** reaches
   `Θ(n log n)` admissibly; the missing bulk is the *dynamic* cascade, so `Φ` must
   price the LIFO-scheduling (amortized/adversary), not be a graph parameter of σ.
   Concrete witness this is real: the n=9 **hot-potato** double-transfer is a forced
   bounce with *no* σ-value-conflict at all (card 1, the global min) — pure buffer
   capacity. See the double-transfer atom in Load-bearing facts.
2. **Resolution schedule** of deferred transfers (the cascade): global/comb-like
   (cheap) or irreducibly tangled?
3. (magnitude half of 1–2) Does the running-`LIS`-excess of `σ` total `Θ(n log n)`,
   matching `opt`? — i.e. is the multi-scale sum the *right size*, not just a bound.
4. Two-open-pile policy → how close to `log(LDS)`? **[ANSWERED — not reachable on the single
   hub.]** Single-pass patience needs `LDS≤2`; the **idealized** recursion hits `log(LDS)`
   (−24% vs merge), but the **realized** recursion (`recreal`) inflates to `~4·len`/level (the
   non-positional split tax) and **loses to merge by 37%** on random — see Constructive
   thread. The `log(LDS)` granularity is not realizable as a sorter; merge's `~1.75` stands.

## Constructive thread — top-down mergesort degrees of freedom

Top-down / Hu–Tucker adjacent-merge tree (`sorters.rs`, cost `2·W` over **ascending
runs**, `W = Σ sᵢ·depthᵢ`) is the best constructive sorter so far (~1.75 const; avg
~484, worst 600 at n=52; it **remains the bar** — all three DOFs below now resolved, none
beats it on random). Two realizable DOFs aimed to push the merge granularity from `r`
(ascending runs) toward the LDS/RSK cover — the §I.4a "merge critique" lever (`log r → log
LDS`, const `1.75 → ~0.8`); **that lever is now refuted as a sorter** (the `log LDS`
granularity is not realizable on the single hub — DOF 1 / recpat below):

1. **Value-fit two-open-piles [ANSWERED — experiment (b), `psort`; detail NOTES §I.3].**
   (Aside: in the merge-tree model the A/B label is a cost-neutral symmetry — `W` depends
   only on leaf depths — and tree shape is already optimal via Hu–Tucker; the real freedom
   is value-aware *non-adjacent* combination.) Use the 2 stacks as 2 **patience piles**
   (place each departing card on the pile it fits under). Sorts in `2n` (one pass + merge)
   **iff `LDS(deck) ≤ 2`** = the `obvious`/interleave class, where it **crushes merge**
   (n=52: `104` vs Hu–Tucker `496`, replay-verified, 4.8×). But `LDS≤2` is a **vanishing
   fraction** (57% of decks at n=4 → 3.5% at n=8 → ~0% by n≥14; random `LDS≈2√n≈14`), so
   it's a **specialist**: never applies to random, and its flat `2n` even *loses* on
   low-run decks (`0` on sorted). **Complementary to reversal:** patience owns the
   interleave extreme (where reversal does nothing); the comb owns the reversed extreme
   (where patience is stuck, `LDS=n`). **Neither touches the random constant** — 2 piles
   overflow at `LDS≈2√n` (the §I.4a wall, now measured). ⇒ sub-merge on random needs the
   **recursion** (sort the split-halves before merging), but both routes there are now closed:
   the *multi-pass* (d) and the *recursive* realization (c', `recreal`) — the latter **built,
   verified, and REFUTED** (loses to merge by 37%; see *Recursive patience* below). = Q4.
2. **Reversal [ANSWERED — experiment (a), `revsort`; detail NOTES §I.3].** Reverse-aware
   adjacent-merge bracketed by an optimal-alphabetic DP charging a size-`s` descending
   leaf `c` per card (`c=0` free LB; `c=∞` = ascending-only baseline; `c≈2` realizable,
   comb-calibrated: whole reversed deck → `208 ≈ opt 204`). Verdict: **reversal helps
   only genuinely descending-structured decks** (reversed `600→208`, embedded long
   descending blocks `380→318`). On **random** the 5.4% free headroom collapses to
   **0.0% at `c=2`** (reverse costs `≈2s`, shred+merge `≈2s·log s`, so reversal wins only
   for runs `s≳3`, which random lacks). On **interleave/`obvious`** the gap is **exactly
   0 at every charge** — reversal is orthogonal to the class where merge is a `log`
   factor off (that's DOF 1, concatenation). The single-hub **parity** (a min-first run
   can't park min-on-top) is why reversal isn't free. *Residual:* exact realizable
   numbers need a replay-verified bidirectional sorter; `c≈2` is comb-calibrated, not
   yet move-emitted.

**Recursive patience [EXPERIMENT (c) idealized `recpat`; (c') REAL `recreal` — DONE, idealized
win REFUTED].** Split the deck into two subsequences each with ~half the LDS (Dilworth cover
into `LDS` increasing chains, 2-color them), recurse, merge → depth `~log₂LDS`, cost
`~2n·log₂(LDS)`. The **idealized** model (charges `2·len`/level) gave random n=52 `367.6` vs
Hu–Tucker `484` (−24%, 5000/5000, margin growing 16→28% over n=16→120). **The real
move-emitting sorter destroys that win.** `recreal` (distribute by chain-2-color onto the two
buffers = the non-positional split; nested-park + recurse each half + remerge, all
merge-by-exact-count; **exhaustively correct n≤8, replay-verified 15 500 random decks**) costs
n=52 **`661.7` = +80% over idealized, +37% over Hu–Tucker, beating merge on `0/5000`**, and
loses at *every* observable n (`real/hut` 1.42@16 → 1.29@120). **Root cause = the single-hub
tax the idealized model omits:** each internal node really costs `~4·len`, not `2·len` —
`distribute` (`len`, separate the interleaved groups) + `bring both groups back to D` (`len`,
to feed each recursion — cards live on buffers between levels) + `merge two now-*stacked*
sorted runs` (`2·len`; you can't merge in place on a LIFO hub — separate to A/B then pour).
Result-to-buffer reframings just move this cost (a wash; `4·len` is the floor). That extra
`2·len`/level is the **non-positional split tax**: merge splits *contiguously* (free), recpat
by *value/chain* (interleaved), and routing interleaved groups apart and back through the one
hub `D` costs 2 moves/card/level merge never pays. So the `log r → log LDS` lever is *real*
(fewer levels, `log₂LDS≈3.5` < `log₂r≈4.7`) but per-level cost **doubles** (4 vs 2), and
`2×0.75>1` ⇒ net loss; asymptotically `real/hut → ~0.9`, so rough *parity* only at
astronomical n, never a practical sub-merge sorter. **Nested parking itself works fine** (not
the retracted "4th stack" barrier) — it's the *hub routing of the interleaved split* that
inflates. ⇒ the §I.4a "merge critique" (`log r → log LDS`, const `1.75→~0.8`) is idealized
accounting that **does not survive the single hub**. Specialists unchanged: `LDS≤2` base =
patience2 (interleave `104`, matches ideal, crushes merge — but that's DOF 1); `reversed`
inflates catastrophically (`1120` vs `600`). **Multi-pass already a dead end [(d), `pass`]:**
iterated full-deck pass reduces LDS only **additively, −1/pass** (≫ merge). With recursion now
also refuted, **both routes to the `log LDS` granularity are closed**; the merge constant
~1.75 stands as the constructive bar, and whether *any* constructive sorter beats it on random
is back to **[OPEN]** (§I.6) with no live candidate.

**NEXT.** The constructive thread's realizable DOFs are now exhausted — reversal (specialist),
patience (specialist), recursion (refuted). No sub-merge-on-random construction is on the
table. The frontier is back to the **main thread**: the instance-sensitive lower bound — a
potential `Φ` pricing the LIFO scheduling/cascade (Q1), not a static function of σ's value
structure (refuted). The single-hub tax that just sank recpat is itself a *positive* clue for
the lower bound: it is a concrete, quantified cost the hub forces on any non-positional
regrouping — the same dynamic, capacity-driven cost the cascade/hot-potato is made of.

## Guardrails (don't repeat)

- "Drain-once / ≤1 bounce per card" is a **small-n artifact** — multi-bounce is
  forced (`avg bounces/card = Θ(log n)`). Don't re-import passes/recursion as an
  *assumption*; the optimum may be unstructured.
- The 2-stack-sorting literature (König–Lübbecke MinUnCut, Mihalák–Pont) is for a
  **different machine** (direct `A↔B` edge, "midnight" constraint) — verified real
  but **does not transfer**. Build the bound from scratch.
- Pursue the **lower bound before** a heuristic `Φ` (an ungrounded `Φ` was the
  rollout's failure).
- **Can't solve n ≳ 15 optimally** — don't propose tracing large-`n` optima.
- **Value-coarsening / dyadic OCT sums are refuted as a lower bound** (`phitest`,
  measured n ≤ 11): the scales double-count (`OCT^(ℓ)` monotone in scale ⇒ nested
  conflicts, shared transfers), and telescoping collapses to the base. Don't retry
  static multi-scale-on-σ; the `Θ(n log n)` is dynamic, not a partition statistic.
- **Don't assume D is transit-only / "bounces = transfers".** Parking is sometimes
  optimal (`park`, forced at n=6); the *transit-only transfer reduction* is lossy for
  the *optimum* — **not** the six-action machine itself (4 base ops + the cost-2
  macros `TA = MB,SA`, `TB = MA,SB`), which represents parking fine as a bare
  `MA`/`MB` onto a non-base D. `OCT ≤ B` is unaffected (the lower bound stands).

## Pointers

- `docs/NOTES.md` — full project record; **§I.4a** is the bounce/cascade theory (the
  `I.4a` tags above point there). `docs/drain-once-investigation.md` — the
  (superseded) reverse-engineering log.
- Core: `src/search.rs::ida_star_path` (exact opt, IDA*); `src/heuristics.rs::h_joint`
  (the `h0 + 2·OCT` admissible bound); `src/bin/analyze.rs` (`traces`, `stats`,
  `bounces`); `src/bin/detsort.rs` (split harness; `obvious` = the interleave class
  where merge is a `log` factor off).
- Scratch bins (each backs a load-bearing fact): `dbx`/`dbx9`/`dbxshow`
  (double-transfer atom; `dbx9` exhaustive n=9, `dbxshow` traces + the `hotpotato`
  necessity check), `park` (parking is sometimes optimal), `phitest` (multi-scale
  OCT-sum refutation), `revsort` (reversal cost-bracket — experiment (a)), `psort`
  (value-fit 2-pile patience sorter — experiment (b)), `recpat` (recursive-patience
  idealized cost model — experiment (c)), `recreal` (the REAL move-emitting recursive
  patience, replay-verified — experiment (c'); refutes the idealized −24%), `pass`
  (multi-pass LDS-reduction test — (d), the dead end).
