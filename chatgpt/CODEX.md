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
  an initial `K=2` benchmark.

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

## Current assignment: DEPTH-LIMITED RHL

Implement the algorithm specified in `ALGORITHMS.md`.

The search unit is one blocker-placement decision:

```text
STAGE
BYPASS
```

not one whole target pass and not one primitive move.

At each real decision:

1. search the next `depth` binary blocker decisions;
2. evaluate every frontier state by exact ordinary consecutive-lookahead
   completion;
3. commit only the first decision;
4. reroot the retained tree;
5. extend the frontier one level to restore the requested depth.

### Required state

A partial-pass planning state must preserve enough information to continue both
arbitrary planning and greedy completion:

```text
current
bucket_low
source and destination endpoint roles
held staged-card count
next_capture
physical A, D, B stacks
```

At the start of a target pass:

```text
held = 0
next_capture = current - 1
```

A staged card advances `next_capture` only when its value equals
`next_capture`.

### Forced moves

When `current` is exposed:

```text
move held staged cards D -> destination
move current source -> D
decrement current
reset held and next_capture
```

Repeat while later targets are exposed. These moves add to cost but do not
consume search depth.

### Terminal evaluator

Use exact greedy completion from the partial state:

```text
GREEDY_COMPLETION_COST(partial_state)
```

This is not an admissible heuristic and does not need to be. It is an exact
upper-bound completion policy.

Reuse the INCREMENTAL RHL base-policy cache where possible, but extend its key
to include the partial-pass control state (`held`, `next_capture`, and endpoint
roles) when necessary.

### Depth recursion

Implement the documented recursion conceptually equivalent to:

```text
F(state, 0) = greedy completion cost

F(state, d) =
    forced cost
    + min over STAGE/BYPASS:
          immediate cost + F(child, d-1)
```

Always evaluate the greedy action first and retain it on exact ties.

Memoize by:

```text
(normalized partial state, remaining depth)
```

### Tree reuse

Do not rebuild the depth tree after every committed blocker.

After committing the chosen root action:

- retain the chosen child subtree;
- discard the other subtree;
- extend the retained frontier by one decision layer;
- update values bottom-up.

Instrument how much work this actually saves.

### Variants and tests

Preserve all existing algorithms as separate selectable variants.

Add at least:

```text
depth 0
depth 1
depth 2
depth 4
depth 6
depth 8
depth 10
depth 12
depth 14
depth 16
```

Depth 0 must exactly match ordinary consecutive lookahead.

For exhaustive small leaves:

- compare the recursive value against a simple nonincremental reference;
- verify retained-tree and rebuilt-tree versions choose identical actions;
- replay every returned plan;
- verify the depth-limited policy never loses to ordinary greedy completion.

Do not assert that larger depth always produces a lower realized move count.
The rollout estimates are monotone; the receding-horizon policies need not be.

### Benchmarks

Benchmark `K=2` first for comparison with existing RHL data, then attempt
`K=1`.

Report:

```text
depth
mean primitive moves
minimum and maximum moves
standard error
planning time
binary nodes expanded
frontier greedy evaluations
greedy-cache hits and misses
depth-cache hits and misses
nodes retained after rerooting
new nodes added per real decision
peak memory
```

Use the existing deterministic benchmark seeds as well as a smaller rapid
development benchmark.

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
4. Implement and benchmark DEPTH-LIMITED RHL at `K=1` and `K=2`.
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
