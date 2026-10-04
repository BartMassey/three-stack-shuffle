# Adaptive natural merge candidate

Historical merge experiments, retained with their original
measurements. The current default uses exact leaves and
bounded split search; [parked leaves](campaign-execution.md)
provide the strongest proved 52-card construction.

Implementation: `merge_candidates.py`, function
`natural_merge(initial, target)`. Inputs are top-to-bottom
sequences of the same unique, hashable labels. Output move
codes name source then destination: AD, DA, DB, BD.

## Construction and proof

Map labels to their positions in the target. Find the
maximal increasing runs in the current D prefix. A correct
target suffix remains on D throughout each pass.

Distribute complete runs alternately to A and B. Each side
now exposes its last run in descending order. Process run
pairs from the last pair toward the first pair. Within each
pair, send the larger exposed rank to D. This builds the
merged run in ascending top-to-bottom order on D. For an
odd number of runs, restore the final unpaired A run first.

If a pass starts with r runs, the next deck has at most
ceil(r/2) increasing runs. Removing a correct final target
suffix cannot increase that run count. Each pass transfers
each active card at most twice. Therefore the direct
candidate uses at most 2n ceil(log2 r) operations, where r
is the initial ascending run count. Sorted decks use zero.
Since r <= n, this proves O(n log n) for all inputs.

Adjacent inverse operations are canceled as the sequence
is generated. They undo the same top-card transfer, so
removing them preserves all subsequent stack states.
This also cancels the needless outward-and-back movement
of the odd final run during a pass.

A second candidate first reverses the whole deck, then
applies the same direct algorithm. Reversal uses DA n
times, then AD followed by DB n times, then BD n times.
The two boundary inverse pairs cancel, giving 4n-4
operations for n >= 2. Choosing the shorter candidate
preserves the direct candidate's proved bound. A reversed
52-card target requires 204 operations with this candidate.

## Verification

A separate simulator represented each stack bottom-to-top.
Every returned operation was checked for a nonempty source.
The final state was checked for empty A/B and exact target D.
Every result was also checked against the proved run bound.

All permutations through n=8 were checked, including n=0.
The complete final-candidate run took 1.783 seconds here.

| n | Permutations | Observed maximum | Exact mean |
|---|---:|---:|---:|
| 0 | 1 | 0 | 0 |
| 1 | 1 | 0 | 0 |
| 2 | 2 | 4 | 2 |
| 3 | 6 | 8 | 5 |
| 4 | 24 | 14 | 9.416666667 |
| 5 | 120 | 20 | 14.366666667 |
| 6 | 720 | 32 | 19.877777778 |
| 7 | 5040 | 38 | 26.274603175 |
| 8 | 40320 | 44 | 33.278323413 |

Random sampling used Python Random seed 27391, continuing
the generator across the rows below. These maxima are
observations, not worst-case guarantees.

| n | Samples | Observed maximum | Mean | Seconds |
|---|---:|---:|---:|---:|
| 52 | 10000 | 610 | 493.3506 | 4.438 |
| 1000 | 100 | 19946 | 18111.3 | 1.425 |
| 10000 | 10 | 252780 | 252384.4 | 2.078 |

For n=52 the simple general guarantee is 624 operations.
The common five-pass case explains much of the observed
cost; inputs with more than 32 initial runs may need six
passes. Exhaustive small-n statistics do not establish
the distribution or maximum for larger n.

Before adding the reversal alternative, n=52 statistics
were identical and runtime was 2.21 seconds for 10000
samples. Thus reversal primarily improves structured
inputs; evaluating it approximately doubles random-case
CPU work without reducing the measured random mean.
The implementation skips reversal when its 4n-4 starting
cost already exceeds or equals the direct result.

## Patience distribution

`patience_merge` tests an alternative to alternating whole
contiguous runs. Each next D card extends an increasing
side subsequence if possible. Among eligible sides, use
the larger top rank. If neither side can extend, restart
the side with the larger top rank. Each side's runs are
recorded independently and merged from their exposed ends.

If a pass does not reduce the natural D run count, finish
with the direct natural algorithm. The same fallback is
used after ceil(log2 n) patience passes. Consequently even
the internal candidate has O(n log n) operation count.
The public function chooses its shorter candidate versus
the direct algorithm, preserving the direct run bound.

Restarting the lower side top was also tested. It was much
worse at large n and is not included in the public plan.
The internal parameter remains available for experiments.

The following random rows used the same generator seed
and continuation convention as the earlier table. Every
plan was simulated independently to verify legality and
the final exact state.

| n | Samples | Higher restart mean/max | Selected mean/max |
|---|---:|---:|---:|
| 52 | 10000 | 439.7682 / 512 | 439.5024 / 510 |
| 1000 | 100 | 16780.74 / 17904 | 16780.74 / 17904 |
| 10000 | 10 | 232656.4 / 233206 | 232656.4 / 233206 |

For n52, higher restart beat the direct plan in 9632 of
10000 cases. Lower restart averaged 708.2718 operations,
with maximum 926; it beat the direct plan zero times.
The batch calculating and simulating all three candidates
took 10.288 seconds, then 3.847 seconds for n1000, and
5.664 seconds for n10000. Selected maxima are empirical.

The final public patience implementation was exhaustively
verified through n8 in 2.364 seconds:

| n | Observed maximum | Exact mean |
|---|---:|---:|
| 2 | 4 | 2 |
| 3 | 10 | 5.333333333 |
| 4 | 16 | 9.083333333 |
| 5 | 26 | 13.183333333 |
| 6 | 32 | 17.808333333 |
| 7 | 40 | 23.018253968 |
| 8 | 48 | 28.742609127 |

## Dynamic-programming partition experiment

`optimal_partition_merge` is an experimental alternative
restricted to n<=64. It computes the globally minimum
total count of increasing runs on the two sides for each
distribution pass. State contains the side just used and
the other side's previous input index. The just-used side
necessarily ends with the most recent input card, so each
input position has O(n) states and the full history needs
O(n²) time and memory. Backtracking recovers assignments.
Ties heuristically prefer balanced side run counts.

The subsequent merging and stopping/fallback rules match
the patience experiment. The public function selects the
shorter DP or direct sequence and retains the direct bound.
For n>64 it returns the direct sequence.

For every permutation through n6, the DP run count was
compared against all 2^n side assignments and was minimum.
Every complete resulting plan was also simulated.
All complete plans for n7 and n8 were also exhaustively
simulated in a combined 5.394 seconds. DP n7 maximum/mean
was 40 / 22.832142857; n8 was 48 / 28.520089286.

For 1000 n52 permutations, freshly seeded with 27391:

| Candidate | Mean | Observed maximum |
|---|---:|---:|
| Greedy higher-top patience | 440.966 | 510 |
| Optimal run partition | 441.046 | 510 |
| Shorter of those two | 433.040 | 500 |

DP beat greedy in 429 of 1000 cases. This batch took
9.564 seconds. Minimizing immediate side run counts is
not the same as minimizing eventual operations: although
DP has no better standalone mean here, its different
choices add useful portfolio diversity. The 500 maximum
is an observed sample maximum, not a proof for n52.
