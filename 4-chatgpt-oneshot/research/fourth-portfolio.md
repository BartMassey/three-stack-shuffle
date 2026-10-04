# Fourth campaign: bounded endpoint portfolios

4 October 2026. This research candidate modestly reduces
sampled move counts at a substantial host-time cost.
It is not registered in the solver and changes no default.

## Fixed finite pilot

The pilot used all 20 fresh uniformly shuffled 52-card
targets from Python `random.Random(2026100405)`, in order.
Widths two and four, six-operation boundary signatures,
the existing split window of four, and the existing
certified splits were fixed before evaluating this sample.
There was no further parameter search or follow-up run.

Each width has a six-second cooperative planning cutoff,
including its mandatory baseline. The pilot stops adding
search when its 280-second total search allowance expires.
Every intended target remains in the report, with timeout
and completed-direction metadata and a baseline fallback.
These checks are cooperative, not hard real-time limits.

The complete pilot took 26.6 seconds. No search timed out;
both directions completed for every width and target.
All 60 returned complete words were independently replayed.

| Controller | Mean | Maximum | Wins | Median seconds |
|---|---:|---:|---:|---:|
| oriented_window | 274.8 | 284 | baseline | 0.102 |
| width two | 274.3 | 284 | 3/20 | 0.475 |
| width four | 273.0 | 284 | 11/20 | 0.754 |

Times include each portfolio's own baseline computation.
Baseline, width two, then width four ran in that order for
each target; caches and host scheduling can affect timings.
This is a paired development pilot, not an independent
population or optimum estimate.

The mean improvements are 0.5 and 1.8 moves, while median
host time grows approximately 4.6 and 7.4 times. Neither
width improves the sampled maximum. This finite gate
does not justify promoting the candidate into the default
controller or extending the campaign. The library remains
available solely as a reproducible research experiment.

See [all pilot words and timings](
../results/fourth-portfolio-pilot20.json).

## Selection rule and scope

Each memoized interval and orientation has two contracts:
central output in the requested order, or parked output
in reverse requested order. Candidates use the existing
central and parked merge constructions. The exact leaves
add reflected central words and the existing alternate
parked table, plus parking a central solution.

For each endpoint pool, group words by their first six
and last six operations. Retain the shortest word in each
group, breaking ties lexicographically. Sort the resulting
representatives by length and then lexicographically;
retain the first two or four. The width is bounded per
endpoint per memoized interval/orientation state.

The shortest available representative always survives.
A longer representative can survive if it has a distinct
boundary and the shorter groups do not fill the width.
Parents evaluate Cartesian products of retained children
and cancel inverse operations across the joins. Central
reflection and the parked merge's reflected central child
are explicitly available. Search is limited to n <= 64.

Six-operation signatures are heuristic summaries. They
do not characterize all possible future cancellation:
an entire child or more than six operations may cancel.
Thus replacing a word by the shortest word with the same
signature is not a proved contextual dominance rule.
Likewise a wider beam does not provably dominate a narrower
one after recursive pruning. No exact global optimum or
local-beam dominance claim follows from this experiment.

The public wrapper first computes the complete existing
`oriented_window` plan, including its special cases and
other registered controller behavior. It compares that
word with each completed forward and backward candidate.
Timeouts retain the best complete word already available;
sizes above 64 use the full baseline without beam search.
This full-plan fallback, rather than local beam retention,
proves pointwise no regression against `oriented_window`
and preserves its universal 352-move bound at n = 52.

## A longer child wins: explicit witness

Use initial top-first order

```text
5 2 4 3 | 8 6 7 1 0
```

with target `0 1 2 3 4 5 6 7 8` on D. The split is
four plus five. The first child parks onto A using

```text
L = DB DA DB DA BD DA BD DA
```

The right child, initially using its A endpoint, has
the following shortest and longer retained candidates:

```text
R13 = DB DB DB DB DA BD DA BD BD DA DA BD DA
R15 = DA DB DB DB DB AD BD DA BD DA BD BD DA DA DA
```

Reflect the right child so it parks onto B, then append
the common central merge:

```text
M = BD BD BD AD AD AD AD BD BD
```

For `L + swap(R13) + M`, the raw length is 30. One
inverse pair cancels, leaving this 28-operation parent:

```text
DB DA DB DA BD DA BD DA DA DA DA DA DB AD
DB AD AD DB DB AD BD BD AD AD AD AD BD BD
```

For `L + swap(R15) + M`, the raw length is 32. Three
inverse pairs cancel, leaving this 26-operation parent:

```text
DB DA DB DA BD DA BD DA DB DA DA DA DA BD
AD DB AD DB AD AD AD AD AD AD BD BD
```

The longer right child costs two additional operations,
but exposes four additional cancelled operations at the
parent merge. Among these retained children, constraining
both children to minimum length gives 28; allowing the
15-operation child gives 26. This is a local witness,
not a claim that 26 is the globally optimal parent plan.

[The exact witness artifact](
../results/fourth-portfolio-witness.json) includes all
child words, reflected words, raw parent words, reduced
parent words, lengths, and cancellation counts. Its
generator independently simulates every child and both
raw/reduced parents with distinct guards under all three
stacks, rejecting any intermediate guard movement.

## Legality and verification

Every retained candidate is assembled from the existing
legal endpoint contracts. A child legal above empty
bases never pops below those bases, so the same word
remains legal over arbitrary protected bases. Reflection
preserves this property. The merge lengths bound the
number of consumed cards, and inverse cancellation
preserves the subsequent machine state. Portfolio
pruning selects whole legal words and changes none of
these invariants.

Five unit tests cover both widths and every permutation
through five cards at all three endpoints, matching the
exact endpoint distance for the shortest leaf candidate.
They replay every returned candidate with guards checked
before every operation. Recursive guarded tests include
9, 13, 26, 52, and 64 cards; both widths are checked at
9 and 13, and width two at the larger sizes. Additional
tests cover the longer-child witness, complete-baseline
fallback, zero-budget timeout, the above-64 bypass, input
validation, and successful-wrapper no-regression cases.

```sh
python3 -m unittest test_endpoint_portfolio -v
python3 tools/fourth_portfolio.py \
  --output results/fourth-portfolio-replay20.json
python3 tools/fourth_portfolio.py --witness \
  --output results/fourth-portfolio-replay-witness.json
```

The runner refuses to overwrite an existing artifact.
The published pilot was run once; the reproduction paths
above deliberately leave its original results intact.
