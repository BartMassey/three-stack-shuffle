# Individual-card budget campaign

The bounded milestone is complete: a new exact
conditional-budget bound improves B by six moves on
all ten independently seeded 52-card targets tested,
and by twelve on the first target. These are instance
bounds, not a new uniform-ensemble theorem.

All four
smallest positive-gap targets, at n=5, admit both an
optimal plan with every card moved at most four times
and a plan keeping the unique maximum two-increasing
set at exactly two moves per card. These properties
cannot hold simultaneously. Thus neither proposed
mechanism, interpreted separately and without a total
budget, explains these first gaps.

For target `(2,1,4,3,0)`, the exact distance is 14 and
the structural bound is 12. The unique maximum set is
`{1,2,3,4}`. Counts `(6,2,2,2,2)` realize an optimum;
so do `(2,4,2,4,2)`. The other n=5 gap targets are
`(2,4,1,3,0)`, `(4,1,0,3,2)`, `(4,2,0,3,1)`.
Every maximum twice set is feasible with unlimited
other-card budgets. Every all-2/4 budget totaling B
is infeasible.

The same coexistence holds for all 64 positive-gap
n=6 targets with no common bottom suffix, and for the
first 30 n=7 targets ordered by descending gap and then
lexicographically. These include all fourteen targets
with gap four. Their maximum twice sets are tested
individually, not inferred from one stored optimum.

For `(2,1,4,3,6,5,0)`, the maximum set `{1,...,6}`
can remain twice with counts `(8,2,2,2,2,2,2)`.
An optimal all-at-most-four plan also exists. Its
distance is 20 and its structural bound is 16.

## Oracle and evidence

`tools/campaign_structure.py` explicitly constructs
the physical state graph using three top-first tuples.
It checks graph distances against the existing exact
table and replays every witness through `Machine`.

The bounded oracle memoizes physical state, remaining
individual budgets, and remaining total budget. Its
only pruning is the exact physical distance to the
target and the sum of unused budgets. It is exhaustive
within those budgets. The unlimited-other-card oracle
is a finite breadth-first search augmenting physical
state only with the selected cards' 0/1/2 move counts.
Its first witness minimizes total moves under that
restriction, and exhausting its states would certify
unrestricted infeasibility. No unrestricted
infeasibility was observed in these batches.

Results are in `results/campaign-structure-n5.json`,
`results/campaign-structure-n6.json`, and
`results/campaign-structure-n7.json`. They retain
every move and per-card count. Commands:

```sh
python3 tools/campaign_structure.py --n 5 --unlimited \
  --output results/campaign-structure-n5.json
python3 tools/campaign_structure.py --n 6 --unlimited \
  --output results/campaign-structure-n6.json
python3 tools/campaign_structure.py --n 7 --limit 30 \
  --unlimited --output results/campaign-structure-n7.json
```

## Sound projected conditional constraint

For an occurrence of any of the four n=5 patterns,
write T for its unique four-card maximum
two-increasing set, and c for its remaining card.
If all cards in T move exactly twice in the full
execution, card c must move at least six times.
Deletion preserves individual move counts, and the
projected target has distance 14 while four twice
cards contribute eight. This argument quantifies
over all executions and permits arbitrary blockers
outside the occurrence.

For patterns `(2,4,1,3,0)` and `(4,2,0,3,1)`, the
exceptional card must actually move at least eight
times. Unrestricted augmented-state BFS establishes
minimum total 16 with those four cards constrained
to two moves. A separate budget DFS exhaustively
rejects exceptional-card budget six and accepts
budget eight. The other two patterns have minimum
exceptional-card budget six. This distinction is not
implied by the scalar exact distance 14 alone.

## Global accounting without repeated charges

Let T be the actual active cards moved twice. The
twice-card lemma requires T to be a union of two
increasing subsequences. For every four-card subset
of T, find all occurrences of the four five-card
patterns for which these are the selected four cards.
Let F6(T) collect exceptional cards forced to at least
six moves, and F8(T) collect those forced to at least
eight. Then F8(T) is contained in F6(T). Neither set
intersects T: the corresponding five-card patterns
cannot themselves be covered by two increasing
subsequences. Therefore every execution satisfies

```text
d >= 4m - 2|T| + 2|F6(T)| + 2|F8(T)|.
```

Each six-move card contributes one extra excursion,
each eight-move card contributes two. Sets, implemented
as bit masks, prevent charging repeated occurrences
to the same excursion. More than eight moves are
permitted; ignoring those further moves is a valid
relaxation.

For I=I2(q), to prove B+2k it suffices to examine T
of sizes I,I-1,...,I-k+1. At size I-j, verify
`|F6(T)|+|F8(T)| >= k-j`. Smaller T already contribute
at least B+2k without conditional charges.

The implementation enumerates those subsets exactly.
It maintains the two sorted last values of increasing
chains. Including a card places it after the largest
last value smaller than the card. This greedy choice
produces coordinatewise smallest sorted last values
among feasible placements, so any future extension
possible after another assignment remains possible.
Consequently each feasible subset has one canonical
path, without omissions or duplicate assignments.
A suffix dynamic program gives the maximum possible
number of future selected cards for each state.

Branches are pruned only when the suffix cannot reach
the requested size, or their already forced extra
excursions reach the needed threshold. Forced sets
only grow, so the latter pruning proves all feasible
completions satisfy the threshold. All arithmetic in
this certification is integer arithmetic. The result
file is a summary of a reproducible exhaustive check,
not a separately serialized proof tree.

The reusable API is
`conditional_bound(target, deficit=2, seconds=55)` in
`tools.campaign_structure`. The target uses initial
ranks `0..n-1`. The returned object retains the full
target and its active prefix separately. A timeout
preserves the strongest already completed layer.

Independent validation checks all 5913 permutations
through n=7. A simpler implementation enumerates all
subsets, uses RSK to recognize two-increasing subsets,
and scans all five-card occurrences directly. Its
relaxation bound agrees with the pruned enumerator,
up to the requested improvement cap, on every target.
Every computed bound is at most the stored exact
distance. The bound is exact through n=5. It improves
B on 67 of the 68 positive-gap n=6 targets and 787
of the 869 positive-gap n=7 targets. Evidence:
`results/campaign-structure-validation.json`.

```sh
python3 tools/campaign_structure.py --check-through 7 \
  --deficit 2 --seconds 55 \
  --output results/campaign-structure-validation.json
```

## Bounded 52-card results

Ten targets use separate Python `Random` seeds
20261004 through 20261013. All ten certify B+6.
Their sample mean B is 165, and the improved sample
mean certified lower bound is 171. The run took 17.97
seconds. This small sample is not representative
evidence for a new exact uniform-ensemble expectation.

For seed 20261004, B is 166. The deeper check certifies
178 in 13.48 seconds, exhausting subset sizes 21
through 16. Its pruning counts are respectively
4, 101, 679, 5925, 23877, and 112585.

An earlier unpruned check independently examined all
132 size-21 subsets and all 3445 size-20 subsets.
Their minimum forced excesses were 22 and 16. It
timed out at size 19 after 55 seconds, while retaining
the valid partial certificate 170. Monotone pruning
was the useful subsequent expansion, not a longer
retry of the same search.

```sh
python3 tools/campaign_structure.py --n 52 \
  --enumerate-bound --deficit 2 --samples 10 \
  --seconds 55 \
  --output results/campaign-structure-52-batch.json
python3 tools/campaign_structure.py --n 52 \
  --enumerate-bound --deficit 5 --seconds 55 \
  --output results/campaign-structure-52-deeper.json
```

An exploratory binary MILP with just the six-move
implications found a relaxation assignment of cost
180 for this target but could certify only B=166
in 30 seconds. Its floating result is not used.
The ordinary continuous relaxation cannot improve
B when I2<=2m/3: the uniform choice t=I2/m and zero
excess satisfies the decreasing-triple and four-card
conditional inequalities. Exact subset structure
is essential for this formulation.

## Frozen 100-target lower-bound holdout

The frozen method was then run on 100 targets from
one `Random(2026100401)` instance, shuffling a fresh
`list(range(52))` each time. This differs from an
execution holdout using `random.sample`, even with
the same seed. The stored full target permutations
are the source of truth for any later paired upper
bounds.

The requested certificate was B+6, with three seconds
per target and a checkpoint after every target.
Results are in
`results/campaign-structure-52-paired.json`.

| Quantity | Result |
|---|---:|
| Sample mean B | 166.86 |
| Sample mean certified bound | 172.56 |
| Mean improvement | 5.70 |
| Targets improved by six | 87 |
| Targets improved by four | 11 |
| Targets improved by two | 2 |
| Improved bound range | 164..184 |
| Total elapsed seconds | 181.96 |

The two timeouts, indices 5 and 74, retain only their
completed layer certificates. Other smaller gains
are completed limits of this conditional relaxation.
The figures above are sample summaries of certified
instance bounds, not a proof of the uniform mean.

```sh
python3 tools/campaign_structure.py --n 52 \
  --paired-batch --samples 100 --seed 2026100401 \
  --deficit 2 --seconds 3 \
  --output results/campaign-structure-52-paired.json
```

No further search expansion was performed in this
track. The timing results above refer to the frozen
five-subset preprocessing implementation, before the
quartet optimization described next.

Root follow-up: quartet preprocessing is implemented.
For sorted values a<b<c<d, only relative quartet orders
(b,a,d,c), (b,d,a,c), and (c,a,d,b) can activate these
rules. Bitmasks select smaller values occurring after
the quartet or larger values occurring before it.
This yields exactly the same forced-card dictionary;
an independent five-subset reference agrees on all
targets through n=7 and one 52-card target. The separate
178 certificate was independently reproduced afterward.

Root ceiling observation: any two-card candidate T
triggers no four-card implication and contributes
4m-4. The current conditional relaxation therefore
cannot exceed 4m-4, even with unlimited enumeration.
This is a firm limit on using it to locate the first
failure of that universal budget. Rules constraining
small twice-moved sets would be needed.

Actual stored holdout targets now have paired upper
plans in `results/campaign-intervals-52.json`: mean lower
172.56, mean upper 317.92, mean width 145.36. All upper
words replay, and every interval is below the universal
410 upper bound. These remain instance/sample results.

## Next gate

The original five-move instance-improvement gate is
passed on all ten pilot targets and 87 of the 100
holdout targets. Next, independently review the
eight-move obstructions and integer enumerator,
then pair upper bounds on the stored holdout targets.
A useful further
stage would compress the proof into a portable
certificate or derive an ensemble count for targets
passing it. Do not claim 171 or 178 as uniform mean
bounds. Larger pattern dictionaries and a larger
requested improvement should follow measured
certificate cost, not unrestricted search expansion.

The smallest remaining failure of the four-pattern
dictionary is `(2,5,0,3,4,1)`: B and the conditional
relaxation both give 14, whereas its exact distance
is 16. This is the unique n=6 target for which the
conditional bound does not equal the exact optimum.
