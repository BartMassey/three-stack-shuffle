# Third campaign: ternary recursion and lower-bound limits

Completed 4 October 2026. The second campaign was committed
as `d1f62f3` before this work began. This campaign is a
separate snapshot, now authorized for commit before the
fourth campaign. No push occurred.

## Main results

The best new construction has move bound
`2n log3(n)+O(n)`, with coefficient about 1.26186 on
`n log2(n)`, improving 4/3. Its 52-card algorithm and
352-move guarantee are unchanged. The deterministic
uniform-mean lower theorem also remains 166.87917.

| Stream | Result |
|---|---|
| Execution | Balanced ternary parked macro recursion |
| Host work | Compute only needed output contracts |
| Residual bound | Consistency proof; faster evaluation |
| Cost partition | Stronger admissible disjoint bounds |
| Increasing pairs | 371 selected cases all feasible |
| Ensemble | Conditional 99% lower confidence 172.08938 |
| Elementary LP | Mean gain at most 0.084995 |
| Stronger LP | Portable 17-row certificate for 176 |
| Exact n=18 | Still [64,66] after bounded follow-up |

## 1. A better merge tree

The previous endpoint contracts satisfy

```text
U(a+b) <= P(a)+P(b)+a+b
P(a+b) <= P(a)+U(b)+2a+b.
```

Use a parked first child of size about n/3, then expand
the central second child into two equal parked children.
The result is three nearly equal parked children with
total merge toll 2n. Summing over ternary levels proves
the new bound. No new physical operation is required.

`--algorithm ternary` implements this recursion above 64,
computing only the requested endpoint. A separate root
early return removes an unused parked calculation from
the old oriented planner and preserves every chosen word.
Regression tests reproduce all 100 frozen holdout words.
The original default remains unchanged.

On three paired 4096-card targets, ternary averages
55,114 moves versus 56,854.67 for oriented; indicative
median planning time is 333 ms versus 1007 ms. These
are small scaling checks, not population estimates.

See the [construction and measurements](third-ternary.md)
and [independent review](third-ternary-review.md). Review
caught an incorrect finite-leaf constant; the corrected
proof uses P(k)<=8k, leaving the leading term unchanged.

## 2. Residual bounds and exact search

The residual structural bound R is consistent, so
pathmax cannot strengthen it. Partitioning cards into
P,Q and adding an exact pattern distance on P to R(Q)
does strengthen the old maximum of the two bounds.
Fixed partitions are consistent. Choosing one partition
adaptively is admissible but can be inconsistent.

Full checks cover 23,115 states and 68,812 directed
edges through six cards. On n=6, full cost partitioning
improves 2002 states; a cheaper selected-partition rule
improves 936. The latter has 166 consistency violations,
which are recorded rather than hidden.

Sparse tail-pair updates and reusing successor estimates
accelerate the unchanged R. The exact n=14 regression
retains all 671,869 search nodes and optimum 50 while
total time falls from 4.917 to 2.230 seconds. At n=18,
the ten-second pilot visits about 2.7 times more nodes.
The justified 45-second follow-up visits 15,142,912
nodes but completes no new threshold. Full partitioning
is too costly there, and staged partitioning also fails
its ten-second gate. The certified interval stays [64,66].

See [the proof, checks, and search records](third-residual.md).
More wall time alone is not the next experiment.

## 3. Increasing-pair obstruction search

The existing group catalog cannot constrain increasing
twice-moved pairs, leaving a 4m−4 relaxation ceiling.
We tested the hardest known seven-card targets, the
complete class with at most three increasing pairs,
and the catalog's eight remaining seven-card misses.

All 371 distinct selected target/pair cases on 94 targets
are feasible: both selected cards move twice and every
other card at most four times. There are 409 stored,
replayed witnesses across overlapping batches. The
oracle also agrees with an independent implementation
on all 1200 five-card target/pair cases. This is not
an exhaustive seven-card result or a general theorem.

See [the pair study](third-pairs.md). No blind expansion
of the obstruction dictionary was justified by this gate.

## 4. Ensemble bounds and compact certificates

Two conclusions must remain separate.

First, under an IID-uniform sampling model, the old
unselected 100-target lower-bound sample supports a
99% lower confidence bound of 172.08938 on the optimal
uniform mean. Exact binomial inversion at three gain
thresholds shares a total error budget of 0.01. Adaptive
certificate searches are safe because each gain is a
pointwise underestimate of a fixed latent statistic.
Seeded pseudorandom sampling does not itself prove the
ideal sampling premise. The deterministic theorem is
still 166.87917, not 172.08938.

Second, the elementary fractional twice-card/extra-move
model with decreasing triples and current group clauses
equals the structural bound whenever I2<=m/2. Exact
shape/suffix counts put 99.83144% of 52-card targets in
this class. Even optimistically treating the remainder,
this relaxation can improve the uniform mean by at most
0.084995. Adding all decreasing-chain constraints avoids
that particular barrier.

A chain-separated pilot yields an exact rational dual
with eight group rows and nine decreasing-chain rows.
It proves one target needs at least 175 moves, hence
176 by parity. This matches the prior integer result;
it does not improve the numerical bound or discovery
time. The useful output is a small standalone proof.
The verifier reconstructs the rows and rational loads,
needs no SciPy, and rejects malformed certificates even
with Python assertions disabled.

See [the ensemble study](third-ensemble.md) for definitions,
exact counts, certificates, and failed counting shortcuts.
In particular, positive structural gap is not preserved
under pattern containment, and local patterns cannot be
assumed independent of RSK shape.

## 5. Next bounded experiments

1. Try compact chain/group duals on a small fresh batch.
   Measure discovery and verification separately. Proceed
   only if stronger bounds or cheaper certificates result.
2. Seek deterministic ensemble gains using full chain
   constraints and joint marked-candidate counts. Do not
   spend more time on the disproved elementary-LP route.
3. Model four-move cards or construct targeted larger
   increasing-pair examples. Small-n feasibility alone
   does not predict 52-card behavior.
4. For n=18, require a stronger initial certificate or a
   materially different search representation before
   another long search. Current speedups did not settle it.
5. Investigate boundary-compatible endpoint portfolios
   with an explicit host-time budget. Ternary recursion
   improves large n, not the present 52-card move counts.

Controller hardness is still open. None of these results
proves NP-hardness, excludes a PTAS, or upgrades XP to P.

## Reproduction

Each linked note records its bounded commands and frozen
JSON artifacts. Use new `build/` destinations for reruns.

Final integration passes all 51 test groups in 16.359s,
plus the five ensemble groups under Python `-O`.
The C++ build is warning-clean. Root independently
repeats the exact n=14 threshold proof with the same
671,869 nodes. All 109 JSON artifacts parse and all
113 local links in the 12 current documents resolve.
Both Pandoc outputs render native MathML without errors.

```sh
python3 -m unittest -v
python3 -O -m unittest test_third_ensemble -v
python3 shuffle.py --n 512 --algorithm ternary --summary
pandoc --defaults tools/report-pandoc.yaml
```
