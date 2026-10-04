# Third campaign: residual bounds and evaluation

4 October 2026. This continues the second campaign's
[residual bound](residual-bounds.md). All search budgets
below are local bounded experiments. The unresolved
n=18 interval remains [64,66]. No 52-card exact search
was performed.

## A consistency theorem

Use the earlier conventions: delete the longest correct
bottom D suffix; let m be the active card count, s the
initial side count, and K the maximum feasible ordinary
card count. The earlier formula can be written

```text
R = 4m - s - 2K.
```

Theorem: R is consistent on every physical move. Thus,
for adjacent states u,v, R(u)<=1+R(v). Because physical
moves are reversible, their R values differ by at most
one. This strengthens the earlier finite consistency
observation to a proof for arbitrary deck sizes.

First consider departing an active top D card x to a
side. The active count stays fixed and s increases by
one. Every feasible ordinary set of the successor gives
a feasible ordinary set of the predecessor of the same
size. If x is omitted, nothing needs changing. If x is
selected, assign its predecessor D departure to that
side. In the successor, x is the selected initial side's
largest card: the selected side order is decreasing.
Every selected D card later assigned there exceeds x by
the separation constraint. Therefore putting x first in
that predecessor D chain preserves increasing order and
its separation from the original side cards. Consequently
K(successor)<=K(predecessor), and the displayed formula
gives R(predecessor)<=1+R(successor).

Next consider returning a top side card x to active D,
without extending the correct suffix. The active count
stays fixed and s decreases by one. Remove x from any
successor ordinary set if it is selected. The remaining
set is feasible in the predecessor: deleting the new D
top preserves its chain constraints, and reinserting
an omitted initial side top preserves all selected side
orders. Hence K(successor)<=K(predecessor)+1, yielding
R(predecessor)<=1+R(successor).

The remaining case is returning x to extend the correct
suffix. Before the return, active D is empty and x is
the largest active rank. Adding x to the optimum ordinary
set after the return preserves decreasing order on its
initial side. Conversely, deleting x from any predecessor
ordinary set leaves a feasible successor set. Thus
K(predecessor)=K(successor)+1, while both m and s fall
by one. R falls by exactly one. Departing a suffix card
is the reverse of this case. These cases cover every
edge in both orientations and complete the proof.

R already has the required physical distance parity.
Its consistency means ordinary pathmax propagation
cannot strengthen it: across an edge, R(parent)-1 is
already at most R(child).

## Disjoint cost partition with a PDB

For a fixed subset P, write Q for its complement and
project the current state onto each subset, relabeling
each projection by its own target order. Let dP be the
exact pattern distance and RQ the projected residual
bound. Then

```text
F(P) = dP + RQ
```

is admissible. Project any complete physical plan onto P
and Q. Both projections are legal complete plans. Every
physical transfer belongs to exactly one projection, so
their lengths sum to the full length. The first projected
length is at least dP and the second at least RQ. Their
sum therefore bounds the full length without counting
any transfer twice.

F(P) is also consistent. One physical move changes one
projection by one legal move and leaves the other fixed.
Exact distance and R are each consistent, so their sum
is consistent on this edge. Taking the maximum over a
fixed pattern family preserves consistency.

Projected Q can have a longer correct suffix than the
full state. Consequently F(P) need not dominate the old
PDB-plus-full-state-mandatory-charge bound. The search
keeps both, together with full-state R, using their
maximum. No overlapping full-state bounds are added.

The cheaper staged variant picks the pattern maximizing
the old PDB-plus-mandatory value at the current state and
evaluates F only for that pattern. It remains admissible,
because whichever pattern is selected supplies a valid
bound. This changing choice does not preserve consistency:
the full six-card check finds 166 violating directed
edges. The search needs admissibility, not consistency,
for its state-based depth pruning. No consistency theorem
is claimed for this staged variant.

## Faster evaluation with the same R formula

The side options now take quadratic side work. Compute
the longest decreasing subsequence starting at each side
position by scanning right to left. For rank threshold t,
the earlier L(S,t) equals the largest starting length
whose starting rank is at most t. A decreasing sequence's
first rank is its maximum, which proves this identity.
Scan starting ranks increasingly, retaining only points
where that maximum rises. A higher threshold with the
same count is dominated by the lower threshold: it gives
the same initial score and permits fewer D extensions.

Fold the tail pair into increasing order. Future D
extensions do not depend on which side owned a tail,
because its initial side count and rank separation have
already been encoded in that tail and its score. Two
orientations with the same unordered tails have identical
future possibilities, so keep their larger score.

Maintain a list of occupied folded tail entries. When
processing a D card x, take the list's previous length
and process only those entries. All new destinations
contain tail x. Existing tails are initial side cards or
previously processed D cards; distinct card ranks mean
no existing entry contains x. Therefore these updates
cannot change an old source entry. In-place insertion is
safe, cannot select x twice in this scan, and removes the
dense table copy. Omission simply keeps existing entries.

Worst-case tail work remains O(d*n^2), and total worst
case remains O(n^3), with O(n^2) memory. The gain is from
quadratic side preprocessing, dominated side options,
symmetry, occupied-entry scans, and avoiding copies.
This is not a new asymptotic complexity claim.

The optional residual search also passes the child
heuristic already calculated for move ordering into its
recursive visit. This preserves its value, move order,
visited nodes, and threshold decisions while avoiding a
duplicate PDB and residual evaluation. The default mode
keeps its previous recursive evaluation behavior.

## Exhaustive validation

[Validation results](../results/third-residual-validation.json)
cover all 23,115 full states through six cards and all
68,812 directed edges. Optimized Python R agrees with
the previous Python formula and C++ on every state. R is
admissible and has no consistency violation. C++ and
Python also agree on the full and staged partition
hybrids. Both remain below the independent BFS distances.

With every four-card pattern at n=6:

| Hybrid | Mean | States improving previous hybrid |
|---|---:|---:|
| Previous max(PDB+charges,R) | 12.039583 | 0 |
| Full disjoint partition maximum | 12.238194 | 2,002 |
| One strongest-base pattern | 12.132440 | 936 |

The exact mean is 12.899504. These means are uniform over
all full six-card states. They do not estimate any
52-card distribution.

[Regression results](../results/third-residual-regression.json)
compare the previous and updated C++ binaries on reversed
targets through six cards, in default mode and optional
mode one. Bounds, words, visited nodes, transpositions,
table entries, interruptions, and exhausted thresholds
all match. The existing n=14 proof also visits exactly
671,869 nodes in both binaries, with two transposition
hits and 312,285 table entries. Host total time changes
from 4.917 to 2.230 seconds in that paired run.

## Larger samples and bounded search gates

The existing recorded witness states and deterministic
random walks were reused with the same 32 patterns.
[Partition samples](../results/third-residual-partition.json)
show these gains above the previous hybrid:

| n | Full mean walk gain | Full wins/128 | Staged wins/128 |
|---|---:|---:|---:|
| 14 | 1.078125 | 46 | 33 |
| 16 | 0.875000 | 41 | 24 |
| 18 | 0.640625 | 34 | 9 |

No variant improves any recorded witness-state bound.
All witness comparisons remain below the verified
remaining path lengths. Walks are heuristic samples,
not independent uniform draws. At n=18 the maximum
partition gain is four; its initial bound remains 64.

The [paired ten-second n=18 pilot](
../results/third-residual-n18-pilot.json) reports:

| Evaluation | Nodes | Exhausted thresholds |
|---|---:|---|
| Previous residual mode one | 1,220,608 | none |
| Optimized residual mode one | 3,305,472 | none |
| All 32 disjoint partitions | 581,632 | none |

The same-bound optimizer traverses about 2.71 times as
many nodes in this bounded comparison. This justified
one [45-second gate](../results/third-residual-n18-45.json).
It visits 15,142,912 nodes, reaches the one-million-entry
table limit, records no transposition hits, and stops
without exhausting threshold 64. The interval stays
[64,66]; more traversal alone supplies no new certificate.

The full partition's stronger samples fail the practical
throughput gate, so it receives no 45-second expansion.
The [staged ten-second gate](
../results/third-residual-n18-selected.json) visits
2,740,224 nodes, with 1,529 transposition hits, but also
exhausts no threshold. It is faster than all-pattern
partitioning and slower than optimized R alone. All
n=18 work stops after these bounded gates.

## Explicit switches and reproduction

The final optional C++ argument now accepts:

| Value | Behavior |
|---|---|
| 0 or omitted | Existing default heuristic |
| 1 | Same residual formula, faster evaluation |
| 2 | Mode one plus all fixed disjoint partitions |
| 3 | Mode one plus the strongest-base partition |

Pattern selection still occurs before enabling any
residual mode. Fixed PDBs, parity, exact keys, inverse
pruning, capacity, and interruption accounting retain
their definitions. No comments or unsafe code were added.

```sh
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic \
  tools/third_residual.cpp -o /tmp/third-residual-check
python -m tools.third_residual validate \
  --binary /tmp/third-residual-check \
  --output results/third-residual-validation.json
python -m unittest test_third_residual
```

`tools/third_residual.py samples` reproduces the larger
comparison using the existing packed eight-card PDB.
Its `pilot` mode requires an explicit search binary and
rejects per-search budgets above 45 seconds. The prior
binary used for paired regression was compiled before
the optimization; archived artifacts record both outputs.
