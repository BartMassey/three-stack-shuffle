# Codex Implementation Handoff

Use [`ALGORITHMS.md`](./ALGORITHMS.md) as the mathematical specification.
This file is the engineering handoff: representation choices, interfaces,
testing requirements, implementation order, and pitfalls that are easy to miss.

The central rule is:

> Every reported cost is the number of primitive legal moves actually replayed
> by the machine simulator.

No algorithm is allowed to update stacks directly in a way that bypasses move
recording.

---

## 1. Scope and recommended implementation order

Implement in this order:

1. Machine model, primitive moves, plan replay, and invariant checks.
2. Common reversal macros.
3. SELECTION SORT.
4. ADAPTIVE SELECTION SORT.
5. BINARY-PRESORT ADAPTIVE SELECTION SORT.
6. MERGE SORT.
7. MSB RADIX SORT.
8. LSB RADIX SORT.
9. NATURAL SORT.
10. SPLIT-MERGE SORT.
11. SIGNED NATURAL SORT, marked experimental.
12. REVERSING SPLIT-MERGE SORT, marked experimental.
13. Exact reverse BFS for small `n`.
14. A* with TRANSPORT HEURISTIC.
15. Pattern databases and stronger heuristics later.

Do not let the experimental algorithms delay completion and validation of the
fully specified algorithms.

---

## 2. Machine model

There are three stacks in a path:

```text
A — D — B
```

The only primitive moves are:

```text
A → D
D → A
D → B
B → D
```

Every primitive move costs exactly one.

A direct endpoint transfer is not primitive:

```text
A → B  means  A → D → B  and costs 2
B → A  means  B → D → A  and costs 2
```

The initial sorting problem has:

```text
A = []
D = a permutation of 1..n, top-to-bottom
B = []
```

The goal is exactly:

```text
A = []
D = [1, 2, ..., n], top-to-bottom
B = []
```

The search code must also support arbitrary intermediate states with cards on
all three stacks.

### Internal stack representation

Either end of a vector may represent the top, but choose one convention and
enforce it everywhere.

Using the end of a vector as the top is efficient. If so:

- parsing and display must convert to the documented top-to-bottom order;
- comparisons with examples in `ALGORITHMS.md` must account for the reversal;
- state hashing and canonicalization must use one documented representation.

Do not mix physical vector order with logical top-to-bottom order.

### State invariant

At all times:

```text
|A| + |D| + |B| = n
```

and the concatenation of the three stacks contains each card `1..n` exactly
once.

Provide a debug assertion that verifies this after every primitive move.

---

## 3. Core types and interfaces

A useful language-neutral design is:

```text
StackId = A | D | B

Move =
    AtoD | DtoA | DtoB | BtoD

State =
    A stack
    D stack
    B stack

Plan =
    sequence of Move
```

Separate two use cases:

### Mutable simulation

A `Machine` owns mutable stacks and a move log.

```text
Machine.apply(move):
    verify the source is nonempty
    verify source and destination are adjacent
    move exactly one top card
    append move to the log
```

All constructive algorithms should operate through this interface.

### Immutable search state

BFS and A* should use compact immutable states as hash keys. Neighbor
generation returns:

```text
(child_state, primitive_move)
```

The mutable simulator and immutable search representation should share tests
but need not share storage.

### Algorithm interface

Each sorting algorithm should expose something equivalent to:

```text
solve(initial_deck) -> Plan
```

The test harness must replay the returned plan from the initial state rather
than trusting the algorithm's final internal stacks.

Report:

```text
plan length
final state
optional algorithm-specific statistics
```

---

## 4. Universal correctness tests

For every algorithm and every tested input:

1. Every move is one of the four primitive moves.
2. Every move has a nonempty source.
3. Replaying the plan preserves the card multiset.
4. Replaying the plan ends in the exact goal.
5. Reported cost equals `|plan|`.
6. A direct `A ↔ B` helper emits two primitive moves and contributes cost two.
7. A sorted input returns an empty plan when the free sorted-input check is
   enabled.

For small `n`, exhaustively test every initial permutation.

Recommended initial exhaustive range:

```text
n = 0..8
```

Increase as practical.

Also test random permutations for larger `n`, especially `n=52`.

---

## 5. Common move macros

Implement macros by emitting primitive moves. Never increment a cost counter
without emitting the corresponding moves.

Each macro should accept explicit active-segment lengths and should assert that
it does not disturb protected cards below the active segments.

Required macros include:

```text
move k cards source → destination
reverse whole D in place
reverse top k cards D → A
reverse top k cards D → B
reverse top k cards A → D
reverse top k cards B → D
reverse top k cards A → B
reverse top k cards B → A
reverse top k cards in place on A
reverse top k cards in place on B
merge two recorded ascending endpoint sequences onto D
```

Regression costs:

```text
whole-D reversal, n >= 2:       4n - 4
D → endpoint reversal, k >= 1:  3k - 2
endpoint → D reversal, k >= 1:  3k - 2
A ↔ B reversal, k >= 1:         2k
endpoint in-place reversal:     4k - 2
```

Test every macro for small segment lengths and with protected sentinel cards
beneath the active segment.

For the whole-D reversal:

```text
n = 52  =>  204 moves
```

---

## 6. Metadata for logical sequences

Several algorithms manipulate logical runs or subsequences whose boundaries
are not recoverable merely by inspecting stack values after arbitrary moves.

Track boundaries explicitly.

A convenient representation is a stack of segment lengths for each endpoint:

```text
A_segments = [bottommost logical segment length, ..., topmost length]
B_segments = [...]
```

When a segment is moved, split, reversed, or merged, update the metadata in the
same operation.

Assertions should verify that the sum of segment lengths equals the number of
active cards represented on that stack.

Do not silently coalesce adjacent logical runs merely because their boundary
happens to be ascending, unless the algorithm definition explicitly allows it.

---

## 7. Algorithm-specific implementation notes

### 7.1 SELECTION SORT

Freeze the maximal correct bottom suffix:

```text
[m+1, m+2, ..., n]
```

and process only cards `1..m`.

Build the output bottom-up in the order:

```text
m, m-1, ..., 1
```

A bypass is always two primitive moves through `D`.

Regression formulas:

```text
cost = 2m + 2Q
worst for active size m = m² + m
global worst = n² + n
```

For `n=52`:

```text
worst = 2756
random expectation without freezing = 1855
random expectation with freezing ≈ 1854.9607689164
```

Test worst-case witnesses, sorted input, reverse input, and permutations where
one sweep deposits multiple consecutive cards.

### 7.2 ADAPTIVE SELECTION SORT

After selecting the next required card, turn around immediately rather than
finishing a sweep.

The unsorted cards are split between `A` and `B`. Since the full input is known,
the implementation may know which endpoint contains the next required card;
this is free planning, not an online-search restriction.

Regression formulas:

```text
cost = 2m + 2Q
global worst = n² + n
E[Q] without freezing = (n-1)(n-2)/6
E[cost] without freezing = 2n + (n-1)(n-2)/3
```

For `n=52`:

```text
Gene's expected shorthand bypass count = 425
expected legal cost without freezing = 954
expected legal cost with freezing ≈ 952.9803844582
worst = 2756
```

A common bug is to count an endpoint-to-endpoint bypass as one move.

### 7.3 BINARY-PRESORT ADAPTIVE SELECTION SORT

Partition once by value:

```text
low  = 1..floor(n/2)
high = floor(n/2)+1..n
```

Process the high bucket first, then the low bucket. The low bucket may remain
protected under temporary cards; no literal “stack low on high” operation is
required.

For:

```text
a = floor(n/2)
b = ceil(n/2)
```

regression formulas are:

```text
expected cost =
    2n + [(a-1)(a-2) + (b-1)(b-2)] / 3

worst =
    2n + a(a-1) + b(b-1)
```

For `n=52`:

```text
expected legal cost = 504
worst = 1404
```

### 7.4 MERGE SORT

Recursive calls operate on a known active top segment of `D`, not implicitly on
the entire stack. Pass segment lengths explicitly.

After sorting a half in `D`, move it to an endpoint so its maximum is exposed.
During merge, repeatedly move the larger exposed card to `D`.

Regression recurrence:

```text
T(0) = T(1) = 0
T(n) = T(ceil(n/2)) + T(floor(n/2)) + 4n
```

For `n=52`:

```text
T(52) = 1200
```

Test all odd sizes carefully.

### 7.5 MSB RADIX SORT

Recursive calls must carry both:

```text
active segment length
consecutive value interval [low, high]
```

Do not scan protected cards below the active segment.

The straightforward implementation uses:

```text
a = floor(k/2)
b = k-a

T(k) = T(a) + T(b) + 2k + 2a
```

For `n=52`:

```text
T(52) = 880
```

This is the specified straightforward implementation, not an invitation to
silently substitute a different orientation-aware variant.

### 7.6 LSB RADIX SORT

Sort on bits of:

```text
card - 1
```

not on `card`, so values `1..n` map naturally to `0..n-1`.

For each bit:

1. distribute all cards from `D` to `A` or `B`;
2. return `B` first;
3. return `A` second.

This ordering places the zero bucket above the one bucket and preserves
stability after the two reversals.

Regression formula:

```text
2n ceil(lg n)
```

For `n=52`:

```text
624 moves
```

Test non-power-of-two sizes.

### 7.7 NATURAL SORT

On the first pass, identify maximal ascending runs on `D`.

Track logical run lengths explicitly thereafter. The baseline analysis assumes
a balanced pairing schedule and full passes; do not opportunistically change
the schedule while still claiming the documented exact cost.

Moving an ascending run to an endpoint reverses its physical order and exposes
its maximum. Merge topmost endpoint runs by repeatedly moving the larger
exposed card.

Regression:

```text
cost = 2n ceil(lg R)
```

where `R` is the initial number of ascending runs under the specified baseline
schedule.

For `n=52`:

```text
worst = 624
random expectation ≈ 520.1955606296
reverse input = 624
```

### 7.8 SIGNED NATURAL SORT

This algorithm is experimental.

Implement it only after ordinary NATURAL SORT is fully tested. Keep its planner
separate from move execution.

Recommended safe mode:

1. generate a complete signed-natural plan;
2. generate the ordinary NATURAL SORT plan;
3. return the cheaper valid plan.

Do not claim a pure worst-case result beyond what `ALGORITHMS.md` establishes.

Required structured regression:

```text
reverse n-card deck may use optimal whole-D reversal: 4n - 4
n=52 => 204
```

### 7.9 SPLIT-MERGE SORT

At the beginning of each phase, identify ascending phase blocks. Candidate
prefixes may end only between phase blocks.

For each split iteration, find the longest whole-block prefix that is either:

```text
one ascending subsequence
two ascending subsequences
```

Tie-breaking must be deterministic:

1. maximum prefix length;
2. prefer one output sequence to two;
3. fixed membership-bitstring order.

A nonfinal iteration should emit two sequences, one to each endpoint. Track
their lengths.

Candidate feasibility can be implemented with dynamic programming. For a
two-increasing-subsequence candidate, the sequence is feasible exactly when it
can be colored with two colors so that each color class is increasing. The
implementation must also recover a deterministic coloring, not merely return a
boolean.

Regression:

```text
at most ceil(lg n) phases
2n moves per phase
worst = 2n ceil(lg n)
n=52 worst = 624
```

### 7.10 REVERSING SPLIT-MERGE SORT

This algorithm is experimental and substantially more complex.

Candidate forms are:

```text
↑
↓
↑↑
↑↓
↓↑
↓↓
```

A singleton is ascending. A genuine descending subsequence has length at least
two.

Candidate order is lexicographic:

1. maximum whole-phase-block prefix length;
2. minimum exact split-and-normalization cost;
3. minimum number of output sequences;
4. minimum number of cards assigned to descending subsequences;
5. fixed membership-bitstring order.

There is no fixed minimum descending length of five.

Implement candidate planning separately from execution. A candidate record
should include at least:

```text
prefix length
case kind
membership assignment
output lengths
descending lengths
exact normalization cost
deterministic tie-break key
```

Before executing a candidate, recompute its cost from the record and assert it
matches the planner's score.

Regression values:

```text
reverse input cost = 4n - 2
n=52 reverse input = 206
certified n=52 worst-case bound = 1872
```

Do not describe the certified bound as the true worst case.

---

## 8. Cost regression table for `n=52`

With the free sorted-input exit, every algorithm has best case zero.

```text
optimal whole-D reversal                         204

SELECTION SORT expected                          1854.9607689164
SELECTION SORT worst                             2756

ADAPTIVE SELECTION SORT expected                 952.9803844582
ADAPTIVE SELECTION SORT worst                    2756

BINARY-PRESORT ADAPTIVE expected                 504
BINARY-PRESORT ADAPTIVE worst                    1404

MERGE SORT fixed cost                            1200
MSB RADIX SORT fixed cost                        880
LSB RADIX SORT fixed cost                        624

NATURAL SORT expected                            520.1955606296
NATURAL SORT worst                               624
NATURAL SORT reverse input                       624

SIGNED NATURAL SORT reverse input                204
SIGNED NATURAL SORT pure expected/worst          unknown

SPLIT-MERGE SORT worst                           624
SPLIT-MERGE SORT reverse input                   624
SPLIT-MERGE SORT expected                        unknown

REVERSING SPLIT-MERGE reverse input              206
REVERSING SPLIT-MERGE certified bound            1872
REVERSING SPLIT-MERGE expected/true worst        unknown
```

Automated tests should distinguish:

```text
exact cost
expected value
certified upper bound
experimental observation
```

Never turn a bound or simulation result into an exact assertion.

---

## 9. Statistical validation

For algorithms with known random-input expectations:

1. generate uniformly random permutations;
2. replay and count primitive moves;
3. report sample count, mean, standard deviation, and standard error;
4. compare the measured mean with the exact expectation.

Use a deterministic seed in regression tests and a configurable seed in
benchmark runs.

A 20,000-run test at `n=52` should be close to the documented expectations;
large discrepancies almost certainly indicate different move accounting or an
algorithmic mismatch.

---

## 10. Exact small-state search

The state graph is undirected because every primitive move has a primitive
inverse.

The number of states for `n` labeled cards is:

```text
n! · C(n+2, 2)
```

Examples:

```text
n=7:       181,440
n=8:     1,814,400
n=9:    19,958,400
n=10:  239,500,800
```

Implement reverse BFS from the goal for small `n`. This provides:

- exact distances for every state;
- an oracle for validating A*;
- optimal small-instance plans;
- tests for admissibility and consistency of heuristics;
- data for comparing constructive algorithms against optimum.

For every BFS state, verify that reversing every generated edge returns to the
parent state.

---

## 11. A* planner

### State and goal

A* must accept arbitrary `(A,D,B)` states and use the exact goal:

```text
([], [1,2,...,n], [])
```

### Reopenings are mandatory

TRANSPORT HEURISTIC is admissible but inconsistent.

Maintain:

```text
best_g[state]
```

If a newly generated path has lower `g`, update the state and push a new queue
entry even if the state was previously expanded.

Do not use a closed set that permanently finalizes a state on first expansion.

### Stale queue entries

Allow duplicate priority-queue entries. On pop:

```text
if queued_g != best_g[state]:
    discard the stale entry
```

### Priority

The primary key is:

```text
f = g + h
```

Use deterministic secondary keys for reproducibility. The particular
secondary order does not affect optimality.

### Parent reconstruction

Store:

```text
parent[state]
move_from_parent[state]
```

for the current best `g`. Replace both when a state is improved.

Replay the reconstructed plan through the mutable machine simulator.

### Pathmax

Optional:

```text
effective_h(child) =
    max(raw_h(child), effective_h(parent) - 1)
```

Pathmax may improve queue behavior, but it does not eliminate the need for
reopenings.

### Frozen suffix warning

The frozen suffix is part of the lower-bound calculation only.

Do not prune moves that disturb it unless a separate proof establishes that
the pruning preserves an optimal solution.

### Optional endpoint symmetry

Swapping `A` and `B` is a graph automorphism preserving the goal.

After the unsymmetrized implementation passes exhaustive tests, states may be
canonicalized as the lexicographically smaller of:

```text
(A,D,B)
(B,D,A)
```

Path reconstruction then needs to track whether each canonicalized state was
reflected, so defer this optimization.

---

## 12. TRANSPORT HEURISTIC

Use the latest agreed interface:

```text
TRANSPORT_HEURISTIC(A, D, B):
    n := |A| + |D| + |B|

    frozen := FROZEN_SUFFIX_LENGTH(D, n)
    X := active prefix of D before that suffix

    base :=
        2|X| + |A| + |B|

    endpoint_extra :=
        2(|A| - LDS_LENGTH(A))
        + 2(|B| - LDS_LENGTH(B))

    center_extra :=
        2(|X| - MAX_TWO_INCREASING_COVER(X))

    return base + endpoint_extra + center_extra
```

Only nontrivial helpers need separate implementation:

```text
FROZEN_SUFFIX_LENGTH
LDS_LENGTH
MAX_TWO_INCREASING_COVER
```

Ordinary sequence and map operations are assumed.

### Important meaning of `MAX_TWO_INCREASING_COVER`

It returns the maximum number of cards in a subsequence of `X` that can be
partitioned into at most two increasing subsequences.

It does not merely test whether all of `X` is coverable.

When implementing the map dynamic program:

```text
next := copy(states)
```

must be a real snapshot. Do not iterate over entries added during the current
card's update.

### Required heuristic regressions

Goal:

```text
A=[]
D=[1,2,...,n]
B=[]

h=0
```

Reversed deck:

```text
A=[]
D=[n,n-1,...,1]
B=[]

h=4n-4
```

Known inconsistency counterexample:

```text
S:
    A=[]
    D=[3,2,1]
    B=[]
    h(S)=8

S' after D→A:
    A=[3]
    D=[2,1]
    B=[]
    h(S')=5
```

Thus:

```text
8 > 1 + 5
```

### Exhaustive heuristic validation

For every state in the small-`n` reverse-BFS database:

```text
0 <= h(state) <= exact_distance(state)
```

Also measure:

```text
maximum consistency violation
fraction of edges violating consistency
average heuristic / exact distance
number of states where heuristic is exact
```

These metrics will guide later heuristic work.

---

## 13. Search output and benchmarking

For each solved instance report:

```text
n
initial permutation or state
optimal move count
move sequence
start heuristic
states generated
states expanded
states reopened
stale queue entries discarded
maximum open-set size
elapsed time
```

For constructive algorithms report:

```text
algorithm name
primitive move count
algorithm-specific statistics:
    sweeps
    bypasses
    phases
    run counts
    reversal counts
```

Provide machine-readable output as well as a human-readable summary.

---

## 14. Avoid premature optimizations

Initially prefer correctness and inspectability over compactness.

A tuple-of-stacks state representation is acceptable for the first BFS/A*
implementation. Optimize to a permutation-plus-separators encoding only after
the exhaustive tests pass.

Do not add unproved pruning rules.

Do not cache heuristic components in a way that changes semantics across
reopened states.

Do not use experimental algorithms as correctness oracles.

---

## 15. Definition of done for the first coding milestone

The first milestone is complete when:

1. All primitive moves and common reversal macros pass exhaustive small tests.
2. The nonexperimental constructive algorithms sort every permutation through
   at least `n=8`.
3. Replayed move counts match the documented formulas.
4. Random `n=52` measurements agree with known expectations.
5. Reverse BFS produces exact distances for a practical small `n`.
6. TRANSPORT HEURISTIC is exhaustively admissible on that database.
7. A* returns the same distances as BFS for every tested state.
8. The known inconsistency counterexample causes no correctness failure.
9. Every returned plan replays legally to the exact goal.
10. Experimental algorithms are clearly labeled and cannot silently replace a
    certified implementation.
