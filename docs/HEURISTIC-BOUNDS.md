# Admissible heuristics for the split‑merge machine, and the `4(n−1)` lower bound

Companion to `SORTING-BOUNDS.md`.

**Machine.** Deck `D` (hub + I/O) and two LIFO buffers `A`, `B`. Moves `SA, SB`
(`D→A`, `D→B`) and `MA, MB` (`A→D`, `B→D`); **no `A↔B`**; cost = number of moves;
goal `D = (1,…,n)`, `A = B = ∅`. Top of a stack = last element.

**Notation.** The *committed base* is the largest `k` with `D[0..k−1] = (1,…,k)`
sitting at the bottom of `D`. A card is *settled* once it occupies its final
`D`‑position and never moves again. `h(s)` denotes an estimate of the moves
remaining from state `s` (the cost‑to‑go, `h` in A*).

---

## 1. Per‑card decomposition (the lever)

For a solution `σ` and a card `c`, let `μ_σ(c)` = number of moves of `σ` that pick
up `c`. Every move picks up **exactly one** card, so `|σ| = Σ_c μ_σ(c)`.

> **Principle.** If for every solution `σ` a card satisfies `μ_σ(c) ≥ m(c)`, then
> `h(s) = Σ_c m(c) ≤ |σ|` for all `σ` — i.e. `h` is admissible.

This localizes the whole estimate to "how few times can each card move," and
(via §6) lets independent obstructions be added.

---

## 2. The checkpoint invariant (foundational)

> **Lemma.** In every solution there are times `τ₁ < τ₂ < … < τₙ` such that
> immediately before `τᵢ` the deck is exactly `D = (1,…,i−1)`, and the move at
> `τᵢ` settles card `i` (an `MA/MB` landing `i` on top). Consequently every
> currently‑in‑`D` card of value `> k` leaves `D` before `τ_{k+1}`, and the
> above‑base deck cards make their **first departures in their current
> top‑to‑bottom order**.

*Proof.* Card `i` ends at height `i` over `(1,…,i−1)`; its last arrival lands it
on top, so just before, the deck is exactly `(1,…,i−1)` — any extra card above
would seat `i` too high. After `i` settles nothing beneath it can move, so
`1,…,i−1` settled earlier: `τ₁<…<τₙ`. At `τ_{k+1}⁻` the deck `(1,…,k)` holds no
value `> k`, so every above‑base card has departed. A card leaves only as the
deck top, and an above‑base card cannot reach the top before the cards above it
have each left at least once; hence first departures follow top‑to‑bottom order.
∎

The lemma constrains **every** solution; nothing below assumes a particular
strategy.

---

## 3. Base charge `h₀` (proven; exact on rotations)

Per‑card lower bounds:

- `m(c) = 0` if `c` is in the base.
- `m(c) ≥ 1` if `c` is in a buffer (it must `MA/MB` home).
- `m(c) ≥ 2` if `c` is in `D` above the base.

*The `≥ 2`:* such a card starts and ends in `D`, so #departures = #arrivals; by
§2 it departs ≥ once, hence arrives ≥ once → ≥ 2 moves. (Parity: a card starting
in `D` moves an even number of times; one starting in a buffer, an odd number.)

```
h₀(s) = 2·(|D| − k) + |A| + |B|.
```

Admissible (BFS: 0 violations, `n ≤ 8`). **Exact on the rotation** `(2,…,n,1)`:
every card moves twice, total `2n`. This is the "one card far out of place"
case — the displaced card moves only twice; the cost is the intervening cards.

---

## 4. Buffer‑bury charge (`+2`, proven)

> **Lemma.** If a card `x` sits in a buffer with a smaller‑valued card `y` below
> it in the same buffer, then `μ_σ(x) ≥ 3` for every `σ`.

*Proof.* `y < x`, so `y` settles before `x`. To extract `y` (below `x`), `x` must
be popped first — an `MA/MB` to the deck. At that instant `y` is unsettled, so
the base is below `x`; `x` is premature and must leave the deck again (`SA/SB`)
and later return to settle (`MA/MB`): ≥ 3 moves. ∎

This is the within‑buffer LIFO inversion, and it is what makes a scrambled
buffer expensive. It is robust: leaving the buffer is the *only* way to free `y`,
and a buffer's sole exit is to the deck.

---

## 5. In‑`D` tangle charge (proven; a Dilworth / two‑buffer bound)

Let `π` be the value sequence of the above‑base deck cards read **top‑to‑bottom**
(their forced first‑departure order, §2); `m = |π|`; `LIS = ` length of the
longest strictly increasing subsequence of `π`.

> **Theorem.** The set `E` of above‑base deck cards that arrive at the deck **two
> or more times** satisfies `|E| ≥ LIS − 2`. Hence
> `Σ_{above‑base c} μ_σ(c) ≥ 2m + 2·max(0, LIS − 2)`.

*Proof.* Take an increasing subsequence `x₁,…,x_L` of `π`: they depart in this
order and, having increasing values, settle in this order. Suppose two of them,
`x < y` (`x` departs and settles first), each arrive at the deck only once — i.e.
each makes a single round trip `D → buffer → D` and rests in one buffer between
departure and settling. If they share a buffer, `x` departed first so `x` lies
below `y`; but `x` must be popped (to settle) before `y`, impossible while `y` is
above it. So two single‑trip chain members lie in **different** buffers. With
only two buffers, at most 2 chain members are single‑trip; the other `≥ L−2` lie
in `E`. Take `L = LIS`. Each `E`‑card makes ≥ 2 round trips (≥ 4 moves), each
other above‑base card ≥ 1 (≥ 2 moves):
`Σ ≥ 4|E| + 2(m−|E|) = 2m + 2|E| ≥ 2m + 2(LIS−2)`. ∎

*(The step "`y` departed before `τ_x`" holds because at `τ_x⁻` the deck is
`(1,…,x−1)`, which excludes the larger‑valued `y`.)*

**Reading.** A *decreasing* run of `π` is one "free" stack (push largest first;
it pops in increasing order). `LIS` is the number of such stacks the sequence
demands; you have two; the deficit `LIS − 2` forces extra round trips. This is
Dilworth's theorem meeting the two‑buffer / no‑`A↔B` constraint.

---

## 6. The composition theorem (the admissible sum)

The charges of §§3–5 bound the moves of **different groups** of cards
(above‑base deck cards; buffer cards). They add:

> **Theorem.** Partition the cards by current location into groups
> `G₁,…,G_r`. If for each `j` there is a value `L_j` with
> `Σ_{c∈G_j} μ_σ(c) ≥ L_j` for **every** solution `σ`, then
> `h(s) = Σ_j L_j` is admissible.

*Proof.* For any `σ`, `|σ| = Σ_c μ_σ(c) = Σ_j Σ_{c∈G_j} μ_σ(c) ≥ Σ_j L_j`.
Minimize over `σ`. ∎

There is **no double counting**, even though all groups compete for the same two
buffers: each `L_j` floors the moves *of its own cards*, and buffer contention
can only raise those move counts, never lower them. Each group bound was proven
valid against *every* solution, so they hold simultaneously and add. Combining
§§3–5:

```
h*(s) = [ 2·(|D|−k) + 2·max(0, LIS(π) − 2) ]                     (above‑base deck)
      + Σ over buffer cards ( 3 if a smaller card sits below it, else 1 ).
```

Admissible by the theorem (and by BFS: **0 violations** across all states at
`n = 6, 7, 8`).

---

## 7. Tightness, the comb construction, and the lower bound

On the reversed deck, `π = (1,2,…,n)`, so `LIS = n`, base `= 0`, and
`h*(reversal) = 2n + 2(n−2) = 4(n−1)`. Admissibility gives
`opt(reversal) ≥ 4(n−1)`. A matching **comb** construction:

1. `SB` cards `1,…,n−1` (deck top first) onto `B`;
2. `SA` card `n` onto `A`;
3. for `j = n−1` down to `1`: `MB j`, and if `j ≥ 2` also `SA j`;
4. `MA` cards `2,…,n` (pour `A`).

sorts the reversed deck in exactly `4(n−1)` moves for all `n` — per‑card profile
`2,4,…,4,2` (extremes 2, interior 4); verified for `n ≤ 29`. Hence:

> **Theorem.** `opt(reversed deck) = 4(n−1)` for all `n`. Therefore the
> worst‑case optimum `M(n) = max_deck opt(deck)` satisfies **`M(n) ≥ 4(n−1)`** —
> at `n = 52`, **`M ≥ 204`** (vs. the counting bound `143`).

Furthermore `max_s h*(s)` equals the goal eccentricity `4(n−1)` for `n = 6,7,8`,
so `h*` is **tight at the diameter** and the reversed deck is a diameter witness.
This makes `M(n) = 4(n−1)` the likely exact answer; the missing half is an
algorithm sorting *every* deck in `≤ 4(n−1)` (the eccentricity guarantees
per‑deck solutions exist at least for `n ≤ 8`).

---

## 8. BFS verification (`n ≤ 8`)

Backward BFS from the goal computes `opt(s)` for all states.

| `n` | states | `h*` admissible | `opt(rev) = h* = 4(n−1)` | eccentricity | worst residual `opt − h*` |
|---|---|---|---|---|---|
| 6 | 20,160 | yes (0 viol.) | 20 | 20 | 8 |
| 7 | 181,440 | yes (0 viol.) | 24 | 24 | 10 |
| 8 | 1,814,400 | yes (0 viol.) | 28 | 28 | 12 |

`h₀ → h*` average improvement is several moves per state; on the scrambled
buffer `((),(),(1,5,6,2,3,4))` (`n=6`, opt 20) it lifts `6 → 16`.

---

## 9. Reframing: moves, arrivals, bounces

> **Identity (every solution).** `|σ| = 2·(arrivals at D) − (n − |D₀|)`, where
> `arrivals = #MA + #MB` and `|D₀|` is the current deck size.

*Proof.* `arrivals − departures = ` net deck growth `= n − |D₀|`; and
`|σ| = arrivals + departures`. Eliminate `departures`. ∎

So lower‑bounding moves ⇔ lower‑bounding arrivals. Each non‑base card arrives ≥ 1
(its settle); an extra arrival is a **bounce** — a premature deck entry that must
be undone. With `B` = total forced bounces, `arrivals ≥ #non‑base + B`, hence

```
|σ| ≥ 2·#non‑base − (n − |D₀|) + 2B = h₀ + 2B
```

(using `2·#non‑base − (n−|D₀|) = 2(|D|−k) + |A| + |B| = h₀`). **Every charge is a
bounce count:** the in‑`D` term is `B ≥ (LIS(π)−2)⁺`, the bury term is
`B ≥ #buried`. Strengthening the heuristic = proving more forced bounces.

## 10. Worked examples (`n = 6`)

| state | deck term | buffer term | `h` | opt |
|---|---|---|---|---|
| rotation `(2,3,4,5,6,1)` | `2·6 + 2(2−2) = 12` (`π=(1,6,5,4,3,2)`, `LIS=2`) | 0 | **12** | 12 ✓ |
| reversal `(6,5,4,3,2,1)` | `2·6 + 2(6−2) = 20` (`π=(1,…,6)`, `LIS=6`) | 0 | **20** | 20 ✓ |
| compound `D=(6,2,5), A=(4,1), B=(3)` | `2·3 + 0` (`π=(5,2,6)`, `LIS=2`) | `1+1+1` | **9** | 17 |

The compound's per‑card optimum is `{1:1, 2:2, 3:1, 4:3, 5:6, 6:4}`: four bounces
(`4`,`5`×2,`6`) that the static charges do not see.

## 11. Strengthening — a pairwise‑conflict (clique) bounce bound

Call two single‑arrival non‑base cards **in conflict** if they must occupy
different buffers. Single‑arrival members of a mutually‑conflicting set then fit
in ≤ 2 buffers, so

> **`B ≥ (largest conflict clique) − 2`.**

The conflict edges, each from §2 + LIFO:

- two deck cards at `π`‑positions `i < j`: conflict iff `val_i < val_j` (an
  *increasing* pair of `π` — this is exactly §5's chain, now as a graph).
- a deck card `d` and a buffer card `q`: conflict iff `q < d`. *(If both were
  single‑arrival in `q`'s buffer, `d` evacuates before `τ_{k+1} ≤ τ_q`, lands on
  top of `q`, and being larger settles later — blocking `q`. Impossible; so
  different buffers.)*
- two cards in the **same** buffer: conflict iff the deeper one is smaller (the
  bury relation).

Combining with the disjoint `LIS`+bury sum (both are valid floors on `B`; take
the max):

```
h_best(s) = h₀ + 2·max( (LIS(π) − 2)⁺ + #buried ,  clique(s) − 2 ).
```

Admissible (BFS: **0 violations**, `n = 6, 7`); dominates `h*`; and **leaves the
diameter exact** (reversal `20`, rotation `12`), so the `M ≥ 4(n−1)` bound is
untouched. It lowers the worst residual `opt − h` from `8 → 6` (`n=6`) and
`10 → 8` (`n=7`). (The clique is found by search at BFS sizes; the graph is
nearly a comparability graph — small‑set conflicts orient smaller‑settles‑first —
so longest‑chain DP should compute it without exponential cost at scale.)

## 12. The joint bound — eliminating the `max`

The `max` in §11 is a confession: `(LIS−2)⁺ + #buried` and `clique − 2` are two
necessary conditions we didn't know how to merge. Bound the object both are
proxies for. Let `U` = the set of non‑base cards that are **single‑arrival** (never
bounce) in a given solution; then `B ≥ #nonbase − |U|`. Two necessary conditions
on `U`:

- **(N1, bury — directional).** A card with a smaller‑valued card below it in its
  buffer must bounce (§4). So `U` contains no buried buffer card.
- **(N2, soft conflict — 2‑coloring).** Give each card in `U` the buffer it
  occupies (buffer cards fixed; deck cards chosen). Two cards are *soft‑conflicting*
  if they could go to different buffers but cannot share one when both are
  single‑arrival; such a pair must then get different buffers. So the
  **soft‑conflict graph** on `U`, with buffer cards pre‑coloured by their buffer,
  is properly 2‑coloured.

Soft edges (each necessary, from §2 + LIFO): deck–deck increasing in `π`; deck `d`
vs non‑buried buffer card `q` with `q < d`. *(Same‑buffer pairs are handled by N1;
two non‑buried same‑buffer cards are compatible, no edge.)* Hence

```
|U| ≤ (#non‑buried buffer + #deck) − OCT_pre,
B ≥ #buried + OCT_pre,
```

where `OCT_pre` = minimum vertices to delete so the soft‑conflict graph becomes
bipartite respecting the pre‑colouring. Then

```
h_joint(s) = h₀(s) + 2·( #buried + OCT_pre(soft‑conflict graph) ).
```

**This is a single quantity — no `max`.** It dominates both former bounds: the
deck cards form a soft clique of size `LIS`, needing `LIS−2` deletions to become
bipartite, so `OCT_pre ≥ (LIS−2)⁺`; any conflict clique of size `c` needs `c−2`
deletions, so `OCT_pre ≥ clique−2`; and `#buried` adds on the disjoint buffer side.

**Results (full BFS, n = 6, 7).** Admissible — **0 violations**. **Dominates the
§11 `max` heuristic at every state.** Average residual gap `opt − h` drops about
35–45 % (n=7: `2.23 → 1.42`); worst residual `8 → 6`. Still **exact on reversal
and rotation**, so the `M ≥ 4(n−1) = 204` lower bound is untouched.

**Scaling note.** The soft‑conflict graph is a comparability graph (edges orient
smaller‑value → larger and the relation is transitive). Without the
pre‑colouring, max induced bipartite subgraph = max union of two antichains =
Greene–Kleitman, which is **polynomial**; the buffer pre‑colouring adds
constraints we currently resolve by a bounded deletion search (cheap at these
sizes). So a polynomial `OCT_pre` at scale is plausible, not just brute force.

## 13. Consistency (admissible but not consistent)

For A* / IDA* with a `g`‑pruning transposition table to be safe without
re‑expansion, the heuristic must be **consistent**: `h(s) ≤ 1 + h(s′)` for every
unit‑cost edge `s → s′` (equivalently `|h(s) − h(s′)| ≤ 1`, the graph being
undirected). The verdict:

| heuristic | max single‑move drop `h(s) − h(s′)` | consistent? |
|---|---|---|
| `h₀` | 1 | **yes** |
| `h*` (`+ LIS + bury`) | 3 | no |
| `h_best` (`+ clique`) | 3 | no |

**`h₀` is consistent.** A move changes only `(|D|−k, |A|, |B|)`. Each of the four
moves shifts `|D|±1` and one buffer `∓1`, and the base `k` changes by at most `±1`
(`SA/SB` can drop the base top; `MA/MB` can extend the base by one). Casework on
`h₀ = 2(|D|−k) + |A| + |B|` gives `Δh₀ ∈ {−1, +1}` in every case. ∎

**`h_best` is not consistent** (verified BFS, `n = 6,7`; admissibility still holds,
0 violations). Witness (`n=6`): `D=(1,2,3,6,5), A=(), B=(4)` has `h=7`; the move
`SA` (5 → A) reaches `D=(1,2,3,6), A=(5), B=(4)` with `h=4` — a drop of 3 across
one edge. Two effects coincide: a card leaves the deck‑above region (`h₀ −1`),
and the conflicting pair `4<5` is split into separate buffers, erasing one forced
bounce (bounce term `−1`, doubled to `−2`). The drop is *legitimate* — that move
is genuinely worth 3 to the bound — but it breaks the unit‑step triangle
inequality. The doubled bounce term is the sole source: any single move can
dissolve one forced bounce, a `−2` swing that stacks with `h₀`'s `−1`.

**Consequences for search.** IDA* optimality needs only admissibility, so
`h_best` is sound for finding optimal costs. A transposition table must then
either (a) be used for cycle/duplicate detection only (never prune by stored
`g` — safe for any heuristic), (b) apply **pathmax**
(`h(child) ← max(h(child), h(parent) − 1)`) to repair consistency along each
search path, or (c) reserve the consistent `h₀` for `g`‑pruning and use `h_best`
only as the bound.

## 14. IDA* search with `h_best`

Search setup: iterative deepening on `f = g + h_best`, with parent‑move pruning,
on‑path cycle detection, and pathmax (`h(child) ← max(h(child), h(parent)−1)`) to
repair the inconsistency (§13) along each path. Admissibility makes the optimal
costs sound.

**Validation.** IDA* returns the BFS‑optimal cost on every tested deck at
`n = 7, 8` (random samples + reversal).

**`h_best` is exact on the reversal path.** IDA* solves the reversed deck in
exactly `4(n−1)` node expansions — i.e. `nodes = cost`, zero backtracking — so
`h_best` equals the true distance at every state on the optimal reversal path.
This holds all the way to the full deck:

| n | reversal opt | nodes expanded |
|---|---|---|
| 12 | 44 | 44 |
| 20 | 76 | 76 |
| 26 | 100 | 100 |
| 40 | 156 | 156 |
| **52** | **204** | **204** |

So the search independently **constructs and certifies an optimal 204‑move
solution for the reversed 52‑card deck**, matching the comb construction and the
lower bound.

**Conjecture test `M(n) = 4(n−1)`.** Beyond the BFS range, random sampling
(300 decks) plus adversarial hill‑climbing from reversal at `n = 8, 9, 10` finds
**no deck exceeding `4(n−1)`**; the reversed deck remains the strict maximum.
Random decks are markedly easier — e.g. at `n = 10`, random cost averages `26.7`
and peaks at `32`, against `4(n−1) = 36` (reversal). Combined with the proven
`M(n) ≥ 4(n−1)` and the exact BFS eccentricity `= 4(n−1)` for `n ≤ 8`, this is
strong evidence that **`M(n) = 4(n−1)` and the reversed deck is the unique
diameter witness**. The remaining gap is a proof that *every* deck is sortable in
`≤ 4(n−1)` (a universal upper bound / eccentricity argument for general `n`).

**Practical strength.** On reversal the heuristic is perfect (no search). On
scrambled decks the cascading‑bounce gap (§11) shows up as search effort but
stays tractable: `n = 10` random decks expand a median of ≈ 600 nodes (max ≈ 74k).

### 14a. Search with the joint heuristic `h_joint` (§12)

Re‑running the same IDA* with `h_joint` (still admissible, verified `n ≤ 7`;
still inconsistent since it dominates `h_best`, so pathmax stays on). It returns
the same optimal costs (matches BFS `n = 7, 8`).

**The tighter bound pays for itself.** Head‑to‑head on 80 identical random decks
at `n = 10`, `h_joint` vs `h_best`:

| heuristic | median nodes | max nodes | total nodes | wall‑clock |
|---|---|---|---|---|
| `h_best` | 810 | 26 568 | 226 480 | 8.1 s |
| `h_joint` | 111 | 10 828 | 36 746 | 6.6 s |

An **84 % reduction in node expansions**, and *faster overall* despite the heavier
per‑node cost — the node savings more than cover the extra work. Optimal costs
identical.

**Scaling bottleneck (and fix).** The `OCT_pre` deletion search is exponential on
a large clique, and the reversed deck's soft‑conflict graph is one clique of size
`n`, so the brute force blows up past `n ≈ 20` (whereas `h_best`'s clique bound is
a polynomial longest‑chain DP — why *it* reached `n = 52`). Fix: a polynomial
fallback for graphs with more than ~15 vertices — the soft graph's own longest
chain gives `clique − 2 ≤ OCT_pre`, admissible. (First cut double‑counted buried
cards inside the full‑graph clique and went *in*admissible — reversal `n = 20`
returned 78 > 76; using the *soft* graph's chain, which already excludes buried
cards, repairs it.) With the fallback, `h_joint` is again exact on the reversal
path: `n = 52 → 204` nodes.

**Conjecture test extended.** With the cheaper search, `n = 11` random + adversarial
search still finds **nothing above `4(n−1) = 40`** (random peaks at 38; hill‑climb
from reversal stays at 40). The `M(n) = 4(n−1)` evidence now runs through `n = 11`.

## 15. Open frontier

The clique bound helps the cross deck/buffer states but still undershoots the
compound (`11` vs `17`): the remaining bounces are **cascading**, not pairwise —
lifting a buried card drops it onto an occupied buffer where it is buried again,
a sequential dependency no single clique captures. The worst residual states
(`((6,),(5,3,1),(4,2))`, etc.) have `D` and **both** buffers disordered at once.
A bound that captures cascades likely needs a *sequential / scheduling* argument
or an exact relaxation, not a static count; by §6 any such bound only needs to
be a valid floor on its group's moves to drop into the sum.

*Method note.* All optimal distances and admissibility checks come from exact
backward BFS over the full state graph; the comb construction and per‑card counts
are obtained by replaying the actual machine moves.
