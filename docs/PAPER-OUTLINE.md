# PAPER OUTLINE — scholarly write-up of the split–merge work

**Status.** Agreed top-level + second-level outline. Not yet drafted. Next step:
flesh out sections into prose. This file is the record to resume from; the
technical content lives in `docs/NOTES.md` (permanent record) and the section
tags there map to the paper sections below.

## Conventions agreed (don't re-litigate)

- **Narrative is chronological and honest** — the order the work actually
  happened, including the things that didn't pan out. No drama; phrase
  everything as plainly and simply as possible.
- **Jargon kept minimal**, especially our invented terms. Introduce at most
  **two** coined names in the whole paper — one for "a card sent back through
  the hub" (the quantity we minimize) and one for "the gap the per-deck bound
  misses." Describe everything else in plain words at first use. Keep the more
  colorful internal names (comb, hot-potato, tangle, OCT, cascade, parking,
  departure order σ) **out of the prose** — describe the ideas instead.
- **The whole-cycle model is NOT its own section.** It only appears as a brief
  note in §3: our first algorithm used full drain-and-refill cycles, and for a
  while we believed good sorting had to be organized into full passes. That
  belief is dissolved in §7–§8. (The cycle model's standalone results — Cayley
  distance over Catalan-many generators, NP membership — are out of scope here;
  candidate for a separate paper.)
- **Related work goes near the end** (§9), after the reader knows the machinery.
- **Two lower bounds, kept sharply distinct throughout** (this is the crux, and
  it drifted once already): the **counting bound** has the right *size*
  (`n log n`) but is *not instance-sensitive* (an "almost all decks" statement;
  false for the easy decks). The **per-deck bound** (Section 6) *is*
  instance-sensitive but only grows like `n`. The open problem (§10.2) is one
  bound with *both* properties. Keep this distinction explicit at 2.3, 5.3, 7.1,
  10.2.
- **Expected length** ~30 pages filled out. Accepted as unavoidable; don't pad.
- **One seam from the reorder:** §5 forward-references §6 (the search heuristic
  in 5.3 is valid only because of the bound proved in 6). Honest to the real
  order of discovery; handle with a one-line forward reference.

## Outline

**1. Introduction**
- 1.1 The machine and the task: sort a deck using a hub and two side buffers
- 1.2 What we wanted: a good algorithm and a matching lower bound
- 1.3 What we found, including what is still open

**2. Modeling the machine**
- 2.1 States and the four moves; the moves come in reversible pairs
- 2.2 Cost is the number of moves; the sorted goal
- 2.3 A counting lower bound: almost every deck needs about `n log n` moves
- 2.4 What the cost really counts: every non-final card costs two moves, plus
  two more each time it is sent back

**3. A first algorithm: sorting in cycles**
- 3.1 One cycle: empty the deck into the buffers, then bring it back in sorted pieces
- 3.2 A sorter built from repeated cycles; it works, in `O(n log n)` moves
- 3.3 This meets the counting bound, so the worst case is `Θ(n log n)`
- 3.4 The assumption we carried for a while: that good sorting had to be
  organized into full passes

**4. A better algorithm**
- 4.1 Merging the deck's existing runs instead of draining blindly
- 4.2 Choosing the cheapest order-preserving merge schedule
- 4.3 The constant it achieves, and its worst cases

**5. The exact cost of reversing, and exact solutions for small decks**
- 5.1 The fully reversed deck: an explicit best method, costing exactly `4(n−1)`
- 5.2 Computing true optima by search, up to about `n = 14`
- 5.3 The lower bound that makes the search practical (developed in Section 6)

**6. A lower bound on the number of moves**
- 6.1 When can a deck be sorted with no card ever sent back?
- 6.2 The bound: fewest send-backs = cards minus the best cover by two decreasing
  pieces (polynomial to compute)
- 6.3 Where it is tight, and the fact that it never overestimates

**7. The gap between the per-deck bound and the optimum**
- 7.1 Two bounds, opposite flaws: the counting bound has the right size but is
  not deck-specific; the per-deck bound is deck-specific but only size `n` — so
  the gap we care about is the per-deck bound's shortfall, `n` vs `n log n`
- 7.2 Why a card sometimes has to sit on the hub rather than pass straight through
- 7.3 Two distinct ways a single card ends up handled three or more times
- 7.4 The shortfall is a cost of move scheduling, not of the deck's value
  pattern — which is also why the "full passes" assumption was wrong

**8. Approaches that did not work**
- 8.1 Building a stronger bound from the deck's value pattern, and why each one
  collapses back to the weak bound
- 8.2 A recursive version of the better algorithm, and why routing through one
  hub makes it cost more than plain merging
- 8.3 Repeated full passes, and why they reduce disorder only slowly
- 8.4 Why computation cannot settle it: the search wall, and why working back
  from the goal just becomes meet-in-the-middle

**9. Related work**
- 9.1 Sorting with stacks and queues, and the two-stack model with a direct
  transfer (and why it does not carry over)
- 9.2 Increasing and decreasing subsequences, patience sorting, and the RSK
  correspondence
- 9.3 Rearrangement distances and breakpoint bounds, as an analogy for an
  instance-sensitive bound
- 9.4 Optimal merge trees, and admissible-heuristic search

**10. Conclusions and future work**
- 10.1 What is settled: the algorithms, the polynomial bound, the exact
  small-case values
- 10.2 The central open problem, stated plainly: a way to charge moves that
  reaches `n log n` and is sensitive to the instance (the counting bound's size
  *and* the per-deck bound's specificity at once)
- 10.3 Whether any algorithm can beat the merge constant
- 10.4 The most concrete next step, and the cost-of-one-hub clue from the failed
  recursion

## Section → NOTES map (for fleshing out)

- §2 → NOTES §0, §I.1, §I.2 (counting bound)
- §3 → §I.3 (the cycle/first sorter), Part II origin note (drain-fill belief)
- §4 → §I.3 (optimal-alphabetic merge)
- §5 → §I.5 (reversal value, exact search), §I.4 (the heuristic)
- §6 → §I.4, §I.4a (the per-deck send-back bound and its structure)
- §7 → §I.4a (the gap, parking, the two triple-handling mechanisms)
- §8 → §I.6, Part III (refutations), and the realized-recursion result (`recreal`)
- §9 → the "does not transfer" notes + acknowledged influences
- §10 → §I.6 open problems, and the abstraction/pattern-database next step
