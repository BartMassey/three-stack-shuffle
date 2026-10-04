# Further research on three-stack permutation routing

*Research plan, 4 October 2026*

Execution status: the first bounded campaign is complete.
See [results and stopping gates](research/campaign-summary.md).
The original baseline and proposed budgets below are
retained as a record; current bounds and measurements
are in [REPORT.md](REPORT.md) and [STATUS.md](STATUS.md).
The [second campaign](research/second-campaign-summary.md)
subsequently proved a 352 upper bound, improved residual
certificates and the XP exponent, and established the
next narrower stopping gates. The baseline below remains
historical rather than being silently revised.
The [third campaign](research/third-campaign-summary.md)
has now executed those narrower gates. It improves the
asymptotic construction to $2n\log_3 n+O(n)$, proves
residual consistency and stronger disjoint cost bounds,
and finds a severe limit of the elementary ensemble LP.
Its 17-row rational certificate matches a previous
176-move instance bound. The deterministic mean theorem
and tested n=18 interval remain unchanged. The summary's
section 5 gives the next bounded experiments.

### Execution outcomes

| Workstream | Completed result |
|---|---|
| Structural bounds | Conditional six/eight-move rules |
| Lower holdout | Mean certificate 166.86 to 172.56 |
| Ensemble bounds | Exact mean theorem unchanged |
| Execution | Parked leaves; universal bound 410 |
| Execution holdout | Mean 322.896 to 318.874 |
| Exact search | Six selected n=10–12 targets solved |
| Complexity | XP in excess excursions; hardness open |

The two holdouts contain different targets. Paired upper
plans on the lower-bound holdout give mean interval
[172.56,317.92]. These empirical means do not replace
the exact uniform-mean lower theorem, 166.87917.
The next priorities and reasons to stop unsuccessful
expansions are in the campaign summary, section 6.

## 1. Objective and pre-campaign position

The first priority is to narrow the gap between lower
bounds and constructions, especially at n=52. This does
not mean trying to prove that the present algorithm is
near optimal. A much better algorithm would be an equally
good explanation of the gap.

The model is exactly that of [REPORT.md](REPORT.md):
three unbounded stacks A–D–B; four adjacent top-card
transfers; every transfer costs one; all cards start and
finish on D. The host knows the target permutation.
No free loading, separate output stream, extra stack,
or restriction on intermediate configurations is allowed.

Write d(pi) for optimal moves, M_n for the maximum over
targets, and mu_n for the uniform-target mean.

| Quantity at n=52 | Pre-campaign evidence |
|---|---|
| Optimal uniform mean | mu_52 >= 166.87917, proved |
| Optimal maximum | 204 <= M_52 <= 444, proved |
| Recommended controller | mean 322.838 on 1,000 targets |
| Recommended sample maximum | 350; not a universal bound |
| Slower controller | mean 310.68 on 50 paired targets |
| Reversal | exactly 204, proved optimal |

The mean and maximum have order Theta(n log n), but their
constants and finite-n behavior are unresolved. Exact
distances through n=9 do not justify extrapolation to 52.
The first n with M_n > 4(n−1) lies between 10 and 212;
the typical-target crossover is a different question.

Allocate approximately 60% of research effort to lower
bounds and certificates, 25% to constructions and host
execution, and 15% to controller complexity. Rebalance
after concrete results. The stages below are a plan,
not a record of experiments already performed.

## 2. Immediate progress: account for individual cards

### 2.1. A proved structural bound

Deleting cards from a legal execution preserves legality
of the surviving moves. This elementary projection lemma
supports both structural bounds and exact abstractions.

Remove the longest common bottom suffix, of length s,
from initial and target orders. Let m=n−s and q be the
remaining target, expressed in initial ranks. The optimal
distance is unchanged: a prefix plan works above the
suffix, and projecting a full plan cannot increase cost.

Every active card must leave D and return. Project onto
the active cards moved exactly twice. All their departures
precede all their returns; otherwise the first returned
card permanently blocks a remaining departure. Within
either side stack the double reversal preserves initial
relative order. Thus these cards form a union of two
increasing subsequences of q.

If I2(q) is the largest size of such a union, then

$$
 d(\pi)\ge B(\pi):=4m-2I_2(q)
                    =4n-2I_2(\pi)-2s.
$$

This proves reversal optimality: I2=2 for a nontrivial
reversal, giving 4(n−1), matching the construction.
The report now contains the proof and exact ensemble
calculation. Greene's theorem and the hook-length formula
reduce the expectation to 281,589 partitions of 52,
rather than 52! permutations. The resulting certified
mean bound is 166.8791708235294... .

Completed validation:

- The inequality holds against all exact distances
  through n=9: 409,113 targets in total.
- An independent two-subsequence dynamic program checks
  the tableau calculation through n=6.
- Exact tableau weights sum to n!, and ensemble sums
  agree with exhaustive enumeration through n=9.

Artifacts: [calculator](tools/structural_bound.py),
[exact result](results/structural-lower-bound-52.json),
and [detailed lower-bound notes](research/next-lower-bounds.md).

### 2.2. What this bound does not explain

B is exact through n=4. Its largest deficit is two moves
at n=5 and n=6, four at n=7 and n=8, and six at n=9.
It accounts for cards needing two or four moves, but
does not explain unavoidable additional excursions or
incompatible choices of the twice-moved cards.

B alone never exceeds 4(n−1). Therefore it cannot locate
the first failure of that universal budget. A tempting
generalization is false: cards moved at most four times
need not form a bounded number of increasing subsequences.
The reversal construction moves every card at most four
times, yet reversal needs n increasing subsequences.

## 3. Priority one: stronger lower bounds

### A. Identify the missing excursion cost

Start with the smallest targets for which d(pi)>B(pi).
Enumerate alternative optimal executions, not just the
single predecessor retained by the current BFS tables.
Record per-card move counts, changes of side, and the
ordering of first departures and final returns.

Separate two mechanisms:

1. A maximum two-increasing-subsequence set cannot all
   be the twice-moved cards of any legal execution.
2. Some card necessarily needs at least six moves,
   even after choosing the twice-moved set freely.

Build a small exact feasibility oracle with individual
card budgets. A proposed obstruction is a projected
permutation together with budgets it cannot realize.
Such obstructions can become hypergraph constraints on
which cards must incur additional cost. Derive a global
charging argument or a relaxation with a checkable dual.
Do not charge the same moves twice.

Finding a six-move card in one optimal plan proves
nothing about other plans. Every proposed theorem must
quantify over all legal executions; use exhaustive code
to find counterexamples before investing in a proof.

Initial scope: two to four research days; n<=8, at most
100 representative targets, one CPU-hour per batch.
Extend a surviving rule through n=9. Continue if it
explains a new family or improves bounds by roughly five
moves on a meaningful fraction of 52-card targets.
Preserve failed conjectures and their smallest witnesses.

### B. Count canonical executions, not arbitrary words

The existing height-state count forgets card identities
and counts many words producing the same target. Improve
it by retaining a language containing at least one
shortest representative of every target, while removing
provably redundant words.

A concrete starting relation is

```text
w = DA DB AD BD
v = DB DA BD AD
```

Both words swap the top two cards of D and leave the
side stacks unchanged; both require at least two cards
on D. Hence w=v as partial transformations, and ww is
identity. A lexicographically least shortest word, with
DA before DB, contains neither v nor ww. Side reflection
lets its first move be DA.

Cross the legal-height automaton with a forbidden-factor
automaton for these relations. Count accepted closed words
using integers, then apply the existing even-distance
tail sum. Fixing the first move already removes the
reflection symmetry; do not divide by two again.

A preliminary calculation gives mean >=156.1566 and
maximum >=158 from this language alone. This is weaker
than the new structural bounds at 52. It is a promising
methodological pilot, not a new headline bound; preserve
and independently verify its counts before publication.

Extend by discovering short relations on symbolic stack
tops, proving their domains and effects above arbitrary
protected bases, and orienting them toward shorter or
lexicographically smaller words. A height loop need not
be identity. Finite examples do not prove a relation in
every stack context. Confluence is unnecessary if each
excluded factor demonstrably contradicts minimality.

As a scale check, a counting threshold near 323 at n=52
would need effective word growth around 1.623 per move,
ignoring prefactors, versus three for nonbacktracking
words. A few local exclusions will not close the gap.
Look for a family of normalizations removing substantial
multiplicity, not indefinitely more isolated relations.

Initial scope: two days, then relation lengths 4,6,8,10
with at most one CPU-hour and 1 GiB per stage. Advance
only for a measurable improvement or a scalable theorem.

### C. Combine ensemble bounds correctly

Compute the exact distribution of B, not only its mean.
Classify targets by maximal common suffix and tableau
shape of the active prefix. Subtract cases with fixed
last card; appending that largest card adds one to the
first tableau row. Check all formulas through n=9.

If H(L) counts targets with B(pi)<=L and U(L) bounds
the number reachable within L moves, then

$$
 |\{\pi:d(\pi)\le L\}|\le\min(H(L),U(L)).
$$

The tail sum using this minimum can improve on either
mean bound separately. Adding the two means is invalid.
This is a cheap first experiment after freezing the new
baseline: one research day, exact arithmetic, no large
simulation. Small improvement is still useful if the
certificate is simple.

### D. Adaptive patterns and verifiable cost sharing

For a card subset S, deletion proves

$$
 \sum_{i\in S}x_i\ge d(\pi|_S),
$$

where x_i is that card's move count. Minimize total x_i
subject to selected subset inequalities and x_i>=2 for
active cards. Existing exact tables supply right-hand
sides through size nine. Start with target-adaptive
disjoint subsets, then use overlapping subsets and
column generation.

The dual certificate assigns nonnegative pattern weights
whose total load on each card is at most one. Verify the
weights and objective with rational arithmetic; floating
LP output by itself is not a proof. Round a verified
lower bound up to the next even integer. Structural
constraints can be included in a common relaxation, but
their objectives cannot simply be added.

There is an important ceiling. Since d_k/k<=32/9 for
k<=9, assigning x_i=32/9 satisfies every such ordinary
subset inequality. At 52 cards the LP cannot exceed
184.888..., or 186 after parity rounding. Even integer
x_i=4 remains feasible, giving a 208 ceiling for that
integer formulation. More computation on these same
constraints cannot certify 300 moves. Larger patterns,
stronger budget obstructions, or interaction constraints
are essential for a substantial breakthrough.

Initial scope: three days, 100 targets, one second per
target and 1 GiB memory. Continue if the certificates
materially improve on B or accelerate exact search.

### E. Retain blockers in coarse abstractions

Deletion loses all blocking caused by omitted cards.
An alternative is to color every card and retain ordered
color words on all stacks, making same-colored cards
indistinguishable. Every concrete move projects to a
legal unit-cost move, so exact abstract distance is a
lower bound. Combine abstractions by maximum unless a
cost-partition proof permits addition.

Try balanced two- and three-color schemes only at small
n first. Even two colors can have exponentially many
states at 52. Compare bound quality against memory, and
investigate bounded interfaces or sparse backward search
before scaling. Budget two days and 1 GiB per pilot;
abandon a representation that only reproduces cheap
deletion bounds at much greater cost.

## 4. Execution and exact certification

### A. Optimize the endpoint a parent actually needs

Current leaves finish sorted on D and are then parked
on a side stack. Compute exact small-block plans whose
goal is already parked, in the orientation needed by
the parent. This can capture savings beyond cancelling
the final run of return moves.

Start with n<=7, then n=8 if memory and measured gains
justify it. Independently replay each table entry above
protected bases and prove its endpoint invariant. Keep
the existing complete parent plan as a candidate: a
shorter isolated leaf can still lose favorable boundary
cancellation. Recompute the finite constants before
claiming any improvement to the 444 universal bound.

### B. Retain boundary alternatives

Keeping one locally shortest word per interval discards
plans that may compose better. First implement an exact
reference frontier for a small, explicitly specified
merge grammar. Retain complete reduced words; deduplicate
identical words. Then try heuristic beam widths 8,32,128.

Equal endpoint, equal last move, or equal short boundary
summaries do not establish dominance. Cancellation can
expose arbitrarily much of a child's interior. Prove an
interface bound before compressing exact frontiers, or
label truncation honestly as heuristic. A capped search
is not exhaustive merely because it exhausted its cap.

Initial scope: two to three days; reference leaves<=5,
parent sizes<=10, one hour and 1 GiB per batch, explicit
frontier-size limit. Advance to n=52 on measured savings.

### C. Obtain exact intervals beyond the complete tables

Prefer target-specific n=10..16 searches to full factorial
BFS. Full n=10 arrays alone need about 1.44 GB; n=11
about 18.7 GB, before overhead. Existing 32-bit state
identifiers also need review beyond n=10.

Use bidirectional BFS as a small-instance reference,
then A* or IDA* with deletion pattern databases. Search
states have cards on all three stacks: D-to-D endpoint
tables alone cannot provide the required residual
distances. Build complete abstract-state distances and
relabel the target to identity. Take maxima, or add only
under a proved disjoint/weighted cost partition. Round
residual bounds to the parity implied by side-stack size.

An alternative is bounded SAT with explicit stack cells,
heights, legal transitions, and frame conditions. Verify
SAT witnesses in the independent simulator. Distinguish
exact-length from at-most-length encodings; padding by
inverse pairs only preserves parity. Use checkable UNSAT
proofs where available. Timeout and solver UNKNOWN do
not improve a lower bound.

Search first for targets with a large certified gap and
for a witness needing more than 4(n−1). A witness at n
proves the crossover is no later than n; it does not
prove that n is the first occurrence.

Pilot budget: 60 seconds and 1 GiB per target, 30 minutes
per method. Expand to ten minutes and 4 GiB per target,
two hours total, only after useful certificates appear.
Return [lower,upper] for every interrupted search.

### D. Improve host time after measuring it

Profile 100 fixed targets before changing representation.
Measure repeated interval planning, inverse candidates,
word copying, cancellation, and lookup decoding. Cache
or eliminate demonstrated duplication first. Compare
planning time separately from replay and data loading.

Keep the cheap balanced plan as an anytime fallback.
Improve it within a host-time budget, retaining the best
validated plan. This preserves practical latency and the
existing move guarantee while allowing experimental
search. Do not sacrifice large-n behavior to optimize
only 52 cards. Detailed implementation priorities are
in [execution notes](research/next-execution.md).

## 5. Computational hardness of the controller

### 5.1. Define the decision problem precisely

THREE-STACK-DISTANCE takes an explicitly listed target
pi and binary integer K, and asks whether d(pi)<=K.
Here n varies. Fixed n=52 is a finite problem, so formal
asymptotic hardness cannot attach to that single size.

Feasibility is trivial: every target is reachable.
Producing a polynomial-length plan is easy. Computing a
shortest plan is the unsettled question.

The problem belongs to NP. Radix gives the polynomial
upper bound U(n)=2n ceil(log2 n). Every yes-instance has
a legal witness of length at most min(K,U(n)), checkable
by simulation. Relevant budgets can be capped at U(n).
Labels can be ranks, so there are no huge numerical
parameters to explain away hardness.

Consequently, an NP-hardness proof would already have
the substance of strong NP-hardness: unary encoding of
all relevant numbers changes size only polynomially.
It would exclude an FPTAS unless P=NP. Indeed, choose
epsilon=1/(2 max(1,U(n))); an integer feasible cost below
OPT+1 must equal OPT, and 1/epsilon is polynomial.

It would **not** by itself exclude a PTAS. A PTAS is
polynomial for fixed epsilon, not necessarily when
epsilon shrinks with n. No-PTAS needs a constant relative
gap or an appropriate approximation-preserving reduction.
An additive gap of two moves over a growing baseline
is insufficient.

There are useful positive constraints on a hardness
claim. Four move choices give an O(4^K poly(n)) algorithm,
so the problem is fixed-parameter tractable in K. Suffix
trimming gives FPT in active prefix length m. The lower
bound 2m and trimmed radix give a polynomial-time
ceil(log2 m) approximation for nonidentity targets.

### 5.2. Audit the nearest reductions before building one

The closest literature is on communicating stack
networks [1–3 below]. It contains strong hardness and
approximation results, but not for these exact rules.

König and Lübbecke use at least four buffer stacks,
complete connectivity, separate input/output, and free
loading/output in their shuffle objective [2]. Their
placement-constrained PSPACE result is a different
problem again. Adding mandatory transfers can destroy
a relative gap; identifying source, buffer, and sink
can create shortcuts. Neither modification is innocuous.

Mihalák and Pont formulate a two-stack variant through
MinUnCut, with a mandatory initial loading phase [3].
This is also a lead toward tractability, not evidence
that unrestricted 3SM has the same formulation.

For an otherwise identical three-stack triangle,
d_triangle <= d_path <= 2 d_triangle, since an A–B move
can be simulated through D. These metric inequalities
do not preserve exact thresholds or automatically prove
PTAS hardness.

Token swapping, pancake sorting, and shortest generator
words offer gadget ideas, but change the graph, primitive
move, or generator set. Their hardness cannot be imported
without an explicit cost-preserving simulation. The
[complexity notes](research/next-hardness.md) give sources
and a model-by-model audit.

### 5.3. Reduction track and stopping rules

First seek a cost-preserving reduction from a close
stack-network or ordering problem. Identify a primitive
choice whose inconsistent alternatives force extra
excursions. Construct tiny guard, choice, and consistency
gadgets, then use exact unrestricted searches to attack
them before composing them.

The soundness proof must exclude moving and restoring
guards cheaply, using D as temporary space, interleaving
gadgets, changing phase order, cancelling across gadget
boundaries, and abandoning the intended simulation for
a global radix or merge route. A construction proves
only the easy direction of a reduction.

For exact hardness, establish source yes => cost<=T and
cost<=T => source yes, with polynomial output size and
the exact four-move model. For no-PTAS, additionally
force cost>=(1+delta)T in no-instances for fixed delta>0.
Track the mandatory transport and guard baseline in T;
replication alone does not create a constant gap.

Initial scope: two days for the literature/model audit,
then three to five days on one plausible reduction.
Reject a route that repeatedly needs extra storage,
free moves, constraints on legal placements, or a
vanishing gap. Record precisely why it failed.

Pursue tractability in parallel. Search for complete
normal forms based on nested excursions, intervals,
or bounded interfaces. Test each restriction against
all small optimal distances before designing a large
DP. Optimizing a chosen merge grammar is not equivalent
to optimizing all legal executions. A polynomial or
parameterized algorithm would be a satisfactory result,
not a failure to prove the conjecture.

## 6. Campaign organization and acceptance criteria

The first research cycle should take roughly one to two
weeks of investigator effort, with concurrent workstreams;
it is not a request for two weeks of unattended CPU time.

| Stage | Deliverable | Gate before expansion |
|---|---|---|
| Freeze baseline | tests, target lists, exact B checks | reproducible |
| Lower-bound work | ensemble and instance certificates | proof and verifier |
| Execution pilot | parked leaves and small frontiers | paired move savings |
| Exact target search | certified intervals or optima | useful pruning |
| Complexity audit | model map and gadget failures | sound primitive |
| Synthesis | revised bounds, timings, next decisions | independent review |

Use three research agents for the structural/counting,
execution/certification, and complexity tracks. Keep
integration and claim review in the root. Agents should
return workspace notes, executable experiments, exact
inputs and seeds, and short summaries. No duplicated
factorial searches. No new long batch without a measured
reason and a stated cap.

Use the existing samples for development. Freeze a new
holdout target list before tuning the next algorithm.
Report paired move differences, mean and uncertainty,
upper quantiles, sample maximum, host-time distribution,
and peak memory. Separate warm-table from cold-start cost.
Replay every plan independently. A sample maximum is not
a worst-case guarantee, and a sampled mean of lower
bounds is not an exact ensemble theorem.

Include adversarial targets: reversals, block reversals,
rotations, many short runs, large tableau column depth,
and targets selected for poor lower/upper ratios. Sweep
n=10,12,16,24,32,52,64,128 before large-n scaling tests.
Report both the excess over 4(n−1) and its uncertainty
where sampling is involved; do not conflate the first
extremal failure with a typical-target transition.

Maintain STATUS.md after each meaningful result or
failed hypothesis. Save the strongest lower and upper
certificate reached before a resource limit. Stop an
approach when its proved ceiling, repeated counterexample,
or measured cost makes its next batch uninformative.

The next decisive outputs would be: a structural reason
for costs beyond four moves per card; an explicit target
certified above 4(n−1); a substantial paired reduction in
52-card moves; or a correctly scoped complexity theorem.
None should be replaced by more repetitions of completed
benchmarks.

## References

The report supplies the basic sorting and tableau
references. The principal additional complexity leads are:

1. Stefan Felsner and Martin Pergel.
   “The Complexity of Sorting with Networks of Stacks
   and Queues.” *ESA 2008*, 417–429, 2008.
   [Author manuscript][felsner].
2. Felix G. König and Marco E. Lübbecke.
   “Sorting with Complete Networks of Stacks.”
   *ISAAC 2008*, LNCS **5369**, 895–906, 2008.
   [Author manuscript][koenig].
3. Matúš Mihalák and Marc Pont.
   “On Sorting with a Network of Two Stacks.”
   *ATMOS 2019*, OASIcs **75**, 3:1–3:12, 2019.
   [doi:10.4230/OASIcs.ATMOS.2019.3][mihalak].

[felsner]: https://page.math.tu-berlin.de/~felsner/Paper/sqsort.pdf
[koenig]: https://or.rwth-aachen.de/files/research/publications/stack-sorting.pdf
[mihalak]: https://doi.org/10.4230/OASIcs.ATMOS.2019.3
