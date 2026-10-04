# Controller complexity: a research plan

Archived proposal, followed by the
[executed complexity study](campaign-complexity.md).
The campaign proves an XP bound in excess excursions and
falsifies initial-drain normalization. Exact-model hardness
and inapproximability remain open; the reduction ideas
below are leads, not established results.

Research note, 4 October 2026. This note concerns the
unrestricted 3SM model in REPORT.md. No NP-hardness or
inapproximability theorem for that exact model has been
established here. The literature below supplies leads,
not a reduction.

## 1. State the problem before guessing its class

Define THREE-STACK-DISTANCE as follows:

- Input: an explicitly listed permutation pi of n distinct
  labels and a nonnegative integer K, encoded in binary.
- Initial state: A and B empty; D contains identity order.
- Goal: A and B empty; D contains pi, top to bottom.
- Operations: AD, DA, DB, BD, one top-card transfer each.
- Question: is there a legal sequence of at most K moves?

All stacks have unbounded capacity. The host may inspect
the entire input and use arbitrary ordinary polynomial
time and memory. There are no forbidden card pairings,
card weights, restrictions on intermediate states, or
extra input/output streams. Every physical transfer costs
one. A-to-B transport uses D and costs two.

Three different questions must remain distinct:

1. Feasibility is trivial: every permutation is reachable.
2. Finding a polynomial-length legal plan is easy: the
   radix construction supplies one in polynomial time.
3. Computing a shortest plan may be difficult; its formal
   complexity remains unsettled in this research.

The current O(n log n) upper bound concerns physical
transfers. It is not a lower bound on host computation.
The n! state count also does not prove NP-hardness: many
problems with exponential state spaces have compact
polynomial-time optimization algorithms.

Complexity statements require n to grow. The family with
n fixed at 52 is finite and admits a constant-size lookup
table in principle, despite an impractical constant.
Asymptotic hardness therefore says nothing quantitative
about how hard a particular 52-card target is.

## 2. Results that follow directly from this model

### Membership in NP, including binary budgets

Let U(n) = 2n ceil(log2 n), with U(1) = 0. REPORT.md's
radix construction realizes every target in U(n) moves.
Consequently every yes-instance has a certificate of
length at most min(K, U(n)). A verifier simulates this
word using three arrays or linked stacks, rejects moves
from empty sources, and checks the exact final state.
It runs in polynomial time in the explicit input size.
For K >= U(n), the verifier can use the universal plan.

Thus THREE-STACK-DISTANCE belongs to NP even when K is
binary. Exponentially long certificates are unnecessary.
This also prevents importing a PSPACE-completeness result
from a different stacking model without a major change
in assumptions, unless NP = PSPACE.

### Fixed-parameter tractability in the move budget

There are only four operation types. Exhaustively testing
all words through length K takes O(4^K poly(n)) time.
Illegal branches can be pruned immediately. Hence the
decision problem is FPT in K. This is an asymptotic
observation, not a practical method for budgets near 300.
It directly rules out transferring a W[1]-hardness result
parameterized by move count from general token swapping.

### An exact suffix reduction and a useful lower bound

Let m be the length of the prefix remaining after removing
the longest suffix on which initial and target orders
agree. Relabel that prefix to identity initial order.
Then its optimal distance is exactly the full distance.

One inequality follows by executing a prefix plan above
the untouched suffix. For the other, project any full
plan onto prefix cards: delete suffix cards from every
state and omit their moves. Each surviving top transfer
is legal in the projected state, so this gives a prefix
plan no longer than the original plan. The suffix is
therefore never needed as a workspace by an optimum.

For a nonidentity target, the initial card at depth m
must leave D. If it never leaves, it and all cards below
it retain their original bottom positions, contradicting
the definition of m. Exposing it forces every card above
it to leave D too. Each of these m cards must eventually
return to D, giving the instancewise bound OPT >= 2m.

Run radix on just these m cards. Its cost is at most
2m ceil(log2 m), so it is a polynomial-time
ceil(log2 m)-approximation. Identity uses the empty plan.
This is a valid worst-case approximation guarantee for
this model; the uniform counting mean is not a substitute
for this instancewise denominator.

Exact BFS on the active prefix also gives FPT in m,
using m! binomial(m+2, 2) states. This can help targets
whose disagreements are confined near the top.

Every complete plan has even length: each move changes
the total number of cards in A and B by exactly one,
and this number starts and ends at zero. Tight decision
budgets and candidate gaps should respect this parity.

A read-only check against existing optimal lookup tables
validated suffix equality, even distances, and OPT >= 2m
on all 46,233 permutations through n=8. The arguments
above, rather than this finite check, justify the claims
for arbitrary n. No solver implementation was changed.

## 3. What hardness would imply about approximation

The phrase "strongly NP-complete, so no PTAS" conflates
separate claims. Use NP-complete for the decision problem
and NP-hard for the optimization problem.

In this particular problem, all relevant numerical
parameters are already polynomially bounded. Labels may
be replaced by 0,...,n-1 and K may be capped at U(n).
Unary encoding of these numbers expands the input only
polynomially. Therefore any NP-hardness proof would
already establish hardness under polynomially bounded
numbers, the usual substance of strong NP-hardness here.
There is no separate source of difficulty in huge weights.

If an FPTAS existed, set epsilon = 1/(2 max(1,U(n))).
For nonzero OPT, its legal plan would satisfy

    OPT <= cost <= (1 + epsilon) OPT < OPT + 1.

Costs are integers, so the result must be optimal. Because
1/epsilon is polynomial in n, this would be a polynomial
exact algorithm. Thus NP-hardness of the exact problem
would exclude an FPTAS unless P = NP. A pseudopolynomial
algorithm polynomial in n and K would also be polynomial
after the budget cap.

A PTAS is different: its running time need only be
polynomial for each fixed epsilon. Using epsilon of order
1/U(n) can make that running time superpolynomial. Exact
or strong NP-hardness alone therefore does not exclude a
PTAS. To exclude a PTAS, establish a constant relative gap
or an appropriate approximation-preserving reduction.

For example, a reduction proving yes OPT <= T and no
OPT >= T+2 is sufficient for exact NP-hardness. When T
grows, the relative gap 2/T tends to zero and says nothing
by itself about a PTAS. A desired no-PTAS theorem needs
yes OPT <= T and no OPT >= (1+delta)T for a fixed
delta > 0, with a polynomial-size construction and a
soundness proof covering every legal 3SM plan.

Likewise, 323 sampled mean moves divided by a uniform
optimal-mean lower bound is neither an instancewise
approximation guarantee nor a proved expected ratio.
Even a proved ratio of expectations is different from
the expectation of the per-instance cost/OPT ratio.
The report distinguishes its proved universal upper
bound, sample means, and lower bounds. The campaign has
since reduced that universal upper bound from 444 to 410.

## 4. Primary literature and the model mismatches

### Communicating stacks: the closest starting point

[Felsner and Pergel, The Complexity of Sorting with
Networks of Stacks and Queues][felsner] analyzes unbounded
communicating stacks with separate source and sink.
For fixed k >= 3, it gives matching Theta(n log n)
worst-case transfer bounds. Its conclusion explicitly
asks about exact optimization and approximation on two
communicating stacks. These are move-count results and
open questions in its model, not a hardness theorem for
three-stack distance.

[König and Lübbecke, Sorting with Complete Networks of
Stacks, ISAAC 2008][koenig] is the most relevant hardness
lead. Section 2 counts only transfers between buffer
stacks: input-to-buffer and buffer-to-output moves are
free. Theorem 2 proves NP-hardness of approximation
within O(n^(1-epsilon)) for each fixed k >= 4 on complete
buffer networks, using circle-graph coloring. Its
separate PSPACE theorem imposes item-on-item placement
constraints. Neither theorem covers unrestricted 3SM.
The crucial audit items are four buffers versus three
total stacks, free loading/unloading, a separate sink,
complete edges, zero-cost optima, and forbidden pairings.

Do not transfer a relative gap by simply adding the
mandatory 2n loading/unloading cost: an additive baseline
can destroy the relative gap. Do not identify their
source, a buffer, and the sink without proving that the
identification preserves all low-cost plans.

[Mihalák and Pont, On Sorting with a Network of Two
Stacks, ATMOS 2019][two-stacks] studies a version where
all input must be loaded before any shuffle or output.
It reduces that optimization to MinUnCut, yielding
randomized O(sqrt(log n)) and deterministic O(log n)
approximations. This suggests exploring graph or dynamic
programming formulations, but the mandatory loading
phase, distinct output, and different cost convention
require a fresh equivalence proof for 3SM.

For the variant with all three stacks and a complete
triangle of edges, let d_triangle use the same endpoints
and unit costs as 3SM. Then

    d_triangle <= d_3SM <= 2 d_triangle.

Replace each direct A-B transfer by A-D-B to prove the
second inequality. These inequalities compare metrics;
they do not preserve an exact threshold, nor automatically
transfer PTAS hardness. They also do not resolve the
source/sink and free-move mismatches above.

### Token swapping: useful gadgets, different freedom

[Miltzow et al., Approximation and Hardness for Token
Swapping, ESA 2016][tokens] proves APX-hardness on general
input graphs and provides a 4-approximation. A move swaps
two adjacent occupied vertices. The graph is part of the
instance and each vertex holds one token; a 3SM move
instead changes stack sizes and only the tops are exposed.

[Bonnet, Miltzow and Rzążewski, Complexity of Token
Swapping and its Variants][token-variants] proves W[1]
hardness in the swap budget on general graphs. It also
records that ordinary labeled token swapping on a path
has optimum equal to the inversion count. Thus even
within token swapping, the graph restriction matters.
The three-node path of unbounded stacks is a different
object from an n-vertex path of one-token positions.

### Short permutation words and pancake sorting

[Even and Goldreich's author description][generators]
states NP-hardness of finding a shortest generator word
when the generators and target group element are supplied
as input. 3SM has four fixed partial operations whose
legality depends on empty stacks. An arbitrary input
generator set cannot simply be declared available.
Any use of this lead needs an explicit simulation by
card gadgets and a lower bound preventing shortcuts.

[Bulteau, Fertin and Rusu, Pancake Flipping Is Hard]
[pancakes] proves NP-hardness of optimal prefix reversal
sorting. A prefix reversal is one move there, irrespective
of its length. 3SM must move individual cards. Simulating
a flip gives a construction, not an equality of optimal
costs; unrestricted interleavings could bypass the flips.

### Container relocation and constrained loading

[Bruns et al., Complexity results for storage loading
problems with stacking constraints][loading], Theorem 4,
proves strong NP-completeness for stack height three and
transitive allowed-placement constraints. Theorem 5 uses
weight or height limits. "Height three" does not mean
"three stacks", and all these restrictions are absent
from 3SM. This is a primary example of a tempting but
invalid complexity transfer.

The original [Caserta, Schwarze and Voß blocks relocation
paper][relocation] is another lead. Its published abstract
reports NP-hardness for minimizing relocations while
retrieving stored blocks in a prescribed order. The full
proof was not accessible in this pass; its exact fixed
stack count and capacity assumptions must be checked
before selecting it as a reduction source. Retrieval
removes blocks from the system, whereas 3SM must retain
every card and put all cards back on D. This lead is not
evidence of hardness for fixed three free buffers.

## 5. Prioritized reduction program

### Milestone A: freeze definitions and structural facts

Write the exact decision definition and NP proof into
the eventual report before asserting any hardness class.
Keep fixed-n experiments separate. Establish and test
suffix projection, parity, reflection, and removal of
immediate inverse pairs as basic normalizations.

Inventory optimal words in existing exact tables through
n=8 and distances through n=9. Search for cheap plans
that violate plausible phase restrictions: returning to
D before distribution finishes, mixing old and new
cards in a buffer, moving apparent separators, sharing
one excursion across several logical gadgets, or keeping
the final bottom suffix fixed. These are counterexample
hunts for proof assumptions, not evidence of hardness.

Deliverable: a one-page model specification, lemmas,
and a list of proposed normal forms with proof status.

### Milestone B: choose a source that survives the model

The first literature track should inspect the circle
graph reduction above in full and ask whether its need
for four buffers and free loading is essential. If it
cannot be encoded with three total stacks, record that
obstruction and move on. Do not call a four-buffer proof
a three-stack proof.

The second track should look for a direct tight-budget
reduction from a combinatorial problem with no large
numerical weights, such as bounded-occurrence SAT or
exact cover. The permutation itself must encode the
choices and consistency checks. Start with exact
hardness, keeping approximation goals separate.

Candidate primitives are a two-way routing choice,
reusable consistency checks, and guards separating
regions. These names are specifications to be proved,
not gadgets already known to work. Guards cannot be
made immovable by fiat: every card may move legally.

Deliverable: one concrete candidate mapping from source
instances to permutations and budgets, plus proof
obligations and rejected model mismatches.

### Milestone C: falsify gadgets before scaling them

For each candidate, exhaustively enumerate every legal
plan up to its local budget with both side stacks free.
Test all intended boundary states, reflected versions,
and neighboring gadgets, rather than only its designed
schedule. Use BFS distances or a bounded SAT/CP encoding
to search for counterexamples to the alleged guard cost.

Then enumerate the smallest satisfiable and unsatisfiable
source instances after reduction and compare exact
distance with the proposed threshold. Padding a guard
with cards increases n and the universal radix bound;
the threshold must remain below an actual no-case lower
bound, not merely below the intended schedule's cost.

Essential escape routes to disprove include:

- moving a separator and restoring it cheaply;
- using D as temporary space during buffer-to-buffer
  transport, or using an otherwise idle side stack;
- interleaving gadgets or reordering logical phases;
- reversing or reusing a region instead of simulating
  the intended primitive;
- choosing a globally different radix or merge route;
- cancelling moves across gadget boundaries.

Finite tests can reject a gadget, but cannot certify it
for arbitrary sizes. Soundness needs a global potential,
charging argument, or proved normal-form transformation
showing that every sufficiently short legal route
encodes a valid source solution.

Deliverable: independently replayable tiny instances,
exact distances, and a falsification log. No reduction
claim until the soundness lemma is proved.

### Milestone D: obtain the precise theorem, then a gap

For exact hardness, prove both implications:

    source yes => a legal 3SM plan of cost <= T
    3SM plan of cost <= T => source yes.

Also prove polynomial output size and construction time,
identity-D start, target-D finish, fixed three stacks,
four allowed transfers, and unit costs. With membership
in NP, this gives NP-completeness and no FPTAS unless
P = NP.

Only afterwards seek a bounded-degree gap source such
as a suitable gap-SAT or APX-hard constraint problem.
If guards contribute baseline B and violated constraints
incur penalty p, show a constant fraction of constraints
must fail in no-instances and that the resulting total
penalty is a fixed fraction of B plus all other costs.
Replicating gadgets without such accounting does not
turn an additive two-move gap into a multiplicative one.

Deliverable: an exact reduction or a written failure
analysis. A no-PTAS claim requires a separate proved
constant-gap argument.

## 6. Pursue tractability in parallel

No available evidence rules out a polynomial exact
algorithm. Give this possibility an active research
track, rather than assuming hardness as the only result.

Investigate whether an optimal route admits a compact
decomposition into excursions from D, intervals, nested
card lifetimes, or bounded interface states. A recurrence
must first be shown complete for arbitrary optimal
plans. Finding an optimum inside a chosen merge-tree
family does not establish a global optimum.

Use exact plans to falsify prospective restrictions.
If a restriction survives, try a direct exchange proof
or projection lemma before implementing a large dynamic
program. The MinUnCut formulation above is a candidate
source of ideas for choosing initial buffer assignments;
it is not yet a formulation of 3SM.

Useful parameters besides K and the active prefix m
include a budget above the 2m lower bound, number of
monotone runs, and a bound on card excursions. Their
algorithmic value is open here. For n=52, an anytime
exact search with admissible pattern-database bounds
could produce per-instance lower/upper certificates
without resolving asymptotic complexity.

Stop pursuing a particular hardness route when its
primitive repeatedly admits a short unrestricted plan,
when it needs extra stacks or placement constraints,
or when its gap vanishes after mandatory transfers are
counted. Preserve these failures as structural evidence.
Success can be a polynomial algorithm, a parameterized
algorithm, an approximation theorem, or a precisely
scoped hardness theorem. None requires endorsing the
strongest conjecture in advance.

[felsner]: https://page.math.tu-berlin.de/~felsner/Paper/sqsort.pdf
[koenig]: https://or.rwth-aachen.de/files/research/publications/stack-sorting.pdf
[two-stacks]: https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.ATMOS.2019.3
[tokens]: https://arxiv.org/abs/1602.05150
[token-variants]: https://arxiv.org/html/1607.07676
[generators]: https://www.wisdom.weizmann.ac.il/~oded/annot/node3.html
[pancakes]: https://arxiv.org/abs/1111.0434
[loading]: https://eprints.whiterose.ac.uk/id/eprint/91278/8/1-s2.0-S0377221715008784-main.pdf
[relocation]: https://doi.org/10.1016/j.ejor.2011.12.039
