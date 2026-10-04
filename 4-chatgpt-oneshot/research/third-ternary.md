# Ternary recursion for a side-output endpoint

Third campaign, 4 October 2026. This continues
[direct-output merging](oriented-merge.md). The 52-card
construction is unchanged; the improvement concerns
large-n asymptotics and avoiding unused endpoint work.

## A three-way macro-step

Recall the legal endpoint recurrences, for a+b=n:

```text
U(n) <= P(a)+P(b)+n
P(n) <= P(a)+U(b)+n+a.
```

For the parked routine choose a about n/3, not n/2.
Implement its U(b) child by a balanced central merge.
If b=c+d, substitution gives

```text
P(n) <= P(a)+P(c)+P(d)+2n.
```

Here a,c,d differ by at most one. The cost 2n is exact
before cancellation: n+a for the side-output merge plus
b for its central child. Thus a parked macro-step has
three parked children and linear toll 2n.

The orientation changes are inherited from the proved
binary endpoint routines. No three-way physical transfer
is introduced. The first child is on the opposite side,
the next two are sorted through a central merge, and the
final merge builds the requested side. Protected bases
make these compositions legal even when different child
outputs share a physical side at different times.

## Asymptotic bound

For powers of three with singleton parked cost P(1)=1,
the unreduced recurrence is exactly

```text
P(n) <= 3P(n/3)+2n = 2n log3(n)+n.
```

For arbitrary n, balanced ternary levels have at most n
total cards per level and depth at most ceil(log3 n).
When n=2, use its four-move parked routine; its charge
is bounded by 2n. Consequently

```text
P(n) <= 2n ceil(log3 n)+2n.
```

Use a balanced central root. Its two parked children
then give

```text
U(n) <= 2n log3(n)+O(n)
     = (2/log2 3)n log2(n)+O(n).
```

The leading coefficient is approximately 1.26186,
improving 4/3. Exact finite endpoint routines through
64 cards change only the O(n) term. A conservative
version takes P(k)<=8k for every leaf k<=64; the stored
finite bounds verify this inequality. The output move
bound does not depend on a favorable cancellation.

### How the thirds arise

A useful trial form is U(n)=c n log2(n)+u n and
P(n)=c n log2(n)+p n. Write delta=p-u. The central
recurrence at split fraction x requires

```text
c H(x) >= 1+delta,
```

where H is binary entropy. Since H(x)<=1, this gives
delta<=c-1. The parked recurrence at first-child
fraction y requires

```text
c H(y) >= 1+y-(1-y)delta.
```

Together these require c[H(y)+1-y]>=2. The bracket is
maximized at y=1/3, with value log2(3); the central
choice is x=1/2. This motivates the ternary macro-step.
The calculation rules out a smaller c within this
constant-linear-term supersolution ansatz. It is not a
machine lower bound or a proof of optimality among all
recurrences; periodic O(n) terms fall outside the ansatz.

## Implementation

[ternary_merge.py](../ternary_merge.py) calls the unchanged
oriented routine for intervals through 64 cards. Above
that cutoff, a central request computes only its two
parked children. A parked request computes one parked
third and one central two-thirds child. It does not
compute both endpoint solutions merely to discard one.

Unlike the original bundled endpoint cache, this recursion
has one disjoint partition per requested state. A parked
macro-level produces three disjoint parked subproblems.
This avoids the overlapping extra work induced by asking
for both central and parked results at every interval.
Materialized words and sorting still give a conservative
O(n log² n) host-time bound.

The finite upper bounds use exactly the implemented
recursion: `ternary_bounds(n)` returns central and parked
bounds. Through 64 it agrees with the previous optimized
bounds, including U(52)=352. For larger sizes the parked
split is floor(n/3) and the central split floor(n/2).

The oriented helper now skips its parked root calculation
when only a central endpoint was requested. A root cannot
be a proper child, so no subsequently needed cached result
is lost. This changes neither the selected central word
nor any side-output request. Regression tests reproduce
all 100 frozen fast holdout words exactly.

The tests check exact word equality with the old routine
on selected small sizes, all three endpoint contracts,
and protected bases during every move of structured and
random recursive cases through 729 cards. The bound is
checked on each replayed output, and a coarse analytic
envelope is checked for all sizes through 10,000.

## Reproduction

```sh
python3 -m unittest test_ternary_merge -v
python3 tools/third_ternary.py --count 3 \
  --output build/third-ternary-scaling.json
```

The experiment records every target, counts, timings,
and SHA-256 of each replayed word. Timings are indicative
measurements on the shared research host. A hash identifies
a reproducible word; it is not by itself a legality proof.

## Completed scaling pilot

Three paired targets at each size use seed 2026100403+n.
All words are independently replayed. After the unused-
root optimization, the measured means are:

| n | Previous oriented | Ternary | Oriented time | Ternary time |
|---|---:|---:|---:|---:|
| 52 | 284.00 | 284.00 | 4.24 ms | 4.19 ms |
| 128 | 917.33 | 917.33 | 21.01 ms | 20.76 ms |
| 512 | 5054.00 | 4930.67 | 110.71 ms | 41.30 ms |
| 4096 | 56854.67 | 55114.00 | 1007.48 ms | 333.03 ms |

Times are bidirectional median planning times. The three
targets per size are a bounded scaling check, not a
population-performance estimate. The n=52 words agree
because its implementation is unchanged except for
discarding work that cannot affect its selected output.

[Before root optimization](../results/third-ternary-scaling.json)
and [after](../results/third-ternary-lazy-scaling.json)
records preserve both measurements. The CLI exposes
`--algorithm ternary`; the existing default is unchanged.
