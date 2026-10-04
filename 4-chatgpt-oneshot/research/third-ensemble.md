# Third campaign: population bounds and barriers

The deterministic uniform-mean theorem remains
166.87917082352942. This track supplies a conditional
statistical result and an exact population barrier for a
natural continuous relaxation. It does not close the gap
to the measured construction mean near 274.

Under independent ideal uniform sampling, the frozen
100-target holdout gives a 99% lower confidence bound
172.08938482352943 for the optimal mean. The exact
arithmetic certificate is in
[the confidence artifact](../results/third-ensemble-confidence.json).
The stored seeded PRNG sequence alone does not establish
the sampling premise, so this is not a replacement for
the deterministic 166.87917 theorem.

For the linearized finite catalog considered below,
99.8314428071% of the entire 52-card population admits
a fractional assignment attaining exactly B. This is
an exact shape-counting theorem, not a sample estimate.
Adding full decreasing-chain constraints escapes that
barrier: one 52-card pilot has a portable, 17-row exact
rational dual certificate proving 176 moves against B=170.

## Concentrate the additional cost only

Write d(pi) for optimal distance and B(pi) for the proved
structural bound. Define the fixed mathematical statistic

```text
G*(pi) = min(6, d(pi)-B(pi)).
```

Then G* belongs to {0,2,4,6}, and pointwise
d >= B+G*. The exact expectation of B is already known.
It is unnecessary to estimate it again from a sample.

Let g_i be any certified improvement, capped at six,
on sampled target Pi_i. Always g_i <= G*(Pi_i).
This permits timeouts, different search budgets, and
adaptive choices of sound instance certificates: the
latent random variables G*(Pi_i) remain fixed functions
of the targets. Selection or replacement of the targets
themselves is not permitted in this argument.

The complete original holdout has 87 gains of six,
11 gains of four, and two gains of two. Its empirical
gain is 5.70. Hoeffding applied to G* in [0,6] gives

```text
mu_52 >= E[B] + 5.70 - 6 sqrt(log(100)/200)
      = 171.66871454589835
```

with one-sided 99% confidence under the sampling model.
No independence between B and the obstruction gain
is used. Their expectations add by linearity.

### Exact simultaneous binomial inversion

Use the identity

```text
E[G*] = 2(P(G*>=2)+P(G*>=4)+P(G*>=6)).
```

For each threshold h, the latent number of successes
K_h is Binomial(100,p_h), where p_h=P(G*>=h).
The observed certified counts k_h only underestimate
K_h. The lower confidence limit is monotone in the
observed count, so replacing K_h by k_h is conservative.

Give each threshold failure probability 1/300. The
script finds a rational p on the millionth grid whose
binomial upper-tail probability at the observed count
is at most 1/300. Every comparison uses integers.

| Threshold h | Observed count | Lower probability |
|---|---:|---:|
| 2 | 100 | 0.944558 |
| 4 | 98 | 0.905985 |
| 6 | 87 | 0.754564 |

Inverting the binomial upper-tail test gives each
one-sided confidence statement. A union bound makes
all three valid simultaneously with probability at
least 0.99. The three indicators are dependent; the
union bound does not require their independence.
Therefore

```text
E[G*] >= 2(0.944558+0.905985+0.754564)
      = 5.210214,
mu_52 >= 172.08938482352943.
```

The observed improvements remain fully deterministic
instance certificates. Only their extension to the
uniform population invokes a sampling model. The four
selected second-campaign cases are not included.
The fixed sample size and equal error allocation are
part of this procedure; repeated testing with fresh
procedures would require its own error accounting.

## Exact population barrier for the elementary LP

Delete the common suffix, leaving m active cards and
I=I2 of the active target. Let t_i indicate a twice-moved
card, and e_i record extra excursions above four for
other cards. A natural continuous formulation minimizes

```text
4m - 2 sum_i t_i + 2 sum_i e_i
```

subject to 0<=t_i<=1, e_i>=0, sum_i t_i<=I,
and sum(t_i on a decreasing triple)<=2.
Linearize each catalog implication with trigger S
and complementary group C as

```text
sum(i in C) e_i >= sum(i in S) t_i - |S| + 1.
```

For the old eight-move singleton implication, multiply
the right side by two. Equivalently, introduce separate
six- and eight-move indicators with the corresponding
implications. The same zero-excess assignment works.
Additional nonnegativity and exclusion constraints
between twice and extra-excursion variables also hold.

Every catalog trigger through six cards has size at
least two. If I<=m/2, choose

```text
t_i = I/m for all i; e_i = 0 for all i.
```

Each trigger of size r has sum(t_i)<=r/2<=r-1,
and each decreasing triple has sum(t_i)<=3/2<=2.
The assignment is feasible with objective B=4m-2I.
Conversely, sum(t_i)<=I and e_i>=0 imply objective>=B.
Thus this LP equals B throughout that population.

This result applies to this specified linearization,
not to every conceivable LP. In particular, an exact
convex hull of admissible maximum two-chain subsets,
lifted inequalities, or constraints on four-move cards
may reject the uniform fractional assignment.
Even adding every decreasing-chain inequality
sum(t_i on the chain)<=2 is outside this theorem.
A chain of length ten rejects a uniform value 0.4.
Such inequalities have polynomial-time separation by
a maximum-weight decreasing-subsequence calculation.

### Count the stuck population exactly

Let A_m(i) be the RSK shape count of permutations with
I2=i, computed as the sum of squared hook dimensions.
The number having precisely m active cards and active
I2=i is A_m(i)-A_(m-1)(i-1). Consequently the exact
number covered by the preceding sufficient condition is

```text
1 + sum(m=2..52) sum(2i<=m)
    [A_m(i)-A_(m-1)(i-1)].
```

The leading one includes identity. The result is

```text
80522220015023436227702926747806497521043143546963335914771952980063
/
80658175170943878571660636856403766975289505440883277824000000000000
= 0.9983144280709016.
```

The computation takes about 22 seconds and visits
partitions rather than permutations. It agrees with
independent exhaustive active-suffix counts through n=7.
See [the population artifact](../results/third-ensemble-population.json).

Even on the remaining population, this catalog has
the proved 4m-4 ceiling: an increasing pair activates
no decreasing pair trigger or larger trigger; reversal
already has B=4m-4. The artifact also counts the weighted
upper limit on any mean improvement from this LP:

```text
sum(m,i:2i>m) (2i-4)
  [A_m(i)-A_(m-1)(i-1)] / 52!.
```

That ceiling is 0.08499479894055594 moves. Thus even
optimally solved instances of this elementary LP cannot
raise its expected certificate above 166.96416562247.

Parity rounding does not remove the ceiling, since
both B and 4m-4 are even. Spending more time solving
this formulation cannot recover the observed six-move
typical gains of integer subset enumeration.

## Stronger chain constraints: a compact certificate

The root review suggested separating all decreasing-chain
inequalities, beyond triples. A bounded follow-up used
the first frozen holdout target, whose B is 170 and I2
is 19. Start with all five-card zero-excess trigger
inequalities and repeatedly add violated decreasing
chains, found by a maximum-weight subsequence DP.

After 22 solves and 260 chain cuts, the zero-excess LP
has floating maximum twice-card count 16.5, below 19.
Its positive trigger duals select eight trigger types.
Retain their 132 projected group occurrences, restore
the extra-excursion variables, and maximize

```text
V = sum_i t_i - sum_i e_i.
```

The resulting floating lower distance is 175. More
significantly, its dual has an exact sparse certificate:
eight group rows and nine decreasing-chain rows, with
weights one or one-half. Their weighted right sides
sum to 16.5. Every t_i has combined coefficient at
least one; every e_i has coefficient at least minus one.
Because these variables are nonnegative, the weighted
inequalities imply V<=16.5 and hence

```text
d >= 4*52 - 2*16.5 = 175;
d is even, so d >= 176.
```

The actual plan supplies t_i in {0,1}: t_i=1 precisely
for cards moved twice. For all other active cards use
e_i=(moves_i-4)/2; set e_i=0 on the twice-moved cards.
Thus d=4m-2 sum(t_i)+2 sum(e_i). Every decreasing
chain contains at most two twice-moved cards, and every
catalog group implication gives its stated linear row.
The dual therefore applies to arbitrary legal executions.

The certificate does not need the total-I2 or singleton
rows: all 17 nonzero rows are projected group or chain
inequalities. A separate verifier reconstructs each
five-card pattern from its actual card labels, checks
membership in the certified finite catalog, checks each
chain against target positions, and recomputes all loads
and the objective with fractions. Invalid right sides
are rejected by a corruption test. The verifier uses
explicit exceptions and remains effective under
`python3 -O`; it rejects malformed targets, coefficient
types, negative weights, invalid chains, and altered
claimed bounds. Verification does not import SciPy.

See [the sparse certificate](../results/third-ensemble-extras.json)
and [the separation pilot](../results/third-ensemble-chains.json).
The two stages take approximately 6.6 and 2.4 seconds.
This matches the previous integer certificate 176; it
does not improve that instance or supply a faster solver.
It does supply a compact, separately verifiable proof,
and shows why the elementary-LP barrier must be scoped
carefully. No broader pilot was launched after this gate.

## Failed shortcuts and explicit counterexamples

Positive d-B is not hereditary upward under pattern
containment. The five-card pattern (2,1,4,3,0) has
d=14 and B=12. Inserting a card yields (2,1,5,4,0,3),
which has d=B=16. Deleting card 3 and standardizing
recovers the five-card pattern. Therefore the event
"contains a positive-gap pattern" does not imply B+2.
One cannot count such occurrences and add their local
gaps to the global B.

RSK shape alone also does not determine the gap:
(2,1,4,0,3) and (2,1,4,3,0) both have shape (2,2,1),
zero common suffix, and B=12; their distances are 12
and 14. Joint tableau and local-pattern data would be
needed beyond the existing hook-weight distribution.

Nor can shape and a fixed local pattern be treated as
independent. Among six-card permutations, the pattern
on ranks 0..4 equals (2,1,4,3,0) with probability 1/120.
Conditional on global I2=4 it occurs in 3/206 cases;
conditional on I2=5 it occurs in 3/356 cases; at I2=6
it occurs in none of 132 cases. These are exact counts.

The reproducible counterexamples and counts appear in
[this artifact](../results/third-ensemble-counterexamples.json).

## Candidate counting and entropy gates

For a fixed k-card subset of a uniform n-card target,
its induced order is uniform. The number of orders
covered by two increasing chains is Catalan(k).
Thus the expected total number of k-card candidates is

```text
binomial(n,k) Catalan(k) / k!.
```

At n=52 this is approximately 339941 at k=20,
22018.8 at k=22, and 886.45 at k=24. A plain union
bound over candidate twice sets is consequently weak
near the typical maximum size. To make it useful,
one must count the surviving candidates after group
constraints, suppressing their multiplicity by several
orders of magnitude. Marginal obstruction frequencies
or an independence assumption do not supply that count.

A word-count argument aimed at a 274-move threshold
needs effective growth roughly exp(log(52!)/274)=1.76943
per move, before finite prefactors. The existing local
canonical language has a mean threshold near 156.
This is a scale check, not an impossibility theorem
for improved canonical representations.

No exact, nonnegligible positive population gain above
E[B] was established here. Tiny direct-sum embeddings
would give positive but negligible gains and were not
promoted as progress. A useful next deterministic gate
is an exact count of marked two-chain candidates with
all zero-excess clauses enforced, or a representation
that constrains increasing twice pairs and four-move
cards. The present catalog cannot establish a target
above 4m-4, regardless of computation time.

## Reproduction and validation

For the statistical methods, see Clopper and Pearson
(1934), [exact binomial limits](https://doi.org/10.1093/biomet/26.4.404),
and Hoeffding (1963),
[bounded-sum inequalities](https://doi.org/10.1080/01621459.1963.10500830).
The paper's bibliography gives full references. Neither
source establishes the sampling premise for our data.

```sh
python3 tools/third_ensemble.py confidence \
  --output results/third-ensemble-confidence.json
python3 tools/third_ensemble.py population \
  --output results/third-ensemble-population.json
python3 tools/third_ensemble.py counterexamples \
  --output results/third-ensemble-counterexamples.json
python3 tools/third_ensemble.py chains --seconds 50 \
  --output results/third-ensemble-chains.json
python3 tools/third_ensemble.py extras --seconds 50 \
  --output results/third-ensemble-extras.json
python3 -m unittest test_third_ensemble.py
```

Five tests pass. They check exact binomial coverage
on small grids, the population count against exhaustive
permutations through seven, the frozen confidence
certificate, the recorded counterexamples, and the
17-row rational certificate with corruption rejection.
