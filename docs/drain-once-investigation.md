# Reverse-engineering the optimal "drain-once" sorter (working log)

Goal: a *deterministic* sort algorithm for the operation-count machine that
beats the merge sorter (~1.9× opt), ideally approaching opt. Method: extract
optimal move sequences for random `n ≤ 14` decks (`analyze` binary, IDA* +
`ida_star_path`) and reverse-engineer the structure.

## Observation 1 — shape of optimal solutions (from `analyze traces`)

Optimal solutions look like **one big drain then one big merge**, with a few
small bounce-interjections during the drain:

```
deck [4,7,2,3,1,9,8,5,6]  opt=22 bounces=2 drains=1 peak=1.00 phases=6
  aaab A bbaa B abb AAABAABBB
  base advances only in the final merge run; buffers fill to peak ≈ n first
```

- `peak_buf ≈ 1.0`: the deck (nearly) fully drains into the two buffers before
  the big merge — the buffers hold almost everything at the peak.
- The committed base advances **only at the end**, during one long merge run.
- Very few bounces (2–5 at n=9), i.e. very few `S`-interjections inside the merge
  or `M`-interjections inside the split.

So the optimum ≈ *distribute the whole deck into the two buffers so they merge
back sorted, paying a bounce only for each card that cannot fit the 2-pile
structure.*

## The framework (operation model — NOT the cycle model)

> Caution: the **cycle model's** "1 cycle sorts a deck iff LIS ≤ 2" uses a
> different reload convention and does **not** transfer — the reversed deck is one
> decreasing run yet needs `4(n−1)`, not `2n`. Reason in the operation model only.

Let `pi` = the departure order = deck read top-to-bottom. A buffer pile that is
**decreasing in departure order** pours back ascending (sorted). So a clean
"split into two piles, then merge" sorts exactly the cards coverable by **two
decreasing subsequences of `pi`**; the rest must bounce.

- Min bounces (clean start deck) `= n − a₂(pi)`, where `a₂` = max cards in a union
  of two decreasing subsequences of `pi`. This is exactly `OCT_pre` on the
  soft-conflict graph, so `h_joint = 2n + 2·OCT = 4n − 2·a₂`.
- A deterministic sorter that bounces each excess card **exactly once** would cost
  `4n − 2·a₂ = h_joint`, i.e. within ~1–2 of opt. (Checks out on the reversal:
  `a₂ = 2`, cost `4n − 4 = 4(n−1)` — the comb.)

So the prize is: **realize the `OCT_pre` bound constructively** — split the deck
into the two best decreasing piles and bounce each of the `n − a₂` excess cards
just once. The crux (and the known-hard part) is making the excess bounce *once*
rather than cascading — my current `rollout` bounces them many times.

## Observation 2 — the OCT bound is a lower bound but NOT achievable

Aggregate feature means (`analyze stats`, opt | merge | rollout):

| n  | cost (o/m/r)        | bounces (o/m/r)   | phases (o/m/r) | deck-drains (o/m/r) |
|----|---------------------|-------------------|----------------|---------------------|
| 14 | 40.3 / 79.4 / 67.8  | 6.1 / 25.7 / 19.9 | 12 / 13 / 42   | 1.4 / 2.6 / 0.9     |

- The optimum has **~4× fewer bounces than the merge sorter** and ~3× fewer than
  my settle `rollout`.
- All three (nearly) fully drain the deck (`peak ≈ 1.0`), so "drain once" is not
  what makes the optimum special — **bounce count** is.
- The `rollout` already **beats merge for small n** (67.8 < 79.4) but with ~3× the
  *phases* (42 vs 12): lots of little cascading bounces. It loses at n=52 (~900 vs
  ~480) because the cascades compound super-linearly.
- **Key correction.** `opt(52)` ≈ 240 ⇒ ~68 bounces, but `OCT_pre ≤ n = 52`, so the
  optimum needs **more bounces than the OCT/2-colouring bound allows**. The
  2-decreasing-partition bound is a genuine *lower* bound that is **not
  achievable** — the LIFO ordering forces extra "cascade" bounces no static
  pairwise/OCT count captures (this is the paper's §15 open frontier). So
  `cost = 4n − 2·a₂` is a floor, not the answer, and at small n it is nearly tight
  only because there are few cascades.

## Deterministic candidates so far (`detsort`)

- `merge` (Hu–Tucker): ~1.76→1.97× opt for n=10→14; ~483 at n=52. Scales (flat
  ~1.9×) but never better.
- `rollout` / `drain_merge` (patience-2 split + bounce; interleaved vs two-phase
  give the same result): ~1.42→1.69× opt at n≤14 — **beats merge** — but ~900 at
  n=52. The cascade barrier.

## Observation 3 — the two-phase structure is near-optimal; the split is the crux

Brute force over all `2^n` split colourings (drain by the colouring, then merge
with bounce):

| n  | opt  | **best two-phase split** | greedy patience | min-bury split |
|----|------|--------------------------|-----------------|----------------|
| 8  | 22.2 | 24.0 (**1.08× opt**)     | 31.8 (1.43×)    | 30.2 (1.36×)   |
| 14 | 42.8 | 51.2 (**1.20× opt**)     | 79.2 (1.69×)    | 76.2 (1.78×)   |

So **the two-phase "drain into two piles, then merge" structure can reach ~1.1×
opt** — there is huge room above the merge sorter (1.9×). The entire difficulty is
**choosing the split colouring.**

- **Min-bury is the WRONG objective.** The min-bury / max-2-decreasing partition
  (= the `OCT`/`a₂` partition) gives only 1.4–1.8× — *worse* than greedy patience.
  Minimising buries does **not** minimise cost, because the phase-2 **cascade**
  (not the bury count) dominates. The good split is a cascade-aware optimisation.
- **Hill-climbing the split finds it.** A trivial bit-flip hill-climb on the
  colouring (cost evaluated by the fast deterministic two-phase sim) reaches
  **1.10× opt (n=10), 1.17× (n=12)** — i.e. essentially the brute-force best — in
  O(n²) evals. So the lever is real and cheap to exploit.

## Observation 4 — at n=52 the *phase-2 cascade* is the bottleneck, not the split

split hill-climb vs opt (n≤14) and merge (n=52):

| n  | split_ls / opt | note                         |
|----|----------------|------------------------------|
| 10 | 1.096          | ≈ brute-force best split     |
| 12 | 1.172          |                              |
| 14 | 1.210          |                              |
| 52 | — (vs merge: split_ls 500 vs merge 484, **1.034×**) | only *matches* merge |

So even with a near-optimal split, the two-phase sorter only **ties** merge at
n=52. The reason: phase-2 ("merge with bounce", evacuating each blocker to the
other buffer) **cascades**, and the cascade grows with n — eating the small-n
advantage. At small n cascades are tiny, so the good split shows ~1.1×; at n=52
they dominate.

## Where this leaves it

- The merge frontier (1.9×) is **not** a structural floor: a two-phase sorter with
  a good split is ~1.1× opt **for small n**. At n=52 it ties merge (~500 vs 484)
  because phase-2 cascades.
- **Two distinct open levers, both needed to beat merge at n=52:**
  1. a *cascade-aware split rule* (min-bury/OCT is not it), and
  2. a *cascade-free phase-2* — a way to merge two imperfect piles back without
     the blocker-evacuation blowing up. This is the same "price the downstream
     relocations" obstruction as the heuristic program (NOTES §I.4/§15).
- The open piece is a **deterministic split rule**. Min-bury/OCT is not it (cascade
  matters). The hill-climbed splits are training data: characterise *which* cards
  the good split puts where, relative to the cascade they would cause.
- Tooling: `ida_star_path` (optimal move sequences), `analyze` (traces + feature
  stats), `detsort` (split harness: brute force, min-bury DP, hill-climb).

## Leads for next session

1. Reverse-engineer the split rule from hill-climbed colourings (a cascade-aware
   scoring per card, not bury count).
2. A cheap cascade-aware split (greedy by predicted cascade, or a DP that charges
   cascade not buries).
3. Better phase-2 (the current "evacuate blockers to the other buffer" is itself
   cascade-prone; a smarter merge could lift the whole two-phase family).
