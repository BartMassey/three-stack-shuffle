# Three-Stack Shuffle

Bart Massey, with various AIs

A work-in-progress study of a three-stack LIFO card-sorting
machine — a deck plus two buffers, four one-card moves —
asking how few moves it takes to sort a shuffled deck (the
headline target is 52 cards), via admissible lower-bound
heuristics, exact search, constructive merge sorters, and a
companion permutation-distance model. Results are partial
and still evolving.

## Current research snapshot

[The three-stack study](4-chatgpt-oneshot/README.md) includes
an executable controller, simulator, exact small-instance
tables, and a [research paper](4-chatgpt-oneshot/REPORT.md).
Its optional direct-output controller guarantees at most
352 moves for every 52-card target. The proved optimal
maximum lies between 204 and 352; the optimal uniform
mean is at least 166.87917. Neither optimum is known.

See the [completed campaign](4-chatgpt-oneshot/research/campaign-summary.md)
for held-out measurements, stronger instance certificates,
and the limits of the approaches tested. Earlier numbered
directories preserve separate stages of the investigation.
The [direct-output follow-up](4-chatgpt-oneshot/research/oriented-merge.md)
improves both the finite bound and the large-n leading term.
The [third campaign](4-chatgpt-oneshot/research/third-campaign-summary.md)
improves that leading term further to `2n log3(n)` and
adds lower-bound cost partitioning and compact rational
certificates. The 52-card universal bound remains 352.
The [fourth-campaign release](4-chatgpt-oneshot/research/fourth-campaign-summary.md)
closes this study with a sharp recurrence coefficient,
fresh certificate comparisons, and a bounded endpoint
portfolio experiment. The portfolio remains research-only;
the practical controllers and their guarantees are unchanged.

## License

This work is made available under the "Apache 2.0 or MIT
License". See the file `LICENSE.txt` in this distribution for
license terms.
