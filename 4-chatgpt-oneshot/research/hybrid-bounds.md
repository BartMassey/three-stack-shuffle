# Guaranteed balanced-hybrid upper bound

Scope: the original controller's 444-move guarantee.
This proof remains valid, but the optional parked mode
now proves 410; see [the execution campaign](campaign-execution.md).
The two guarantees concern different constructions.

The existing `hybrid_merge` controller with `leaf_limit=8`
has a guaranteed upper bound of 444 moves for any 52-card
target. This improves the bound of 488 obtained by adding
all leaf costs and parking/merging costs without accounting
for cancellation. No algorithm modification is needed.

The machine-readable calculations and complete leaf
effective-cost distributions are stored in
`results/hybrid-bounds.json`.

## Exact leaf analysis

For a chosen exact-table plan, let L be its length and r
the number of consecutive identical source-to-D moves at
its end. The identity plan has L=r=0. Every nonempty plan
ending with empty side stacks must finish with AD or BD,
so r>=1. Reflection exchanges A and B without changing
L or r or the final permutation in D.

When the sorted child is subsequently parked onto a
particular side, reflect its plan so the last source is
that parking side. Its last r returns and the first r
parking moves cancel, saving 2r moves. Since r cannot
exceed the child's number of cards, the parking block
always contains enough inverse moves.

Define E(n) as the maximum L-2r over the actual selected
exact-table plans for n cards. Exhaustive table analysis
gives:

| n | max L | E(n) |
|---|-------|------|
| 1 | 0 | 0 |
| 2 | 4 | 2 |
| 3 | 8 | 4 |
| 4 | 12 | 8 |
| 5 | 16 | 12 |
| 6 | 20 | 16 |
| 7 | 24 | 20 |
| 8 | 28 | 24 |

The bound uses the selected plans' actual tails rather
than assuming that maximum length and minimum tail occur
on the same target. For example, the six-card plan for
`[5,0,4,3,2,1]` has L=18, r=1 and effective cost 16.
The seven-card target `[6,0,5,4,3,2,1]` similarly attains
effective cost 20 with L=22 and r=1.

These E values depend on the stored optimal plans, because
different shortest plans can have different final tails.
The existing `exact_solver` uses these exact same tables.

## Composition proof

For a split into children of sizes a and b, where a+b=n,
the raw combining plan is:

```
left child plan
park left onto A: a moves
right child plan
park right onto B: b moves
merge both children into D: n moves
```

Each child plan remains legal with other cards buried
beneath its active cards. A locally legal child plan never
pops beyond its own locally available stack heights.
Reflection preserves this legality and its sorted output.

The existing `_combine` tests all four independent choices
of child reflection and returns the shortest reduced word.
One choice aligns the left tail with A and the right tail
with B. Its two tail/parking cancellations occupy disjoint
blocks in the raw word, and can both be deleted. Hence

```
B(n) <= 2n + E(a) + E(b).
```

Additional boundary cancellations can only improve this
bound. The selected shortest reflected composition also
satisfies it, even if another candidate would have a more
favorable final tail.

For any selected nonempty composite plan with L<=B(n),
its final return run has at least one move. Therefore
its effective parked cost is at most B(n)-2. An empty
plan has effective cost zero, giving the safe recurrence

```
E(n) <= max(0, B(n)-2), for n>8.
```

This uses only one guaranteed return move for composite
children; it does not assume their final merge run is long.
Reflection of the whole composite preserves its length
and final run length when it is parked at the next level.

## The 52-card tree

Balanced splits produce eight exact leaves: four of size
6 and four of size 7. The recurrence is:

| n | children | B(n) | E(n) upper bound |
|---|----------|------|------------------|
| 13 | 6+7 | 26+16+20 = 62 | 60 |
| 26 | 13+13 | 52+60+60 = 172 | 170 |
| 52 | 26+26 | 104+170+170 = 444 | 442 |

The root uses B(52), because it is returned in D rather
than parked afterward. The value 442 is relevant only if
that whole plan becomes a child in a larger composition.

The uncancelled estimate is 488: leaf maxima total 176,
and parking plus merging at the three internal levels
total 312. Leaf-tail savings guarantee 32 fewer moves;
the six nonroot composite children save at least another
12, yielding 444. Further savings are common, but were
not used in this universal guarantee.

This is an algorithmic upper bound, not the exact worst
optimal distance or a claim that the controller attains
444 on some target. Controllers retaining this baseline
as a candidate and selecting a shorter verified plan
inherit its guarantee.

## Verification

All 46,233 stored plans for n=1..8 were scanned. Every
nonempty plan ended in AD or BD and had r<=n. The exact
effective-cost histogram and a maximizing witness for
each leaf size were recorded in JSON.

An exhaustive additional check composed every ordered
3+4-card split of a seven-card deck, using the existing
`_combine` and exact-table child plans. All 5,040 cases
replayed to the sorted full deck with empty side stacks
and stayed below the proved bound of 26 moves. Their
observed maximum was 26 moves.
