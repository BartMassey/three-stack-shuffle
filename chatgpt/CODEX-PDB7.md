# Codex Transient Handoff: Seven-Card Additive Pattern Heuristic

This is a focused experimental assignment. Do not turn it into a broad search
or algorithm refactor, and do not replace the permanent project documentation
until the experiment has produced results.

## Read the repository as it now stands

The implementation has already been simplified in ways that make this
experiment small:

- `State` is the canonical three-stack state, with stacks stored
  top-to-bottom.
- `State::neighbors()` already defines the complete unit-cost graph.
- `ReverseBfs::build(n)` already computes exact goal distances for every
  `n`-card state.
- `ReverseBfs::distance()` already performs exact lookup.
- A* with reopenings already exists in `src/search.rs`.
- The perfect-leaf code already projects contiguous value intervals and
  relabels `low..=high` to `1..=k`.

Reuse these pieces. Do not build a second abstract search engine, introduce
another machine representation, or implement a compact microcontroller table
format in this first experiment.

The initial prototype may use the existing `HashMap<State, usize>` reverse-BFS
databases. Compact indexing and serialization are follow-up work only if the
heuristic proves useful.

## Goal

Implement and measure an admissible additive pattern-database heuristic based
on disjoint value intervals of at most seven cards.

For a full state `S`, partition card values into disjoint, contiguous,
approximately equal-sized intervals:

```text
P1, P2, ..., Pr
```

with every interval size at most:

```text
PATTERN_SIZE = 7
```

For `n = 52`, do not use seven size-7 patterns plus a size-3 tail. Balance the
eight intervals so that their sizes differ by at most one:

```text
7, 7, 7, 7, 6, 6, 6, 6
```

The exact order of the four size-7 and four size-6 intervals must be
deterministic; larger intervals first is fine.

For each interval `P`:

1. Delete all cards not in `P`.
2. Preserve the remaining cards' stack locations and top-to-bottom order.
3. Relabel the interval to `1..=|P|`.
4. Look up the exact distance of the projected state.
5. Sum the interval distances.

Define:

```text
H_PDB(S) = sum over intervals P of exact_distance(project_P(S))
```

Also test:

```text
H_MAX(S) = max(transport_heuristic(S), H_PDB(S))
```

Do not add the transport and PDB values. They may charge the same physical
moves.

## Why the sum is admissible

This proof is part of the implementation contract.

For a fixed interval `P`, take any legal full solution and erase every move
whose moved card is not in `P`. Also erase all non-`P` cards from the states.

The remaining moves form a legal solution of the projected `P` problem:

- a retained card that was exposed in the full state remains exposed after
  other cards are deleted;
- deleting cards removes obstructions and cannot introduce one;
- the final retained cards are sorted on `D` relative to one another.

Therefore the exact projected distance for `P` is no greater than the number
of moves made by cards in `P` in any full solution.

The intervals are disjoint, and each primitive move moves exactly one card.
Thus moves counted for one interval are never counted for another interval.
Summing the exact projected distances is therefore no greater than the length
of any full solution.

Cross-pattern cooperation is allowed. Projection gives each pattern even more
help by deleting every other card entirely.

For a fixed partition, `H_PDB` should also be consistent: a primitive move
changes the projection of exactly one interval by one graph edge, while all
other projected states are unchanged.

## Implementation shape

Keep this primarily in `src/search.rs`.

### 1. Generalize interval projection

There is already interval-projection code in `src/algorithms.rs` for perfect
leaf extraction. Avoid maintaining two subtly different implementations.

Move or expose a shared helper with behavior equivalent to:

```text
project_interval_state(state, low, high) -> State
```

For every retained card:

```text
projected_label = card - low + 1
```

The result must be a valid `State` containing exactly labels
`1..=high-low+1`.

Update the existing perfect-leaf code to use the shared helper without changing
its behavior.

### 2. Build exact databases once

Add a small container such as:

```text
PatternDatabases {
    by_size: databases for sizes 0..=7
}
```

The simplest first implementation may contain `ReverseBfs` instances.

Build each required size once and reuse it for all heuristic calls. Do not
construct a reverse BFS inside the heuristic.

Sizes 6 and 7 are sufficient for the `n=52` balanced partition, but supporting
all sizes `0..=7` makes tests and arbitrary `n` straightforward.

Expected state counts include:

```text
size 6:  6! * C(8,2) = 20,160
size 7:  7! * C(9,2) = 181,440
```

Report the observed maximum exact distance in each database. Do not assume
`u8` storage until the maximum has been checked, although it is expected to
fit.

### 3. Deterministic balanced interval partition

Implement a helper conceptually equivalent to:

```text
balanced_value_intervals(n, maximum_size)
```

Let:

```text
count = ceil(n / maximum_size)
small = floor(n / count)
large_count = n mod count
```

Produce `large_count` intervals of size `small+1` and the rest of size
`small`, covering `1..=n` exactly and without overlap.

Required `n=52`, `maximum_size=7` result:

```text
8 intervals
four of size 7
four of size 6
```

Test the helper over a broad range of `n`.

### 4. Heuristic object or context

The PDB heuristic needs shared database state, so avoid a global mutable
singleton.

Use a borrowed context or closure, for example:

```text
pdb.heuristic(&state, 7)
```

Refactor A* minimally so it can accept a heuristic callback or strategy:

```text
astar_with_heuristic(start, heuristic)
```

Preserve the existing public `astar(start)` behavior as the transport-heuristic
wrapper.

Add explicit wrappers or modes for:

```text
transport only
PDB only
max(transport, PDB)
```

A* must retain its existing reopening behavior. `H_PDB` alone should be
consistent, but `H_MAX` may inherit transport's inconsistency.

### 5. Generic heuristic validation

The current validation routine is hardwired to TRANSPORT HEURISTIC. Refactor
it to validate a supplied heuristic while preserving the existing wrapper or
test behavior.

Collect the existing metrics:

```text
admissibility failures
maximum consistency violation
violating-edge fraction
average h / exact_distance
number of exact states
```

## Required tests

### Projection tests

Test arbitrary three-stack states, not only initial decks.

Verify that projection:

- preserves stack location;
- preserves top-to-bottom order;
- removes unselected values;
- relabels the selected interval correctly;
- produces a valid state.

Also verify that the shared helper leaves existing perfect-leaf tests
unchanged.

### Exactness for one pattern

For every state through `n=7`:

```text
H_PDB(state, maximum_size=7) == exact_distance(state)
```

There is only one interval, so this must be exact.

### Exhaustive admissibility

Using the existing reverse-BFS oracle, exhaustively verify for practical small
`n` that:

```text
H_PDB(state) <= exact_distance(state)
H_MAX(state) <= exact_distance(state)
```

Choose a test maximum pattern size smaller than `n` in at least one exhaustive
test so the additive case is actually exercised, for example:

```text
n=6, maximum_size=3
```

### Consistency

For a fixed balanced value partition, exhaustively verify:

```text
H_PDB(parent) <= 1 + H_PDB(child)
```

on every directed edge in the tested graph.

Do not require this of `H_MAX`, because TRANSPORT HEURISTIC is already known
to be inconsistent.

### Known states

At minimum test:

```text
goal: H_PDB = 0
```

and compare transport, PDB, and max on:

```text
reversed decks
random initial permutations
states with cards on all three stacks
```

Do not assume PDB dominates transport on every state.

## Benchmark experiment

The point of this task is to measure heuristic quality, not merely compile it.

### Database construction

Report for every built size:

```text
state count
construction time
maximum distance
rough process-memory change if readily available
```

At minimum report totals for sizes 6 and 7.

### Random `n=52` initial states

On the same deterministic random sample, report distributions for:

```text
transport
seven-card additive PDB
max(transport, PDB)
```

Include:

```text
sample count and seed
mean
minimum
maximum
standard deviation
standard error
fraction PDB > transport
fraction transport > PDB
fraction equal
mean and maximum improvement of max over transport
```

Use the repository's existing deterministic RNG.

### A* comparison

On the largest tractable common set of states, compare:

```text
transport only
PDB only
max(transport, PDB)
```

Report:

```text
optimal distance
start heuristic
generated
expanded
reopened
stale
max open
elapsed time
```

Use exactly the same input states for all three modes.

Start with `n=8` or another practical size. Increase only if runtime remains
reasonable. The experiment is still useful if `n=52` A* is completely
intractable.

## Optional second partition only after the baseline

After the balanced contiguous-value partition works, one small comparison is
allowed:

- add one fixed interleaved value partition of the same pattern sizes;
- compute its additive value;
- take the maximum of the two fixed-partition heuristics.

The maximum of fixed admissible heuristics is admissible, and the maximum of
fixed consistent heuristics is consistent.

Do not implement state-dependent partition optimization, local swapping,
random pattern search, overlapping-pattern LPs, or portfolio algorithms in
this task.

## Microcontroller follow-up, not part of the first implementation

The mathematical seven-card database contains only:

```text
181,440
```

states. If the measured maximum distance fits in one byte, a dense table would
need about 177 KiB before indexing metadata.

Do not optimize for this yet. First establish:

1. heuristic correctness;
2. heuristic strength;
3. A* expansion reduction;
4. database maximum distance.

If promising, the next task can design:

- a dense rank for permutation plus two stack separators;
- a distance-only table;
- serialization and a generated static byte array;
- a no-allocation lookup suitable for a microcontroller.

## Deliverables

1. Working implementation with no regressions.
2. Focused tests proving exactness, admissibility, and consistency.
3. Benchmark output comparing transport, PDB, and their maximum.
4. A concise result note stating whether the seven-card PDB materially improves
   heuristic quality and A* search.
5. Do not permanently rewrite `ALGORITHMS.md` yet. Report results first so the
   documented design can reflect what actually worked.
