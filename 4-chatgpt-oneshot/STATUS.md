# Work status

Updated: 2026-10-04. First autonomous campaign completed.
Tests, certificate replay, documentation links, and browser
rendering are verified. No searches remain running.
The user authorized
execution without intervention; bounded experiments and
their stopping gates are recorded in
`research/campaign-summary.md`.

## Objective and constraints

Explore algorithms for the A–D–B three-stack machine.
Only adjacent top-card transfers count as operations.
Every run starts and finishes with all cards on D.
Realize any specified permutation, with uniform shuffling
implemented by selecting a uniform target permutation.

Prioritize clear algorithms, practical host computation,
low mean and maximum moves at n=52, and reasonable
large-n asymptotics. Use code for exhaustive searches;
do not extrapolate tiny-n results to 52 cards. Investigate
the reported change in behavior across deck sizes.

The user has authorized a local commit of this project
and relevant documentation. No push is authorized.
Exclude the unrelated untracked `../3-generic/` work.
No unsafe code.
No account-wide usage percentage is available to this
session, so the requested approximate 20% usage cutoff
cannot currently be measured. Keep experiments bounded.

## Completed work

- Root: simulator, controller, CLI, tests, and benchmarks.
- algorithms agent: radix and exact-leaf hybrid methods;
  independent final integration review found no blockers.
- exact_search agent: BFS, lookup tables, counting bounds,
  local-shortcut experiment, refined hybrid upper bound.
- adaptive_merge agent: natural/patience merge, partition
  DP, and bounded hybrid split-search comparisons.

## Current recommendation

Run `python3 shuffle.py --n 52 --summary`.
See `README.md` for usage, proofs, and reproduction.

Default: exact lookup through eight cards; otherwise
merge sort with exact leaves, midpoint +/-4 split search,
reflection, cancellation, and forward/inverse candidates.
A cheap single-pass candidate helps structured decks.
Search falls back to balanced recursion above 64 cards.

Original development sample (seed 20261003):

| Mode | Targets | Mean | Sample max | Plan time |
|---|---:|---:|---:|---:|
| fast | 1000 | 357.254 | 386 | 0.34 ms |
| recommended | 1000 | 322.838 | 350 | 53.24 ms |
| thorough | 50 | 310.680 | 328 | 571.73 ms |

All use the same seed; thorough's 50 targets are the first
50 of the larger sample. The default mean on those same
50 is 324.04. Sample maxima are not worst-case proofs.
Timing excludes simulation and is indicative for this host.
The final structured-deck shortcut changes none of these
1000 random plans; timing was recorded before that addition.

Proved bound for every 52-card target: **444 moves**.
The new optional `--algorithm parked` improves this to
**410 moves**, retaining the old plan as a candidate.
Proved optimal maximum lower bound: **204 moves**.
Proved optimal uniform mean lower bound: **166.87917**.
Reversal is optimal at exactly 4(n-1) for every n.
All modes use O(n log n) moves, with conservative balanced
host time O(n log² n) and plan storage O(n log n).
Full interval DP is bounded to 64 cards and is heuristic,
not a globally optimal controller.

Verification: all 25 unittest groups pass, including all
46,233 exact-table plans, independent simulator checks,
untouched stack buffers, arbitrary initial labels,
recursive merges, and Fisher–Yates choice enumeration.
All benchmark plans were independently simulated.
Scaling samples reached 4096 cards; structured tests
include identity, reversal, rotations, adjacent swaps,
interleaving, and reversed blocks.

Artifacts: `results/comparison-52.json`,
`results/thorough-52.json`, `results/scaling.json`,
`results/structures-52.json`, and `results/hybrid-bounds.json`.
No commits or pushes were made during the research
campaign; the user has now requested its local commit.

## Research log

- 2026-10-04 documentation and commit preparation: refreshed
  the project and repository indexes, marked historical
  studies and plans with their current successors, and
  reconciled completed campaign status. Preserved original
  measurements and prompts. Added research navigation and
  reproduction guidance. Generated HTML, local previews,
  build products, and unrelated sibling work are excluded
  from the requested local commit; no push is authorized.
- Documentation verification: all 25 tests pass (12.509s).
  All 98 local Markdown links resolve. Pandoc renders both
  the report and plan without warnings; the report has
  67 MathML expressions and no MathML error elements.
  Local editor autosaves are preserved but excluded along
  with generated previews. No research result was rerun
  or replaced during this documentation pass.
- Campaign integrated result: optional parked controller
  has a proved 410 bound (root split 22+30; four seven-card
  and three eight-card leaves). Frozen holdout of 1,000:
  recommended mean 322.896/max 350 versus parked 318.874/
  max 340; 762 wins, 238 ties, mean saving 4.022, approximate
  95% CI [3.800,4.244]. Median planning 53.5 versus 104.2 ms.
  Default unchanged because this is a moves/time tradeoff.
  All words replayed. Cache clearing in new and old interval
  searches preserves words; fresh 100-case run VmHWM 25 MB.
- Campaign integrated result: conditional six/eight-move
  obstructions give mean bound 172.56 versus 166.86 on
  100 distinct held-out targets. Gains 87×6, 11×4, 2×2;
  two three-second timeouts retain completed layers. This
  is not a new exact uniform-mean theorem (still 166.87917).
  Root paired actual target upper plans: mean 317.92,
  interval width 145.36; all 100 upper/lower ratios <1.989.
  Root independently reproduced one 166→178 certificate.
  Quartet preprocessing matches an independent five-subset
  reference on all targets through seven and a 52-card case.
  This relaxation cannot exceed 4m−4: a two-card twice set
  triggers no four-card rule. Larger searches alone cannot
  locate the transition.
- Campaign integrated result: all six selected nonreversal
  n=10–12 targets solved exactly, saving 2–6 moves. Full-state
  eight-card PDB plus disjoint omitted-card charges is
  admissible; 16 validation targets and an interruption
  test pass. n=14/16/18 nearly reversed targets time out
  after 45 seconds with unchanged intervals [48,50],
  [56,58], [64,66]. Stopped n=20 externally without claiming
  a completed certificate.
- Campaign gates: canonical count + structural CDF gains
  exactly zero; ordinary pattern LP gains only 0.28 in
  sample mean with a 186 ceiling; color abstraction gives
  tiny n=8 gain; terminal-run alternatives save zero moves
  on 20 cases and stay disabled. Scaling stops at these gates.
- Campaign delivery: REPORT.md now includes the parked
  algorithm, 410 proof, conditional bound, XP theorem,
  exact search and holdout results. README exposes the new
  mode; RESEARCH_PLAN links the completed campaign summary.
  Detailed proofs/results are `research/campaign-*.md` and
  `results/campaign-*.json`. All 25 tests pass (12.5s).
  Final Chromium checks at 1280px and 390px: 67 MathML
  expressions, zero math errors, responsive SVG widths,
  no page overflow. Report and campaign artifact links
  resolve. Root independently replays every upper witness
  in the exact-search pilot, validation, and larger cases.
- Campaign checkpoint: exact structural-bound distribution
  through n=52 computed by tableau shapes and maximal-suffix
  subtraction. Canonical legal-word counting now forbids a
  top-swap duplicate and doubled swap, and reflects each
  closed component separately. Product-state BFS verifies
  shortest-word coverage for all targets through n=6.
  Counting mean improves to 156.1873229093522, but combining
  its CDF with the structural CDF gives exactly zero gain
  over 166.8791708235294. Preserved the negative result in
  `results/campaign-counting-52.json`; no further small-rule
  expansion is justified without a stronger normalization.
- Campaign checkpoint: ordinary adaptive size-nine pattern
  LPs produced 100 exactly verified rational duals in 8.2s.
  On the development sample, structural mean 166.98 becomes
  167.26 by taking the maximum, improving 10/100 targets.
  This is a paired sample result, not a uniform-mean theorem.
  Stored certificates in `results/campaign-patterns-52.json`.
- Campaign checkpoint: parked-leaf agent independently
  replayed 92,466 protected-base words (both sides) and
  checked BFS against an independent implementation through
  n=5. Proposed universal 52-card bound 420 for balanced splits,
  410 with optimized fixed splits. Root review subsequently
  completed; the proof and tests are integrated above.
- Campaign checkpoint: complexity agent proved exact
  event-schedule feasibility via bipartite excursion
  crossing graphs and an XP algorithm in excess excursions
  r above 2m, with time f(r)(m+1)^(5r+2). Excess-one program
  matches all 873 targets through n=6. A smallest drain-first
  counterexample at n=5 disproves that normalization. Notes:
  `research/campaign-complexity.md`. Target-specific exact
  search with deletion PDBs was subsequently completed;
  see the integrated result above.
- 2026-10-04 campaign start: retained the verified controller
  and all previous artifacts. Created independent workstreams
  with separate files. Development seed remains 20261003;
  the execution holdout seed is frozen at 2026100401 before
  tuning. Initial per-search cap 60 seconds, typically
  <=1 GiB, with progress gates before larger batches.
- 2026-10-04: fixed Pandoc input math delimiters and added
  reproducible offline MathML output, embedded SVG/CSS,
  responsive figure sizing, and narrow-screen equation
  scrolling. Build: `pandoc --defaults tools/report-pandoc.yaml`.
  The output is ignored `REPORT.html`; source stays Markdown.
  Final Chromium checks at 1280px and 390px show no page
  overflow, two correctly scaled SVGs, 62 MathML nodes,
  zero math errors, and all six equation labels visible.
  Replaced TeX tags with portable explicit labels because
  Pandoc's MathML conversion otherwise drops the tags.
  Visually checked desktop equations and phone layout.
  Also generated ignored `RESEARCH_PLAN.html`.
- 2026-10-04: delegated independent lower-bound, execution,
  and complexity reviews. Synthesized `RESEARCH_PLAN.md`:
  lower bounds first; bounded experiments, resource gates,
  proof obligations, and no-PTAS versus NP-hardness audit.
  Detailed notes are `research/next-*.md`.
- 2026-10-04: proved the twice-moved-card bound
  d(pi)>=4n-2I2(pi)-2s, where s is common bottom suffix
  length and I2 is the maximum two-increasing-subsequence
  union. It proves reversal optimality for all n. Exact
  tableau enumeration raises the uniform mean lower bound
  at n=52 to 166.8791708235294... . Added the theorem and
  bibliography to REPORT.md; preserved older counting
  bounds for their asymptotic role. Calculator:
  `tools/structural_bound.py`; new exact artifact:
  `results/structural-lower-bound-52.json`.
  Independently reran all 409,113 targets through n=9;
  I2 checked by a separate DP through n=6; tableau weights
  sum to n! and exact small-n ensemble sums agree.
  Partition calculation at n=52 takes about four seconds.
  All 15 existing unittest groups pass (6.7 seconds).
- 2026-10-04: rewrote `REPORT.md` as a self-contained
  research paper, with formal model, propositions/proofs,
  experiments, discussion, bibliography, and reproduction
  appendix. Corrected the ambiguous reversal discussion:
  its 4(n-1) construction is valid for all n; the counting
  obstruction concerns the stronger all-permutations bound.
  Verified primary bibliographic sources with two research
  agents. Added original SVG and PNG diagrams in `figures/`
  and their generator, `tools/report_figures.py`.
  Mathematical review and document checks performed without
  changing the algorithms or rerunning large experiments.
  Independent review found no substantive mathematical
  errors. Both figures were rendered and visually checked;
  local links, citations, SVG structure, and wrapping pass.
- 2026-10-04: wrote `REPORT.md` for a math/CS audience,
  emphasizing derivation, the recursive buffer invariant,
  expected and worst-case costs, and unresolved gaps.
  Cross-checked numbers against saved results and code.
  No algorithm changes or new benchmark runs in this pass.
- Stable binary radix is a simple scalable baseline.
  Partition D by a target-rank bit into A and B, then
  return B followed by A. The two reversals preserve
  each group's order. Low-to-high bit passes sort ranks.
  Cost: 2n ceil(log2 n), or 624 moves at n=52.
- Simulator and exact-count reversal validation pass.
- Exact BFS is complete through n=9; see
  `research/exact-search.md` and `results/exact-n*.json`.
  Maximum optimum is 4(n-1) through n=9. At n=9,
  reversal first shares that maximum with another target.
  This does not establish a large-n transition.
- Radix runs can share a code when their target order
  agrees with their initial order. Gaps in the codes,
  untouched suffixes, and flexible A/B assignment improve
  the basic algorithm. Proofs are in
  `research/algorithms.md`.
- Initial integrated n=52 benchmark, 1,000 seeded targets:
  adaptive radix mean 478.088, sample max 518;
  natural merge mean 493.328, sample max 594;
  four-candidate portfolio mean 470.630, sample max 504.
  All emitted moves independently simulated and verified.
  Results are in `results/baseline-52.json`.
- A patience-based merge variant has independently tested
  around 439.5 mean moves at n=52; retained as a comparison.
- Exact small-n plan tables and bounded local path
  shortcuts are complete. All 46,233 lookup plans through
  n=8 independently replayed and match exact distances.
  Shortcuts saved zero moves on 100 random n=52 plans,
  costing about 0.23 seconds each; excluded from defaults.
- Major improvement: recursively merge sorted blocks,
  using optimal lookup for leaves of at most eight cards.
  Existing side-stack contents act as untouched buffers.
  Reflection and adjacent inverse cancellation reduce
  node costs. `hybrid_merge.py` implements the method.
  Balanced n=52 mean 363.386, sample max 402 over 10,000.
  Initial proved bound: 488 moves, later tightened to 444.
- Interval split DP improves moves but takes about 0.26
  seconds/deck. Near-balanced split search is promising:
  midpoint +/-2 mean 337.784, sample max 368 over 500,
  around 7.4 ms/deck in the agent's comparison.
- Legal nonbacktracking path counting proves n=52 optimal
  maximum >=156 and uniform optimal mean >=154.9453.
  Counting rules out universal 4(n-1) by n=212, but does
  not locate the true transition. See
  `research/lower-bounds.md`.
- Final bound refinement measures exact leaf plans' final
  return runs. Reflecting each child to its parking side
  guarantees cancellations. Leaf effective costs are 16
  and 20 for sizes six and seven; the recurrence gives
  B(13)=62, B(26)=172, B(52)=444. See
  `research/hybrid-bounds.md`. Tests check the exact tables
  still satisfy the effective-cost constants.
- Final structured tests exposed a cheap improvement:
  distribute into two increasing subsequences when possible
  and merge once. All modes now use 104 moves for the 26
  disjoint adjacent swaps at n=52. The default uses 204
  for reversal, 104 for tested rotations, and 102 for the
  tested interleaving.

## Open research and useful next steps

1. Locate the actual small-to-large-n change. The n=9 tie
   in hardest targets and counting obstruction at n=212
   do not locate the true transition. The 64-card search
   cutoff is an implementation choice, not evidence of it.
2. Close the large gap between the proved optimal uniform
   mean lower bound 166.87917 and measured controller mean
   about 319 at n=52. Conditional instance certificates
   narrow the gap, but optimality remains unresolved.
3. Explore retaining several boundary-compatible plans
   per interval. Current DP keeps only one shortest local
   plan, which can lose better parent cancellations.
4. Strengthen arbitrary-state residual bounds before
   expanding targeted exact searches. The nearly reversed
   n=14–18 trials stalled despite two-move endpoint gaps.

`research/campaign-summary.md` records completed work and
narrower next gates. The immediate priority is to constrain
executions with few twice-moved cards, beyond the current
conditional relaxation's 4m−4 ceiling. Ordinary
fractional size-nine subset patterns have a proved
186-move ceiling after parity rounding at n=52.
NP membership is proved; NP-hardness remains unproved,
and no-PTAS would require a separate constant-gap result.
The new excursion formulation proves XP in the excess
above the 2m baseline, not FPT in that parameter.
