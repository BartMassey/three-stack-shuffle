# Exact three-stack search

This records exhaustive endpoint distances through n=9
and shortest-plan tables through n=8. The later
[target-specific search](campaign-exact.md) solves six
selected n=10–12 instances, not all targets at those sizes.
The exhaustive results below are unchanged.

The reusable program is `tools/exact_search.cpp`. Its JSON
outputs are `results/exact-n1.json` through `exact-n9.json`.
The n=9 run was already underway when the coordinator capped
further full searches at n=8; it completed in 3.56 seconds.
No runs above n=9 were performed.

## Model and conventions

The three stacks form the path A-D-B. A move removes exactly
one top card from one stack and places it on top of an adjacent
stack. The legal directed pairs are A->D, D->A, D->B, B->D.
There is no direct A-B move and no whole-stack operation.

All sequences are listed top to bottom. Initially A and B
are empty and D contains `[0,1,...,n-1]`. Every target has
empty A and B and a specified permutation in D. Distance is
the number of individual legal card moves. All moves cost 1.

The graph is undirected, so BFS from the common initial state
computes the optimal distance to every final permutation.
The reported mean is the uniform mean over all n! targets,
including the initial ordering at distance zero. It concerns
an offline, target-aware optimal plan, not the mean cost of
a particular randomized shuffling algorithm.

## Representation and completeness

A state is represented by a permutation obtained by
concatenating A, D, B in that order, plus the lengths of A
and D. The length of B follows from n. This representation is
bijective: the stack cuts reconstruct all three sequences.
There are n! * (n+1)(n+2)/2 states. Permutations are indexed by
their zero-based lexicographic Lehmer ranks; stack cuts are
indexed by nested loops over A length, then D length.

Four prefix or segment rotations implement the four legal
moves. BFS visits every represented state in every run.
Every central-stack target has even distance, since a move
changes the number of cards in D by exactly one and its
initial and final values are equal. The program asserts
this parity property and full-state reachability. It also
checks that reversal has distance 4(n-1) in each sampled run.
That last check verifies the sampled instances; it is not
a proof of the formula for arbitrary n.

## Exact results

| n | targets | exact mean | decimal mean | maximum |
|---|---------|------------|--------------|---------|
| 1 | 1 | 0 | 0 | 0 |
| 2 | 2 | 2 | 2 | 4 |
| 3 | 6 | 5 | 5 | 8 |
| 4 | 24 | 97/12 | 8.0833333333 | 12 |
| 5 | 120 | 111/10 | 11.1 | 16 |
| 6 | 720 | 5081/360 | 14.1138888889 | 20 |
| 7 | 5040 | 5414/315 | 17.1873015873 | 24 |
| 8 | 40320 | 410341/20160 | 20.3542162698 | 28 |
| 9 | 362880 | 4287301/181440 | 23.6293044533 | 32 |

The JSON stores the complete distance histogram and a distance
for each target in lexicographic permutation order. Exact
means are reproducible from `distance_sum / target_count`.
`mean_numerator` and `mean_denominator` store that same
fraction without reduction; decimal means are supplemental.

For n=8, the histogram is:

| distance | count |
|----------|-------|
| 0 | 1 |
| 4 | 1 |
| 6 | 3 |
| 8 | 10 |
| 10 | 36 |
| 12 | 139 |
| 14 | 566 |
| 16 | 2412 |
| 18 | 7234 |
| 20 | 13093 |
| 22 | 12213 |
| 24 | 4318 |
| 26 | 293 |
| 28 | 1 |

For n=9, the histogram is:

| distance | count |
|----------|-------|
| 0 | 1 |
| 4 | 1 |
| 6 | 3 |
| 8 | 10 |
| 10 | 36 |
| 12 | 139 |
| 14 | 566 |
| 16 | 2412 |
| 18 | 10666 |
| 20 | 36624 |
| 22 | 84433 |
| 24 | 117986 |
| 26 | 85148 |
| 28 | 23602 |
| 30 | 1251 |
| 32 | 2 |

Reversal is the unique maximum-distance target for n=2..8.
At n=9 it shares the maximum with the non-reversal target
`[6,8,4,7,2,5,0,3,1]`. Both require 32 moves. This is a
concrete change in the extremizers, although it does not
establish an asymptotic phase transition.

The observed maximum remains 4(n-1) through n=9. Neither
that observation nor the roughly linear small-n means
justifies extrapolation to n=52. Factorial state growth
prevents this full BFS from approaching deck size 52.

## Reproduction and resource use

Build with a C++20 compiler:

```sh
g++ -std=c++20 -O3 -march=native -Wall -Wextra \
    -pedantic tools/exact_search.cpp \
    -o /tmp/three-stack-exact-search
/tmp/three-stack-exact-search 8 results/exact-n8.json
```

The CLI accepts `N [OUTPUT.json] [PLANS.json]`; without the
output argument it prints JSON to stdout. Summary statistics
go to stderr. The optional plans output requires n<=8.
Its accepted range is 1..10, but n=10 has not been tested.
It reserves the complete BFS queue, using about six bytes
per state for the queue and distance arrays, before small
overheads and the final per-target distance vector.

Recorded times use GCC 14.2.0 with the options above. They
exclude JSON serialization, as the timing field is recorded
immediately after BFS and aggregate calculation.

| n | states | seconds | BFS array bytes |
|---|--------|---------|-----------------|
| 6 | 20160 | 0.00165 | 120960 |
| 7 | 181440 | 0.01688 | 1088640 |
| 8 | 1814400 | 0.19169 | 10886400 |
| 9 | 19958400 | 3.55860 | 119750400 |

The anticipated n=10 BFS arrays require 1,437,004,800 bytes;
larger runs need explicit resource planning. These figures
are allocation calculations, not measured peak RSS.

## Independent verification

A separate Python reference BFS represented states as tuples
of three explicit top-to-bottom tuples. For n=1..5 it
generated neighbors by removing `state[source][0]` and
prepending that card to `state[destination]`, without using
permutation ranks, cuts, or rotations. All target distances
and total visited-state counts agreed with the C++ outputs.
That comparison covered 1, 2, 6, 24, and 120 targets and
3, 12, 60, 360, and 2520 full states, respectively.

## Compact optimal plans

`results/plans-n1.json` through `plans-n8.json` contain an
optimal plan for every target in lexicographic order. Plans
are recovered by repeatedly following an adjacent state
whose BFS distance is one smaller. The resulting reverse
path is inverted into an executable forward path, requiring
no additional full-state parent storage.

Move codes are `0=AD`, `1=DA`, `2=DB`, `3=BD`. Encoding starts
at the integer 1. Each move in execution order appends two
bits with `(packed << 2) | code`. To decode, take low two
bits, shift right two, repeat until reaching sentinel 1,
then reverse the decoded move list. The empty plan is 1.
The largest n=8 plan uses 28 moves and 57 bits including
the sentinel, fitting unsigned 64-bit storage. JavaScript
consumers must use BigInt or another exact integer reader;
some plan integers exceed the exact range of Number.

All 46,233 emitted plans for n=1..8 were independently
decoded and replayed with the shared Python `Machine`.
Every final target and empty-side-stack condition passed,
and every plan length equaled its BFS optimal distance.
For arbitrary distinct labels, map each target label to
its index in the initial ordering, rank that normalized
permutation, and retrieve its plan. Operations themselves
are label-independent.
