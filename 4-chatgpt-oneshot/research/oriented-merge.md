# Merge directly onto the required output stack

4 October 2026. Second campaign construction, implemented
in [oriented_merge.py](../oriented_merge.py). Independent
review is recorded in [oriented-review.md](oriented-review.md).

## Result

There is a constructive upper bound of **352 moves** for
every 52-card target, improving the previous 410 bound.
Balanced recursion gives

    (4/3) n log2(n) + O(n)

moves for every size. The algorithm still uses only
AD, DA, DB, and BD. There is no free side-to-side transfer.
The implementation retains balanced oriented recursion
above 64 cards, so the asymptotic improvement is executed,
not just a hypothetical recurrence.

## Why the earlier construction did extra work

A central-output merge has two sorted children on A and
B, then merges them onto D. The previous parked parent
first produced that central output and moved it to a
side. Cancellation saves at least two moves, but this
still visits an endpoint that its caller never needed.

Instead, distinguish two recursive contracts:

- U: sort the active segment onto D in the requested order.
- P: put the reverse requested order onto a specified side.

The reversal in P is intentional: two such children can
be merged onto D by taking the larger exposed rank first.
Either contract can request ascending or descending order.
Reversing the requested order is a choice of subproblem,
not a free physical reversal.

## The new side-output merge

Suppose the required output is decreasing on A, with
all ranks expressed relative to the final target.
Split the current top segment into an initial a cards
and the following b cards.

1. Park the first child on B in **increasing** order.
   Do this with its P routine requesting decreasing order.
2. Sort the second child on D in increasing order.
   The first child's cards remain a protected base of B.
3. Repeatedly take the smaller of the exposed B and D
   ranks. If it is on D, transfer DA. If it is on B,
   transfer BD followed immediately by DA.

The selected ranks arrive on A in increasing order, so
the final A stack is decreasing from top to bottom.
Each first-child card costs two merge transfers, and
each second-child card costs one. Thus the merge toll
is 2a+b, not the cost of merging onto D and reparking.

For example, before merging let B=(1,4) and D=(2,3),
both top first, with A empty above its protected base.
The word

```text
BD DA   DA   DA   BD DA
```

selects 1,2,3,4 and leaves A=(4,3,2,1), with B and D
empty above their bases. Six transfers equal 2*2+2.

During BD DA, the B card temporarily covers the D head
and is immediately removed again. No simultaneous top
access or direct B-to-A transfer is being assumed.

## Protected bases and induction

Every child consumes only its specified active segment.
An otherwise legal local word cannot pop below a base:
the local source would have been empty at that step.
This applies equally when the first child is stored
under temporary workspace on B while the second runs.

The merge loops use the two child lengths explicitly.
They never consume an underlying base. Reflection swaps
A and B, and handles a requested B endpoint. Adjacent
inverse cancellation preserves every subsequent state.
These observations prove both recursive contracts by
induction from the exact small endpoint tables.

## Universal recurrence

For n<=8 use the verified bounds

    U(n) = 4 max(0,n-1)
    P(0..8) = 0,1,4,7,12,15,20,23,28.

For a+b=n, central merging gives

    U(n) <= P(a)+P(b)+n.

The new construction gives

    P(n) <= P(a)+U(b)+2a+b.

The old alternative P(n)<=max(n,U(n)+n-2) remains valid.
The maximum covers the empty central word; the direct
construction also always costs at least n.
Optimize each recurrence independently over fixed splits
through n=64. Their splits need not agree. At n=52,

    U(52) <= 52 + P(23) + P(29)
          <= 52 + 125 + 175 = 352.

The implementation always includes those certified split
choices among its candidates. Locally shortest choices
can alter boundary cancellation opportunities, but cannot
violate this unreduced upper-bound recurrence. Sample
maxima do not enter the calculation.

## Large-n analysis

Choose a=floor(n/2), b=ceil(n/2). Set

    V_U(n)=U(n),       V_P(n)=P(n)-n/3.

The central recurrence becomes a sum of child V_P costs
plus 4n/3. The parked recurrence becomes child V_P(a)
plus V_U(b), with toll

    4n/3 + (2/3)(a-b) <= 4n/3.

Thus either state has adjusted toll at most 4n/3 per
balanced level. There are at most ceil(log2 n) levels;
the sum of segment sizes on each level is at most n.
Singleton adjusted costs are at most 2/3. Therefore

    U(n) <= (4/3)n ceil(log2 n) + (2/3)n,

and P has only an additional n/3 term. Exact leaves and
better bounded split choices can only improve this
construction. The leading coefficient is 4/3 rather
than the original balanced merge's 2.

Memoized interval/orientation states prevent duplicated
recursive work. Above the fixed search cutoff, only
balanced splits are used; finite small-size optimization
changes constants, not asymptotics. Copying materialized
words gives a conservative O(n log² n) host-time bound.
The returned word occupies O(n log n) space; this is not
a claim that all memoized intermediate words together
occupy only the output space.

## Validation and development measurements

Tests independently replay all three endpoint contracts,
with protected bases, for every target through n=6.
Recursive tests cover identity, reversal, random targets,
and sizes through n=257. Every checked output satisfies
the corresponding U or P bound. Independent review also
checks that no guard is moved at an intermediate step.

On 20 original development targets, bidirectional planning
with midpoint and certified splits, but no split window,
averages 284.4 moves (maximum 296), versus 320.5 (338)
for the previous parked controller. The new construction
wins on all 20. Median planning is about 8.7 ms versus
101.6 ms in that run.

A window of four gives mean 272.1 and maximum 284 on the
same 20 targets, at about 140 ms. These are development
measurements, not estimates of the optimal mean. Fresh
validation uses seed 2026100402, frozen after these two
settings were selected. Times are indicative host costs,
not isolated timing guarantees.

The fresh 100-target results are now complete:

| Controller | Mean | Maximum | Median planning |
|---|---:|---:|---:|
| Previous parked | 318.74 | 340 | about 103 ms |
| Direct-output fast | 284.72 | 300 | 8.8 ms |
| Direct-output window four | 273.80 | 288 | 134.9 ms |

Both new versions win on all 100 targets. Records are
[fast holdout](../results/oriented-holdout100.json) and
[window holdout](../results/oriented-window4-holdout100.json).
Every winning word is stored and independently replayed.
The new CLI modes are `oriented` and `oriented_window`;
both add the existing exact and structured special cases.

```sh
python3 -m unittest test_oriented_merge -v
python3 tools/oriented_experiments.py --count 20 \
  --output build/oriented-development20.json
python3 tools/oriented_experiments.py --count 100 \
  --fresh-seed 2026100402 \
  --output build/oriented-holdout100.json
```

The experiment retains the old parked plan when comparing
portfolios. No runtime default is changed by this note.
