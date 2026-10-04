# Bounded local shortcut experiment

Historical negative experiment. This shortcut remains
excluded from runtime defaults. See the
[campaign summary](campaign-summary.md) for subsequent
execution improvements and current stopping gates.

`shortcut.py` exports
`shorten(initial, operations, window=12, time_limit=0.25)`.
It preserves the exact full three-stack endpoint, including
nonempty side stacks. It first removes repeated-state loops,
then searches shorter replacements for windows of 4..12
moves using bidirectional BFS to depth at most four per side.
It verifies its result by replaying both endpoint states.

Each BFS is strictly bounded. The optimizer checks its
wall-clock deadline before each window search. Input replay,
loop removal, and final verification are outside the search
deadline, and a single bounded BFS may finish just after it.
This is a practical runtime cap, not a hard real-time bound.

The local search caches results by operation word and stack
heights capped at the word length. Card labels are irrelevant
because the machine only moves distinct cards and never
branches on their values. At most the window length of top
cards can participate in either the original segment or a
strictly shorter replacement. Symbolic distinct labels
therefore suffice to preserve the full actual endpoint.

The cache retains at most 50,000 replacement words, not BFS
graphs. Maximum replacement length is eight moves. Thus the
search can miss a ten-move improvement to a twelve-move
segment, as well as all longer-range improvements. This
bounded experiment cannot prove that the input is optimal.

## Random 52-card natural-merge trial

The experiment used 100 targets generated with Python's
`random.Random(20261003)`, each obtained by shuffling
`list(range(52))` independently. The initial deck was
`range(52)` and the source planner was `natural_merge` from
`merge_candidates.py`. Each shortened plan was replayed and
checked against its target using the shared `Machine`.

| quantity | result |
|----------|--------|
| number of targets | 100 |
| original mean moves | 493.12 |
| shortened mean moves | 493.12 |
| targets improved | 0 |
| maximum moves saved | 0 |
| total elapsed seconds | 23.127 |
| maximum call seconds | 0.250381 |
| cache hits | 305757 |
| cache misses | 106655 |

This experiment found no useful savings. The runtime cost
is too high for an interactive controller when compared
with constructing the source plan. The bounded optimizer
is retained as an experimental function, but is not
recommended for the default controller pipeline. Further
local search was abandoned after this negative result.
