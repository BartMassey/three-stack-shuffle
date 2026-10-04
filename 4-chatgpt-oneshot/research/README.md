# Research notes and reproduction

Final research release, 4 October 2026, after four
bounded campaigns. No further experiment is scheduled.
Run commands from the `4-chatgpt-oneshot` directory.

## Read these first

- [Paper](../REPORT.md): model, constructions, proofs,
  performance, limitations, and bibliography.
- [Release summary](fourth-campaign-summary.md): final
  results, stopping decision, and verification commands.
- [Campaign summary](campaign-summary.md): completed
  experiments, negative results, and next priorities.
- [Second campaign](second-campaign-summary.md): stronger
  construction, residual bounds, and revised next gates.
- [Third campaign](third-campaign-summary.md): ternary
  recursion, cost partitioning, and ensemble limitations.
- [Living log](../STATUS.md): decisions and checkpoints.
- [Original plan](../RESEARCH_PLAN.md): historical baseline,
  proposed experiments, and an execution outcome table.

The files in [prompts/](../prompts/) are original problem
statements, not a statement of the latest research results.
Earlier numbered repository directories are separate
historical investigations; this index covers this project.

## Current evidence

| Topic | Detailed note |
|---|---|
| Sharpness of the numeric recurrence coefficient | [Recurrence closure](fourth-recurrence.md) |
| Fresh rational proofs and capped-search controls | [Certificate pilot](fourth-certificates.md) |
| Small endpoint portfolios and cancellation witness | [Portfolio pilot](fourth-portfolio.md) |
| Ternary recursion and lower host cost | [Ternary merge](third-ternary.md) |
| Residual consistency and cost partitioning | [Residual follow-up](third-residual.md) |
| Selected increasing-pair feasibility | [Pair follow-up](third-pairs.md) |
| Confidence, LP barriers, rational duals | [Ensemble follow-up](third-ensemble.md) |
| Direct-output merge and the 352 construction | [Oriented merge](oriented-merge.md) |
| Intermediate-state structural bounds | [Residual bounds](residual-bounds.md) |
| Improved XP exponent 2r+2 | [Parameter algorithm](parameter-complexity.md) |
| Pair/triple-triggered group constraints | [Obstructions](excursion-obstructions.md) |
| Parked endpoints and the 410 construction | [Execution](campaign-execution.md) |
| Conditional six/eight-move certificates | [Structure](campaign-structure.md) |
| Ensemble counts, LPs, color abstractions | [Counting](campaign-counting.md) |
| Target-specific exact search | [Exact search](campaign-exact.md) |
| Excursion schedules, XP, model mismatches | [Complexity](campaign-complexity.md) |

At n=52 the proved optimal maximum lies in [204,352].
The exact uniform-mean lower theorem remains 166.87917.
The parked controller's 318.874 mean and 340 maximum
are measurements on 1,000 held-out targets, not optima.
The separate 100-target lower holdout has mean certified
interval [172.56,317.92]. Stored target lists identify
the samples: sharing an RNG seed is not sufficient.
Hardness for this exact machine is still unproved.
The direct-output controller averages 284.72 moves on
100 new targets, or 273.8 with a wider split window.
This later sample is distinct from both original holdouts.

The new large-n construction uses `2n log3(n)+O(n)` moves.
Its finite routine through 64 is unchanged. A 99% lower
confidence bound of 172.08938 assumes IID-uniform targets;
it is separate from the exact mean theorem. The stronger
chain/group LP supplies a portable certificate of 176
on one target, matching its prior integer certificate.
The tested n=18 exact interval remains [64,66].

The fourth campaign proves the leading coefficient sharp
for the specified additive recurrence, not the machine.
All 20 fresh LP targets yield portable rational proofs;
higher-cap integer controls reverse most apparent LP
wins. A width-four endpoint portfolio saves 1.8 mean
moves on a separate 20-target pilot but costs about
7.4 times the host time. It remains research-only.

## Earlier studies

These retain their original proofs and measurements:

- [Radix constructions](algorithms.md).
- [Natural and patience merging](merge.md).
- [Bounded split search](hybrid-variants.md).
- [The original 444 guarantee](hybrid-bounds.md).
- [Exhaustive endpoint search through n=9](exact-search.md).
- [Original word-counting bounds](lower-bounds.md).
- [Local shortcuts: a negative pilot](shortcuts.md).

The planning notes on [lower bounds](next-lower-bounds.md),
[execution](next-execution.md), and
[hardness](next-hardness.md) explain the campaign's
derivation. The first includes the still-current exact
structural mean theorem. Proposed stages are not evidence
that every experiment was performed; consult the campaign
notes for outcomes and stopping gates.

[Stack-source notes](bibliography-stacks.md) and
[shuffle-source notes](bibliography-shuffling.md) record
source checks and model caveats. The paper contains the
consolidated bibliography for its citations.

## Dependencies and safe reproduction

Runtime and ordinary tests use Python 3.10+ and the
standard library; measurements used Python 3.13.5.
Two optional discovery tests use SciPy when available
and skip otherwise. `python3 -S -m unittest -q` verifies
the suite with third-party site packages disabled.
C++20 is needed for table generation and the exact-search
tool. NumPy and SciPy are optional dependencies of LP/MILP
experiments, not the controller or integer conditional
bound. Matplotlib regenerates figures. Pandoc renders
the report as offline MathML with embedded SVG and CSS.

Start with the bounded checks:

```sh
python3 -m unittest -v
python3 shuffle.py --n 52 --algorithm parked --summary
pandoc --defaults tools/report-pandoc.yaml
```

The committed JSON files are frozen evidence. For new runs,
prefer `build/` outputs to avoid replacing that evidence:

```sh
mkdir -p build
python3 tools/campaign_execution.py holdout --count 10 \
  --output build/parked-smoke.json
python3 tools/campaign_structure.py --n 52 \
  --paired-batch --samples 3 --seed 2026100401 \
  --deficit 2 --seconds 3 \
  --output build/lower-smoke.json
python3 tools/campaign_intervals.py \
  --input results/campaign-structure-52-paired.json \
  --output build/paired-intervals.json
```

The interval script reads Linux `/proc/self/status` to
record its process memory high-water mark. Timing and
timeout-sensitive certificates may vary by host. Only
completed search layers contribute new lower bounds.
Use the stored words and certificates to verify an
existing result instead of inferring it from a timeout.

For bounded exact-search validation:

```sh
mkdir -p build
c++ -O3 -std=c++20 -Wall -Wextra -Wpedantic \
  tools/campaign_exact.cpp -o build/campaign_exact
python3 tools/campaign_exact.py \
  --binary build/campaign_exact --mode validate \
  --seconds 15 --patterns 32 \
  --output build/exact-validation.json
```

The Python driver validates upper witnesses. The raw C++
interface instead trusts supplied lower and upper bounds;
it must not be called with conjectural certificates.
Do not interpret selected n=10–12 solutions as exhaustive
tables. Full BFS beyond n=9 is not part of this campaign.

The detailed notes contain full-run regeneration commands.
Some explicitly overwrite named `results/` artifacts;
change their output paths when comparing a new run.
Generated HTML, previews, build products, and Python
caches are ignored; source figures and evidence are kept.
