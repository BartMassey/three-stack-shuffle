# Exact ensemble and canonical-word experiments

4 October 2026. This track implements Sections 3B–3E of
RESEARCH_PLAN.md. All numerical claims below refer to
preserved artifacts, not an inferred optimal distance.

## Exact distribution of the structural bound

Let A_m(t) count permutations of m cards with I2=t.
The Robinson–Schensted shape weights compute this exactly:
sum (f^lambda)^2 over shapes with lambda1+lambda2=t.
Set A_0(0)=1.

Targets whose maximal common bottom suffix leaves m
active cards correspond to permutations of m whose last
card is not fixed. Among all m-card permutations with
I2=t, the fixed-last cases number A_(m-1)(t-1): appending
the largest card increases I2 by one. Consequently the
active count is A_m(t)-A_(m-1)(t-1), contributing to
B=4m-2t. Identity contributes one target at zero.

The program checks nonnegativity, each active population
m!-(m-1)!, total population n!, and the distribution
against exhaustive targets through n=6. The n=52 mean
agrees exactly with the previously stored rational bound.

## A smaller covering language

Write w=DA DB AD BD and v=DB DA BD AD. Both swap the
top two D cards and restore the sides, with the same
legality condition: at least two cards on D. Hence w=v
as partial transformations and ww is identity.

Choose the lexicographically least shortest word for
each target, ordering DA before DB. It has no adjacent
inverse, no factor v, and no factor ww: each would admit
a shorter or lexicographically smaller legal replacement.
At every return to empty side stacks, its next nonempty
closed component starts DA. Otherwise reflecting that
component leaves its final D order unchanged and makes
the whole word lexicographically smaller.

These requirements hold simultaneously by minimality.
Thus every target has a shortest representative in the
language, even though the language still contains many
noncanonical words. Cross a forbidden-factor suffix
automaton with legal stack heights, requiring DA after
every empty-side state. The DP counts accepted closed
words exactly. Do not divide by two: reflection is
already normalized.

An independently represented physical-state/automaton
BFS verifies shortest representative coverage for every
target through n=6, including 117,910 product states at
n=6. This finite check validates the implementation;
the replacement argument supplies the covering proof.

At n=52 this yields:

- Counting mean lower bound 156.1873229093522.
- Counting maximum lower bound 158.

Both are weaker than the current structural results.
Let H(L) be the exact cumulative distribution count of
B, and U(L) the new covering-language count. Using
min(H(L),U(L),52!) in the even tail sum gives *exactly
zero* improvement over the structural mean. This is an
exact rational comparison, not rounding away a small gain.

Artifact: `results/campaign-counting-52.json`.
Reproduction:

```sh
python tools/campaign_counting.py --n 52 \
  --check-through 6 --output results/campaign-counting-52.json
```

The complete run took about 23 seconds. Further isolated
short relations were not pursued: the certified pilot
does not improve the priority n=52 lower bound. A larger
canonical structure would be needed before expansion.

## Ordinary pattern LPs

Deletion gives sum(x_i, i in S)>=d(pi restricted to S).
We selected 1,500 random size-nine subsets, solved the
LP, and used current card prices to guide single-card
subset replacements. A second solve supplies dual
weights. Forced active-card charges enter as singleton
constraints with right-hand side two.

Only the dual is used for certification. Nonnegative
weights are rounded down to integer multiples of 1e-9;
the common denominator is enlarged if needed so every
card's total weight is at most one. A separate verifier
recomputes every exact pattern distance and every card
load using integers, then evaluates the rational bound.
The result is rounded upward to even parity.

All 100 development certificates verify. Their mean
bound is 164.68, versus 166.98 for B. Taking the maximum
gives 167.26 and improves ten targets. Total computation
was 8.2 seconds, below 0.1 second per target. A separate
100-target n=6 check never exceeds the exact table.

Artifacts: `results/campaign-patterns-52.json` and
`results/campaign-patterns-validation-6.json`.
The script `tools/campaign_patterns.py` needs NumPy and
SciPy only for this experiment. Its LP interface follows
the [official HiGHS documentation][highs]. Runtime routing
and the conditional-card certifier do not depend on them.

The ordinary fractional formulation has the proved
32n/9 ceiling with size-nine tables, so further scaling
was not pursued after this modest gain. Rational duals
remain useful standalone instance certificates.

## Coarse color abstractions

Coloring cards preserves every move but forgets their
individual identities. Exact abstract distance is a
lower bound. We tried six fixed balanced two- or
three-color assignments at n=8: periodic, contiguous,
and two-card block patterns, then took their maximum.

All 40,320 concrete targets satisfy the resulting bound.
It improves B on 351 targets, raising its mean from
19.7918154762 to 19.809375; the color bound alone averages
17.9262896825. The largest abstract graph has 25,200
states. Artifact: `results/campaign-colors-8.json`.

The very small gain and exponential growth of balanced
color-word spaces do not justify direct scaling to 52.
Retaining blocker information remains an interesting
idea, but needs a different compact abstraction.

[highs]: https://docs.scipy.org/doc/scipy/reference/optimize.linprog-highs.html
