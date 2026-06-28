# Codex Current Work Handoff

Use [`ALGORITHMS.md`](./ALGORITHMS.md) as the mathematical specification and
[`ADAPTIVE-SELECTION-SORT.md`](./ADAPTIVE-SELECTION-SORT.md) for the selection
family and benchmark history. [`INCREMENTAL-RHL.md`](./INCREMENTAL-RHL.md) and
[`DEPTH-LIMITED-RHL.md`](./DEPTH-LIMITED-RHL.md) summarize the two RHL follow-up
experiments.

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
- INCREMENTAL RHL, including exact-equivalence tests, planning counters, and
  an initial `K=2` benchmark;
- DEPTH-LIMITED RHL, including baseline planning counters and initial `K=1`
  and `K=2` benchmarks.

Do not reimplement or broadly refactor these components unless specifically
asked.

## Completed experiment: INCREMENTAL RHL

INCREMENTAL RHL is implemented as a separate experimental algorithm and keeps
ordinary brute-force RHL as its reference implementation.

It is a computationally equivalent version of ordinary
receding-horizon rollout using:

- algebraic successor construction;
- persistent memoization of deterministic rollout suffixes;
- rerooting after each committed placement;
- forced-target and endpoint-symmetry normalization;
- deduplication or batched evaluation of the rollout DAG;
- the bottommost-mask-bit quotient.

It must select the same masks and emit the same primitive move sequence as the
existing brute-force RHL under the same tie rule.

### Completed validation

1. Exact score and mask equivalence is tested exhaustively through six-card
   leaves.
2. Complete move-sequence equivalence is tested on random `K=2`, `K=3`, and
   `K=4` runs.
3. The initial benchmark was intentionally limited to `K=2`.
4. The benchmark reports:

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

5. The existing RHL implementation remains the reference implementation.

The 2,000-sample, 52-card, seed-`24301` `K=2` benchmark measured `8.133`
seconds for brute-force RHL and `4.795` seconds for incremental RHL, with the
same `343.931` mean primitive moves. See `ALGORITHMS.md` for counters.

The measured speedup is useful but not enough to make exhaustive `K=1` mask
enumeration practical by itself. Keep its persistent greedy-completion cache
available for later algorithms, but do not spend more time optimizing this
variant unless requested.

## Completed experiment: DEPTH-LIMITED RHL

DEPTH-LIMITED RHL is implemented as a separate experimental algorithm:

```text
depth-limited-rhl-2k-partition-lookahead-selection-experimental:<K>:<depth>
```

The search unit is one blocker-placement decision:

```text
STAGE
BYPASS
```

not one whole target pass and not one primitive move. Depth `0` is ordinary
consecutive lookahead; depth `1` compares the next blocker decision against
greedy completion.

The implementation uses exact greedy completion as the terminal evaluator and
keeps all existing algorithms as separate selectable variants.

### Completed validation

1. Depth `0` exactly matches ordinary consecutive lookahead.
2. Exhaustive small-leaf checks compare the memoized recursion against a simple
   nonincremental reference.
3. Exhaustive small-input checks replay every returned plan.
4. Exhaustive small-input checks verify the depth-limited policy does not lose
   to ordinary greedy completion.

### Current implementation note

The planner memoizes depth values using full physical partial states. Greedy
terminal evaluation is exact but no longer runs the whole suffix physically:
it finishes the current partial pass, then uses the incremental-RHL
deterministic suffix cache for the remaining consecutive-lookahead bucket.
The implementation does not yet maintain an explicit retained tree object
after each committed blocker, so the retained-tree counter is currently zero.
A compact algebraic state and explicit rerooted tree remain possible
optimizations if deeper search is revisited.

### Initial benchmarks

The rapid benchmark used 200 random 52-card permutations with seed `24301`.

`K=2`:

| depth | mean moves | stderr | min | max | elapsed |
|---:|---:|---:|---:|---:|---:|
| 0 | 387.760 | 1.552 | 318 | 448 | 0.004 s |
| 1 | 350.690 | 1.101 | 304 | 402 | 0.114 s |
| 2 | 348.800 | 1.058 | 302 | 402 | 0.206 s |
| 3 | 346.940 | 0.981 | 302 | 402 | 0.343 s |
| 4 | 345.600 | 0.935 | 302 | 382 | 0.579 s |
| 5 | 344.730 | 0.909 | 302 | 382 | 0.932 s |
| 6 | 344.360 | 0.899 | 302 | 378 | 1.432 s |
| 7 | 343.760 | 0.884 | 302 | 376 | 2.241 s |
| 8 | 343.340 | 0.873 | 302 | 376 | 3.570 s |
| 9 | 342.880 | 0.856 | 302 | 376 | 5.689 s |
| 10 | 342.390 | 0.848 | 302 | 376 | 9.147 s |
| 11 | 341.990 | 0.835 | 302 | 376 | 14.891 s |
| 12 | 341.750 | 0.825 | 302 | 372 | 23.734 s |
| 13 | 341.610 | 0.823 | 302 | 372 | 40.261 s |

`K=1`:

| depth | mean moves | stderr | min | max | elapsed |
|---:|---:|---:|---:|---:|---:|
| 0 | 465.320 | 3.114 | 344 | 584 | 0.004 s |
| 1 | 348.710 | 1.775 | 290 | 430 | 0.528 s |
| 2 | 342.440 | 1.718 | 288 | 418 | 0.898 s |
| 3 | 339.900 | 1.711 | 272 | 404 | 1.656 s |
| 4 | 336.240 | 1.595 | 282 | 396 | 2.685 s |
| 5 | 334.130 | 1.600 | 280 | 396 | 5.088 s |
| 6 | 332.370 | 1.591 | 280 | 394 | 10.072 s |
| 7 | 331.310 | 1.612 | 270 | 400 | 19.745 s |

The main finding is that most of the benefit comes from short lookahead.
`K=2` improves sharply from depth `0` to depth `1`, then tapers. `K=1` starts
much worse at depth `0`, but depth `1` already beats `K=2` depth `1`, and depth
`7` reaches `331.310` moves. Deeper `K=1` runs are probably not worth the
current planner cost unless the retained-tree/compact-state optimization is
implemented first.

After adding the exact suffix-cache terminal evaluator, larger 5,000-sample
tail checks with the same seed measured:

| variant | mean moves | stderr | min | max | elapsed |
|---|---:|---:|---:|---:|---:|
| `K=1`, depth 6 | 330.478 | 0.303 | 260 | 406 | 71.363 s |
| `K=1`, depth 7 | 328.829 | 0.298 | 260 | 412 | 126.549 s |
| `K=2`, depth 7 | 342.798 | 0.177 | 294 | 394 | 35.666 s |

The suffix-cache shortcut substantially improved compute time without changing
the policy, but these larger samples show the expensive tail remains. `K=1`
has the better mean but still produces cases above 400; `K=2` has much lower
variance but still reached 394 in 5,000 samples.

### Optional experimental terminal evaluators

The default evaluator must remain exact greedy completion.

After that baseline is measured, alternate learned or handcrafted terminal
evaluators may be added as separate variants. They need not be admissible, but
they lose the policy-improvement guarantee and must not replace the baseline.

## Deferred idea: optimized partition trees

Keep this noted but do not implement it yet:

- mask branch-and-bound;
- pattern-database mask bounds;
- symbolic mask families or decision diagrams;
- choose value-bucket boundaries and an alphabetic partition tree jointly;
- trade partition depth against leaf extraction cost;
- consider both distribution-optimized and per-instance variants.

Portfolio algorithms are explicitly out of scope for now.

## Remaining research directions

These are open topics, not automatic implementation assignments:

1. Strengthen TRANSPORT HEURISTIC while preserving admissibility.
2. Investigate consistent relaxations, reopenings, and pathmax behavior.
3. Add disjoint additive pattern databases for exact abstractions.
4. Optimize DEPTH-LIMITED RHL with compact partial states and explicit retained
   tree reuse, if deeper horizons become interesting again.
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
