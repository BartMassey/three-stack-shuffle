# Three-stack shuffle

The research paper is [REPORT.md](REPORT.md). To render it:

```sh
pandoc --defaults tools/report-pandoc.yaml
```

This writes `REPORT.html` with embedded figures and CSS.
Math is rendered as native MathML, with no network scripts.
Use a current browser, such as Chromium or Firefox.
The Markdown uses `$...$` and `$$...$$`, which Pandoc's
default reader recognizes. A custom HTML conversion also
needs `--mathml` or `--mathjax`; recognizing math on input
and rendering it on output are separate settings.
Figure widths scale to the page, and the HTML stylesheet
preserves aspect ratios on narrow screens and in print.
The first bounded research campaign is complete. See its
[results and stopping gates](research/campaign-summary.md)
and the [research index](research/README.md).
[RESEARCH_PLAN.md](RESEARCH_PLAN.md) preserves the original
plan, with an outcome summary and links to the evidence.
The second campaign adds
[direct side-output merging](research/oriented-merge.md):
a **352-move** universal 52-card bound and a leading
move term `(4/3)n log2(n)`, rather than `2n log2(n)`.
Its fast version averages 284.72 moves on 100 fresh
targets; window-four search averages 273.8. These are
sample means, not optimality theorems.
See its [results and next gates](research/second-campaign-summary.md).
It can be rendered with the same stylesheet:

```sh
pandoc RESEARCH_PLAN.md -s --embed-resources --mathml \
  --metadata pagetitle="Three-stack research plan" \
  --css tools/report.css -o RESEARCH_PLAN.html
```

Executable exploration of the prompts in `prompts/`.
The original default is merge sort with optimal
small-block plans and a bounded search over merge splits.
It handles arbitrary target permutations, is optimal for
up to eight cards, and has O(n log n) card-move complexity.
The new optional `parked` mode solves the actual child
parking endpoints. It improves the proved 52-card bound
from 444 to 410 and saves about four moves on average,
at roughly twice the planning time. The faster default
is unchanged.

Every algorithm uses only AD, DA, DB, and BD transfers.
Each transfer moves one top card. Inputs and targets are
listed top to bottom. All cards finish on D; A and B finish
empty. There is no direct A–B transfer or extra card store.
The host keeps ordinary bookkeeping and a planned move list.

## Run

Python 3.10+ and its standard library are sufficient.
The supplied lookup tables need no compiler at runtime.
Development and measurements used Python 3.13.5.

```sh
python3 shuffle.py --n 52 --summary
python3 shuffle.py --n 52 --seed 42 --output plan.json
python3 shuffle.py --n 52 --algorithm parked --summary
python3 shuffle.py --n 52 --algorithm oriented --summary
python3 shuffle.py --n 52 --algorithm oriented_window --summary
python3 shuffle.py --n 4 --target 3 1 4 2
python3 shuffle.py --initial 40 10 30 --target 10 30 40
python3 -m unittest -v
```

The default labels are 1 through n. `--initial` overrides
that deck. Every emitted plan is replayed in an independent
simulator before output. Without `--summary`, JSON includes
the full move sequence as well as its count and target.

Three useful controller settings:

- `--algorithm fast`: balanced merge trees, both directions.
- `--algorithm recommended`: nearby merge splits, both
  directions. This is the default.
- `--algorithm thorough`: also search all contiguous splits
  for n<=64. This costs more CPU and can save more moves.

A fourth setting, `--algorithm parked`, retains the
recommended plan and adds exact parked-leaf candidates
in both directions. On a frozen 1,000-target holdout,
mean moves fell from 322.896 to 318.874 and sample maximum
from 350 to 340. Median planning rose from 53.5 to 104.2 ms.
Its 410-move guarantee is proved, not a sample estimate.
`solver.move_bound(52, "parked")` returns 410; the default
`solver.move_bound(52)` still returns 444.

These four use exact lookup through n=8, recognize identity
and reversal, and try a single merge pass when cards can
be distributed into two increasing subsequences. This
handles patterns such as disjoint adjacent swaps cheaply.
They retain the balanced construction's bound.
Above 64 cards, the search modes use balanced recursion.
The cutoff bounds host work; it is not a machine transition.

Two new research settings are now available:

- `--algorithm oriented`: direct-output merging with
  midpoint and certified splits in both directions.
- `--algorithm oriented_window`: also search midpoint
  plus/minus four at sizes up to 64.

Both keep the exact small-n and structured special cases,
and guarantee 352 at n=52. Above 64, both use balanced
oriented recursion, retaining the improved asymptotic
constant. `solver.move_bound(52, "oriented")` returns 352.
On 100 fresh targets the two versions average 284.72 and
273.8 moves, with maxima 300 and 288. Indicative median
planning is 8.8 and 134.9 ms. The existing default remains
unchanged for reproducibility; specify a new mode to use it.

For library use:

```python
from solver import recommended
from three_stack import Machine

initial = [1, 2, 3, 4]
target = [3, 1, 4, 2]
operations = recommended(initial, target)
machine = Machine(initial)
machine.run(operations)
machine.verify(target)
```

## Uniform shuffle versus routing

Fisher–Yates selects the target first, with an unbiased
integer choice at each step. Each of its n! equally likely
choice sequences gives a different permutation. Routing
that target deterministically therefore preserves uniformity.

The default uses `SystemRandom` for the integer choices.
The uniformity proof assumes ideal uniform random bits.
`--seed` selects a reproducible pseudorandom generator for
experiments, not a claim of exact randomness over seeds.
Benchmark means sample uniform targets, including the
identity when it occurs. There is no random-walk mixing
assumption or rejection of costly target permutations.

## The recommended construction

Map each card to its position in the desired target.
For a top segment of at most eight cards, look up a shortest
plan that sorts those ranks. Tables come from exhaustive
BFS of the full legal machine state graph.

For a larger segment:

1. Recursively sort its first part on D, then move it to A.
2. Recursively sort the second part on D, then move it to B.
3. Merge the two parts back onto D, taking the larger
   exposed rank each time. This builds increasing order
   from the bottom upwards.

Each recursive plan balances its use of all three stacks.
Previously parked cards lie below the active cards and
are never touched, so the same plans work with those
buffers present. Merge loops consume only their two
specified part lengths.

Each merge node costs at most twice its segment size.
Reflecting a child plan across A and B is free and legal.
Canceling adjacent inverse moves preserves every later
machine state. The controller chooses reflections that
make these cancellations more effective.

The default also considers splits within four cards of
the midpoint, caching interval results. It compares the
result with the balanced plan. Finally it tries planning
from target to initial and reverses/inverts that sequence.
That is another valid plan from initial to target.

Selecting one locally shortest interval plan is a heuristic:
different child plans can interact differently at their
boundaries. Even the all-splits mode is not globally optimal.

## Bounds and the reversal warmup

For n>=2, reversal is:

```text
DA repeated n-1 times
DB once
(AD, DB) repeated n-2 times
AD once
BD repeated n-1 times
```

The final D order is reversed, with exactly 4(n-1) moves.
The structural lower bound in the report proves this
optimal for reversal at every n.
For one card, do nothing. The implementation also accepts
the empty deck and uses zero moves.

For balanced hybrid merge, let B(n) bound the move count:

```text
B(n) = 4 max(0,n-1)                         for n<=8
B(n) = 2n + B(floor(n/2)) + B(ceil(n/2))     otherwise
```

The small cases are verified exhaustively. The recurrence
proves O(n log n) moves. At n=52 there are three merge levels
and eight leaves, four of size six and four of size seven:

```text
B(52) = 3*104 + 4*20 + 4*24 = 488 moves.
```

Accounting for guaranteed cancellations tightens the
52-card bound to **444 moves**. If a child plan ends in r
consecutive returns from one side, reflecting it to match
the next parking side cancels 2r moves. Exhaustive lookup
table analysis bounds the leaf cost minus those savings by
16 for six cards and 20 for seven cards. A larger nonempty
child always saves at least two moves when parked:

```text
B(13) <= 26 + 16 + 20 = 62
B(26) <= 52 + 2*(62-2) = 172
B(52) <= 104 + 2*(172-2) = 444
```

All recommended modes preserve this bound by retaining the
balanced candidate. `solver.move_bound(n)` computes it.
The exact table analysis is reproduced in
`research/hybrid-bounds.md` and checked by the tests.
Observed maxima do not replace this worst-case guarantee.

The Python balanced planner builds and copies move lists;
a conservative host-time bound is O(n log² n), with
O(n log n) plan storage. Bounded search through n=64 adds
a fixed-size regime. Exact lookup uses under 1 MB of table
files and costs O(n² + output length) for n<=8.

## Measurements and limits

Original development sample: 1,000 shared 52-card targets,
seed 20261003. These are not the later parked holdout:

| Algorithm | Mean moves | Sample max | Plan time |
|---|---:|---:|---:|
| Ordinary radix, simplified | 608.506 | 624 | 0.17 ms |
| Adaptive radix | 478.088 | 518 | 0.16 ms |
| Patience merge | 437.528 | 510 | 0.48 ms |
| Balanced hybrid | 363.164 | 402 | 0.18 ms |
| Fast controller | 357.254 | 386 | 0.34 ms |
| Recommended controller | 322.838 | 350 | 53.24 ms |

The default's sample mean standard error is 0.308 moves.
In a separate 50-target comparison, thorough averaged
310.68 moves (sample maximum 328), versus 324.04 for the
default on those same targets. Planning took about 0.57
seconds/deck for thorough. This smaller sample is stored
in `results/thorough-52.json`.

Scaling checks reached n=4096, averaging 78,734 moves with
the balanced controller on 20 targets. Structured tests
cover reversal, rotations, adjacent swaps, interleaving,
and reversed blocks. Records are `results/scaling.json`
and `results/structures-52.json`. Reproduce them with:

```sh
python3 tools/profile_algorithms.py --suite scaling \
  --output results/scaling.json
python3 tools/profile_algorithms.py --suite structures \
  --output results/structures-52.json
```

Full records are in `results/comparison-52.json`; ongoing
experiment history is in `STATUS.md`. Reproduce with:

```sh
python3 benchmark.py --n 52 --samples 1000 \
  --algorithms radix runs_radix_flexible patience \
  hybrid fast recommended \
  --output results/comparison-52.json
```

Every measured plan is simulated and checked. Reported
planning times exclude simulation and are indicative
single-host measurements, not isolated timing guarantees.
Seeded sample maxima are not exhaustive maxima.

Exact optimum means for n=8 and n=9 are about 20.3542 and
23.6293 moves; maxima are 28 and 32. Reversal is uniquely
hardest through eight cards. At nine, another target ties
it. No extrapolation to 52 cards is justified by this.

For n=52, the structural lower bound proves that the
optimal uniform mean is at least 166.87917 moves. Reversal
proves that the optimal maximum is at least 204. The
uniform bound is evaluated exactly over tableau shapes,
not estimated from random targets. Reproduce it with:

```sh
python3 tools/structural_bound.py --n 52 --check-through 9
```

There remains a large gap between lower bounds and our
constructive algorithms. Counting legal nonbacktracking
words independently gives the weaker mean bound 154.9453.
The campaign's canonical-word count improves this to
156.1873, still below the structural mean theorem.
Conditional excursion rules improve certified bounds on
a separate 100-target holdout from mean 166.86 to 172.56.
Upper plans on those exact targets average 317.92 moves;
the mean certified interval is therefore [172.56,317.92].
These sample bounds are not a new uniform-mean theorem.
We have not located the proposed phase transition or proved
the 52-card optimum. A counting bound rules out a universal
4(n-1) guarantee by n=212, without identifying the first
actual failure. Asymptotically, counting proves that both
optimal worst and uniform mean costs are Omega(n log n).

## Research map

- [Executed campaign](research/campaign-summary.md): results,
  certificates, negative experiments, and stopping gates.
- [Research index](research/README.md): historical notes,
  current results, dependencies, and reproduction routes.
- [Direct-output merging](research/oriented-merge.md):
  352 bound, endpoint contracts, and better asymptotics.
- [Residual bounds](research/residual-bounds.md): admissible
  structural bounds for intermediate physical states.
- [Improved XP algorithm](research/parameter-complexity.md):
  fixed event skeleton and exponent 2r+2.
- `parked_leaf.py`: exact parked endpoints and 410 bound.
- `tools/campaign_structure.py`: conditional card-budget
  certificates; standard-library Python, no LP solver.
- `tools/campaign_exact.cpp`: bounded target-specific exact
  search with arbitrary-state deletion pattern databases.
- `three_stack.py`: strict simulator, cancellation, reversal.
- `exact_solver.py`: shortest-plan lookup through n=8.
- `hybrid_merge.py`: balanced and all-splits hybrid plans.
- `hybrid_variants.py`: bounded split-search experiments.
- `solver.py`: controller choices and experimental registry.
- `radix_candidates.py`: stable radix constructions.
- `merge_candidates.py`: natural and patience merge variants.
- `shortcut.py`: bounded local BFS experiment; no observed
  benefit in its tested sample, excluded from defaults.
- `research/`: detailed proofs and experiment notes.
- `results/`: tables and machine-readable measurements.
- `tools/exact_search.cpp`: regenerate exact BFS tables.
- `tools/counting_bound.py`: reproduce lower bounds.
- `tools/structural_bound.py`: exact structural mean bound.

To regenerate the n=8 exact distances and plans:

```sh
mkdir -p build
c++ -std=c++20 -O3 -Wall -Wextra -pedantic \
  tools/exact_search.cpp -o build/exact_search
build/exact_search 8 results/exact-n8.json \
  results/plans-n8.json
```

The exact graph has n! (n+1)(n+2)/2 states. Full BFS is
restricted to small n; n=10 alone would allocate roughly
1.44 GB for the queue and distance arrays. It is never
part of runtime shuffling.
