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

*(Tooling note: `a₂(σ) = (g(reversed-equivalent) …)`; the static bound is
`h_joint`. The cascade is `B_opt − OCT(T)`, which `ida_star` gives exactly for
small `n` and which the multi-bounce theorem lower-bounds as `Θ(n log n)`.)*
