# CURRENT — active working context

**What this is.** Volatile saved context: the thread we are *actively* thinking
about, terse but enough to restart cold without re-reading everything. **Read this
first when resuming.**

**Workflow (important).** `docs/NOTES.md` is the permanent record and is updated
from this file **only when something is removed here** — i.e. when a question is
settled, migrate its conclusion into NOTES/structure.md and delete it from
CURRENT. The detailed formal record already lives in `docs/structure.md`; this
file summarizes the frontier and points there, it does not replace it.

---

## The thread

Understand the operation-count optimum well enough to get **either** an
instance-sensitive lower bound **or** a sub-merge constructive algorithm. Current
focus: the **cascade** — the part of the bounce cost the static (tangle/OCT) bound
misses, which is where essentially all the difficulty is.

## Load-bearing facts (established; detail at `structure.md §`)

- Cost `g = 2m + 2B`; minimize **bounces** `B`. (§1)
- **Settle-time lemma:** at each settle the deck holds only the base; *all*
  unsettled cards are in the two buffers. (§8)
- **Reduction:** `B` = number of **inter-buffer transfers** ⇒ "sort two stacks by
  transfers." (§8)
- `σ` = departure order = deck top-to-bottom. **Single-arrival (bounce-free) ⟺ two
  `σ`-decreasing subsequences**; static bound `B ≥ OCT = n − a₂(σ)`, polynomial
  (Greene–Kleitman). (§2–3)
- **tangle** = increasing pair in `σ` (deck); **buried** = inversion in a buffer.
  Same inversion, two locations. (§3–4)
- **Where the complexity is:** `OCT = Θ(n)` (`a₂ = Θ(√n)`, Ulam–Hammersley) but
  `opt = Θ(n log n)` (counting LB for almost-all + merge UB). The static bound is an
  asymptotically vanishing fraction; the whole bulk is `cascade = opt − OCT`. (§5)
- **Safe/forced boundary:** placing departing `v` keeps a pile sorted iff
  `v < top(A)` or `v < top(B)`; **forced** iff `v >` both tops. All-defer (`B=0`) ⟺
  `LIS(σ) ≤ 2`. (§10)
- **At a forced event:** peel-to-fit = insertion sort = `O(n²)` (bad); **bury =
  O(1) deferred**, optimal on reversed (the comb). `#forced = OCT = Θ(n)` but
  `min-transfers > #forced` (the cascade) — deferred transfers re-bury, and the
  resolution schedule is **global** (comb reverses a whole pile), not per-event
  greedy. (§10a)
- **Merge critique:** ascending runs over-count; right granularity is the
  increasing-subsequence cover `= LDS ≤ r`. `log r → log(LDS)` moves the constant
  `1.75 → ~0.8` on random — but two buffers hold only 2 open piles (`LDS ≈ 2√n`),
  and reversed shows `LDS` over-counts the *dual* way. (§9)

## Hard constraint — observability

Exact `opt` computable only to **n ≈ 14** (IDA*); `OCT = n − a₂` is poly at any
`n`. So the cascade is measurable only where it's `~1` bounce (n ≤ 14) and
**unmeasured where it dominates** (n ≥ 20). All large-`n` cascade/opt numbers are
**extrapolation**. ⇒ no `n` is both solvable and cascade-rich, so the cascade
**cannot** be reverse-engineered from optimal traces — this is a theory problem.

## Open (active) sub-questions

1. **Potential `Φ`** with `|ΔΦ| = O(1)`/move and `Φ(random) = Θ(n log n)` ⇒ the
   missing instance-sensitive lower bound. Single value-cuts are *vacuous* (a
   binary projection needs 0 transfers); the bound must be **multi-scale** (≥3-way
   distinctions force transfers).
2. **Resolution schedule** of deferred transfers (the cascade): global/comb-like
   (cheap) or irreducibly tangled?
3. Running-`LIS`-excess of `σ` vs `opt` — does it total `Θ(n log n)`?
4. Two-open-pile lazy policy: how close to `log(LDS)` rounds can it get?

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

## Pointers

- `docs/structure.md` — the formal theory (the `§` numbers above).
- `docs/NOTES.md` — full project record; `docs/drain-once-investigation.md` — the
  (superseded) reverse-engineering log.
- Tooling: `src/search.rs::ida_star_path`; `src/bin/analyze.rs` (`traces`, `stats`,
  `bounces`); `src/bin/detsort.rs` (split harness; `obvious` = the interleave
  class where merge is a `log` factor off).
