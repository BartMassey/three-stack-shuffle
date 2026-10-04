# Second campaign: direct outputs and residual bounds

4 October 2026. This continues the
[first campaign](campaign-summary.md). The user has now
requested this completed snapshot's local commit before
a third campaign. No push is authorized. Frozen
first-campaign artifacts remain unchanged.

## Main results

| Question | Before | Second campaign |
|---|---|---|
| Universal 52-card upper bound | 410 | 352 |
| Balanced leading move term | 2n log2(n) | (4/3)n log2(n) |
| Fast new controller | unavailable | mean 284.72 |
| Window-four new controller | unavailable | mean 273.80 |
| Uniform-mean lower theorem | 166.87917 | unchanged |
| Nearly reversed n=14 | [48,50] | exact 50 |
| Nearly reversed n=16 | [56,58] | exact 58 |
| Nearly reversed n=18 | [64,66] | unchanged |
| XP exponent in excess r | 5r+2 | 2r+2 |

The two new controller means use 100 fresh targets, seed
2026100402. The previous parked controller averages
318.74 on those same targets. These are sample means,
not optimal means or universal mean upper bounds.
Neither NP-hardness nor an unrestricted polynomial-time
algorithm has been established.

## Direct side-output merging

The previous composite parked routine sorted onto D and
then moved the result to its side. Instead, recursively
park its first child on the opposite side in increasing
order, sort its second child on D in increasing order,
then merge the smaller exposed ranks onto the output
side. A side card costs two transfers and a D card one.

For central and parked costs U and P, respectively:

```text
U(a+b) <= P(a)+P(b)+a+b
P(a+b) <= P(a)+U(b)+2a+b.
```

Exact leaves through eight give U(52)<=352. Balanced
recursion gives the leading constant 4/3. Independent
review checked endpoint orientation, every protected
base, the recurrence, and its all-size analysis.

Both new settings win all 100 held-out comparisons.
Fast mean/max are 284.72/300; window-four gives 273.8/288.
Indicative median planning is 8.8/134.9 ms. At n=4096,
three replayed cases average 56,882.67 moves versus
78,710 for the old parked fallback. The new controller
takes about 1.05 seconds versus 0.11 seconds there:
the large-n host-time tradeoff differs from n=52.

On the original lower-bound holdout, the new window mode
gives mean interval [172.56,271.76], width 99.2, versus
the previous [172.56,317.92]. The largest per-instance
upper/lower certificate ratio is 1.69412. These claims
refer only to those recorded 100 targets.

Use `--algorithm oriented` or `--algorithm oriented_window`.
The existing default is unchanged. See
[construction and proof](oriented-merge.md),
[independent review](oriented-review.md), and
[paired intervals](../results/oriented-intervals-52.json).

## Residual-state lower bound

After deleting a correct bottom D suffix, assign mandatory
cost one to each initial side card and two to each D card.
Cards achieving exactly those minima obey coupled order
constraints: two increasing D subsequences, decreasing
initial-side subsequences, and strict separation between
each side's retained D and initial-side ranks.

Maximizing the number K of such ordinary cards gives
mandatory cost plus twice the number of remaining cards.
This generalizes the endpoint structural bound. A weighted
tail-pair DP evaluates it in O(n³), and it combines with
the PDB bound by taking a maximum, not adding overlapping
charges.

Exhaustive Python/C++ checks cover all 23,115 states through
n=6. An independent subset implementation checks the
relaxation through n=5. The new bound settles the old
n=14 target at 50 and n=16 at 58 by exhausting the lower
thresholds and retaining replayed upper witnesses.
The n=16 run takes about 43.66 search seconds. The n=18
ten-second pilot still cannot complete its threshold;
no larger expansion was performed after the n=16 gate.

See [proof and experiments](residual-bounds.md),
[independent review](residual-review.md), and
[n=16 certificate](../results/residual-search-n16-45.json).
The C++ switch remains optional and off by default.

## Group obstructions

The new finite catalog includes pair and triple triggers:
if certain cards remain twice-moved, at least one card in
a specified group must move six times. Charge disjoint
groups to avoid double counting. Five-card rules repair
the earlier unique six-card miss; all bounds through six
cards now match exact distances. The full catalog through
six matches 5032 of 5040 seven-card optima.

Four deliberately selected 52-card targets gain 2,2,2,4
moves over their old certificates. This is not a random
sample or a new ensemble lower theorem. Every pair trigger
in the complete catalog is decreasing. An increasing
two-card candidate therefore still evades the clauses,
so the relaxation retains its 4m-4 ceiling. Larger search
budgets cannot remove this representational limit.

See [proof and selected pilot](excursion-obstructions.md).

## Fixed excess versus unrestricted polynomial time

Every plan has a fixed skeleton: all first departures in
initial order, then all final returns in reverse target
order. Its extra events are balanced temporary D blocks
inside skeleton gaps. Enumerating their labels and gaps
gives f(r)(m+1)^(2r+2), improving the old XP exponent.
Exhaustive checks recover every optimum through six cards.

For fixed r this is polynomial. For unrestricted inputs,
r grows and so does the exponent. Binary search over K
needs only logarithmically many calls; the unresolved
problem is making each such decision call inexpensive.
Even a linear upper bound would not cure the parameter
dependence. See [the theorem](parameter-complexity.md).

## Next gates

1. Improve the residual bound or its evaluation before
   extending the stalled n=18 search. The n=16 run already
   consumed nearly all of its 45-second allowance.
2. Find constraints on increasing two-card twice sets,
   or directly constrain the four-move cards. The current
   catalog cannot witness the proposed phase transition.
3. Seek a population argument for the conditional gains;
   selected instance improvements are not an ensemble proof.
4. Investigate whether balanced-block schedules permit
   FPT compression or a genuine hardness reduction. The
   smaller XP exponent alone settles neither question.
5. Profile oriented interval caching at large n. Keep
   host time and physical moves as separate objectives.

All experiments at these gates have stopped. The project
retains reproducible tools and explicit negative results;
no account-wide usage percentage was available.
Final verification passes 36 tests, 130 local Markdown
links, result JSON parsing, and Pandoc MathML rendering.
