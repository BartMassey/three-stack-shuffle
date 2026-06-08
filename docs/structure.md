# Structure of the bounce-minimization problem

Working directly on the algorithm (operation-count model). We formalize the two
obstructions that force a card to move more than the minimum — **tangle** (deck
disorder) and **buried** (buffer disorder) — derive the static lower bound they
give, and then locate where the actual complexity is. Notation: deck `D`, buffers
`A`, `B`; sort a start deck `π` (in `D`, `A=B=∅`) to the identity; cost = #moves.

## 1. Cost = base + 2·bounces

**Checkpoint invariant** (proven). Every solution settles the cards onto the base
in value order: there are times `τ_1 < … < τ_n` with `D = (1,…,i−1)` just before
card `i` settles. So each above-base card must *leave* `D` before its turn and
*return* to settle.

Let `e_c` = number of times card `c` enters `D` (arrivals). A card that starts and
ends in `D` departs and arrives equally often, so it makes `2·e_c` moves, and

```
g(π) = 2·Σ_c e_c .
```

Write `m` = number of non-base cards (each needs `e_c ≥ 1`). A **bounce** of `c` is
an entry beyond the one that settles it: `b_c = e_c − 1 ≥ 0`. With total bounces
`B = Σ b_c`,

```
g(π) = 2m + 2B            (B ≥ 0; "h0" = 2m).
```

**Minimizing moves ≡ minimizing total bounces `B`.** Everything below is about `B`.

## 2. Single-arrival, and the 2-decreasing-cover

Let `σ = ` the deck read top-to-bottom = the **departure order** (by the invariant,
above-base cards first-depart in this order). A card is **single-arrival** if
`e_c = 1` (`b_c = 0`): it goes `D → buffer → D`-settle, once.

> **Theorem (single-arrival set).** A set `U` of cards can all be single-arrival
> iff `U` partitions into **two subsequences, each decreasing in `σ`**.
>
> *Proof.* (⇒) Two single-arrival cards in the same buffer: the earlier-departing
> sits below; they must pop in value order (smaller first), so the smaller must be
> on top = depart later. Hence each buffer's `U`-cards decrease in `σ`; two buffers
> give two decreasing subsequences. (⇐) Route each decreasing subsequence to its
> buffer in `σ`-order — each card is smaller than the current top, so it lands on
> top and the buffer is decreasing bottom-to-top, i.e. pops in increasing (settle)
> order; free `MA`/`MB` interleaving then settles `U` in value order. ∎

So the largest single-arrival set has size `a₂(σ)` = **max union of two decreasing
subsequences of `σ`**, and

```
B  ≥  m − a₂(σ)                       (the static lower bound).
```

## 3. Tangle = the deck's increasing structure

Build the **tangle graph** `T` on the above-base cards: an edge `x—y` iff `x, y`
are an *increasing pair* in `σ` (`x` before `y` and `x < y`). This is the
comparability graph of the 2-D dominance order `(σ-position, value)`, hence
**perfect**. In `T`:

- an increasing subsequence of `σ` = a **clique** (a chain); `ω(T) = LIS(σ)`;
- a decreasing subsequence = an **independent set** (an antichain);
- a single-arrival set = a set inducing a **bipartite** (2-colourable) subgraph.

So the static bound is an **odd-cycle transversal**: `B ≥ OCT(T) = m − a₂(σ)`,
where `a₂` = max union of two antichains. On a perfect comparability graph this is
**polynomial** (Greene–Kleitman / a min-flow; `a₂` = sum of the two longest
columns of the RSK shape). *This is exactly what `h_joint` computes* (`h_joint =
2m + 2·OCT(T)`). The "tangle" is the irreducible, statically-knowable bounce core.

## 4. Buried = the same inversion, in a buffer

`OCT(T)` lives in the deck. Its dynamic twin lives in the buffers. A card `x` is
**buried** if a smaller card `y` sits below it in the *same buffer*: `y` settles
first but `x` blocks it, so `x` must bounce. Buried and tangled are the **same
relation — an inversion forcing a re-entry — read in two places**: `tangle` = an
increasing pair in the *departure* order (`σ`, in `D`); `buried` = an increasing
pair in the *placement* order (in a buffer). The soft-conflict graph of
`heuristics.rs` is just `T` plus the buffer cards pre-coloured by their buffer and
the buried ones forced out; its `OCT` is the unified static bound.

For a **clean start deck** there is no buried yet — it is *all tangle*. Buried is
what tangle *turns into* when, during the sort, we place a card onto a smaller one.

## 5. Where the complexity is — NOT in tangle/buried

Here is the point of the formalization. Compare the two quantities for a typical
(uniform random) deck:

- **Static bound (tangle/buried).** `a₂(σ) = Θ(√n)` — the longest decreasing
  subsequence of a random permutation is `~2√n` (Ulam–Hammersley / Logan–Shepp–
  Vershik–Kerov), and two of them cover `Θ(√n)`. So
  `B_static = m − a₂ = n − Θ(√n) = Θ(n)`, i.e. `h_joint ≈ 4n − 2a₂ = Θ(n)`
  with the constant `→ 4`.
- **The optimum.** `g = Θ(n log n)` for **almost every** deck, not just the worst
  case: the ball of radius `L` holds `≤ 3^L` decks, so for any `ε>0` all but a
  vanishing fraction of the `n!` decks have `g ≥ (1−ε)·log₃(n!) = Ω(n log n)`
  (counting); the merge sort gives the matching `O(n log n)`. So
  `B_opt = (g − 2m)/2 = Θ(n log n)` typically (measured `≈ 0.8·n log₂ n`).

Therefore

```
B_static / B_opt  =  Θ(n) / Θ(n log n)  →  0.
```

**The static obstructions are an asymptotically vanishing fraction of the work.**
Tangle/buried prove only that *almost every card must bounce at least about once*
(you can single-arrival just `Θ(√n)` of them). They are blind to the fact that the
average card bounces `Θ(log n)` times (§ the multi-bounce theorem). The entire
`Θ(n log n)` bulk — and all the open difficulty — is the **cascade**: the dynamic
fact that *evacuating a buried card re-enters it into `D`, where re-placing it
creates new buried cards*, recursively. The static graph `T` has no edges for
"bounces created by handling other bounces."

This is why `h_joint` is tight at small `n` (cascade `≈ 0` until `n ≈ 30`) and a
vanishing fraction asymptotically; why no static/2-colouring construction can
match opt; and why the merge sorter — which is *entirely* cascade (every card
re-handled `⌈log₂ r⌉` times) — is within a constant factor of optimal while the
tangle bound is a `log n` factor below it.

## 6. So the real object is the cascade

The decomposition:

```
B_opt  =  OCT(T)            +            cascade(π)
          └ static, poly,                └ dynamic, Θ(n log n),
            Θ(n), "tangle"                 the open hard core
```

To "understand the complexity" we must formalize **cascade**: choosing, over time,
which buffer each bounce goes to and when, so the re-entries it forces are
minimized. **We do not assume it is pass- or recursion-structured.** The merge
sorter *is* recursive (uniform passes; every card paid `⌈log₂ r⌉`), and that
uniformity is precisely its `1.75` waste. The optimum need not look like that at
all: the deck and buffer sizes may move up and down freely, cards settling
opportunistically the instant they become accessible, with the `Θ(log n)`-average
re-entries spread *unevenly* (most cards once, a few many times) rather than
log-deep across all. We genuinely cannot observe this at scale — `opt` is
uncomputable past `n ≈ 15` — so a clean global structure there is unfalsifiable
and not to be presumed. The live possibility: the optimum is the trajectory of a
simple *local/online policy* (settle-when-you-can + good buffer routing) whose
emergent path is unstructured. Two structure-agnostic handles: a **potential**
`Φ` with `|ΔΦ| = O(1)` per move and `Φ(start) = Θ(n log n)` (a per-instance lower
bound tighter than `OCT`, and a hint at the policy), and direct design+analysis
of such a local policy. The counting lower bound is itself structure-agnostic, so
nothing forces passes.

## 7. The lower-bound question, and where graph theory enters

**Pursue the lower bound before the heuristic `Φ`.** The gap is *entirely* here:
`OCT = Θ(n)` is the only instance-sensitive bound we have, and it's a `log n`
factor short; the counting `Θ(n log n)` is uniform (same for every deck) so it
can't see that reversed is `Θ(n)` and random is `Θ(n log n)`. There is **no
instance-sensitive `Ω(n log n)` bound** — that's the missing keystone. And a
lower-bound potential and a steering potential are plausibly the *same* `Φ`, so
the bound is the higher-leverage object; a `Φ` not grounded in one risks being
loose-but-plausible (as the rollout was).

**What actually needs bounding — and it is not the static conflict.** The cascade
is the cost of *LIFO-scheduling* the 2-colouring, not of its existence. Tellingly
it is **not monotone in `a₂`**: it is `0` at both extremes — reversed
(`a₂ = 2`, fully tangled but *regular*, the comb pays no cascade) and sorted
(`a₂ = n`) — and `Θ(n log n)` in the middle, for *generic/irregular* permutations.
So the right bound is **incompressibility/entropy-flavoured** (a random `π` carries
`Θ(n log n)` bits the machine must "pay off"; structured families are cheap),
*instance-sensitive*, and unrelated to `OCT`. The counting argument is the global
shadow of this; we need its per-instance, structural form.

**Candidate graph-theory tools (to mine, not yet established here):**
- **The existing stack-sorting literature does not transfer** (checked). This is a
  shuffle-minimisation problem in a stack network (Tarjan 1972), but the nearby
  hardness/approximation results are for *different* machines: König–Lübbecke
  (ISAAC 2008) prove `k ≥ 4` *complete* networks NP-hard via Min-`k`-Partition on
  circle graphs; Mihalák–Pont (ATMOS 2019) handle *two stacks with a direct `A↔B`
  edge* under the "midnight" constraint (an `O(√log n)` *approximation*, not a
  lower bound). Neither models our reusable hub / no-`A↔B`-edge machine, and the
  `k = 2, 3` exact complexity is open — so there is **no off-the-shelf cut
  formulation to borrow**; the cut idea below must be built from scratch.
- **Buffer occupancy = a cut/separation profile.** `|A|+|B|` over departure time is
  a vertex-separation / cutwidth-like quantity of `π`; the cost integrates it. The
  cutwidth / Minimum-Linear-Arrangement / pathwidth literature is the natural home
  for turning that profile into an `Ω(n log n)` bound.
- The **full Greene–Kleitman hierarchy** `a_1, a_2, …` (whole RSK shape), not just
  `a_2` — the "2" is the two buffers, the missing `log` should come from iterating
  the chain/antichain structure, if it is a graph parameter at all.

## 8. A clean reduction, and why naive cuts fail

**Settle-time lemma.** Just before card `i` settles, `D = (1,…,i−1)` *exactly*
(any extra card would seat `i` too high). So at the instant `i` settles, the deck
holds only the committed base and **all `n−i` unsettled cards are in the two
buffers**. The deck is thus never a place to *store* unsettled cards across a
settle — only an I/O port and a one-card transit slot.

**Bounces = transfers between two stacks.** Consequently every non-settling deck
entry is a card going `A → D → B` (or symmetric): a **transfer** of a stack top to
the other stack, costing 2 moves. With `g = 2n + 2B`, **`B` = number of
inter-buffer transfers**, and the whole problem reduces to:

> Drain the deck (one stack, in departure order `σ`) into two stacks and extract
> `1,…,n` in increasing order; the only nontrivial operation is moving a stack-top
> to the other stack (cost 2). **Minimize transfers.** (The split is itself free
> to choose.)

This is *sorting two stacks by transfers* — a cleaner object than the full machine,
for both the bound and the policy.

> **[CORRECTION — the reduction is LOSSY] (`src/bin/park.rs`, exhaustive n ≤ 8).**
> The "one-card transit slot" claim is **false**: `D` is itself a LIFO stack, and
> some optima must *park* a card in it — arrive a buffer card onto a `D` that still
> holds unsettled cards (using `D` as a third stack), then carry on. First forced
> at **n = 6** (`[1,3,5,6,4,2]`, opt = 12: the transit-only model needs `B = 2`, but
> parking sorts it in `B = 1`); parking is necessary in >50% of random decks by
> n = 9–10. So `B_true` can be **strictly below** the minimum inter-buffer transfer
> count of the two-stack model — the reduction *over-counts* and is not a faithful
> model of the optimum. It survives only as the source of the `OCT` **lower** bound
> (`OCT ≤ B_true`, proven and tested); use it only that way. The settle-time lemma
> itself (`D = (1,…,i−1)` at the settle instant) is unaffected — parking happens
> *between* settles.

**Why a single value-cut proves nothing.** Project values to two classes
(`≤ k`, `> k`). The projected instance has only two distinct values, so it needs
**zero** transfers (push one class to each stack; equal values never bury). So
`B(π) ≥ B(\text{binary projection}) = 0` — vacuous. Bounces are not forced by any
coarse 2-way separation; they require a **3-way** distinction (three values can
have `LIS = 3`, exceeding the two stacks). This is the crux: the cost lives in the
*fine, multi-scale* order, not in any single cut — which is exactly why it is
`Θ(n log n)` (a hierarchy of `≈ log n` refinements) and why no single
separator/cutwidth quantity can capture it.

**So the lower bound we want is multi-scale.** Not one cut but a *hierarchy*: e.g.
sum over a dyadic refinement of the value range of the transfers forced at each
level (3-way `OCT` of each refinement), `Θ(n)` per level over `Θ(log n)` levels for
random, but `Θ(n)` total for reversed (whose refinements stay monotone, hence
transfer-free under reversal-by-pour). Making that sum a valid lower bound — i.e.
showing transfers at different levels cannot be shared — is the open core; it is
where an entropy / amortized-potential argument, not a static graph parameter,
seems required. The reduction above is the right arena to attempt it.

> **[REFUTED — the value-coarsening instantiation] (`src/bin/phitest.rs`, n ≤ 11).**
> The transfers *are* shared, so the naive sum fails. `OCT^(ℓ) = m − a₂` of σ
> value-coarsened to blocks of size `2^ℓ` is **monotone non-increasing in `ℓ`**
> (coarsening only merges adjacent values, which can only enlarge the
> two-decreasing cover), so coarse-scale conflicts are a *nested subset* of fine
> ones and the sum charges each transfer once per surviving scale. Measured: it
> overshoots `B_opt` by ~2× on the reversed deck at every n and on 39/40 random
> decks by n = 11. The telescoping repair `Σ(OCT^(ℓ) − OCT^(ℓ+1))` collapses to the
> base `OCT^(0)`. Since `OCT^(0)` already exhausts σ's static conflicts, **no
> value-partition hierarchy reaches `Θ(n log n)` admissibly**: the missing bulk is
> wholly the *dynamic* cascade, and the bound must be amortized over the schedule,
> not a static parameter of σ at any resolution. (Recorded in NOTES Part III.)

## 9. Runs vs. increasing-subsequence cover — "leave a run for later"

The merge sorter pays `2·W(T)` for a binary tree `T` over the `r` **ascending
runs**; Hu–Tucker minimises `W ≈ n·H(\text{run lengths}) ≤ n·log r`. But ascending
runs are a **positional** artifact and *over-count* the disorder.

> **Critique.** Ascending runs are *one* partition of the deck into increasing
> subsequences (each run is increasing), using `r` pieces. The **minimum**
> increasing-subsequence cover is `LDS` (Dilworth's dual; `LDS` = longest
> decreasing subsequence), and `LDS ≤ r` always — often `≪` (interleave deck:
> `LDS = 2` vs `r = n/2`; random: `LDS ≈ 2√n` vs `r ≈ n/2`). The merge sorter
> *chops* coherent increasing subsequences into positional runs and re-merges them.

**"Leave a run for later" = keep an increasing subsequence intact.** A card only
ever has to move when a **smaller** card must pass it; *within* an increasing
subsequence no card sits above a smaller one, so the whole subsequence pours out
sorted for free (`0` transfers). So the cost is the number of *cross-subsequence*
blocking events, and the `r − LDS` extra run-boundaries are exactly runs that
needn't be a separate merge leaf — they can be absorbed into a longer subsequence
("left for later") instead of merged early.

**Quantitatively this is the right lever.** Swapping `log r` for `log(LDS)` takes
the constant from the merge sorter's `~1.75` (`log(n/2)`) toward `log(2√n) =
½ log₂ n + O(1)` — i.e. roughly halves it, into the neighbourhood of the measured
optimum `~0.8`. Your instinct points straight at the optimal constant.

**Two honest caveats.**
1. *`LDS` is not the whole story* — it over-counts the *dual* way. Reversed has
   `LDS = n` yet `opt = Θ(n)` (it's one *decreasing* run, sorted by
   reversal-by-pour). So the cost is governed by the interaction of increasing
   *and* decreasing structure (the full RSK shape), and `LDS` alone over-counts
   decreasing-structured inputs exactly as `r` over-counts the interleave.
2. *Realisation is the open obstruction.* Two buffers can keep only **two**
   increasing subsequences "open" at once, but a random deck has `≈ 2√n` of them;
   the rest must be merged, and whether that merge runs in `~log(LDS)` rounds
   *without* the binary-search-over-pile-tops random access patience needs is the
   open Greene–Dilworth question. The fresh, concrete sub-question the transfer
   view poses: **with exactly two open piles and lazy (deferred) merging, how
   close to `log(LDS)` can an online 2-stack policy get?**

## 10. The safe/forced boundary — when must a transfer happen?

Before any lazy-merge policy, pin down when a transfer is *forced* vs *safely
deferrable*. Setup (transfer view + settle-time lemma): cards depart the deck in
order `σ`; each, as it departs, is placed on a buffer; the buffers are stacks; a
transfer (bounce) moves a top across. Keep a buffer "sorted" = decreasing
bottom-to-top (min on top), so it pours out increasing.

**Boundary.** Placing the departing card `v` keeps a buffer sorted iff `v` goes
*below a current top* (`v < top(A)` or `v < top(B)` — it becomes that pile's new,
smaller top). So:

- **Safely deferrable (free):** `v < max(top(A), top(B))` — `v` fits below a top,
  extends that sorted pile, costs nothing. *While every arrival fits, both piles
  stay sorted and you pay zero transfers.*
- **Forced:** `v > top(A)` *and* `v > top(B)` — `v` exceeds both tops, the apex of
  an increasing triple `(top, top, v)`. It cannot extend either sorted pile; it
  *must* bury one (a future transfer); the only freedom is which pile to break and
  where the displaced card goes.

**Consequence.** Each pile in arrival order is a *decreasing* subsequence of `σ`,
so "both piles stay sorted for the whole sort" (zero transfers) is possible **iff
`σ` is a union of two decreasing subsequences, i.e. `LIS(σ) ≤ 2`** — exactly the
`B = 0` class. The first arrival exceeding both tops is the first forced transfer:
the event "the live arrival-stream's `LIS` reaches 3."

**Dynamic form of the static condition.** Static `OCT` measures the `LIS ≤ 2`
violation of the *whole* `σ`; the real cost is the *running* count of
exceeds-both-tops events over the live stream — which is why static `OCT`
under-counts (§5): global violation, not the integral of local ones.

**The useful (and slightly deflating) precision:** the *deferral* freedom is
narrow. A departing card is the deck top — you must place it *now*; you cannot
hold a run "open" on the deck. So the policy best-fit-places until an
exceeds-both event, and **all real decisions are concentrated at the forced
events**: (i) which pile to break, (ii) where the displaced card is routed so it
re-merges cheaply. That — not "when to merge" — is the optimization, and it *is*
the cascade. "Lazy merge" therefore reduces to "forced-event resolution."

**Precise open sub-questions.** (a) Is `min transfers` = the number of forced
events under best placement, or strictly more (does resolving one force later
ones)? (b) The optimal pile-to-break and displaced-card destination at a forced
event. (c) Magnitude: the running-`LIS`-excess of `σ` vs `opt` — `Θ(n log n)` for
random `σ`?

### 10a. What to do at a forced event (answers (a))

Two extreme local actions, both wrong:

- **Peel-to-fit** (transfer small tops away until a pile top exceeds `v`, then
  free-place `v`): this is **insertion sort → `O(n²)`**. To free-place `v` you must
  expose an *already-placed* card `> v` by relocating the smaller cards above it;
  on increasing `σ` (reversed) no card `> v` exists yet, so it can't free-place at
  all, and elsewhere it relocates `O(n)` small cards per event. Confirmed bad.
- **Bury** (place `v` on a pile, deferring one transfer): `O(1)` locally and
  *exactly optimal on reversed* (it is the comb — each card transfers once).

So the right local action is **one deferred transfer (bury), not a peel.** But that
does **not** make `min transfers = #forced events`. The number of forced events is
exactly `OCT = n − a₂` (the cards outside the best 2-decreasing cover) `= Θ(n)`,
yet `opt = Θ(n log n)`. The gap is the **cascade**: the deferred transfer, when it
finally executes (`v` moves to expose the card it buried), can re-bury on the other
pile, forcing more. So:

> Forced events are the right *trigger* (they count the `Θ(n)` first-order
> transfers, `= OCT`) but the wrong *granularity for cost* — the `Θ(n log n)` bulk
> is the resolution of the deferred transfers, and it is **not local**: the comb
> resolves globally (pile everything on `B`, reverse the whole pile onto `A`, each
> card once), not bury-by-bury. **(a) answered: `min transfers` is strictly more
> than `#forced events` whenever the cascade is nonzero — which is already true at
> small `n` (measured `opt − h_joint ≈ 1` even at `n ≤ 14`), growing from a
> minority of the bounces to the dominant term asymptotically.**

> **Observability caveat.** Exact `opt` is computable only to `n ≈ 14`; `OCT = n−a₂`
> is poly at any `n`. So the cascade `= opt − OCT` is *measurable* only where it is
> still small (`~1` at `n ≤ 14`) and is *unmeasured* where it dominates (`n ≳ 20`).
> Its large-`n` magnitude is **inferred from proven asymptotics** (`opt = Θ(n log n)`
> by counting-LB + merge-UB; `OCT = Θ(n)` by `a₂ = Θ(√n)`, Ulam–Hammersley), not
> computed. Finite-`n` cascade values and any "crossover `n`" are extrapolation.

So the object to understand is the **resolution schedule** of the deferred
transfers (where each buried card goes when it must move), which is the cascade —
and it is global, not a per-event greedy.
