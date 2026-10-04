# Autonomous campaign: results and handoff

4 October 2026. This is the synthesis of the first
bounded execution of RESEARCH_PLAN.md. It is not a claim
that the optimal 52-card costs or controller complexity
have been settled. The subsequent documentation pass
prepares this snapshot for the user's requested local
commit; it does not authorize a push.

## Main results

| Question | Before | After this campaign |
|---|---|---|
| Universal 52-card construction | 444 | 410 |
| Held-out mean, same 1,000 targets | 322.896 | 318.874 |
| Held-out sample maximum | 350 | 340 |
| Median planning time | 53.5 ms | 104.2 ms |
| Lower-bound mean on separate 100 targets | 166.86 | 172.56 |
| Exact uniform-mean lower theorem | 166.87917 | unchanged |
| Exact target search beyond tables | none | six n=10–12 targets |
| Controller hardness | unproved | still unproved; XP result |

The measured mean improvements and sample maxima are
not universal theorems. The 410 construction and the
individual lower certificates are proved bounds.

Use the new controller with:

```sh
python shuffle.py --n 52 --algorithm parked --summary
```

The faster default remains unchanged because neither
controller dominates the other in both moves and host
time. Both keep exact lookup through eight cards and
O(n log n) move complexity at large n.

## 1. Better constructions

The parent needs a child parked on a side, not sorted
on D. Computing that endpoint directly produces a
better small-block library. Every stored word was
replayed above protected stack bases, for both sides;
independent BFS agrees through n=5.

The exact parked maxima for sizes 1 through 8 are
1,4,7,12,15,20,23,28. Balanced composition proves 420.
Optimizing a fixed split tree gives 410, with a root
split 22+30 and four seven-card plus three eight-card
leaves. REPORT.md contains the inductive proof.

The frozen 1,000-target comparison gives 762 wins and
238 ties, with no losses because the original plan is
retained. The paired mean saving is 4.022, approximate
normal 95% interval [3.800,4.244]. Detailed timing scope
and all targets are preserved. The sample maximum 340
does not prove a universal 340 bound.

The local recursive caches now clear after extracting
the chosen word. This preserves outputs while promptly
releasing cached plans. A fresh 100-target certification
run used approximately 25 MB process high-water memory.
Some earlier getrusage maxima include inherited launcher
accounting; they must not be interpreted as measured
hundreds-of-megabytes savings from the cache change.

An additional shortest leaf maximizing its terminal
parking run was implemented and tested. It saved zero
moves on 20 development cases, while adding work, and
remains disabled. This is not an exhaustive negative
result for arbitrary boundary frontiers.

Proofs and implementation:
[execution note](campaign-execution.md),
[parked controller](../parked_leaf.py), and
[holdout data](../results/campaign-execution-holdout1000.json).

## 2. Stronger instance lower bounds

The first small structural gaps are joint obstructions.
An optimum may keep all cards within four moves, or
keep the maximum two-increasing set at two moves, but
cannot necessarily do both. Four five-card patterns
force an exceptional card to six or eight moves when
four specified cards remain twice-moved.

For a candidate twice-moved set, union the forced-card
masks. Counting each extra excursion once avoids the
double counting that would invalidate occurrence sums.
Enumerate near-maximum two-increasing sets, pruning when
their forced cost already meets the requested bound.
An interrupted layer contributes nothing new.

The implementation matches independent brute enumeration
on every target through n=7, and is exact through n=5.
The frozen 100-target lower-bound holdout has mean
172.56 instead of 166.86. Gains are +6 on 87 targets,
+4 on eleven, and +2 on two. Two three-second timeouts
retain their completed layers. These are certified
per-instance gains, not a uniform-ensemble theorem.

Root replay of a separate deeper certificate confirms
lower bound 178, versus its previous 166. Its new upper
plan has 338 moves. Thus the substantial remaining gap
is explicit, not hidden by the improved mean.

The lower-bound holdout uses repeated `shuffle`, while
the execution holdout uses repeated `sample`. The same
seed does not make them the same targets. We therefore
ran upper plans on the actual stored lower-bound targets:
mean interval [172.56,317.92], mean width 145.36, largest
per-instance ratio certificate below 1.989. These ratios
apply to the recorded targets only.

Quartet preprocessing now replaces enumeration of all
five-card subsets by rank/position bitmasks. Its output
matches the independent five-subset implementation on
all permutations through n=7 and a 52-card target.
The published frozen timing refers to the old method;
the optimization changes no mathematical rule.

There is a decisive ceiling: any two-card twice-moved
set triggers none of these four-card implications.
The relaxation therefore never exceeds 4m−4. It cannot
locate the crossover beyond that budget, no matter how
long it runs. Further rules must constrain executions
with few twice-moved cards, not just large such sets.

Proofs and certificates:
[structural note](campaign-structure.md),
[lower holdout](../results/campaign-structure-52-paired.json),
[paired intervals](../results/campaign-intervals-52.json).

## 3. Exact target-specific search

The eight-card PDB includes all distributions over the
three stacks. Its distance charges only retained cards;
mandatory moves of omitted cards can safely be added.
IDA* uses that bound, parity, and exact transposition
keys containing the previous move. Every accepted plan
is independently replayed.

Six selected n=10–12 targets were solved exactly, saving
two to six moves over their previous thorough plans.
The largest completed search used 2,370 nodes after the
approximately 0.35-second database build. Sixteen known
targets through n=9 and an interrupted-iteration case
validate the implementation and conservative bounds.

Nearly reversed targets at n=14,16,18 each reached the
45-second cap without exhausting the first threshold.
Their intervals remain [48,50], [56,58], [64,66]. The
n=20 job was stopped without a completed certificate.
No failure of the 4(n−1) universal budget was found.

The bottleneck is now clear: a strong endpoint bound
does not make a weak residual-state heuristic strong.
Further runs on that family need a new residual bound,
not a larger timeout. No full n=10 factorial BFS was run.

[Exact-search proof and reproduction](campaign-exact.md).

## 4. Controller complexity

A complete chronological excursion schedule is feasible
exactly when its D-only simulation succeeds and its
excursion crossing graph is bipartite. Coloring supplies
the side choices. This yields a constructive exact
primitive rather than a model-changing analogy.

With excess r above the mandatory m excursions, at most
r cards are exceptional and contribute at most 4r
events. Ordinary-card event order is fixed. Enumerating
exceptional identities and event interleavings gives
time f(r)(m+1)^(5r+2): XP in r, not FPT in r. The complete
excess-one algorithm matches all 873 targets through n=6.

The proposed initial-complete-drain normal form is false:
(2,3,1,4,0) costs 12 unrestricted but 14 under that rule.
This blocks a direct transfer from mandatory-loading
stack-network results. The targeted literature audit
did not establish exact-model NP-hardness. Strong
NP-hardness would exclude an FPTAS, but no-PTAS would
still need a separate relative-gap argument.

[Complexity proofs and counterexamples](campaign-complexity.md).

## 5. Other completed gates and limits

[The counting/abstraction note](campaign-counting.md)
records the full calculations and reproduction commands.

- Exact tableau/suffix enumeration computes the complete
  distribution of B at 52. Combining it with the improved
  canonical-word count gives exactly zero mean gain.
- Ordinary pattern LPs give verified rational duals in
  under 0.1 second per target, but only +0.28 mean moves
  when combined with B on 100 development targets.
  Their size-nine fractional ceiling is 186 at n=52.
- Six color abstractions improve the eight-card mean
  bound by only 0.01756. Their exponentially growing
  spaces do not justify direct 52-card expansion.
- The continuous conditional-budget LP has a uniform
  fractional solution that collapses to B on typical
  targets. A 30-second integer pilot supplied no stronger
  certified dual. The successful replacement is the
  exact combinatorial enumerator, not a solver timeout.

These are reasons to stop the tested expansions, not
claims that entire families of approaches are useless.

## 6. Verification and remaining decisions

The integrated suite passes 25 tests. These include old
optimal plans, parked bases and independent BFS/DAG,
the registered controller, conditional bounds, and the
quartet optimization. Runtime remains standard-library
Python. C++20 is used for finite BFS and exact search;
NumPy/SciPy only for separate LP/MILP experiments.

The strongest next research directions are now narrower:

1. A residual-state structural bound for exact search,
   including cards already on the sides.
2. Obstructions that force extra excursions even when
   only zero, one, or two cards are twice-moved.
3. A compact, sound interface for richer boundary plans;
   the tested terminal-run heuristic was insufficient.
4. A counting argument for the population passing the
   new conditional test, to convert instance gains into
   an exact uniform-mean theorem.
5. A cost-preserving hardness primitive consistent with
   the excursion model, or a stronger parameterized
   algorithm. Existing nearby reductions do not suffice.

REPORT.md, README.md, and STATUS.md incorporate the
verified results. The original research plan remains
as the campaign specification, with its execution status
linked at the top. No long search is left running.
