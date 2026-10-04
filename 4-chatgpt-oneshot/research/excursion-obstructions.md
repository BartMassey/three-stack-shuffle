# Small twice-set budget obstructions

The new finite obstruction catalog repairs the unique six-card
miss of the previous conditional relaxation. Its five-card
rules alone recover every exact distance through six cards.
The full five/six-card catalog improves 80 seven-card targets
beyond the old method, matching 5032 of 5040 exact distances.
These are proved instance bounds. They do not break the
`4m-4` ceiling or establish a new ensemble theorem.

On four deliberately selected stored 52-card targets, the
five-card rules add 2, 2, 2, and 4 moves to the previous
certificates. This establishes a useful bounded gate for
the new clauses, without expanding the seven-card catalog.

## Finite budget oracle

`tools/excursion_obstructions.py` reuses the physical graph
and individual-budget DFS of `campaign_structure.py`.
For target q and a selected set S, give cards of S budget
two and all remaining cards budget four. The oracle searches
all executions of length at most the sum of these budgets.
Any execution respecting the individual budgets necessarily
satisfies this total cap. Exhaustion therefore proves
infeasibility without assuming a particular move schedule.

The exact physical distances prune the DFS and independently
agree with every stored endpoint distance through six cards.
The catalog considers every S that is a union of two
increasing subsequences. A failed S permits skipping all
supersets because their budgets are smaller. Failure of a
smaller projected rule also permits skipping the full case.
Cases excluded by the two-increasing test are already
excluded by the twice-card lemma.

There are no budget failures through four cards. At five
cards there are 16 deletion-minimal rules: three with a
two-card trigger, ten with a three-card trigger, and three
with a four-card trigger. At six cards there are 17 further
deletion-minimal rules, all with three-card triggers.
The entire catalog took 8.08 seconds and about 3.34 million
memoized DFS visits. Its exact caps and visit counts are in
`results/excursion-catalog-n6.json`.

Here deletion-minimal means that no rule in the complete
smaller-card catalog already proves the same failure after
deleting cards. Trigger-minimality is also checked by
ascending subset size. Neither notion claims a unique
irreducible representation of all budget constraints.

## Projected group constraints

For any failed pair (q,S), let C contain the other cards
of that occurrence. Every full execution satisfies:

```text
if every card of S moves exactly twice,
at least one card of C moves at least six times.
```

Proof: delete all other cards from the full execution.
Deleting a card deletes its moves; every retained move
remains legal in the projected three-stack execution.
Individual move counts of retained cards are preserved.
If S stayed within two moves and C within four, the
projection would contradict the exhausted budget oracle.
Each card's move count is even because it starts and ends
on D, so exceeding four means at least six. Arbitrary
outside blockers are allowed by this deletion argument.

The three new two-card trigger rules are:

| Five-card target q | Trigger S | Group C |
|---|---|---|
| `(2,4,3,1,0)` | `{0,1}` | `{2,3,4}` |
| `(4,1,3,2,0)` | `{0,4}` | `{1,2,3}` |
| `(4,3,0,2,1)` | `{3,4}` | `{0,1,2}` |

An augmented BFS constraining only S to at most two moves
finds minimum total 18 in all three cases. The other cards
are unrestricted. The BFS is finite: a physical state and
the selected cards' counts completely determine a node.
Its first endpoint witness proves the minimum total, rather
than merely showing that a particular capped budget fails.

`results/excursion-conditional-costs.json` records all 33
minimum conditional totals, witnesses, and per-card counts.
Some minima equal the rejected total budget. There is no
contradiction: an unrestricted witness can move another
card twice, freeing two moves for a different card's sixth
move. Thus the group clauses contain information that the
scalar conditional minimum alone does not capture.

## Accounting for distinct extra excursions

Let T be the actual active cards moved exactly twice.
An active card outside T must move at least four times.
The common bottom suffix is deleted before this accounting;
the active-card mandatory-movement lemma is the same one
used by the existing structural bound.

Let F6 and F8 be the previously established forced-card
sets, with F8 contained in F6. These supply a lower bound
of `|F6|+|F8|` on extra excursions above four moves per
card outside T. Repeated occurrences are deduplicated.

Activate a group clause when its trigger is contained
in T. Remove cards of T from its group. An empty group
rules out T. A singleton group forces its one remaining
card into F6. Union all such singletons with F6 before
counting any additional groups.

Discard groups intersecting this augmented F6: an already
charged card can satisfy them. From the remaining groups,
choose any collection P of pairwise disjoint groups.
Every group needs a six-move card. Disjointness makes those
cards distinct, and none can be in F6. Therefore:

```text
d >= 4m - 2|T| + 2(|F6| + |F8| + |P|).
```

The implemented packing greedily visits groups by increasing
size and then integer bitmask. Maximality or optimality of
the packing is unnecessary for soundness. More elaborate
minimum hitting-set calculations could strengthen a given
T, but are not used for the 52-card certificates.

The enumerator visits feasible near-maximum T using the
existing two-chain suffix DP. An already attained packing
threshold permits pruning an entire branch. Its validity
persists under completion: selected cards can only shrink
each activated group, forced-card sets can only grow, and
the currently disjoint groups still require distinct
charged cards. This argument does not assume that the
greedy packing's numerical output is itself monotone when
recomputed after adding cards.

Only completed subset layers increase the certificate.
The maximum requested gain is `2*(deficit+1)`. A smaller
candidate T supplies the separate baseline cost for all
unexamined subset sizes.

## The remaining six-card miss

For q=`(2,5,0,3,4,1)`, both the old bound and B are 14;
the exact distance is 16. There are exactly two maximum
twice-card candidates, omitting either card 1 or card 5.

Project onto `{0,1,2,4,5}`. The relative target becomes
`(2,4,0,3,1)`. Its new triple rules give these full-card
clauses:

```text
T contains {0,1,4} => some card in {2,5} moves >=6;
T contains {2,4,5} => some card in {0,1} moves >=6.
```

If T omits 5, the first clause forces card 5 to six
moves. If T omits 1, the second forces card 1 to six
moves. Either maximum T therefore costs at least 16.
Any T of size at most four already has baseline 16.
An existing 16-move witness supplies the matching upper
bound. No genuinely six-card rule is needed for this case.

## Exhaustive validation and bounded pilot

Independent validation enumerates every candidate T and
computes an exact minimum hitting set for the active
complement groups. It checks that the pruned packing
certificate is no stronger than this exact relaxation,
and that the exact stored endpoint distance is never
violated. This covers all 5913 permutations through seven
cards and takes 3.71 seconds.

With all 33 rules, the bound is exact through six cards.
At seven cards it improves 80 targets beyond the old bound
and matches 5032 optima; the remaining total gap is 16.
With only the 16 five-card rules it improves 76 targets
beyond the old bound and matches 5028 optima. No seven-card
rules were mined. Result files retain the bound sums and
exact distance sums, not just success counts.

The 52-card pilot uses only five-card rules. Two ordinary
stored holdout targets tie their previous certificates.
Then four targets were selected because their previous
completed relaxation gained less than six moves; none
was an earlier timeout. The gain cap stayed at B+6.

| Holdout index | B | Old certificate | New certificate |
|---|---:|---:|---:|
| 26 | 170 | 174 | 176 |
| 30 | 166 | 170 | 172 |
| 51 | 170 | 174 | 176 |
| 78 | 174 | 176 | 180 |

These checks took 2.42, 2.26, 2.23, and 4.33 seconds.
This intentionally selected sample supports instance
gains, not a representative expected improvement. Results
are in `results/excursion-52-selected-pilot.json`; the two
ordinary cases are in `results/excursion-52-pilot.json`.

## Ceiling and stopping gate

All two-card triggers in the complete catalog through
six cards occur in decreasing order in their targets.
If an active target has any increasing pair, choosing
that pair as T activates no group rule and no old quartet
rule. The candidate therefore costs `4m-4` in this
relaxation. If there is no increasing pair, the target is
fully reversed and B itself is `4m-4`.

Consequently this expanded relaxation still cannot prove
any nonidentity target exceeds `4m-4`. It constrains small T
and gives better certificates, but does not resolve the
universal-budget question. The next relevant gate needs
constraints on increasing two-card candidates, or a
different representation that also charges four-move
cards. Mechanical expansion of this dictionary is not
justified merely by the present small-n gains.

## Reproduction

```sh
python3 tools/excursion_obstructions.py catalog \
  --through 6 --seconds 50 \
  --output results/excursion-catalog-n6.json
python3 tools/excursion_obstructions.py costs \
  --rules results/excursion-catalog-n6.json --seconds 50 \
  --output results/excursion-conditional-costs.json
python3 tools/excursion_obstructions.py validate \
  --rules results/excursion-catalog-n6.json --through 7 \
  --deficit 2 --independent --seconds 50 \
  --output results/excursion-validation-independent.json
python3 tools/excursion_obstructions.py pilot \
  --rules results/excursion-catalog-n6.json \
  --max-pattern 5 --indices 26,30,51,78 \
  --deficit 2 --seconds 15 \
  --output results/excursion-52-selected-pilot.json
python3 -m unittest test_excursion_obstructions.py
```

The three short integration tests rebuild the five-card
catalog, verify the conditional minimum 18 for all pair
rules, check disjoint and singleton accounting, and compare
all six-card bounds against independent hitting enumeration
and the exact distance table. They take about 0.4 seconds.
