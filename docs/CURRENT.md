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
- **Merge critique:** ascending runs over-count; right granularity is the
  increasing-subsequence cover `= LDS ≤ r`. `log r → log(LDS)` moves the constant
  `1.75 → ~0.8` on random — but two buffers hold only 2 open piles (`LDS ≈ 2√n`),
  and reversed shows `LDS` over-counts the *dual* way. (I.4a)

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
4. Two-open-pile lazy policy: how close to `log(LDS)` rounds can it get?

## Constructive thread — top-down mergesort degrees of freedom

Top-down / Hu–Tucker adjacent-merge tree (`sorters.rs`, cost `2·W` over **ascending
runs**, `W = Σ sᵢ·depthᵢ`) is the best constructive sorter so far (~1.75 const; avg
~484, worst 600 at n=52). Two realizable DOFs aim to push the merge granularity from
`r` (ascending runs) toward the LDS/RSK cover — the §I.4a "merge critique" lever
(`log r → log LDS`, const `1.75 → ~0.8`):

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
   overflow at `LDS≈2√n` (the §I.4a wall, now measured). ⇒ sub-merge on random needs >2
   open piles (impossible) or the cascade (recursive/multi-pass patience = the open hard
   problem). = Q4.
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

**Recursive patience [EXPERIMENT (c) — idealized, promising; `recpat`].** Split the deck
into two subsequences each with ~half the LDS (Dilworth cover into `LDS` increasing
chains, 2-color them), recurse, merge → depth `~log₂LDS`, cost `~2n·log₂(LDS)`. Idealized
result (charges `2·len`/level; like revsort's free bound): random n=52 **367.6 vs
Hu–Tucker 484 (−24%), winning on 5000/5000**, matching `2n·log₂LDS=367` exactly, and the
**margin GROWS with n** (16%→28% over n=16→120) — the real `log r → log LDS` lever, the
first construction that would beat the merge constant on random. Still ~`367` vs opt
`~250`, so LDS-halving isn't the whole story (full RSK, per §I.4a). **Realizability is the
open crux:** merge sort is realizable because its splits are *positional* (contiguous
blocks park as inert stacked runs, "merge by exact count"); the LDS-halving split is
*non-positional* (interleaved subsequences) and can't park that way — sorting one half
seems to need to store the other, i.e. a 4th stack (the same 2-buffer wall). **Next:** is
there a realizable *multi-pass* analog — a full-deck pass (D→A,B→D) that halves LDS — that
would make the 24% real? That's the concrete open question.

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
  idealized cost model — experiment (c)).
