# Next execution and certificate experiments

Archived pre-campaign proposal. Its terminal-state
experiment is now complete: [parked leaves](campaign-execution.md)
prove 410 instead of 444. [Exact search](campaign-exact.md)
solves six selected n=10–12 targets. The baseline claims
and proposed experiments below are retained as history;
[current priorities](campaign-summary.md) supersede them.

Planning note, 4 October 2026. No algorithm changes or
new benchmark runs were performed for this note.

## Priority and the current evidence

The next main research effort should improve lower bounds.
Execution work earns priority when it also closes exact
small-instance gaps or cheaply improves the usable upper
bound. Host speed and physical transfers are different
objectives; report them separately.

Concrete implementation limitations:

- `hybrid_dp` stores one word per interval. Its all-splits
  search is not an exact optimization over merge trees.
  A locally longer child can cancel more with its parent.
- `hybrid_window` has the same limitation. Its memoized
  interval state contains no boundary context.
- `exact_solver.optimal` selects one shortest D-to-D word.
  The generator chooses the first available shortest-path
  predecessor, without optimizing terminal runs or other
  compositional boundary behavior.
- Every `_combine` first finishes a child on D and then
  parks it. The physically useful endpoint is often a
  reversed sorted segment already on a specified side.
- `_join` can cancel through complete intervening parts.
  A last-move or final-run signature alone does not capture
  all of its behavior.
- `thorough` repeats balanced planning inside both full DPs
  after `recommended` has already evaluated it. Window
  planning also recomputes sorted intervals, reflections,
  merge words, and full reduced words for candidates.
- The fixed n<=64 cutoff gives no explicit wall-clock or
  memory bound; work depends strongly on interval count,
  split count, word lengths, and retained frontier size.
- The earlier shortcut trial saved zero moves on natural
  merge plans and spent about 0.25 seconds per target.
  Repeating that unchanged experiment is low priority.

The current 444-move bound is a guarantee inherited by
choosing the balanced baseline. Sample maxima near 350
and the 50-case thorough mean 310.68 are empirical results.
They neither bound optimal costs nor certify worst cases.

## First experiment: terminal states instead of detours

Generate exact small-block distances and plans to these
three endpoints, with active cards normalized by target:

1. Sorted on D, both sides empty.
2. Reversed sorted on A, D and B empty.
3. Reversed sorted on B, D and A empty.

The second endpoint is exactly what the first child needs
before sorting the second; the third precedes the merge.
The existing protected-base lemma applies because every
move in the isolated plan uses an active source card.
The bases can differ between stacks without invalidating
that plan. All endpoint contracts must specify active
card order, active stack sizes, and restored bases.

Use BFS at n<=8 first. A direct shortest parked plan is
no longer than the current shortest-D-plan-plus-parking
word, even after legal inverse cancellation. This is a
local endpoint guarantee. Choosing one parked shortest
word can still sacrifice cancellations against preceding
or following parent context; retain the old candidate
and baseline when testing an end-to-end planner.

Collect parked cost histograms, final and initial runs,
the number of useful ties, and gains over the current
composition. Recompute effective leaf worst-case constants
from the new tables before proposing a tighter universal
bound. A lower sample mean alone cannot tighten 444.

Pilot budget: n<=7, 30 minutes wall time, 512 MiB RSS.
Extend to n=8 only after replay and tiny independent BFS
agree. Cap that extension at 1 hour and 1 GiB. The current
n=8 full graph has 1,814,400 states; its existing dense
distance and queue arrays occupy about 10.9 MB, so these
budgets allow substantial validation overhead.

## Second experiment: boundary-aware frontiers

Separate two proposed variants and label claims honestly.

An exact reference frontier retains distinct full reduced
words for every interval and endpoint contract, including
reflection. It enumerates all selected splits and all
child pairs, then joins and reduces the full word.
Identical full words can be deduplicated. Equal endpoints
alone are not a safe dominance criterion.

This reference can optimize only the explicitly defined
finite grammar and leaf library. It cannot certify global
3SM optimality. Including all shortest leaf words still
omits longer leaves that might compose better. Expanding
leaf lengths requires an explicit finite bound or a proof
that omitted leaves cannot improve any allowed context.

Start at n<=10 with leaf sizes <=5 and compare the existing
one-word DP to this exhaustive grammar reference. Search
for and save an actual interval/parent counterexample in
which the selected shorter child loses globally. This
turns a known structural concern into a measured loss.

A practical beam frontier can then retain, for example,
8, 32, or 128 words per endpoint and interval, ranking
length plus observed boundary benefits. Such caps make
the method heuristic. Compare capped frontiers against
the exact grammar reference before trying n=52.

Possible signature: length, exact prefix and suffix of
K moves, endpoint, and reflection class. Identical finite
prefix/suffix signatures do not generally permit exact
dominance: cancellation can consume the visible portion
and expose a hidden middle. A validity proof must bound
the total left and right cancellation available in every
permitted context, and handle overlapping visible ends
or completely consumed words. Otherwise signatures are
beam-selection features, not certificates.

For one known following parking block of k equal moves,
the terminal run and length determine that immediate
boundary saving. They do not determine subsequent joins.
Full-word equality is a safe initial deduplication rule;
any stronger dominance rule needs a contextual proof.

Pilot budget: 1 hour, 1 GiB, at most 100,000 retained words
overall and a small n<=10 target set. Exceeding a cap ends
the exhaustive claim for that instance; log it as unknown.
Do not silently prune and report the result as exact.

## Target-specific certificates at n=10 through 16

Avoid full-state BFS beyond a deliberately budgeted n=10
run. The current generator supports only n<=10, 32-bit
state IDs, and <=8 compact plan output. Full n=10 BFS has
239,500,800 states and requires about 1.44 GB just for
its queue and distance arrays. n=11 would need about
18.7 GB with that layout; growth soon exceeds its IDs.

For target-specific searches, keep a certified interval
LB<=d<=UB, a legal replayable upper-bound word, node and
time budgets, and a precise termination status. A timeout
does not certify optimality. Passing all depth limits
below UB does, provided every search and pruning rule is
complete and sound.

Start with bidirectional BFS for small/easy targets as a
reference. Meet on the complete three-stack state, not
merely D order or stack heights. Reflection exchanges A
and B while fixing endpoints; quotienting requires exact
canonicalization and path reconstruction with orientation.
Pruning immediate inverse moves is sound, but any pruning
that depends on the preceding move must be compatible
with visited-state handling and search completeness.

Then test IDA* and memory-bounded A* with deletion pattern
databases. For a selected subset of cards, delete all
others from each stack. A real move projects to one legal
abstract move or a stutter. Therefore the exact abstract
distance is an admissible, consistent lower bound.

Take the maximum of several such bounds. Do not sum
overlapping bounds without a cost-partition proof. Even
disjoint subsets require a justified per-move partition.
The safe initial design uses their maximum plus the
maximum with a+b, the minimum returns needed when cards
remain on side stacks. Round admissible remaining bounds
up to the necessary parity, (a+b) mod 2.

Existing `optimal` tables are insufficient for this PDB:
they contain only D-to-D plans, whereas search states
project to arbitrary three-stack states. Retain complete
distance arrays from BFS to the abstract sorted endpoint.
An n=8 uint16 distance array needs about 3.6 MB; n=9 needs
about 40 MB. Reuse arrays across targets by relabeling
each pattern according to its target order.

Select patterns deterministically using target adjacency,
spread across target ranks, and difficult projected
orders; compare against fixed/random patterns on holdout
cases. Do not infer n=52 distributional bounds from a
handful of exact n=10..16 targets.

SAT is an independent certificate route for hard targets.
Encode all stack cells and heights, one legal move each
time step, top-card transfer, untouched cells, distinct
card occupancy, and the exact final endpoint. Require
exact horizon and test every smaller compatible horizon,
or pad with explicit no-ops to encode at-most horizon.
An arbitrary forced first move must be justified by side
reflection. Do not add a forced last move without checking
that the chosen symmetry constraints remain compatible.

Prefer a SAT solver with independently checkable UNSAT
proofs when an exported certificate matters. CP-SAT is
useful as an independent exact model and incumbent source,
but timeout/UNKNOWN and lack of a portable proof must be
reported. Solver optimality is not itself a checked proof
artifact. Replay every SAT/CP-SAT witness independently.

Stages: validate through n<=8 against all saved distances
where feasible; then 20 prespecified targets at n=10;
then n=12 and n=16 only if certified-node throughput and
bound gaps justify expansion. Include reversal and the
non-reversal n=9 maximum as regression cases.

Initial cap: 60 seconds and 1 GiB per target, 30 minutes
total per method. Escalation cap: 10 minutes and 4 GiB
per selected hard target, 2 hours total. No prediction of
success at n=16 is implied by these resource budgets.

## Host improvements and evaluation

First profile 100 fixed targets to locate time in joins,
reflection, sorting, table ranking, and repeated baselines.
Reuse per-interval sorted values, reflected words and
merge words. Avoid generating repeated balanced baselines
inside an orchestration call. Try byte-coded operations
or persistent word fragments only if profiling identifies
allocation as material. A semantics-preserving speedup
must return the identical word or an equivalent verified
word of the same move count on a fixed corpus.

Introduce explicit planning budgets for experimental
frontiers and exact searches. The existing balanced plan
is an immediate valid incumbent with the proved bound.
Interrupting after a completed valid candidate preserves
correctness. Report budget overshoot and distinguish a
practical cap from hard real-time execution.

Use a fixed tuning corpus, then a separate precommitted
holdout seed and target list. Existing seeds and samples
are tuning data after repeated comparisons. Compare the
same targets with paired move differences and confidence
intervals; do not compare the 50-target thorough mean
directly against the 1,000-target recommended mean.

Record cold and warm timing, median/p95/max planning time,
RSS, mean/p95/max physical moves, win rate, and certificate
rate where relevant. Timing in `benchmark.py` excludes
simulation, while the older hybrid variant note includes
simulation; keep those scopes explicit.

Portfolio additions can include packed/aligned trees,
terminal-state leaves, and capped boundary frontiers.
Retain every original baseline candidate. Start with a
100-case tuning pilot; promote only after useful paired
move savings at an agreed planning budget. Run 1,000
holdout n=52 targets only after selecting the method.
An optional acceptance target is at least two moves of
mean improvement within the current roughly 53 ms budget;
choose this threshold before examining holdout outcomes.

Stress-test maximum cost using rotations, alternating
ranks, bit reversal, reversed blocks, long increasing
runs with difficult boundaries, and saved hardest samples.
Use swap/insertion hill climbing to maximize the returned
planner cost; save every incumbent target and replay.
Keep this adversarial corpus separate from uniform mean
estimation. An adversarial observed maximum is still not
a worst-case proof. Preserve and independently check the
444 bound through the selected baseline, and recompute
finite leaf constants if tables change.

Deliverables should include corpus identifiers, resource
caps, legality replays, proofs for any dominance/pruning
rules, and precise unresolved gaps. Lower-bound work
retains priority unless these bounded execution pilots
produce a clear certificate or practical improvement.
