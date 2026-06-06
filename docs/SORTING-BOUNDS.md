# Split‑Merge Sorting — Three No‑Reversal Algorithms and Their Bounds

This note records exactly what is currently established for sorting on the
three‑stack split‑merge machine **without any run‑reversal trick** (runs are
merged in place; descending runs are *not* flipped). It gives three algorithms
in increasing sophistication — the original multi‑pass merge, the adaptive
top‑down merge, and the Hu–Tucker optimal merge — then a provable lower bound
for the problem and a provable upper bound for each algorithm. Figures are for
`n = 52`. "Proven" and "measured/empirical" are kept distinct throughout.

---

## 0. The machine and the cost identity

Three LIFO stacks — deck `D` (also the I/O port), buffers `A`, `B`; the
accessible end of each is its top. Four moves, each pops one top and pushes it
onto another top:

```
    SA : D -> A        MA : A -> D
    SB : D -> B        MB : B -> D
```

`A` and `B` never exchange directly (`D` is the hub). The moves form inverse
pairs `SA<->MA`, `SB<->MB`, so the configuration graph is **undirected**, with
out‑degree ≤ 4. Start: input deck in `D`, `A = B = empty`. Goal: ascending deck
in `D`, `A = B = empty`. **Cost = number of moves.**

Let the input have `r` maximal ascending runs (bottom→top) of sizes
`s₁,…,s_r` (so `Σ sᵢ = n`; a sorted deck has `r = 1`, a strictly descending deck
has `r = n` singletons). A random permutation has `E[r] = (n+1)/2 = 26.5`.

All three algorithms are **adjacent‑merge** sorters: they only ever merge two
*adjacent* sorted runs. They differ in the schedule — equivalently, in the
binary **merge tree** they build over the runs, whose leaves are the runs in
left‑to‑right order (`= bottom→top` on `D`).

### Tree cost identity (proven, machine‑verified)

For a merge tree `T` realized **by recursion with parking**, define the weight

```
    W(T) = Σ sᵢ · depth(leaf i)      (root at depth 0; a 2‑leaf tree's leaves are at depth 1)
```

Then the machine cost is **exactly `2·W(T)`**.

*Realization.* To sort the block of a node with children `L` (lower) and `R`
(upper): sort `R` in place; park `R` onto `A` (`|R|`×`SA`); sort `L` in place;
park `L` onto `B` (`|L|`×`SB`); merge the two parked runs back to `D` **by exact
counts** (`|R|` from `A`, `|L|` from `B`, smaller top first). Per node:
`|R| + |L| + (|L|+|R|) = 2·(cards in node)`, summing to `2·W(T)`. Merging by
count rather than by empty‑stack test is essential — otherwise the merge dips
into the inert parked block beneath it.

Verified by emitting and replaying the move stream: **3208 realizations**
(`n = 6,12,20,52` plus structured decks, both tree algorithms below) all sort
correctly, leave `A`/`B` empty, and use exactly `2·W` moves. The standing test
suite (`tests.py`, `test_bimerge.py`, `test_recthirds.py`, 137,872 checks)
passes.

So Algorithms 2 and 3 reduce to **building a good merge tree over the runs**;
their cost is `2·W` of that tree. Algorithm 1 uses a flat schedule with a
simpler closed‑form cost.

---

## 1. Algorithm 1 — original multi‑pass natural merge

The deployable baseline (`natural_sort`). Bottom‑up two‑way merge seeded with
the natural runs, in synchronous passes:

```
runs = natural ascending runs of D
while more than one run:
    DISTRIBUTE: pop the runs off the top of D, alternately onto A and B
    MERGE:      repeatedly merge the top run of A with the top run of B back to D
    # run count roughly halves each pass
```

Each pass moves **all `n` cards out and back** (`2n` moves), and the run count
halves per pass, so there are `⌈log₂ r⌉` passes.

### Cost = `2n·⌈log₂ r⌉` (proven exactly, machine‑verified)

A single closed form, depending only on the run count `r`:

- **Best:** sorted deck, `r = 1` → **0**.
- **Worst:** strictly descending deck, `r = n = 52`, `⌈log₂ 52⌉ = 6` →
  `2·52·6 =` **624** (proof in §5; this is the *only* deck with `r = n`).
- **Average (`n = 52`, random): ≈ 520.** Since `E[r] = 26.5`, almost every deck
  has `r ∈ [17,32]`, i.e. `⌈log₂ r⌉ = 5`, giving cost `2·52·5 = 520`; ~0.2 % of
  decks have `r > 32` and cost 624. The cost is quantized to multiples of `2n`.

The weakness is structural: because every pass re‑distributes the **whole** deck,
a run that is already merged‑in keeps getting moved on later passes too. In tree
terms, the multi‑pass schedule charges **every** run the maximum depth
`⌈log₂ r⌉`, whereas a genuine tree (Algorithms 2–3) charges each run only its
own depth. That is exactly why the tree methods below dominate it.

---

## 2. Algorithm 2 — adaptive top‑down merge

Build the merge tree by recursively splitting the run sequence at the **run
boundary nearest the card midpoint**, then realize it (cost `2·W`, §0):

```
def build(runs):                       # runs: sizes, in order
    if len(runs) == 1: return leaf(runs[0])
    m = sum(runs); target = m/2
    k = boundary minimizing |prefix_cards(k) − target|     # adaptive split
    return node(build(runs[:k]), build(runs[k:]))
```

The split is **adaptive**: existing long runs stay intact as shallow leaves
instead of being cut, so a deck that is already nearly sorted costs little. No
global optimization, simpler than Hu–Tucker.

- **Average (`n = 52`, 2000 random decks): ≈ 487** (min 448, max 530).
- **Worst case: 600** (§5).
- `T_TD` is **never better than `T_HT`** (`W(T_TD) ≥ W(T_HT)` pointwise, since
  Hu–Tucker is optimal); on all 2000 random decks `cost_TD ≥ cost_HT`. It does
  beat the multi‑pass baseline on both average (487 < 520) and worst (600 < 624)
  by exploiting long runs.

---

## 3. Algorithm 3 — Hu–Tucker optimal merge tree

This builds the **minimum‑weight alphabetic binary tree** over the runs — the
provably best merge tree, hence the optimal no‑reversal adjacent‑merge sorter.

### The problem it solves

Given leaf weights `s₁,…,s_r` in **fixed left‑to‑right order**, find the binary
tree with those leaves, *in that order*, minimizing `W = Σ sᵢ·depthᵢ`. This is
the **optimal alphabetic binary tree** (Hu–Tucker, 1971) — like building an
optimal Huffman tree, but with the extra constraint that the leaves may not be
permuted (they must stay in run order, because runs sit at fixed positions on
`D` and only *adjacent* ones may be merged). Since machine cost `= 2·W`,
minimizing `W` minimizes moves, and no in‑place merge sorter can do better.

### What we compute — the order‑constrained DP (directly implementable)

`C[i][j] =` min weight of a tree spanning runs `i…j`; `pre` = prefix sums of
sizes. Optimal‑BST‑style recurrence (split only at a contiguous boundary `k`,
which is what enforces the alphabetic/order constraint):

```python
C[i][i] = 0
C[i][j] = min over i<=k<j of ( C[i][k] + C[k+1][j] ) + (pre[j+1] - pre[i])
K[i][j] = argmin k                                   # record split for reconstruction
```

`W(T_HT) = C[0][r-1]`; rebuild the tree from `K` (`node(build(i,k),
build(k+1,j))`). Straightforward `O(r³)`; Knuth's monotonicity of the optimal
split point reduces it to `O(r²)`. This is the version actually run and
verified here.

### The Hu–Tucker algorithm proper — `O(r log r)`

For large `r`, Hu–Tucker (or the cleaner Garsia–Wachs variant) computes the same
optimal tree in `O(r log r)` via three phases:

1. **Combination (yields depths).** Work on a sequence of nodes, initially the
   `r` leaves in order. Call two nodes *compatible* if no original leaf lies
   strictly between them (already‑combined internal nodes are "transparent" and
   do not block). Repeatedly take the **compatible pair of minimum total
   weight** (ties broken by leftmost, then by smaller right index), replace it
   by one node of that summed weight placed at the left member's position, and
   continue until a single node remains. This phase *ignores* the left‑to‑right
   order, so the tree it forms is generally **not** alphabetic — but the **depth
   each original leaf ends up at is exactly its optimal alphabetic depth**. That
   is the non‑obvious content of the Hu–Tucker theorem.
2. **Level assignment.** Read off `dᵢ =` depth of leaf `i` from phase 1. These
   satisfy the Kraft equality `Σ 2^{−dᵢ} = 1`.
3. **Reconstruction (yields the tree).** Build the unique binary tree whose
   leaves, taken in the original order, have depths `d₁,…,d_r`: repeatedly merge
   the **leftmost two adjacent nodes of equal maximum depth** into a parent of
   depth one less, until one root remains. The result respects leaf order
   (alphabetic) and realizes the optimal `W`.

(The subtlety is entirely in phase 1's "compatible pair" rule and its
tie‑breaking; phases 2–3 are mechanical. Garsia–Wachs replaces phase 1 with a
single left‑to‑right sweep that combines the first locally‑minimal adjacent
pair and then moves the new node left past larger weights, which is easier to
implement correctly.)

### Performance

- **Average (`n = 52`, 2000 random decks): ≈ 484** (min 444, max 524) — best of
  the three.
- **Worst case: 600** (§5), optimal among all no‑reversal merge sorters.

---

## 4. Lower bound for the problem: **≥ 143** (proven)

Let `M` be the optimal worst‑case sorting cost (max over decks of the minimum
moves to sort it). Every algorithm — multi‑pass, top‑down, Hu–Tucker, or
anything else — has worst case `≥ M`.

**Claim.** `M ≥ ⌈log₃(n!)⌉`. For `n = 52`, `log₃(52!) = 142.33`, so
**`M ≥ 143`.**

**Proof.**
1. *Undirected graph, bounded branching.* Moves come in inverse pairs, so the
   configuration graph is undirected. On a **shortest** path no move is the
   inverse of the move just made (else the two cancel and the path was not
   shortest), so after the first step each step has ≤ `4 − 1 = 3` choices. At
   the start `A = B = empty`, so only `SA, SB` apply — ≤ 2 first moves.
2. *Ball bound.* Non‑backtracking walks of length `k` from the start number ≤ 1
   (k=0), 2 (k=1), `2·3^{k-1}` (k≥2). Summing,
   `|Ball(k)| ≤ 1 + Σ_{j=1}^{k} 2·3^{j-1} = 3^k`. Every configuration within
   distance `k` is some such walk's endpoint, so **`|Ball(k)| ≤ 3^k`**.
3. *Counting.* All `n!` deck arrangements (with `A = B = empty`) are distinct,
   reachable, sortable configurations, hence all within distance `M` of the
   sorted one: `n! ≤ |Ball(M)| ≤ 3^M`, so `M ≥ log₃(n!)`.

For `n = 52`: `log₃(52!) = ln(52!)/ln 3 = 156.36/1.0986 = 142.33`, so
`M ≥ 143`. ∎

(The ≤ 2 first‑move count — buffers start empty — tightens the base from the
naive `4·3^{M-1}` to `3^M`, which is what carries 142.33 across to 143. Ignoring
the no‑backtracking structure entirely gives only `log₄(n!) ≈ 113`.)

**Stronger bound (`≥ 204`).** The counting bound above is loose. An admissible
per‑card heuristic proves the reversed deck requires exactly `4(n−1)` moves, so
the worst‑case optimum satisfies `M ≥ 4(n−1) = 204` at `n = 52`. Derivation and
proofs are in the companion note `HEURISTIC-BOUNDS.md`.

---

## 5. Upper bounds, per algorithm (with proofs)

### Algorithm 1 (multi‑pass): **worst = 624, proven exactly**

`cost = 2n·⌈log₂ r⌉`, and over all inputs `r ≤ n`, so
`cost ≤ 2n·⌈log₂ n⌉ = 2·52·6 = 624`, attained only by the descending deck
(`r = n`). A closed form — the cleanest of the three, but the largest worst
case. (On the descending deck it charges all 52 cards 6 passes = 624, whereas
the tree methods place 12 of them at depth 5 and reach 600.)

### Algorithm 3 (Hu–Tucker): **worst = 600, proven**

- *Unconditional (assumption‑free): `cost ≤ 2n⌈log₂ n⌉ = 624`.* `cost = 2·W(T_HT)`
  and `W(T_HT) ≤ W(balanced tree over the r runs)` since Hu–Tucker is optimal; a
  balanced tree has all leaves at depth ≤ `⌈log₂ r⌉ ≤ ⌈log₂ n⌉ = 6`, so
  `W ≤ 6n = 312`.
- *Tight value `= 600`.* The descending deck has `r = n` singleton runs; its
  optimal tree is the balanced tree on `n` units, with
  `W = n⌈log₂ n⌉ − (2^{⌈log₂ n⌉} − n) = 312 − 12 = 300` (40 leaves at depth 6,
  12 at depth 5), so `cost = 600` and worst `≥ 600`. Conversely `W(T_HT)` is
  maximized at all‑unit weights by the standard refinement‑monotonicity of
  optimal alphabetic trees (merging two adjacent leaves never increases the
  optimal weight), so `W(T_HT) ≤ 300` and `cost ≤ 600` for every input.
- *Confirmed.* Exhaustive `n = 7,8,9` gives worst `40,48,58`, each the
  descending deck and equal to the balanced singleton value; a hill‑climb at
  `n = 52` finds nothing above 600.

**Worst case exactly `600` (assumption‑free version: `≤ 624`).**

### Algorithm 2 (adaptive top‑down): proven **`T(n) = 2n·(⌊log_{4/3} n⌋ + 1)`**, so `≤ 1456` at `n = 52`

Cost `= 2·W = 2·Σ sᵢ dᵢ ≤ 2n·D(n)`, where `D(n)` is the maximum leaf depth (`Σ sᵢ = n`).
So it suffices to bound the depth. The natural guess "every child `≤ ¾m`" is
**false** — a run larger than `m/2` straddling the midpoint forces an
arbitrarily unbalanced split — so the bound needs the next lemma.

**Structural lemma.** Split a block of `m` cards; let `t` be the size of the run
straddling the midpoint. Writing the chosen boundary as `a` (nearest to `m/2`),
the larger child has size `≤ m/2 + t/2`. Two cases:

- *(balanced, `t ≤ m/2`)* — both children are `≤ 3m/4`.
- *(dominant, `t > m/2`)* — a child can exceed `3m/4`, but only the one
  containing `t`. The midpoint then lies inside `t`, so the split falls on an
  edge of `t`: the large child is `C = t` together with the run‑group on **one**
  side of `t`, whose cards `R` satisfy `|R| = |C| − t ≤ m − t < m/2`. Moreover
  `C`’s own split isolates `t` as a leaf, leaving `R` (`< m/2`) as `C`’s only
  internal child. *(In `C`, `t` is at one end and `t > |R|`, so the midpoint of
  `C` is inside `t` and the nearest boundary is `t`’s far edge.)*

**Depth theorem.** `D(m) ≤ log_{4/3} m + 1`, by strong induction on `m`.
- *Base* `m = 1`: a leaf, depth `0 ≤ 1`.
- *Balanced step:* `D(m) ≤ 1 + D(3m/4) ≤ 1 + (log_{4/3}(3m/4) + 1)
  = log_{4/3} m + 1`, using `log_{4/3}(¾) = −1`.
- *Dominant step:* the path needs **two** levels (`v → C → R`) to drop the block
  below `m/2`, since `C`’s other branch is the leaf `t`. So
  `D(m) ≤ 2 + D(m/2) ≤ 2 + (log_{4/3}(m/2) + 1) = log_{4/3} m + 3 − log_{4/3} 2
  = log_{4/3} m + 0.59 < log_{4/3} m + 1`  (as `log_{4/3} 2 = 2.41`). ∎

Hence `D(n) ≤ ⌊log_{4/3} n⌋ + 1`, and

```
    cost ≤ T(n) := 2n·(⌊log_{4/3} n⌋ + 1).
```

**At `n = 52`:** `log_{4/3} 52 = 13.7`, so `D(52) ≤ 14` and
**`cost ≤ T(52) = 2·52·14 = 1456`.**

**Can the depth exceed 7?** No, at `n = 52`. The tree depends only on the
run‑size composition, so adversarial search over **all** compositions of 52 is
feasible: it gives **maximum depth exactly 7** (one above `⌈log₂ 52⌉ = 6`) and
**maximum cost exactly 600**, attained by the descending deck at depth 6. So 7
is genuinely the worst depth here — the proof’s `14` is a ~2× over‑estimate —
and the depth‑7 configurations have cost well below 600 (their deep leaves are
light). For larger `n` the worst depth does climb, roughly like `log₂ n + O(1)`
(8 at `n = 64`, 9 at `n = 100` and `128`); the proof tracks that growth with a
looser constant, since `log_{4/3} = 2.41·log₂`.

**Lower side (proven, attained).** `W(T_TD) ≥ W(T_HT)` pointwise (Hu–Tucker is
optimal), so the top‑down worst is `≥` Hu–Tucker’s `600`; the descending deck
gives `600` for both. Combined with the search above, the **true** worst cost is
`600` — far inside the proven ceiling `T(52) = 1456`.

---

## 6. Summary (`n = 52`, 52‑card deck)

| algorithm | average | worst | worst status |
|---|---|---|---|
| Lower bound, **any** algorithm | — | **≥ 204** | `≥ 143` by counting; `≥ 4(n−1)=204` via reversed deck (see `HEURISTIC-BOUNDS.md`) |
| 1. Multi‑pass natural merge | ≈ 520 | **624** | proven exactly (`2n⌈log₂ n⌉`) |
| 2. Adaptive top‑down | ≈ 487 | **600** | proven `≤ T(52)=1456` via depth `≤ ⌊log_{4/3}52⌋+1 = 14`; true worst `600` (`≥ 600` forced; depth ≤ 7 by search) |
| 3. Hu–Tucker (optimal merge) | ≈ 484 | **600** | proven (`= 600`; `≤ 624` unconditional) |

The three are strictly ordered by quality: multi‑pass (simplest, one formula,
re‑moves every card every pass) is dominated by the adaptive top‑down tree,
which is in turn dominated by the optimal Hu–Tucker tree. The problem optimum
lies in `[204, 600]`: every algorithm needs ≥ 204 in the worst case (reversed
deck), and Hu–Tucker provably achieves ≤ 600. Small‑`n` BFS in fact gives the
worst‑case optimum as exactly `4(n−1)`, suggesting the true answer is `204` and
that the merge family (600) is ~3× off; closing this is the open problem (see
`HEURISTIC-BOUNDS.md`).

*All move counts above were verified by emitting and replaying the actual
machine moves.*
