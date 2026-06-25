# Codex Current Work Handoff

Use [`ALGORITHMS.md`](./ALGORITHMS.md) as the mathematical specification and
[`ADAPTIVE-SELECTION-SORT.md`](./ADAPTIVE-SELECTION-SORT.md) for the selection
family and benchmark history.

This file intentionally contains only current and future work. The original
implementation roadmap has been completed and has been removed.

## Current status

The repository already contains:

- the three-stack machine and primitive move accounting;
- constructive sorting algorithms through the current experimental variants;
- benchmark and random-input tooling;
- reverse BFS and A* search support;
- TRANSPORT HEURISTIC and its validation machinery;
- ordinary and receding-horizon lookahead selection variants;
- the complete specification of INCREMENTAL RHL; implementation and benchmarking are still pending.

Do not reimplement or broadly refactor these components unless specifically
asked.

## Current assignment: INCREMENTAL RHL

The INCREMENTAL RHL specification is complete; the implementation experiment
is not.

Implement it as a computationally equivalent version of ordinary
receding-horizon rollout using:

- algebraic successor construction;
- persistent memoization of deterministic rollout suffixes;
- rerooting after each committed placement;
- forced-target and endpoint-symmetry normalization;
- deduplication or batched evaluation of the rollout DAG;
- the bottommost-mask-bit quotient.

It must select the same masks and emit the same primitive move sequence as the
existing brute-force RHL under the same tie rule.

### Required implementation sequence

1. Verify exact score and mask equivalence against brute-force RHL on exhaustive
   small leaves.
2. Verify complete move-sequence equivalence on random `K=2`, `K=3`, and `K=4`
   runs.
3. Attempt `K=1` with 26-card leaves.
4. Report:

   ```text
   masks visited
   distinct algebraic successors
   distinct normalized successors
   base-cache hits and misses
   base states stored
   forced targets removed
   peak memory
   planning time per target and per bucket
   primitive move count
   ```

5. Preserve the existing RHL implementation as a reference implementation.

Do not add pruning, pattern databases, or a changed policy in this first
experiment. Once this baseline has been measured, the next round may add safe
mask pruning, partial-mask dynamic programming, or state-space lower bounds.

## Deferred idea: optimized partition trees

Keep this noted but do not implement it yet:

- choose value-bucket boundaries and an alphabetic partition tree jointly;
- trade partition depth against leaf extraction cost;
- consider both distribution-optimized and per-instance variants.

Portfolio algorithms are explicitly out of scope for now.

## Remaining research directions

These are open topics, not automatic implementation assignments:

1. Strengthen TRANSPORT HEURISTIC while preserving admissibility.
2. Investigate consistent relaxations, reopenings, and pathmax behavior.
3. Add disjoint additive pattern databases for exact abstractions.
4. After the INCREMENTAL RHL benchmark, develop safe pruning or dynamic
   programming for large RHL mask spaces if needed.
5. Determine exact or tighter expected and worst-case results for the
   lookahead-selection family.
6. Determine the exact worst and expected costs of SPLIT-MERGE SORT.
7. Analyze HU–TUCKER NATURAL MERGE SORT on random input.
8. Complete the pure SIGNED NATURAL SORT phase rule.
9. Tighten REVERSING SPLIT-MERGE SORT bounds and macros.
10. Improve orientation-aware MSB RADIX SORT.
11. Search optimal small-`n` programs and use them as block macros or pattern
    databases.
12. Improve the general lower bound beyond

    ```text
    max(ceil(log_3(n!)), 4n-4).
    ```

Do not start one of these without a specific instruction selecting it.

## Permanent implementation guardrails

When modifying the repository:

- Count only primitive legal moves:

  ```text
  A → D, D → A, D → B, B → D.
  ```

- A direct `A ↔ B` transfer costs two primitive moves through `D`.
- Every reported algorithm cost must equal the length of a replayable primitive
  move sequence.
- Preserve deterministic tie rules exactly when comparing algorithms.
- Distinguish exact values, certified bounds, mathematical expectations, and
  measured sample means.
- Do not turn a heuristic, simulation result, or upper bound into an exact
  claim.
- Keep experimental algorithms clearly labeled.
- Validate returned plans by replaying them to the exact goal.
- TRANSPORT HEURISTIC is admissible but inconsistent; A* must support
  reopenings.
- The frozen suffix is a heuristic concept, not an established pruning rule.

## Handoff procedure for a new task

For each new Codex assignment:

1. Read the relevant section of `ALGORITHMS.md`.
2. State the exact algorithmic or experimental change being made.
3. Preserve the existing algorithm as a separate selectable variant when the
   change alters semantics.
4. Add targeted correctness tests and cost regressions.
5. Benchmark with primitive move counts and clearly identified sample sizes.
6. Update the documentation with results, including failures or negative
   findings.
