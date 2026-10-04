# Fourth campaign: fresh rational certificates

All 20 fresh 52-card targets produced exact rational
certificates within the fixed discovery budget. They
prove improvements of four to ten moves over B. This
extends the previous single 17-row pilot to a fresh
batch, but does not establish a stronger mean theorem
or show that LP discovery outperforms integer search.

The original comparison capped the integer improvement
at six moves. A subsequent, bounded check with a higher
integer cap reverses four of five selected LP wins.
One of those five retains a two-move LP advantage.

## Fixed experiment and budgets

The targets are the first 20 sequential shuffles of
`range(52)` by Python `random.Random(2026100404)`.
Targets were fixed before solving, with no replacements.
All 20 happen to have no common fixed bottom suffix.
The implementation supports suffixes by trimming them
and verifying the remaining active permutation.

Per target, LP discovery receives ten seconds and
`conditional_bound(deficit=2)` receives three seconds.
The latter searches three layers, with excess caps
3, 2, and 1, and can certify at most B+6. Its argument
does not mean a four-move cap. Both algorithms retain
only sound bounds when a deadline stops discovery.

The shared experiment guard is 420 seconds. Deadlines
are cooperative: solver calls and periodic enumeration
checks can slightly overshoot. No such overshoot affected
the total cap. The original batch took 119.782 seconds;
the selected fairness check took another 9.771 seconds.
There was no second batch or broader discovery search.

## Certificate pipeline

The code projects the existing certified five-card
catalog once per target. The first LP maximizes the
twice-moved card count with zero extra excursions,
including trigger inequalities and the structural I2
bound. A maximum-weight decreasing-subsequence DP adds
violated chain inequalities. Positive trigger duals
select the projected group rows for the second LP.

The second LP restores the extra-excursion variables
and maximizes `sum(t)-sum(e)`. It uses the selected group
rows, all discovered decreasing chains, I2, and singleton
upper bounds. The zero-excess phase reserves 2.5 seconds
of the ten-second budget for this second phase.

Floating solver output is only a discovery aid. Dual
weights become nonnegative fractions with denominator
at most one million before repair. If any e capacity
is exceeded, all weights are scaled down exactly.
Singleton weights then fill any missing t coverage.
The final denominators can exceed the initial cap.

The unchanged `verify_extra_certificate` reconstructs
each group pattern, checks catalog membership, validates
every decreasing chain, and recomputes all coefficients
and objectives using exact fractions. Each accepted
dual proves a rational bound, rounded upward to the
next even integer. No floating objective is accepted
as a certificate. These proofs depend on the existing
certified finite catalog and structural lemmas.

## Results

| LP improvement over B | Targets |
|---|---:|
| 4 | 4 |
| 6 | 8 |
| 8 | 6 |
| 10 | 2 |

The sample mean B is 165.3. The mean certified LP bound
is 171.9; the capped integer mean is 171.2. Taking the
stronger original bound per target gives mean 172.3.
LP wins nine original comparisons, ties seven, and
loses four. These comparisons include different caps
and discovery times; they are not an algorithm ranking.

LP discovery totals 115.113 seconds, with median 5.637
and maximum 7.613 seconds. Integer discovery totals
3.663 seconds, with median 0.133 and maximum 0.668.
Integer search completed all requested layers or found
a zero-excess witness, without timing out.

Three zero-excess chain-separation phases reached their
solver time limit: indexes 2, 6, and 11. Their final
group LPs still returned exactly verified certificates.
The remaining 17 chain phases finished separation.
There were no final LP unknowns, failed verifications,
unrun targets, or integer timeouts.

Certificates contain 13 to 50 nonzero rows, mean 29.35.
Verification inside discovery totals 0.0179 seconds.
A separate fresh-process verification of all 20 takes
0.0156 seconds, excluding process startup. This was run
with `python3 -S -O`, disabling site packages and Python
assertions. The portable verifier needs no SciPy.

### Higher-cap check on selected targets

The first five original LP-winning targets were rerun
using the existing integer method, ten seconds each,
`deficit=5`, allowing improvements through B+12.
This selected subset is not a new random sample.

| Index | B | LP | Integer +6 cap | Integer +12 cap |
|---:|---:|---:|---:|---:|
| 0 | 160 | 168 | 166 | 172 |
| 1 | 166 | 174 | 172 | 178 |
| 4 | 164 | 174 | 170 | 176 |
| 6 | 170 | 176 | 174 | 174 |
| 9 | 160 | 170 | 166 | 172 |

All five reruns completed without a timeout; their
times range from 0.315 to 3.651 seconds. Four original
LP advantages disappear under the higher integer cap.
Index 6 retains a valid LP176 versus integer174 gap.
This shows that the two certificate methods can differ
on an instance, not that either computes its optimum.
Taking every valid bound, including these selected
reruns, gives descriptive sample mean 172.9.

No population confidence calculation is performed.
The deterministic uniform-mean theorem is unchanged.
The seeded sequence alone supplies no ideal independent
uniform-sampling premise. This bounded experiment
supports portable certificates and records their cost.

## Artifacts, validation, and reproduction

- [Original batch](../results/fourth-certificates.json)
- [Higher-cap check](../results/fourth-certificates-fairness.json)
- [Independent verification](../results/fourth-certificates-verification.json)
- [Implementation](../tools/fourth_certificates.py)
- [Tests](../test_fourth_certificates.py)

Discovery used SciPy 1.15.3 and its HiGHS backend.
Timing and discovered dual bases can vary by machine.
The stored exact certificates remain independently
verifiable regardless of floating solver versions.
Every command below refuses to overwrite an output.

```sh
python3 tools/fourth_certificates.py \
  --output results/fourth-certificates-rerun.json
python3 tools/fourth_certificates.py \
  --fairness results/fourth-certificates-rerun.json \
  --output results/fourth-certificates-fairness-rerun.json
python3 -S -O tools/fourth_certificates.py \
  --verify results/fourth-certificates.json \
  --output results/fourth-certificates-reverified.json
python3 -m unittest test_fourth_certificates
python3 -O -m unittest test_fourth_certificates
python3 -S -m unittest test_fourth_certificates
```

Four tests pass normally and with assertions disabled.
They cover the frozen 17-row certificate and deliberate
corruption, exact tiny distances including a trimmed
suffix, selected five-card patterns, rational repair,
and deterministic target generation. Without SciPy,
the two discovery tests skip and the two standard-library
verification tests pass. Existing result files from
earlier campaigns were not modified.
