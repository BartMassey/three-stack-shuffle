# The Split–Merge Permuter — Theory

**Scope.** This is the formal write-up for the *LIFO* split–merge machine: the only
operations are "pop the top card of one pile, push it onto the top of another pile."
There is no flip, no peek, no per-pile orientation choice. The document supersedes
the earlier working notes; where those notes hedged about conventions, the resolution
is recorded here.

Claim status is marked **[PROVEN]** (proof given), **[VERIFIED]** (exhaustive
computation, stated range of *n*), **[CONJECTURE]**, or **[OPEN]**.

---

## 1. The machine

There are three piles of distinct cards, each a **stack** (LIFO) with an accessible
top: the **Deck** *D*, **Stack A**, **Stack B**. A configuration is the three stack
contents. There are exactly four operations, each moving one card from the top of a
source pile to the top of a destination pile:

- `SA` : pop top of *D*, push onto *A*.
- `SB` : pop top of *D*, push onto *B*.
- `MA` : pop top of *A*, push onto the output.
- `MB` : pop top of *B*, push onto the output.

A **cycle** runs in two phases. The **split** phase issues a sequence of `SA`/`SB`
that empties *D* (exactly *n* ops). The **merge** phase issues a sequence of `MA`/`MB`
that empties *A* and *B* into the output (exactly *n* ops). The output then *becomes*
the Deck for the next cycle. (The "empty the deck each split" restriction is part of
the model; relaxing it is noted as future work in §6.)

**Reload convention (fixed).** The Deck is read top-to-bottom; the split processes the
current top first. During merge, the first card emitted becomes the **new top** of the
output. With this convention each stack emits its contents in the *reverse* of the
order in which those cards left the Deck.

Throughout, a deck of *n* distinct cards is written as a sequence top-to-bottom and
identified with a permutation in **one-line notation** (position ↦ card) after
relabeling the cards `1..n` by their rank.

---

## 2. What one cycle computes  **[PROVEN]**

Write the Deck top-to-bottom as `d = (d₁,…,dₙ)`. The split assigns each position,
processed `i = 1..n`, to *A* or *B*. Let `α` be the subsequence sent to *A* (in
increasing index order) and `β` the subsequence sent to *B*.

**Lemma 1 (one-cycle image).** After one cycle the new Deck is an interleaving of
`reverse(α)` and `reverse(β)`, and *every* such interleaving is achievable by a
suitable choice of `MA`/`MB`.

*Proof.* The split pushes `α₁,…,α_p` onto *A* in that order, so *A*'s top is `α_p`;
the merge therefore pops `α_p,…,α₁`, i.e. `reverse(α)`. Likewise for *B*. The `MA`/`MB`
choices realize exactly the order-preserving merges (interleavings) of the two
pop-streams. ∎

This already pins the one-cycle reachable set. The next statement is the operational
heart of the theory: it removes the explicit reference to splits and interleavings.

### 2.1 The relative-permutation reformulation **[PROVEN, VERIFIED n ≤ 6]**

For two decks `d, e` of the same cards, define the **relative permutation**
`rel(d,e)` by relabeling each card with its position in `d` (so `d` becomes the
identity) and reading off `e` in those labels. Concretely, with decks viewed as
permutations, `rel(d,e) = d⁻¹e`.

**Theorem 2.** `e` is reachable from `d` in exactly one cycle **iff**
`LIS(rel(d,e)) ≤ 2`, i.e. iff the relative permutation is 123-avoiding.

*Proof.* By Lemma 1, `e` is reachable iff the cards can be 2-colored (the split
choice) so that within each color class the order in `e` is the *reverse* of the order
in `d` (a stack reverses its class) while across classes the interleave is free. Two
same-colored cards must therefore form an *inverted* pair (opposite relative order in
`d` and `e`); same-colored ⇒ inverted, and any inverted-only class is exactly a class
on which `e` reverses `d`. Build the **agreement graph** on the cards, with an edge
between every pair that keeps the *same* relative order in `d` and `e`. A valid cycle
is exactly a proper 2-coloring of this graph (every agreeing pair must be
bichromatic). After relabeling by `d`, the agreement graph is the increasing-pairs
(non-inversion) graph of `rel(d,e)`; this is the comparability graph of a permutation,
which is perfect, so its chromatic number equals its clique number, namely
`LIS(rel(d,e))`. It is 2-colorable iff `LIS(rel(d,e)) ≤ 2`. ∎

*Verification.* For every deck `d` (sampled) at `n = 4,5,6`, the brute-force one-cycle
neighbor set equals `{ e : LIS(rel(d,e)) ≤ 2 }` exactly, and `|neighbors(identity)|`
equals the Catalan number `Cₙ` (`14, 42, 132` for `n = 4,5,6`).

**Corollary 2a (symmetry of the relation).** `d → e` in one cycle iff `e → d` in one
cycle. Indeed `LIS(d⁻¹e) = LIS((d⁻¹e)⁻¹) = LIS(e⁻¹d)`. Thus *the one-cycle
reachability relation is symmetric*, even though an individual operation sequence is
not its own inverse. **[PROVEN; VERIFIED n ≤ 7, 0 violations.]** This settles the
"not obviously symmetric" worry from the old notes: the *graph* is undirected.

**Corollary 2b (one-cycle sortability).** A deck `d` can be put in sorted order in one
cycle iff `LIS(d) ≤ 2`, equivalently iff `d` is a union of two decreasing
subsequences (Dilworth). Symmetrically, the decks reachable *from* the identity in one
cycle are exactly the 123-avoiding permutations, counted by `Cₙ`. **[PROVEN.]**

---

## 3. The SPLIT-MERGE PERMUTATION problem

**Instance.** A source deck `S` and target deck `T`, both permutations of the same `n`
cards, and an integer `k ≥ 0`.

**Decision question.** Can the machine transform `S` into `T` in at most `k` cycles?

**Optimization question.** Compute the minimum such `k` (and a witnessing operation
sequence).

### 3.1 Normalization and the algebraic form **[PROVEN]**

Relabel each card by its rank in `S`, so `S` becomes the identity and `T` becomes a
permutation `π`. Define

> `f(π) = minimum number of cycles to reach π from the identity`,  `f(identity) = 0`.

The decision question is exactly: **is `f(rel(S,T)) ≤ k`?**

By Theorem 2 and Corollary 2a, two decks `g, g′` are one cycle apart iff
`g⁻¹g′ ∈ C`, where

> `C = { σ ∈ Sₙ : LIS(σ) ≤ 2 }`  (the 123-avoiding permutations; `|C| = Cₙ`).

`C` is symmetric (`LIS(σ) = LIS(σ⁻¹)`) and contains the identity. Hence the reachability
graph is the **undirected Cayley graph** `Cay(Sₙ, C)`, it is vertex-transitive, and

> **`f(π)` is the word length of `π` over the generating set `C`** — the minimum `k`
> with `π = c₁c₂⋯c_k`, each `cᵢ` a 123-avoiding permutation.

So SPLIT-MERGE PERMUTATION is the **shortest-factorization / Cayley-distance** problem
for the symmetric group with the Catalan-many 123-avoiding generators. The diameter
`D(n) = max_π f(π)` equals the eccentricity of the identity (by vertex-transitivity).

### 3.2 Membership in NP **[PROVEN]**

**Lemma 3 (two-cycle single-card move).** For any deck, any one card can be removed and
reinserted at any chosen position in exactly **two** cycles, leaving the relative order
of all other cards unchanged.

*Proof / construction.* Let the chosen card be `c` and `others` the rest in deck order.
Cycle 1: send `c` alone to `B`, all of `others` to `A` (deck order); merge by emitting
`c` first, giving `(c, reverse(others))` — both halves are single reversed classes, so
`LIS(rel) ≤ 2`. Cycle 2: send `c` alone to `B`, the `reverse(others)` block to `A`; `A`
pops `reverse(reverse(others)) = others` in original relative order, and the free
interleave inserts `c` at any desired position. ∎  **[VERIFIED exhaustively n ≤ 6.]**

Since any permutation is reachable from the identity by at most `n−1` single-card
insertions, Lemma 3 gives the unconditional bound

> **`f(π) ≤ 2(n − 1) = O(n)`  [PROVEN].**

Therefore a YES-certificate is a list of at most `min(k, 2(n−1))` intermediate decks,
each consecutive pair checkable by an `O(n log n)` `LIS` test, so

> **SPLIT-MERGE PERMUTATION ∈ NP.**

The remaining question is its exact complexity inside NP (P vs NP-hard); see §5.

---

## 4. What is known about `f` (verified data)

**Termination / generation.** `C` generates `Sₙ`; every permutation is reachable.
Confirmed by exhaustive BFS reaching all `n!` permutations for every `n ≤ 9`.

**Diameter** `D(n) = max_π f(π)`, exhaustive BFS over the full Cayley graph:

| n    | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|------|---|---|---|---|---|---|---|---|
| D(n) | 1 | 1 | 2 | 2 | 2 | 3 | 3 | 3 |

`D(8)` and `D(9)` are new here (n ≤ 7 reproduces the old notes). BFS layer sizes for
`n = 9`: `1, 4862, 261807, 96210` (sums to `9! = 362880`).

**Diameter formula — status updated.** The two live hypotheses were:

- triangular / `√(2n)`-type: `D(n) = (least c with c(c+1)/2 ≥ n) − 1`, predicting
  `D(9) = 3`;
- `⌈log₂ n⌉`, predicting `D(9) = 4`.

The computed **`D(9) = 3` refutes the `⌈log₂ n⌉` formula** and is consistent with the
triangular formula. **[VERIFIED n ≤ 9.]** Note, however, that `n ≤ 9` cannot separate
the *asymptotic* growth `Θ(√n)` from `Θ(log n)` — the two formulas first diverge by
more than one only around `n ≈ 16`. The triangular formula predicts `D(10) = 3` and a
first jump to `D = 4` at `n = 11`; both are **[OPEN]** (n = 11 BFS is ~10¹² ops).

**`f` is not a function of `LIS`** — made precise. Multiset of `LIS` values per
distance:

| n | dist 1 | dist 2 | dist 3 |
|---|---|---|---|
| 7 | LIS 1–2 | LIS 3:2107, 4:1821, 5:421, 6:36 | **LIS 3:225** |
| 8 | LIS 1–2 | LIS 3:9800, 4:16332, 5:6105, 6:841, 7:49 | **LIS 3:4537, 4:1225** |

`LIS ∈ {3, 4}` occurs at *both* distance 2 and distance 3. So `LIS` determines `f`
only for `LIS ≤ 2` (⇒ `f ≤ 1`) and large `LIS` (close to identity); on the values
`{3,4}` there is a genuine second-order correction. **[VERIFIED n ≤ 8.]**

**`LIS` is *not* submultiplicative under composition.** `LIS(ab) ≤ LIS(a)·LIS(b)` fails
(16/4000 random pairs at `n = 6`). Consequently the tempting lower bound
`f(π) ≥ log₂ LIS(π)` is **invalid** and must not be used. **[VERIFIED — negative.]**

---

## 5. The two roads (current frontier)

The decision problem is in NP (§3.2). Its placement is **[OPEN]**: no polynomial
algorithm and no hardness reduction is known.

### Road A — polynomial algorithm

What is settled and what blocks the rest:

- **`k = 0`:** `π = identity`. `O(n)`. **[PROVEN.]**
- **`k = 1`:** `LIS(π) ≤ 2`. `O(n log n)`. **[PROVEN]** (Corollary 2b).
- **`k ≥ 2`:** no poly characterization known. The natural reduction is: `f(π) ≤ 2`
  iff there exists an intermediate deck `g` with `LIS(g) ≤ 2` and `LIS(g⁻¹π) ≤ 2`.
  The obstacle is that `g` ranges over the `Cₙ ≈ 4ⁿ` 123-avoiding permutations, and §4
  shows `LIS(π)` alone cannot decide `k = 2` vs `k = 3`. A poly algorithm therefore
  needs a structural invariant strictly finer than `LIS` (a candidate: the
  agreement-graph / interval structure of the inversions of `π`). **[OPEN.]**

A positive resolution of `k = 2` in poly time would be the natural first milestone and
might generalize, given the very small diameter.

### Road B — NP-hardness

No reduction exists yet. The structure that a reduction must exploit: a single
generator is "two reversed runs freely interleaved," and `f` counts how many such
moves compose to `π`. Plausible source problems are other permutation
shortest-factorization / sorting-distance problems known to be hard (e.g.
prefix-reversal / pancake distance is NP-hard), but this generator set
is unusually rich (Catalan-many), which tends to make distances *small* and
hardness *harder to inject*. **[OPEN.]**

**Honest assessment.** The `O(n)` diameter bound, the trivial `k ≤ 1` test, and the
empirically tiny diameter all lean (weakly) toward tractability, but the failure of
`LIS` to decide even `k = 2` shows there is real combinatorial structure to capture.
Neither road is close to settled.

---

## 6. Open problems (priority order)

1. **Complexity of `f`.** Is SPLIT-MERGE PERMUTATION (LIFO) in P or NP-hard? Settle
   `k = 2` first (poly test, or hardness gadget).
2. **Characterize `f` exactly.** Find the `LIS`-plus-correction invariant; the
   correction lives precisely on `LIS ∈ {3,4}` per §4.
3. **Asymptotic diameter.** Compute `D(10)` (predicted 3) and `D(11)` (predicted 4) to
   separate `Θ(√n)` from `Θ(log n)`; `n ≤ 9` cannot.
4. **Heuristic with a proven cycle bound.** A constructive algorithm using
   `O(√n)` (or whatever the truth is) cycles, with per-cycle runtime.
5. **Relax "exhaust the deck each split"** and re-derive Lemma 1 / Theorem 2.

---

## 7. Reproducible tooling

**One-cycle neighbors (LIFO), and the verified equivalent test.** A deck `e` is a
one-cycle neighbor of `d` iff `LIS(d⁻¹e) ≤ 2`. Brute force (for cross-checking) and the
`LIS` test agree for all sampled `d` at `n = 4,5,6`, with `|neighbors(identity)| = Cₙ`.

```python
import itertools, bisect

def lis(p):
    t = []
    for x in p:
        i = bisect.bisect_left(t, x)
        if i == len(t): t.append(x)
        else: t[i] = x
    return len(t)

def rel(d, e):                       # relative permutation d^{-1} e
    pos = {c: i for i, c in enumerate(d)}
    return tuple(pos[x] for x in e)

def one_cycle_ok(d, e):              # admissible single cycle d -> e ?
    return lis(rel(d, e)) <= 2
```

**Diameter / distance** is BFS in `Cay(Sₙ, C)`, neighbors of `g` being `g∘c` for each
`c ∈ C = {σ : LIS(σ) ≤ 2}`. Python BFS is fine to `n = 7`; `n = 8,9` need a compiled
BFS (permutations packed as nibble codes in a `u64`, open-addressing visited set,
right-multiplication by the precomputed `Cₙ` generators). That program produced the
`n = 8,9` rows of §4 (`n = 9`: ≈1.8×10⁹ compositions, ~30 s).

---

## 8. One-line summary

You are studying the **LIFO** two-stack split–merge permuter. **One cycle ⟺ the
relative permutation is 123-avoiding (`LIS ≤ 2`)**; the reachability graph is the
**undirected Cayley graph `Cay(Sₙ, {LIS ≤ 2})`**, and `f(π)` is the word length there.
**Proven:** sortable-in-one ⟺ `LIS ≤ 2`; `f(π) ≤ 2(n−1)`, so the problem is in **NP**.
**Verified:** diameters `1,1,2,2,2,3,3,3` for `n = 2..9` — `D(9)=3` kills the
`⌈log₂ n⌉` formula; `LIS` decides `f` only outside `LIS ∈ {3,4}`. **Open:** P
vs NP-hardness (start at `k = 2`), the exact invariant, and asymptotic diameter.
