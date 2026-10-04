# Controller complexity: executed research

4 October 2026. Exact model: three unbounded stacks A-D-B,
four unit-cost top transfers, identity and target entirely
on D. This note records proved structural results and
completed computations. It asserts no hardness theorem.

## 1. Main result: polynomial time for fixed excess

Let m be the active prefix length after deleting the
longest common identity suffix. Its distance equals the
original distance. For a nonidentity target, all m active
cards must leave D and return at least once. Set

    r = floor((K - 2m) / 2).

If K < 2m, answer no. Identity is immediate. Otherwise
THREE-STACK-DISTANCE is in XP parameterized by r: it can
be decided in

    f(r) (m + 1)^(5r + O(1))

time, returning a legal plan on yes-instances. Thus each
fixed excess above the 2m bound has a polynomial exact
algorithm. This is **not** an FPT claim in r, and it is
not a polynomial algorithm when r grows with m.

Second-campaign update: a fixed skeleton for all cards
and balanced temporary D blocks reduce the exponent
to 2r+2. See [the improved enumeration](parameter-complexity.md).
The original proof below remains valid but is superseded
as an enumeration bound. The XP/P distinction is unchanged.

### Fixed excess does not imply membership in P

For each fixed r, the exponent 5r+2 is a constant as m
grows. Polynomial time for unrestricted input would
require one exponent independent of both m and r.
The distinction is called XP versus P here; an FPT
algorithm would instead have time g(r)m^c with a single
constant c, although g(r) could still make growing r
intractable. The displayed bound establishes XP only;
it does not rule out a better algorithm.

Binary search on K is valid because feasibility is
monotone. The radix upper bound is O(m log m), so only
O(log m) decision queries are needed. The difficulty is
the cost of each query: r can grow as O(m log m), and
substituting that value in the XP bound does not give
polynomial time. Even a hypothetical universal upper
bound 4m would leave r as large as m. A polynomial-size
search interval is not a polynomial-time decision oracle.

At m=52, the first campaign's 410 upper bound corresponds
to r=153; the newer 352 construction reduces that to
r=124. These are still large parameters. The theorem is
therefore a result
about a restricted low-excursion regime, not a practical
exact algorithm across the present 52-card interval.

### Scope of the derivation

The derivation below is specific to the current model.
It does not depend on a reduction from stack-network
literature. No claim of novelty across that literature
is made.

### Excursions and exceptional cards

An excursion is one departure of a card from D to A or B
and its next return to D. Because A and B have no edge,
every card's moves alternate departures and returns.
Every completed excursion costs exactly two moves.

A plan of length at most 2m+2r has at most r extra
excursions beyond one per active card. Call a card
exceptional if it has more than one excursion. There
are k <= r exceptional cards. If they collectively have
j extra excursions, they have k+j excursions and
2(k+j) <= 4r departure/return events.

Each remaining ordinary card has exactly two events.
Their projected event order is uniquely determined:

1. Departures in initial order, restricted to ordinary
   cards.
2. Returns in reverse target order, restricted to
   ordinary cards.

To prove this, initial ordinary cards can only be exposed
in their original order. Once an ordinary card returns,
it never moves again; it permanently blocks any ordinary
card still below it on D. Therefore all ordinary
departures precede all ordinary returns. Permanent
returns build the final stack from bottom to top.
Deleting exceptional cards preserves these facts.

This is a normalization of the **projected ordinary
events**, not of all physical transfers. Exceptional
returns may precede the last initial departure, and the
experiment in Section 3 shows that forbidding this can
lose optimality.

### An exact event-schedule feasibility primitive

Suppose a chronological event schedule specifies, for
every card, alternating departures from and returns to D.
Assume each card starts and ends on D. The schedule does
not yet choose A or B for an excursion.

First simulate D alone. At a departure, the specified
card must be the current top; pop it. At a return, push
the specified card. Reject any illegal departure or a
wrong final D order.

For every excursion form the interval (a,b), where a
and b are its departure and return times. Construct an
undirected graph with one vertex per excursion and an
edge exactly when two intervals cross:

    a < c < b < d, or c < a < d < b.

The schedule is realizable on A-D-B if and only if this
graph is bipartite and the D simulation succeeds.

Necessity: crossing intervals assigned the same side
stack would return the older card while the newer card
still covers it. Hence they must use different sides.
Actual side choices give a proper two-coloring.

Sufficiency: assign the two colors to A and B. Within
one color, excursion intervals are disjoint or nested.
When an interval returns, every later departure to its
side has either already returned, or would cross this
interval. The latter is excluded by the coloring.
Thus the returning card is exposed. D simulation already
certifies every departure, so all four transfer types
are legal. The final D check certifies the target.

This proof permits repeated excursions by the same card,
including different side choices in different excursions.
No immovable guards, extra buffers, or free moves are
assumed. A direct implementation builds the graph and
finds a coloring in O((m+r)^2) time and space.

### Enumeration and completeness

For every j=0,...,r:

1. Choose k <= j exceptional cards, with k=0 if j=0.
2. Allocate j extra excursions among them, each at least
   one. Card i has 1+t_i excursions, sum t_i=j.
3. Enumerate their event orders, respecting each card's
   alternating departures and returns.
4. Interleave those events with the fixed ordinary
   event sequence described above.
5. Apply the exact schedule-feasibility primitive.

Every accepted schedule gives a plan of length 2m+2j.
Conversely, every plan within the requested budget has
some enumerated exceptional set, excursion allocation,
exceptional event order, and interleaving. Its D simulation
and side coloring succeed. This proves both directions.

There are at most m^r exceptional-set choices times a
factor depending only on r for allocations. At most 4r
exceptional events are placed into 2m+2r chronological
positions, with event labels included in the enumeration.
A crude bound is f(r) m^r (2m+2r)^(4r) schedules.
Multiplying by the quadratic feasibility check yields
f(r)(m+1)^(5r+2), sufficient for XP. Stream the schedules
to keep working memory polynomial in m+r.

Parity is handled by the floor in r, and enumerating all
j<=r handles at-most budgets without padding assumptions.
For K above the trimmed radix upper bound, one may simply
return that universal plan before enumerating anything.

## 2. Tight baseline and an implemented excess-one case

The r=0 case simplifies further:

    d(target) = 2m
    iff its active prefix is a union of two increasing
    subsequences
    iff its active prefix avoids decreasing triples.

Necessity follows because each card moves exactly twice;
all departures precede all returns, and double reversal
preserves the initial relative order within each side.
For sufficiency, color the two increasing subsequences,
depart in initial order to their assigned sides, then
return cards in reverse target order. Within either side,
the next requested card is its top, so the plan is legal.

The equivalence between two increasing subsequences and
avoiding decreasing triples also has a direct constructive
proof. Give each element the length of the longest
decreasing subsequence ending there. These lengths are
at most two; elements of either fixed length increase.
Recognition and a two-chain assignment can also be done
with two greedy tails in linear time.

[The experiment program](../tools/campaign_complexity.py)
implements this construction and the entire r<=1 decision
algorithm. For r=1, choose the exceptional card and
positions for its four alternating events; the remaining
events have fixed order. It uses the graph test above,
then independently replays every accepted word using
the physical three-stack simulator.

Against the existing complete exact tables, all 873
targets through n=6 agree with the excess-one decision.
Every returned word has exactly the tabulated optimum.
The program considered 2,310,697 nontrivial schedules;
the run took about 4.1 seconds, with a 50-second cutoff
and a 512 MiB address-space limit.

| n | Targets | d <= 2m+2 | Enumerated schedules |
|---|---:|---:|---:|
| 1 | 1 | 1 | 0 |
| 2 | 2 | 2 | 0 |
| 3 | 6 | 6 | 17 |
| 4 | 24 | 23 | 2,265 |
| 5 | 120 | 99 | 88,931 |
| 6 | 720 | 439 | 2,219,484 |

The tight 2m characterization was separately checked on
all 5,913 targets through n=7. Counts attaining it were
1, 2, 5, 14, 42, 132, 429. Every constructive witness was
replayed. The finite checks support the implementation;
the proofs above establish the arbitrary-size statements.

Artifacts:

- [Excess-one results][excess-results].
- [Phase and tight-baseline results][phase-results].

Reproduce from the project root:

```sh
timeout 55s python tools/campaign_complexity.py \
  --mode excess-one --max-n 6 --seconds 50 \
  --output results/campaign-complexity-excess-one.json
timeout 55s python tools/campaign_complexity.py \
  --max-n 7 --seconds 50
```

## 3. A falsified mandatory-loading normalization

Candidate: after suffix trimming, some optimum always
drains all of D before the first return to D.

This is false. The smallest counterexamples have n=5.
One is target (2,3,1,4,0), whose unrestricted distance
is 12 but whose restricted distance is 14. An optimal
unrestricted word is

```text
DB DB DA DA BD DA DA BD AD AD AD AD
```

After its fourth move, the state is

    A=(3,2), D=(4), B=(1,0).

It returns card 1 to D and sends it to A before card 4
first leaves D. This early transfer changes its place
relative to cards 2,3,4 in A. It uses one extra excursion;
the other four cards move exactly twice.

The target contains a decreasing triple (2,1,0), ruling
out a 10-move plan by Section 2. Parity and the replayed
12-move word independently establish the unrestricted
optimum 12. The restricted optimum 14 is established by
complete multi-source BFS: initialize every pure initial
distribution of the n cards to A and B at cost n, then
allow all legal transfers. This exactly enumerates plans
with the proposed initial-drain requirement.

All active targets through n=7 were checked. Only targets
with active prefix equal to n enter this comparison, so
the counterexamples cannot be blamed on unnecessarily
moving an already correct bottom suffix.

| n | Active targets | No penalty | +2 | +4 |
|---|---:|---:|---:|---:|
| 2 | 1 | 1 | 0 | 0 |
| 3 | 4 | 4 | 0 | 0 |
| 4 | 18 | 18 | 0 | 0 |
| 5 | 96 | 93 | 3 | 0 |
| 6 | 600 | 539 | 59 | 2 |
| 7 | 4,320 | 3,476 | 769 | 75 |

The other n=5 targets are (3,1,4,2,0) and (3,4,1,2,0),
also with unrestricted distance 12 and restricted 14.
The JSON records unrestricted and restricted witnesses
and full unrestricted state traces. BFS visits all
181,440 states at n=7. The entire phase experiment took
about 0.55 seconds under the same memory cap.

This rejects the direct strategy of importing a
mandatory-loading theorem by claiming that its loading
phase can always be imposed on an unrestricted optimum.
It does not rule out some different cost-preserving
reduction.

## 4. Focused primary-literature audit

Full bibliographic records are collected in the
[stack-source notes](bibliography-stacks.md).

The sources below were opened through web research during
this campaign. Search queries included the exact titles,
communicating stacks with optimization hardness, and
three-stack minimum sorting complexity. No located source
proved NP-hardness for the exact fixed A-D-B endpoint-D
unit-cost model. This is a bounded search result, not
proof that no such theorem exists.

[König and Lübbecke, ISAAC 2008][koenig], Section 2 and
Theorem 2, use a separate source and sink, free source
loading and sink output, a complete buffer network, and
at least four buffers. Their coloring construction uses
overlap intervals for source-to-output lifetimes. It is
not the same as the excursion graph in Section 1: our
event schedule is itself an optimization variable and
uses two actual side stacks. Their Section 3 hardness
therefore cannot simply be instantiated on A-D-B.

[Mihalák and Pont, ATMOS 2019][two-stacks] explicitly
require all input to enter the two buffers before any
shuffle or output. Their reduction to MinUnCut gives
approximation algorithms under that assumption. Section 3
supplies an exact counterexample to imposing the analogous
initial-loading phase here. Additionally, their output
is a separate stream and their shuffle costs differ.

[Felsner and Pergel, ESA 2008][felsner] study communicating
buffers with source and sink. Their conclusion asks about
computing and approximating shortest sorting routes for
two communicating stacks. The worst-case move bounds in
that paper are not computational hardness proofs for the
present controller problem.

[Biedl et al., 2010][biedl], Section 4, describe bidirectional
series of stacks with input and output connected to the
stacks. Theorem 5 gives asymptotic move bounds. Even this
path-like network retains separate input/output access;
it does not provide an exact-distance hardness theorem
with both endpoints fixed to the middle stack.

The new XP result constrains prospective reductions:
unless P=NP, an NP-hard family cannot have universally
bounded excess above 2m. A growing excess remains fully
compatible with NP-hardness. Neither that possibility
nor any future exact NP-hardness proof excludes a PTAS
without a separate relative-gap argument.

[excess-results]: ../results/campaign-complexity-excess-one.json
[phase-results]: ../results/campaign-complexity.json
[koenig]: https://or.rwth-aachen.de/files/research/publications/stack-sorting.pdf
[two-stacks]: https://drops.dagstuhl.de/entities/document/10.4230/OASIcs.ATMOS.2019.3
[felsner]: https://page.math.tu-berlin.de/~felsner/Paper/sqsort.pdf
[biedl]: https://doi.org/10.1016/j.dam.2010.06.007
