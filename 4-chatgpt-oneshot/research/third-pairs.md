# Targeted increasing-pair gate

No increasing-pair obstruction was found. The three completed
seven-card batches contain 409 pair checks, representing 371
distinct target/pair cases on 94 targets. Every case has a
stored legal execution moving each selected card exactly
twice and every other card at most four times. These are
upper witnesses for the conditional budgets, not new lower
bounds or an arbitrary-size construction.

## Selection and completed coverage

All targets have no common identity bottom suffix. Pairs
are selected by increasing value and must occur in that
same increasing order in the target. Every such pair in
each selected target was checked.

| Selection | Targets | Pair checks | Seconds |
|---|---:|---:|---:|
| Largest exact-minus-B gap | 14 | 132 | 2.03 |
| At most three increasing pairs | 76 | 193 | 8.07 |
| Remaining group-catalog misses | 8 | 84 | 2.24 |

The hard batch orders targets by descending exact-minus-B
gap, then descending exact distance, then lexicographically.
Its fourteen targets are exactly the targets with gap four.
The miss batch contains every target whose exact distance
exceeds the complete five/six-card group relaxation.
Four of those targets also occur in the hard batch.

The structured batch exhausts the specified class at n=7.
There are one, six, twenty, and forty-nine active targets
with respectively zero, one, two, and three increasing
pairs. The reversal has no increasing pair and contributes
no pair check. Thus every selected pair in every member of
this finite class admits the requested conditional budget.
This is exhaustive class coverage at seven cards, rather
than a lemma covering larger sizes.

## Finite oracle and validation

`tools/third_pairs.py` builds the existing physical graph
once per batch. At n=7 the graph has 181,440 states. For
each target it computes one exact distance table, checks
the initial distance against the stored exact table, and
reuses that distance table across the target's pairs.

The depth-first oracle stores remaining card budgets in
three bits per card. Memo keys consist of physical state
and all remaining budgets. Remaining total cost is the sum
of those budgets, so no extra cost coordinate is required.
Every recursive move decreases this sum. A goal is accepted
with unused budgets allowed. Exact physical distance is the
only feasibility pruning rule. Search exhausts every legal
budgeted continuation before recording a failed memo key.
Consequently a completed rejected root proves budget
infeasibility, independently of any proposed schedule.

Each oracle has a 300,000 failed-state memo limit, and each
batch has a fifty-second deadline. Either interruption
raises an exception and records `unknown`, never
`infeasible`. No interruption occurred. The three batches
together visited 178,643 recursive nodes; the largest
single memo contained 2,850 failed states. Shell invocations
also imposed a one-GiB address-space limit and a
fifty-five-second process limit.

Independent oracle validation checks all 1,200 target/pair
combinations at n=5 against the earlier tuple-budget DFS.
It recovers exactly the three known failures:

| Target | Twice cards |
|---|---|
| `(2,4,3,1,0)` | `{0,1}` |
| `(4,1,3,2,0)` | `{0,4}` |
| `(4,3,0,2,1)` | `{3,4}` |

All three pairs are decreasing in their targets. Every
accepted test witness is replayed through the physical
simulator. A separate artifact test replays all 409 saved
seven-card words, checks their per-card counts and final
targets, and verifies complete increasing-pair coverage
within each stored target. Tests also verify that deadline
and memo-limit interruptions raise rather than reject.

## Interpretation and stopping point

The completed catalog through six cards contains no
increasing-pair trigger, and this targeted seven-card gate
finds none in the hardest gaps, all remaining relaxation
misses, or the specified near-reversal class. A new
seven-card increasing-pair obstruction would therefore
have to occur outside this coverage. Most seven-card
target/pair combinations remain unchecked.

There is no infeasible increasing pair to minimize by
deletion, no additional group clause, and no constructive
lemma extending these finite witnesses to all sizes. This
does not imply a universal all-at-most-four budget. The
separate counting obstruction already rules out a universal
`4n-4` total budget at sufficiently large n. The targeted
gate is therefore complete as a negative finite search,
with no basis here for enlarging the obstruction dictionary
or repeating an unrestricted seven-card mining campaign.

## Reproduction

```sh
python3 -m unittest test_third_pairs.py
timeout 55s bash -c 'ulimit -v 1048576;
python3 tools/third_pairs.py --mode hard --limit 14 \
--seconds 50 --output results/third-pairs-hard-n7.json'
timeout 55s bash -c 'ulimit -v 1048576;
python3 tools/third_pairs.py --mode structured --limit 5040 \
--seconds 50 --output results/third-pairs-structured-n7.json'
timeout 55s bash -c 'ulimit -v 1048576;
python3 tools/third_pairs.py --mode miss --limit 14 \
--seconds 50 --output results/third-pairs-miss-n7.json'
```
