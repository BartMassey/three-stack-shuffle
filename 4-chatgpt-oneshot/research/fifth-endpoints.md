# Fifth campaign: boundary endpoint selection

4 October 2026. One fixed width-four heuristic was tested
on ten fresh 52-card targets, seed `2026100406`.
No parameter sweep or follow-up sample was run.

Both policies keep the shortest representative for each
existing first-six/last-six signature, with lexical ties.
The new optional `boundary` policy always retains the
globally shortest representative, then ranks remaining
representatives by length minus twice the larger initial
or final run of identical moves, with length/word ties.
This is heuristic cancellation potential, not a dominance
rule. Runs can exceed the signature length, and future
contexts need not cancel the favored boundary at all.

The default remains `shortest`. Every portfolio retains
the complete existing `oriented_window` fallback, so its
returned plan cannot be longer than that baseline.
The solver registration and default are unchanged.

| Controller | Mean | Maximum | Wins vs baseline | Median s |
|---|---:|---:|---:|---:|
| oriented_window | 274.2 | 288 | baseline | 0.102 |
| width four, shortest | 273.4 | 288 | 4/10 | 0.760 |
| width four, boundary | 271.2 | 284 | 8/10 | 0.807 |

Boundary selection beat shortest selection on seven
targets and tied on three, a mean gain of 2.2 moves.
Its mean improvement over baseline was 3.0 moves.
All thirty complete words were replayed successfully.
No six-second cooperative call cutoff was reached, and
all twenty searches completed both directions within
the fixed 150-second campaign search allowance.

Each target ran baseline, shortest, then boundary.
Portfolio times include their own baseline computation;
cache state and scheduling can influence timings.
This paired ten-target pilot supports retaining the
optional experiment, not a population or optimum claim.
Boundary's roughly eightfold median baseline runtime
and small sample do not justify changing the default.
Stop this campaign here; no additional sweep is warranted
by this finite gate.

The focused tests check explicit/default policy equality,
validation, recursive guarded endpoints, and fallback.
See [targets, words, timings, and cutoffs](
../results/fifth-endpoints.json), reproduced by
`python tools/fifth_endpoints.py --output NEW_PATH`.
