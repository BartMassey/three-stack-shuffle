# Target-specific exact-search campaign

4 October 2026. This executes RESEARCH_PLAN section 4C
without changing the runtime solver. The results are now
integrated into [REPORT.md](../REPORT.md), section 6.4.

## Results

All six selected nonreversal targets at n=10,11,12 were
solved exactly. Their existing thorough-plan costs fell
by two to six moves. These are target-specific results,
not universal bounds on these deck sizes.

| n | ID | Old upper | Exact | Search nodes |
|---|---|---:|---:|---:|
| 10 | A | 36 | 32 | 106 |
| 10 | B | 34 | 30 | 31 |
| 11 | C | 38 | 32 | 75 |
| 11 | D | 36 | 34 | 1,639 |
| 12 | E | 42 | 36 | 2,312 |
| 12 | F | 44 | 38 | 2,370 |

Targets, top to bottom:

```text
A: 9,3,5,2,6,8,4,1,7,0
B: 9,5,2,4,1,6,8,3,0,7
C: 3,1,8,2,5,10,4,9,7,6,0
D: 10,4,0,7,2,3,6,1,9,8,5
E: 1,11,5,3,2,7,4,6,8,10,9,0
F: 11,8,1,5,7,3,10,9,4,0,6,2
```

The target selection was deterministic: at each size,
generate 100 shuffled permutations with Python RNG seed
20261004; rank them by thorough-plan cost minus structural
bound, then by thorough-plan cost and target order; select
the first two. Reversal was also included at each size,
with exact distances 36,40,44 already certified by the
structural theorem and construction. Reversal required
no search in the pilot.

The complete eight-card database took approximately
0.35 seconds per invocation. Additional search on these
six targets took less than 0.02 seconds each. Each had a
20-second search allowance and a 1 GiB address-space cap.
No target in this pilot exceeds the budget 4(n-1).

Artifacts:

- [C++ search](../tools/campaign_exact.cpp).
- [Python driver and replay](../tools/campaign_exact.py).
- [Initial pilot results][pilot].
- [Validation and interruption results][validation].
- [Larger structured targets][extended].

The larger target family lists all odd labels in
descending order, followed by all even labels in
descending order. It exposes a practical limit:

| n | Retained interval | Nodes | Search seconds |
|---|---|---:|---:|
| 14 | [48,50] | 10,735,616 | 45 |
| 16 | [56,58] | 9,986,048 | 45 |
| 18 | [64,66] | 10,178,560 | 45 |

Each run interrupted its initial structural threshold;
none completed an iteration or improved its lower bound.
Each filled the one-million-entry transposition cache.
The initial PDB-plus-remainder values were 40,44,48,
respectively, significantly below the structural seeds.
Thus this residual heuristic becomes too weak on the
nearly reversed family even though its endpoint interval
is only two moves wide.

Scaling was stopped on instruction from the root agent.
The n=18 run had already completed its timeout; the
active n=20 subprocess was externally interrupted.
There is no completed n=20 search result or claimed new
bound for it. Completed JSON entries were preserved.
Further time on this family needs a stronger arbitrary-
state bound, rather than merely a larger cache or timeout.

## Full-state pattern database

The PDB covers all states of eight cards, including
arbitrary distributions over A,D,B. Its

    8! * binomial(10,2) = 1,814,400

states use the existing permutation-rank/stack-cut
representation. Breadth-first search from identity on D
computes exact unit-transfer distances. All graph edges
are reversible. This is not an endpoint-only distance
table. For n<8, the analogous full n-card PDB is built.

The main problem is relabeled by target positions, so
its goal is identity on D. A pattern selects eight card
labels. Delete all other cards from a state and relabel
the selected cards in goal order; the resulting state
can be looked up directly in the PDB. Projection of any
physical path is legal and charges only selected cards.

All eight-card subsets are scored at the initial state.
They are ordered by decreasing initial heuristic value,
breaking ties by increasing numeric subset mask. The
first 32 are retained, or all subsets when there are
fewer. This fixed collection is used throughout search.
Choosing only some patterns weakens the bound but cannot
invalidate it. No sum of overlapping PDB distances is
used.

## A stronger admissible residual bound

Let C be the longest bottom suffix of the current D
stack agreeing with the final goal's bottom suffix.
Assign each card a mandatory residual charge:

- Zero if it belongs to C.
- Two if it is on D above C.
- One if it is currently on A or B.

Every card on a side stack must return to D, proving
the charge one. A D card above C must leave and return:
if it never leaves, neither can any card below it, so
that entire bottom segment remains unchanged forever
and must already be a goal suffix. This contradicts
its position above the longest such suffix.

For each pattern S, use

    PDB(project_S(state)) + sum(charge(card), card not in S).

The PDB term charges moves by cards in S. The remainder
charges disjoint cards, each by a proved minimum. Their
sum is therefore admissible. Take the maximum over all
selected patterns and the mandatory-charge total.

Every path to the goal has length congruent modulo two
to the current number of side-stack cards. Round the
bound upward to this parity. This also remains admissible.
The structural endpoint bound B(target) is an additional
seed for the initial search threshold; it is not used
without justification as an arbitrary-state heuristic.

## IDA* and exact transposition handling

IDA* searches successive even thresholds from the initial
lower bound up to, but excluding, the known upper bound.
The upper bound always comes with a replayed legal word.
Children are considered in increasing heuristic order.
Immediate inverse moves are omitted, since removing
such a pair preserves state and strictly shortens a plan.

Each iteration has its own transposition table, keyed
by the complete state **and the preceding move**. Including
the preceding move makes the permitted successor set
identical for equal keys despite inverse-move pruning.
Store the smallest encountered depth for each key.
A visit at greater or equal depth has no larger residual
budget, and its continuations are available from the
earlier visit. It may therefore be pruned. A lower-depth
visit replaces the depth and is expanded again.

When an equal key is still on the recursion path, the
later visit contains a removable cycle; pruning it also
preserves a shortest solution. When it is no longer on
the path, its search has completed unless an interruption
occurred. Interruption immediately unwinds the entire
iteration, so partially searched entries cannot support
a claim that the iteration was exhausted.

The table retains at most one million keys. After it
fills, lookups and improvements to retained keys continue,
but new keys are simply not cached. This loses speed,
not completeness. Hash collisions use exact key equality.

For n<=20, state keys use two 64-bit fields: five bits
per card, stack heights, and the preceding move. The
first field stores up to 12 cards; the second stores
the rest and metadata. This is injective at fixed n
and avoids factorial-rank overflow for larger decks.
Only the small PDB uses ranked integer state IDs.

## Validation and interruption accounting

Sixteen solved validation targets at n=5,8,9 match the
existing exact tables. These include four nine-card
instances requiring the initial lower threshold to be
fully exhausted before reaching the optimum. Every
returned word is independently replayed by the Python
three-stack simulator. Initial upper words deliberately
include a canceling move pair during validation, so the
search must find and return its own shorter witness.

The n=12 reversal is an explicit interruption test. Its
true optimum is 44. Disable the structural seed and give
the search a zero-second allowance, checked every 4,096
nodes. The initial PDB bound is 36; that threshold is
fully exhausted, but threshold 38 is interrupted.
The returned search interval is [38,46], where 46 comes
from a padded reversal witness. In particular it does
not incorrectly promote the lower bound to 40.

In general, fully exhausting threshold T proves the
distance is at least T+2. Interrupting threshold T proves
nothing new beyond earlier completed thresholds and the
initial admissible bounds. If the next threshold reaches
the known upper cost, equality follows without searching
that upper threshold. Timeout is always reported explicitly.

The deadline excludes PDB construction and pattern setup.
The wrapper separately imposes an external process timeout
of search allowance plus ten seconds. Deadline checks may
overshoot by at most one block of 4,096 visited nodes.
The completed threshold list, interruption flag, node
count, retained table size, and full upper witness are
recorded in each JSON result. These are reproducible
computational certificates, not SAT-style proof logs.

## Reproduction

```sh
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic \
  tools/campaign_exact.cpp -o /tmp/campaign_exact
python tools/campaign_exact.py --mode validate \
  --seconds 15 --patterns 32
python tools/campaign_exact.py --mode pilot \
  --seconds 20 --patterns 32 \
  --output results/campaign-exact-pilot.json
python tools/campaign_exact.py --mode extended \
  --seconds 45 --patterns 32 \
  --output results/campaign-exact-extended.json
```

A single C++ invocation takes target CSV, certified upper,
certified lower, seconds, pattern count, and cache capacity:

```sh
/tmp/campaign_exact \
  11,8,1,5,7,3,10,9,4,0,6,2 44 34 20 32 1000000
```

The raw C++ interface trusts its supplied lower and upper
bounds; the driver supplies a proved structural lower
bound and a replayed upper witness. Independent callers
must preserve that contract.

[pilot]: ../results/campaign-exact-pilot.json
[validation]: ../results/campaign-exact-validation.json
[extended]: ../results/campaign-exact-extended.json
