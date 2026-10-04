# Exact parked leaves: execution campaign

4 October 2026. The runtime defaults are unchanged.

## Result and scope

Exact parked leaves give a verified construction with
at most **410 transfers at n=52**, improving the earlier
444 construction bound by 34. This bounds the optimum
from above; it does not prove the controller optimal.

The experimental implementation is `parked_leaf.py`.
`parked_recommended` retains the existing recommended
controller and adds both directions of a parked-leaf
window search. It also includes splits chosen by the
proved recurrence below. Its window has radius four.

The first 100 development targets, seed 20261003, gave
322.92 moves for recommended and 319.00 for the new
portfolio. There were 82 wins, 18 ties, and no losses.
The paired mean saving was 3.92 moves, with approximate
normal 95% interval [3.32, 4.52]. Median warm planning
time rose from 53.4 ms to 104.0 ms. Replay is excluded
from these timings. This fails the optional unchanged
53 ms latency target despite useful transfer savings.

The method was frozen before inspecting holdout targets
from seed 2026100401. That seed is separate from tuning.

On 1,000 paired holdout targets, recommended averaged
322.896 moves and the new portfolio averaged 318.874.
The saving was 4.022 moves, approximate normal 95%
interval [3.800, 4.244], with 762 wins, 238 ties, and
zero losses. The sample maxima were 350 and 340; these
are empirical maxima, not the universal 410 guarantee.
The respective 95th-percentile costs were 338 and 334.

Warm median planning times were 53.5 and 104.2 ms;
95th-percentile times were 57.1 and 110.5 ms. The new
portfolio's longest observed planning time was 132.8 ms.
All plans were replayed independently after timing.
The 100-case holdout pilot is the exact prefix of this
1,000-case corpus, and its move counts agree. It is not
an additional independent sample. Results preserve all
targets and paired measurements for reproduction.

A cheap bound-tree candidate alone averaged 335.04
on development targets in 2.87 ms median. Adding it
to recommended saved only 0.08 moves per target. The
window-two portfolio saved 1.14 moves in about 80 ms.
The window-four method was selected before holdout
because its larger empirical savings justified testing
the slower option; no default latency promise changed.

## Host profile and bounded boundary pilot

The saved profile covers 100 development targets.
The old controller makes about 2.02 million joins;
the new portfolio makes 4.95 million. Joining and
copying words dominate. Fresh-process first-target
planning took 59.5 ms and 116.9 ms respectively.
Those isolated processes reported peak RSS of about
17.5 MiB and 23.7 MiB; profiling overhead is excluded
from the cold timing but affects the detailed profile.

After the frozen benchmark, the experimental recursive
cache is explicitly cleared when its winning word has
been copied. All 100 saved holdout costs remain exactly
unchanged and every execution replays successfully.
That validation process had Linux VmHWM 23,176 KiB.
The earlier benchmark's getrusage values all equal
652,592 KiB, suggesting inherited launcher accounting.
Do not interpret their difference from VmHWM as a
measured reduction in planner memory. The saved raw
results preserve both observations without conflation.

A further bounded pilot added one shortest parked leaf
word maximizing its terminal parking run. A dynamic
program over the BFS distance DAG finds that word;
only edges decreasing the exact distance are eligible.
The original word remains available, and full words
are retained. This is a two-word leaf library, not an
exhaustive merge-grammar frontier or a dominance proof.

Both sets of shortest leaf words agree in length, and
all 46,233 alternative words replay through n=8.
Independent tuple-state distance-DAG calculation checks
the maximum terminal-run objective through n=5.
Twenty development targets showed zero additional
savings over the frozen portfolio, for about 55 ms
of extra two-direction planning per target. Therefore
`tail_alternative` remains disabled; no holdout tuning
or broader frontier search was performed.

## Endpoint contract and protected bases

The active cards initially occupy a prefix of D. Their
target ranks are distinct integers 0 through n-1.
The parked-A endpoint has active cards n-1 through 0,
top to bottom on A, and no active cards on D or B.
Reflecting A and B supplies the parked-B endpoint.

Place any fixed stack contents below the active cards
on each stack. Every move in the isolated execution
has a nonempty active source, so induction on the word
shows that it moves the same active card above the same
protected bases. None of those bases moves or changes.
This permits a parked sibling to be part of a base
while the next sibling is processed on D.

The parent first parks its first child on A, then its
second child on B. Taking the larger exposed target
rank at each step merges the children onto D in sorted
order. This uses exactly n transfers. Cancelling two
adjacent inverse transfers preserves the endpoint and
legality, including legality above protected bases.

The implementation retains both reflected old D plans
followed by parking as leaf alternatives. It adds one
exact parked plan. Full words are joined and reduced.
The old complete root candidate is retained because
a locally shortest child need not compose best with
external context. This remains a heuristic search over
trees and words, not exact global optimization.

## Table generation and validation

`tools/campaign_execution_tables.cpp` performs BFS from
the parked-A endpoint in the complete state graph.
States are permutations with two cuts, representing
the top-to-bottom contents of A, D, and B. Thus there
are n! times (n+1)(n+2)/2 states. Edges are exactly the
four adjacent top-card transfers, each with cost one.

All edges are reversible. Distances from the fixed
goal are therefore exact remaining distances. Starting
at each D-only permutation and following decreasing
distances yields a shortest legal parked word.

Generation through n=7 took less than 0.05 seconds per
size; n=8 visited 1,814,400 states in 0.435 seconds.
The n=8 dense distance and queue storage is 10.9 MB,
excluding small overhead and streamed output. Each run
used an external 60-second limit; all finished normally.

Independent tuple-state Python BFS agrees with all
parked distances through n=5. The test suite replays
all 46,233 table entries through n=8, for both sides,
above three distinct nonempty protected bases. It checks
that no move ever draws from a base and that each base
and the active endpoint are correct. These are 92,466
protected executions. Each exact parked word is also
no longer than either reflected old D plan followed
by parking and reduction.

The verified finite maxima are:

| n | Exact parked maximum P(n) | Old parked bound |
|---|---:|---:|
| 1 | 1 | 1 |
| 2 | 4 | 4 |
| 3 | 7 | 7 |
| 4 | 12 | 12 |
| 5 | 15 | 17 |
| 6 | 20 | 22 |
| 7 | 23 | 27 |
| 8 | 28 | 32 |

At n=8, 27,769 of 40,320 permutations improve locally.
The greatest local gain is ten transfers. Histograms
of gains, costs, and initial/final runs are saved in
`results/campaign-execution-leaves.json`. The table
generator chooses one shortest path; it does not count
all useful ties or claim a complete boundary frontier.

## Proof of the improved universal construction bound

Let U(n) bound a D-to-D sorted construction and P(n)
bound the corresponding reversed sorted parked endpoint.
For n<=8 use the verified table maxima above and the
existing D-to-D upper bound U(n)=4 max(0,n-1).
Set U(0)=P(0)=0.

For n>8 and any nonempty split k+(n-k), the protected
construction proves

    U(n) <= n + P(k) + P(n-k).

A nonempty D-to-D execution ends with AD or BD.
Reflect its side names if necessary so this final
return is inverse to the requested parking move.
Parking the n sorted cards then cancels at least one
inverse pair. An already sorted segment instead needs
only n parking transfers. Therefore

    P(n) <= max(n, U(n) + n - 2).

Choose k to minimize P(k)+P(n-k). Both subproblems
are smaller, making this a finite inductive recurrence.
The code `parked_bounds` implements it directly, and
the planner's `bound_splits=True` includes the chosen
split and its mirror. Every child retains enough
alternatives to meet this recurrence irrespective of
which equal-length word it selects: either an exact
leaf parked word or both reflections of its returned
D word followed by parking. Additional cancellation
can only improve this upper bound.

At n=52 the chosen root split is 22+30 and U(52)=410.
The concrete arithmetic is:

    P(14) <= 26 + 23 + 23 = 72
    P(16) <= 30 + 28 + 28 = 86
    P(22) <= 42 + 28 + 72 = 142
    P(30) <= 58 + 72 + 86 = 216
    U(52) <= 52 + 142 + 216 = 410

This tree has four seven-card and three eight-card
leaves, so the 52-card bound needs only those two
verified finite constants and the protected-base lemma.
Restricting every split to the balanced split instead
gives U(52)=420. These are construction bounds from
verified finite leaves and induction, not extrapolated
sample maxima.

The implemented bound applies with the full leaf library
(`leaf_limit=8`), bound splits enabled, and `max_n` at
least the requested size. The default max_n=64 includes
n=52. Larger inputs currently use the old fallback;
the mathematical recurrence still describes a valid
construction at those sizes if its tree is executed.

## Reproduction

Build the generator with a C++20 compiler:

```sh
g++ -O3 -std=c++20 tools/campaign_execution_tables.cpp \
  -o /tmp/campaign_execution_tables
```

For each n=1 through 8:

```sh
timeout 60s /tmp/campaign_execution_tables "$n" \
  "results/campaign-execution-plans-n$n.json"
timeout 60s /tmp/campaign_execution_tables "$n" \
  "results/campaign-execution-tail-plans-n$n.json" tail
```

Validate and run the development comparison:

```sh
python -m unittest test_parked_leaf -v
python tools/campaign_execution.py leaves \
  --output results/campaign-execution-leaves.json
python tools/campaign_execution.py development --count 100 \
  --output results/campaign-execution-development.json
```

The execution experiment created no commits or pushes.
Root integration exposes `parked` as an optional mode;
`recommended` remains the default.
The root also clears local memoization caches in the
old interval searches after extracting their chosen words.
