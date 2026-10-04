# Next lower-bound research

Archived planning note and structural-bound derivation.
The 166.87917 exact mean theorem derived here remains
current. The later [structural campaign](campaign-structure.md)
and [counting campaign](campaign-counting.md) record which
proposals succeeded and which reached stopping gates.
Baseline algorithms and sample numbers below are historical.

Planning notes, 2026-10-04. This note proposes bounded
research stages and establishes one immediate structural
improvement. Existing planners and result files were not
changed; a new calculator and result artifact were added.

## The question to answer

For uniformly chosen targets at n=52, the previous proved
optimal mean is at least 154.9453333327. The main planner
averages 322.838 on its 1,000-target sample; every target
has a proved construction of at most 444 moves.

These numbers describe different objects. The objective
is to narrow the optimality gap, not to force a lower
bound up to a heuristic's measured mean. An optimal mean
near 200 would also be a major result if accompanied by
better constructions. Reversal costs only 204, so large
inversion count alone cannot certify typical costs near
323.

Keep three outputs separate:

- A theorem about the uniform ensemble mean.
- A theorem or certificate for the worst target.
- Computable bounds for particular sampled targets.

The immediate result developed below is a uniform mean
bound of 166.87917082352942, computed exactly, and an
elementary proof that reversal's optimal cost is 4(n-1).
The next deliverable is a table of certified instance
bounds paired with existing plan lengths.

## Priority 1: certify the twice-moved-card bound

The following lemmas have elementary proofs. The root
agent independently reviewed them and included the bound
and exact expectation in REPORT.md, Section 5.4.

### Deletion and move accounting

For a card subset S, delete cards outside S from every
stack throughout a legal execution. Ignore moves of
deleted cards. Every retained move remains legal: a card
that was physically exposed is exposed after deletion.

Write m_i for the number of moves of card i. Initial and
final location D imply every m_i is even. If pi|S is the
relative target permutation on S, then

```
d_|S|(pi|S) <= sum(i in S) m_i.
```

For disjoint subsets, these inequalities may be added.
Deletion is a relaxation: it removes blockers. A bound
proved only for a more restrictive routing algorithm is
not automatically a lower bound for 3SM.

### Which cards must move

If card i is never moved, all cards initially beneath i
remain inaccessible and also never move. Thus untouched
cards form an initial bottom suffix, which is also an
identical final bottom suffix.

Let s be the length of the longest common bottom suffix
of the initial and target decks, and set m=n-s. Every
card in the initial first m positions must move at least
twice. Cards within the common suffix may also move;
the lemma does not assume that a chosen plan leaves the
whole common suffix untouched.

### Cards moved exactly twice

Project a plan onto its cards moved exactly twice. Each
such card goes D -> A or B -> D exactly once. All outward
moves must precede all return moves: after the first
return, that card is permanently on top of D and blocks
any further departure of an as-yet-unmoved projected card.

The projected execution therefore distributes the cards
once and returns them once. Within each side stack, the
two reversals preserve the cards' original relative
order in the final D order. The target subsequence on
these cards is a union of two increasing subsequences.

Let I2(q) be the largest number of entries in q that can
be covered by two increasing subsequences. Restrict pi
to the initial first m cards, retaining their initial
ranks; call this restriction q. At most I2(q) of those
cards cost two moves, and the remaining cards cost at
least four. Consequently,

```
d_n(pi) >= B(pi) := 4m - 2 I2(q).
```

Since the common bottom suffix consists of the largest
s initial ranks appended in increasing order, I2(pi)
equals I2(q)+s. Any union of two increasing subsequences
contains at most I2(q) active cards and s suffix cards;
conversely, append all suffix cards to one increasing
subsequence of a maximizing active selection. Hence

```
B(pi) = 4n - 2 I2(pi) - 2s.
```

This formula includes identity, whose I2 and s both
equal n and whose bound is zero.

This is an immediately provable lemma, not an inference
from the small exact tables. It requires no assumption
that a planner has a particular sequence of phases.

For reversal and n>=2, s=0 and I2=2, giving 4(n-1).
Together with Proposition 2's construction, this proves
the exact reversal distance for every n. For n=1 the
distance is zero.

### Bounded checks completed for this note

Every target distance through n=9 satisfied B(pi)<=d_n(pi).
An independent dynamic program for I2 agreed with the
RSK row calculation through n=6. B is exact for every
target through n=4.

| n | Mean B | Exact optimal mean | Largest gap |
|---|--------|--------------------|-------------|
| 5 | 11.0333333333 | 11.1 | 2 |
| 6 | 13.925 | 14.1138888889 | 2 |
| 7 | 16.8369047619 | 17.1873015873 | 4 |
| 8 | 19.7918154762 | 20.3542162698 | 4 |
| 9 | 22.7855158730 | 23.6293044533 | 6 |

A separate seed-41004 pilot of 10,000 Fisher-Yates-style
targets at n=52 produced mean B=166.9594, sample standard
error 0.0384393, and range 152..180. This is a sample
estimate of a proved per-instance bound. It is not yet
a proved uniform mean of 166.9594.

### Exact ensemble calculation

Greene's theorem gives I2(pi)=lambda_1+lambda_2 for the
Robinson-Schensted shape lambda. For uniform pi, the
shape has weight (f^lambda)^2/n!, where

```
f^lambda = n! / product(cell c in lambda) hook(c).
```

Define the exact rational expectation

```
G_n = sum(lambda partitions n)
      (lambda_1 + lambda_2) * (f^lambda)^2 / n!.
```

The suffix identity above gives the exact expectation
of the per-instance bound, with no correlation assumption:

```
mu_n >= E[B] = 4n - 2 G_n - 2 E[s],
E[s] = sum(k=1..n) (n-k)! / n!.
```

`tools/structural_bound.py` now computes this expectation
with exact integer hook products and rational arithmetic.
At n=52 it enumerated 281,589 shapes in 3.73 seconds and
verified sum (f^lambda)^2=52!. The exact reduced bound is

```
47731097137113840908862262779607325979589169862469413070151949515219
/
286021897769304533942058995944694209132232288797458432000000000000
= 166.87917082352942...
```

The exact mean, I2 sum, and suffix sum match exhaustive
enumeration for every n<=9. The suffix identity was also
checked on every one of those targets. The result and
validation summary are saved in
`results/structural-lower-bound-52.json`.

Reproduce the bounded calculation with:

```sh
python3 tools/structural_bound.py --n 52 \
    --check-through 9 --max-seconds 60 \
    --output results/structural-lower-bound-52.json
```

The previous mean lower bound is improved by about
11.93 moves. Remaining budget: one day for independent
proof review and integration into the report. The first
success criterion, an exact mean above 160, is achieved.

A subsequent exact distribution of B can combine this
argument with counting. If H(L) counts targets with
B<=L and U(L) is any certified word-capacity bound, then
the number of targets with optimal distance at most L
is at most min(H(L),U(L)). Apply the existing even-valued
tail sum to this minimum; adding the two mean bounds
would double-count their information.

H can be obtained without enumerating permutations.
Let F_m(r) count permutations of m cards with I2=r,
using the same shape weights. Targets whose maximal
common suffix leaves m>=2 active cards have an active
permutation with nonfixed last card. Their I2 histogram
is F_m(r)-F_(m-1)(r-1), because appending the largest
fixed last card raises I2 by one. Each such count enters
B's histogram at 4m-2r. Add the identity separately;
m=1 is impossible. This is a proposed extension, not
an additional computation performed in this task.

## Priority 2: use move accounting to find missing cost

The twice-moved lemma captures why many cards need four
moves. It does not detect why any card must need six,
eight, or more. That is the structural research question
most likely to produce a larger jump.

Inspect the n=5..9 targets where d-B is positive. Recover
several optimal plans for each, and record per-card move
counts, side-switch histories, and first/last departure
times. Ask whether the gap is caused by incompatible
choices of the cards allowed only two moves, or by a
genuinely unavoidable third excursion for some card.

A useful candidate theorem would lower-bound the number
of cards requiring at least six moves after all possible
choices of a maximum two-increasing-subsequence set.
Its formulation must quantify over every legal plan.
Finding six-move cards in one optimal plan does not prove
their inevitability in every optimal plan.

Do not assert that cards moved at most 2r times form at
most 2r increasing subsequences. Reversal supplies a
counterexample for r=2: the known reversal plan moves
every card at most four times, but the target needs n
increasing subsequences.

Try obstruction hypergraphs: vertices are cards; an
edge represents a projected pattern that cannot be
realized with specified per-card excursion budgets.
Weighted covers can charge incompatible local demands
without double-counting moves. The missing ingredient
is an exact feasibility oracle with individual card
budgets, not merely the ordinary optimal distance.

Budget: two to four days; initially only n<=8, at most
100 representative gap targets, and one hour of bounded
enumeration per batch. Continue if a pattern rule survives
every exact target through n=9 and improves 52-card bounds
by at least five moves. Otherwise document the failed
rules and move on.

## Priority 3: canonical traces and height automata

The present DP counts all legal closed words without
immediate inverses. It distinguishes many words with
identical effects. A canonical representative for every
reachable target would count a smaller covering language.

### A concrete valid local relation

Set

```
w = DA DB AD BD,
v = DB DA BD AD.
```

Both are legal exactly when D initially has at least
two cards, and both swap D's top two cards while leaving
the rest of D and both side stacks unchanged. Thus w=v
as partial state transformations, and ww is identity.

Choose a lexicographically smallest shortest word for
each target, with DA ordered before DB. Reflection makes
its first move DA. It cannot contain v, because replacing
v by w preserves legality and endpoints while decreasing
the word lexicographically. It cannot contain ww, because
deleting it makes the word shorter.

Count words with first move DA, no immediate inverses,
and no factors v or ww. The height DP crossed with a
prefix automaton for these factors has 11 suffix states.
Use its closed-word counts C_L directly:

```
U(2k) = 1 + sum(j=1..k) C_(2j).
```

Do not divide these counts by two: reflection has already
been removed by fixing the first move. The identity is
included separately. Canonical words need not be unique
among all counted words; a covering language suffices.

A bounded Python check at n=52 through length 204 took
about 2.8 seconds. Exact integer counts, fed into the
existing tail-sum formula, gave exploratory values

```
maximum lower bound: 158
mean lower bound:    156.15662479267044
```

These numbers have not been added to official results.
The relation proof is elementary; the implementation
still needs an independently written reference enumerator
and preserved exact counts before publication.

### Extensions to test

- At every return to a=b=0, the next nonempty closed
  component can be reflected independently. Require
  each component to start DA, not only the whole word.
- Enumerate equal endpoint transformations on symbolic
  stack tops for short words, then prove candidate
  relations for arbitrary protected bases.
- Add shortening rules and lex-decreasing equal-length
  rules to a finite forbidden-factor automaton.
- Add bounded stack-content information only when the
  abstraction has a proved representative-preservation
  argument. Forgetting information must enlarge, rather
  than accidentally shrink, the covering language.

Never exclude a loop solely because it returns to the
same heights: w is a closed height loop that changes
the permutation. Height data alone cannot identify
identity transformations. Likewise, a relation observed
on a finite deck may depend on boundary emptiness and
need not be universal.

A lexicographic minimum avoids a need to prove confluence
for local equal-length substitutions. Every excluded
factor must, however, have a legal lex-smaller replacement
or a legal shorter replacement in every allowed context.

The entropy scale gives a useful reality check. Ignoring
finite prefactors, a 323-move counting threshold at n=52
would require effective growth about

```
exp(log(52!)/323) = 1.6226965078 per move.
```

The basic nonbacktracking rate is three. Eliminating a
few short factors is unlikely to close the whole gap.
That is a quantitative motivation for canonical traces
that quotient substantial, repeated multiplicity.

Budget: two days for the first certified automaton;
then a capped relation search with lengths 4, 6, 8, 10,
at most one CPU-hour and 1 GB per stage. Continue beyond
the first stage only if it adds at least two mean moves
or reveals a scalable family of normalization rules.

## Priority 4: subset LP and pattern databases

For a selected family F of subsets, solve the relaxation

```
minimize sum_i x_i
subject to sum(i in S) x_i >= d_|S|(pi|S), S in F,
           x_i >= 0.
```

Actual per-card move counts give a feasible solution,
so the optimum is a valid lower bound. Round it upward
to the next even integer. Forced-prefix constraints
x_i>=2 may be added. Cost constraints cannot simply be
added to B(pi), since both charge the same moves.

The dual produces a compact certificate:

```
w_S >= 0,
sum(S containing i) w_S <= 1 for each card i,
d_n(pi) >= sum_S w_S * d_|S|(pi|S).
```

Use exact rational weights, or round nonnegative floating
weights downward and verify every per-card capacity
with exact arithmetic. A floating solver objective alone
is not a rigorous certificate. For combined constraints,
verify the appropriate expanded dual including forced
prefix constraints.

Start with target-adaptive disjoint partitions into
subsets of size 6..9. Add overlapping patterns by column
generation: search for subsets whose table distance
exceeds their current dual card prices. A failed search
for more columns does not certify global optimality of
the LP, but a feasible dual already certifies its bound.

### Important ceilings

A fixed partition of 52 cards into five nine-card sets
and one seven-card set has exact expected bound

```
5 mu_9 + mu_7 = 135.3338238536.
```

Thus fixed disjoint patterns alone are weaker on average
than current counting. Target-adaptive selection may
help, but it has a finite ceiling.

For all available patterns of size at most nine,
d_|S|<=4(|S|-1). Moreover d_|S|/|S|<=32/9.
Setting every x_i=32/9 satisfies all subset constraints,
including x_i>=2. This LP cannot exceed

```
52 * 32/9 = 184.8888888889.
```

Even rounding can raise that certificate to at most
186 moves. This remains far below the measured plan mean.

Imposing even integer move counts still admits x_i=4
for every card, so these ordinary pattern constraints
cannot certify more than 208 moves. Breaking that ceiling
requires larger patterns or additional structural
constraints that capture more than subset total cost.

For optimal search, store distances to all abstract
states, not only final D permutations. A size-nine
byte distance array has 19,958,400 entries; the BFS's
queue raises build memory to the roughly 120 MB already
documented. A size-eight array has 1,814,400 entries.
Relabel the target to identity and use undirected-state
distances from the standard initial state.

Projected moves of ignored cards are zero-cost self
steps; retained moves cost one. Disjoint patterns charge
each physical move at most once. Overlapping patterns
need verified cost sharing, or take their maximum.

Budget: three days, <=1 GB memory, and a pilot of 100
uniform 52-card targets with <=1 second per target.
Continue as an instance-certification tool if it beats
B by five moves on at least a quarter of the pilot,
or materially accelerates exact searches. Do not invest
in it as a route to 300 using only size-nine tables.

## Priority 5: coarser partition abstractions

Color each card by a partition class and retain the
ordered color words on all three stacks. This is a
different relaxation from deleting cards: all moves
remain, but cards within a color become interchangeable.
Map the initial and target states to their color states.
Every concrete move maps to an abstract move, so abstract
shortest-path distance lower-bounds concrete distance.

Use two or three color classes chosen by target order
or by difficult local patterns. The representation must
retain ordered color sequences. Merely recording class
counts in each stack often loses the target-order
obstruction entirely.

Even two balanced colors yield a huge word space at
52 cards. Start with ten or twelve distinguished cards
and one anonymous color, or merge colors further while
preserving the transition relaxation. Give every omitted
constraint an explicit simulation proof.

This direction can preserve blocking and interactions
that deletion removes. Its likelihood of improvement
must be established on exact small instances before
paying for 52-card abstractions.

Budget: two days of small-n evaluation, <=1 GB, and no
abstract graph exceeding ten million states initially.
Continue only if its bound captures a substantial part
of d-B on n=8 or n=9 while staying cheap to query.

## Statistical and certificate protocol

For any proved instance bound b(pi), E[b(Pi)]<=mu_n.
Its sample mean estimates E[b], not the optimal mean.
Paired differences between plan length and b provide
upper bounds on each sampled instance's optimality gap.

Predeclare an independent uniform evaluation sample,
separate from targets used to invent the bound. Retain
all sampled targets, including expensive timeouts. A
timeout can fall back to a weaker proved bound; replacing
the target by an easier one biases the sample.

For a statistical ensemble claim use a one-sided
confidence bound. If 0<=b<=444 is the only guaranteed
range, Hoeffding gives

```
E[b] >= sample_mean
        - 444 sqrt(log(1/alpha)/(2N))
```

with confidence 1-alpha for N independent ideal uniform
targets. This conservative radius is large; use a tighter
proved range for the chosen statistic or a documented
empirical-Bernstein bound. A normal standard-error
interval is a useful estimate but not a distribution-free
certificate. Seeded PRNG reproducibility is separate
from mathematical independence and uniformity.

For B specifically, the proved range at n=52 is 0..204:
the active restriction has m=0 or m>=2, and I2>=2 when
m>=2. Replace 444 by 204 when evaluating this statistic.

Adaptive stopping or trying many bounds on the same
sample requires a confidence sequence, correction for
multiple tests, or a fresh final sample. Prefer exact
ensemble formulas whenever they are feasible.

Worst-case certificates need no sampling: one concrete
target and one sound certificate suffice. To claim a
worst-case value rather than a lower bound, also prove
an upper bound for all targets.

Store certificate inputs, exact table identifiers and
hashes, model conventions, rule proofs, and the evaluator
version. For LPs store the feasible rational dual. For
automata store transition rules, exact integer counts,
and the covering proof. For bounded search store a
complete exhaustion certificate or replayable proof;
failure to find a shorter plan is not itself a lower
bound.

## Recommended sequence and stopping gates

1. Review B's proof and the completed exact mean.
   Publish this independently verified improvement first.
2. Certify the short-factor automaton and compare it
   with B on exactly the same targets.
3. Classify small optimality gaps d-B, focusing on
   incompatible two-excursion assignments and necessary
   third excursions.
4. Prototype adaptive pattern certificates with their
   184.89/208 ceilings kept visible.
5. Pursue coarser blocking abstractions only after a
   small-n pilot shows additional information.

After roughly two weeks, require either an exact mean
bound near 170 or above, a certified worst-case target
above 204, or a concrete structural theorem that scales
beyond the four-moves-per-card barrier. If none emerges,
redirect effort toward improving the constructive upper
bound and learning whether the current planner's gap is
mainly algorithmic.

## Primary literature checked

[Greene's original paper record][greene] identifies the
1974 extension of Schensted's theorem. [Dukes and
Mullins][bump], Theorems 3 and 4, state the union-of-k-
subsequences and hook-length formulas used here. Their
paper also states the Robinson-Schensted bijection with
pairs of tableaux. These are combinatorial tools; the
3SM lower-bound lemma above is derived separately.

[Korf and Felner][disjoint] establish additive pattern
databases when each operator affects only one disjoint
pattern. [Felner, Korf, and Hanan][additive] discuss
static and dynamic additive partitions. The deletion
proof and move-accounting constraints above establish
the required validity for this machine.

[Albert and Bousquet-Melou][parallel] use canonical
operation sequences to enumerate permutations through
two parallel stacks. Their separate input/output model
differs from 3SM's shared D storage. Their canonical
rules are research inspiration, not directly applicable
forbidden factors or enumeration bounds here.

[greene]: https://scholarship.haverford.edu/mathematics_facpubs/172/
[bump]: https://link.springer.com/article/10.1007/s00026-024-00708-z
[disjoint]: https://doi.org/10.1016/S0004-3702(01)00092-3
[additive]: https://arxiv.org/abs/1107.0050
[parallel]: https://dmtcs.episciences.org/2425/pdf
