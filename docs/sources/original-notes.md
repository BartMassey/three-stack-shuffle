# Split-Merge Permuter — Working Notes

**Status:** handoff document for a fresh start. Everything below is marked
either **[PROVEN]**, **[VERIFIED]** (exhaustive computation, small n),
**[CONJECTURE]**, or **[OPEN]**. Earlier discussion produced several claims for
a *different, stronger* machine; see "Corrections / pitfalls" before trusting
any remembered numbers (e.g. "4 cycles for 52 cards" is **wrong** for this
machine).

---

## 1. The machine (formal definition)

Three ordered piles of distinct cards: the **Deck**, **Stack A**, **Stack B**.
Each pile is a LIFO stack with an accessible **top**. Four operations:

- `SA` — pop the top of the Deck, push it onto the top of Stack A.
- `SB` — pop the top of the Deck, push it onto the top of Stack B.
- `MA` — pop the top of Stack A, push it onto the top of the output (new Deck).
- `MB` — pop the top of Stack B, push it onto the top of the output (new Deck).

A **cycle** is: a sequence of `SA`/`SB` that empties the Deck into A and B
(exactly `n` split ops; the Deck must be fully exhausted — this "exhaust the
deck" restriction is assumed *for now* and may be worth relaxing later),
followed by a sequence of `MA`/`MB` that empties both stacks into the output
(exactly `n` merge ops). The output becomes the Deck for the next cycle.

**Convention (load-bearing — read §3 before changing it).** The Deck is read
**top-to-bottom**; the split processes the top card first. The output's
**first-emitted card becomes the new top**. With this convention each stack
emits its contents in the *reverse* of the order the cards left the Deck.

---

## 2. What one cycle computes  **[PROVEN]**

Write the Deck top-to-bottom as `d = (d_1, ..., d_n)`. A split assigns each
position to A or B (processed `i = 1..n`). Let `alpha` = the subsequence of `d`
sent to A (in increasing index order), `beta` = the subsequence sent to B.

**Lemma 1.** After the cycle, the new Deck is some interleaving of
`reverse(alpha)` and `reverse(beta)`, and *every* such interleaving is
achievable by appropriate `MA`/`MB` choices.

*Proof.* The split pushes `alpha_1, ..., alpha_p` onto A in that order, so A's
top is `alpha_p`; the merge pops `alpha_p, ..., alpha_1`, i.e. `reverse(alpha)`.
Same for B. The `MA`/`MB` choices realize exactly the order-preserving merges of
the two pop-streams. ∎

**Corollary (one-cycle reachability).** The decks reachable from `d` in one
cycle are exactly the interleavings of two reversed subsequences of `d`.

**Corollary (sortable in one cycle).** A deck `d` can be put in sorted order in
one cycle **iff `LIS(d) <= 2`** (longest increasing subsequence), equivalently
iff `d` is the union of two decreasing subsequences (Dilworth). Symmetrically,
the permutations reachable *from* the identity in one cycle are exactly those
with `LIS <= 2` (the 123-avoiding permutations; count = Catalan `C_n`).
**[VERIFIED]** for n <= 7.

---

## 3. CRITICAL: the convention determines everything

The four operations do not by themselves fix the theory; the *reload
convention* (how the emitted sequence becomes the next Deck) does. There are two
natural choices and they give **different machines**:

- **LIFO reload (what §1–§2 assume):** new Deck = emission order. One cycle =
  interleave of two *reversed* subsequences. This is the machine we have mostly
  analyzed. Its complexity appears to be the hard/interesting one (see §5).

- **FIFO reload:** if instead the emitted pile is flipped before the next split
  (new Deck = reverse of emission order), one cycle becomes
  `interleave(alpha, beta)` with **no** reversal — i.e. a classic **riffle
  shuffle**. That machine has clean, *polynomial* theory: minimum cycles to
  reach a target = `ceil(log2(number of rising sequences))` (Bayer–Diaconis),
  diameter `ceil(log2 n)`, and it is **not** NP-hard. **[PROVEN in literature]**

**Action item for the restart:** decide which reload the physical machine
actually does. If FIFO, the problem is essentially solved by known riffle-shuffle
theory and the "prove NP-hardness" goal is unattainable (it's in P). If LIFO,
the theory below applies and the interesting questions are open.

There is **no third "flip per pile" option** on this machine: a true stack
forces one reversal per pile and gives no per-pile orientation choice. Any model
with such a choice (call it the "opt"/flip model) is a *strictly stronger,
different* machine — see §6.

---

## 4. The SPLIT-MERGE PERMUTATION problem (formal)

**Instance:** a source deck `S` and target deck `T`, both permutations of the
same `n` cards.

**Question (decision):** is there a sequence of `k` cycles transforming `S` into
`T`? **(optimization):** find the minimum such `k`, and a witnessing op
sequence.

**Reduction to identity.** Relabel each card by its rank in `S` (so `S` becomes
the identity `1..n`). Then `T` becomes a permutation `pi`, and the problem is:
minimum cycles to reach `pi` from the identity. So WLOG study `f(pi) :=` min
cycles from identity. All bounds below are stated for `f`.

Note the machine is **not** obviously symmetric (a cycle is not its own inverse),
so "sort `pi` to identity" and "build `pi` from identity" need separate care,
though for the LIFO machine the one-cycle sortability condition (`LIS <= 2`) is
symmetric.

---

## 5. What we know about `f` (the LIFO machine)

**[VERIFIED] Termination.** Every permutation is reachable; one cycle strictly
decreases `LIS` toward the fixed point, so the process always terminates.

**[VERIFIED] `f` is governed primarily by `LIS`.** Exhaustive BFS from identity:

| dist `f` | LIS values of permutations at that distance |
|---|---|
| 0 | n (identity only) |
| 1 | 1, 2 |
| 2 | 3, 4, 5, 6 |
| 3 | 3 (n=7 only so far) |

The LIS=3-at-distance-3 entry (n=7) shows `f` is **not** a pure function of LIS;
there is a second-order correction not yet characterized.

**[VERIFIED] Diameter** `D(n) = max_pi f(pi)` by exhaustive BFS:

| n | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|
| D(n) | 1 | 1 | 2 | 2 | 2 | 3 |

**[CONJECTURE] Diameter formula:** `D(n) = (least c with c(c+1)/2 >= n) - 1`
(≈ `sqrt(2n)`), which matches n <= 7. **Untested for n >= 8.** Computing
`D(9)` or `D(10)` would distinguish this `sqrt(2n)` growth (predicts 3) from
logarithmic `ceil(log2 n)` (predicts 4) — a cheap, decisive experiment if
neighbor generation is optimized.

**[VERIFIED — negative] One cycle does NOT cleanly halve LIS.** The natural
guess "min reachable LIS in one cycle = `ceil(LIS/2)`" fails on a large fraction
of permutations (≈37% at n=6). So do **not** assume an `f = ceil(log2(·))`
form; the diameter data (sub-logarithmic at n=5,6) is consistent with that
warning.

---

## 6. Corrections / pitfalls (things that bit us)

- **The "monotone cover / `ceil(log2(cover))` / 4-cycles-for-52" results are for
  a DIFFERENT machine** — one allowing a *flip* so each run can be presented
  ascending or descending. That "opt" machine reaches 512 permutations in one
  cycle at n=6; **this** machine reaches only `C_6 = 132` (`LIS <= 2`). Do not
  carry those numbers over.
- **`min monotone cover` is NP-hard**, but that belongs to the flip model, not
  this one. NP-hardness of SPLIT-MERGE PERMUTATION (LIFO) is **[OPEN]** — no
  reduction yet.
- Distinguish three models cleanly: **fifo/riffle** (poly, known), **lifo**
  (this machine, mostly open), **opt/flip** (stronger, monotone-cover theory).
  They are genuinely different; small-n reachable-set sizes separate them.

---

## 7. Open problems (in priority order)

1. **Pin the reload convention** (LIFO vs FIFO) for the real machine. Decides
   whether there's anything open at all.
2. **Characterize `f` exactly** for the LIFO machine: the LIS-plus-correction
   quantity. Candidate directions: rising/falling-sequence counts under the
   reversal twist; relation to `f(pi)` vs `f(reverse(pi))`.
3. **Settle the diameter growth:** compute `D(8..10)`; confirm or kill the
   `sqrt(2n)` conjecture.
4. **Complexity of optimal `k`:** is SPLIT-MERGE PERMUTATION (LIFO) NP-hard, or
   poly like the riffle case? No evidence either way yet.
5. **A heuristic with a proven big-O cycle bound** (requirement still unmet). A
   constructive sorting algorithm that provably uses `O(sqrt n)` or `O(log n)`
   cycles, whichever is true, plus its per-cycle runtime.

---

## 8. Reproducible tooling

Exact one-cycle neighbor generator (LIFO machine), Deck as a tuple top-to-bottom:

```python
import itertools, bisect
from collections import deque

def neighbors(deck):
    """All decks reachable from `deck` in one cycle (LIFO reload)."""
    n = len(deck); res = set()
    for mask in range(1 << n):
        A = [deck[i] for i in range(n) if not (mask >> i) & 1]   # deck order
        B = [deck[i] for i in range(n) if (mask >> i) & 1]
        ra, rb = A[::-1], B[::-1]                                # stacks reverse
        la, lb = len(ra), len(rb)
        for combo in itertools.combinations(range(la + lb), la):  # interleavings
            out = [None] * (la + lb); s = set(combo); ia = ib = 0
            for p in range(la + lb):
                if p in s: out[p] = ra[ia]; ia += 1
                else:      out[p] = rb[ib]; ib += 1
            res.add(tuple(out))
    return res

def f_distances(n):
    """BFS distances from identity (exhaustive; feasible to ~n=7)."""
    start = tuple(range(1, n + 1)); dist = {start: 0}; q = deque([start])
    while q:
        c = q.popleft()
        for nb in neighbors(list(c)):
            if nb not in dist:
                dist[nb] = dist[c] + 1; q.append(nb)
    return dist

def lis(p):
    t = []
    for x in p:
        i = bisect.bisect_left(t, x)
        if i == len(t): t.append(x)
        else: t[i] = x
    return len(t)
```

Notes on scaling: full BFS is fine to n=7 (`C(2n,n)` candidate neighbors per
state). For n>=8 use layered forward reachability from identity and stop once a
target layer is filled, or bidirectional search; neighbor generation is the
bottleneck and dedupes heavily.

---

## 9. One-line summary for the next instance

You are studying the **LIFO** two-stack split-merge permuter. **Proven:** one
cycle = interleave of two reversed subsequences; sortable-in-one ⟺ `LIS <= 2`.
**Verified small-n:** diameters 1,1,2,2,2,3 for n=2..7. **Everything else**
(exact optimal-cycle quantity, diameter growth, NP-hardness, heuristic bound) is
**open** — and beware: the monotone-cover / log-bound results from prior chatter
were for a stronger flip-enabled machine and do **not** apply here.
