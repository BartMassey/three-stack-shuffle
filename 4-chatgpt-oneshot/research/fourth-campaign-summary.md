# Fourth campaign: closure and release

Completed 4 October 2026. The third snapshot was committed
as `e3ffafb` before this bounded closure campaign. The
user authorized a final local release commit afterward.
No push, tag, or external release publication is implied.

## Release position

This is a good stopping point, not a claim of optimality.
The main algorithm, proofs, simulator, exact tables,
experiments, and negative results form a coherent study.
At n=52, the proved optimal maximum remains in [204,352]
and the deterministic uniform-mean lower bound remains
166.87917. Neither optimum is known. The best proved
large-n construction uses `2n log3(n)+O(n)` moves.

No default changed. Use `oriented` for fast 52-card
planning, `oriented_window` to spend more host time on
moves, or `ternary` for large decks. The historical
`recommended` CLI default remains for reproducibility.
The new portfolio stays an experimental library.

## 1. The recurrence coefficient is now settled

The new proof establishes that `2/log2(3)` is the best
leading coefficient for the explicitly defined numeric
endpoint recurrence, allowing every positive integer
split at every size above the fixed 64-card leaf cutoff.
It holds with or without the fixed repark credit.

The key is an inductive lower potential for both output
contracts. Every central split has entropy surplus
`c[1-H(x)]`; every parked split has surplus
`2-c[H(y)+1-y]`. Both are nonnegative when
`c=2/log2(3)`. Thus arbitrary size-dependent split choices
and oscillating linear corrections cannot evade the
bound. The ternary construction supplies the matching
upper term.

This does not lower-bound machine words: numeric leaf
charges are upper guarantees and the model excludes
additional boundary cancellation. It settles this
recurrence family, not all shuffling algorithms.
Exact all-split minimization through 4096 verifies the
finite implementation; the proof itself covers all n.

See [the theorem and reproducer](fourth-recurrence.md).

## 2. Compact certificates: useful, not a replacement

The fixed batch contains 20 fresh 52-card targets, seed
2026100404. Each LP pipeline has a ten-second discovery
budget; the earlier integer method has three seconds
and its original +6 cap. All 20 LP outputs yield exact
rational certificates, verified independently without
SciPy and with Python assertions disabled.

| Method | Mean certified bound |
|---|---:|
| Structural B | 165.3 |
| Integer, original cap | 171.2 |
| Chain/group LP | 171.9 |
| Maximum of the two | 172.3 |

These are sample certificate means, not population
theorems. LP discovery is slower: about 5.76 seconds
on average, versus 0.18 seconds for capped integer search.
LP has nine wins, seven ties, and four losses; the cap
prevents reading those wins as general superiority.

A fairness check therefore takes the first five LP wins
on these same targets and gives integer search ten
seconds with a +12 cap. Four apparent LP wins reverse;
one remains, at 176 versus 174. This selected subset
does not support a population comparison. The methods
can complement each other, but the experiment does not
justify replacing the fast integer method.

Base discovery and the fairness check together take
about 130 seconds. All searches stop at this gate.
The durable output is 20 portable proofs and a measured
distinction between finding and checking a certificate.
See [the certificate study](fourth-certificates.md).

## 3. Endpoint portfolios: modest gains, expensive

A separate fixed sample uses 20 targets, seed 2026100405.
Each interval retains two or four representatives with
distinct six-operation prefix/suffix signatures. Parents
try child combinations and cancel inverse operations.
An explicit complete `oriented_window` fallback proves
pointwise no regression and preserves the 352 guarantee.
The signature rule itself is heuristic, not dominance.

| Method | Mean moves | Sample max | Median seconds |
|---|---:|---:|---:|
| oriented_window | 274.8 | 284 | 0.102 |
| Width two | 274.3 | 284 | 0.475 |
| Width four | 273.0 | 284 | 0.754 |

There are no timeouts. All 60 recorded words replay.
Width four saves 1.8 mean moves at about 7.4 times the
median host cost and does not improve the sample maximum.
That is insufficient to promote it into the default or
justify extending this campaign.

There is a useful pedagogical result: an explicit legal
nine-card composition where a 15-move child yields a
26-move parent, while its 13-move alternative yields 28.
The longer child exposes three cancellation pairs instead
of one. This explains why one locally shortest plan can
lose without asserting a global optimum for that example.

See [the portfolio study and witness](fourth-portfolio.md).

## 4. Final verification and stopping decision

All 64 test groups pass. With third-party site packages
disabled, the suite also passes, skipping only two
optional SciPy discovery tests. Portable certificate
verification remains available in that environment.
The C++ exact-search build is warning-clean.
Root independently verifies all 20 new certificates,
checks them against replayed upper plans, and replays
all 60 stored portfolio words. All 115 result JSON files
parse; 199 local links in 41 documents resolve. Pandoc
renders 108 report and four plan MathML nodes without
math errors. Whitespace checks pass.

The paper, README, research index, historical plan, and
living log distinguish current conclusions from archived
measurements. Earlier frozen artifacts remain unchanged.
Generated HTML is reproducible from the supplied Pandoc
configuration rather than committed as a second source.

There is no further campaign scheduled. A useful restart
would need new structure: constraints on four-move cards,
a stronger ensemble-counting argument, substantially
better endpoint contracts, or a concrete hardness
reduction. Longer runs of the same searches are not
presently justified. Hardness, a no-PTAS result, the
52-card optima, and the first failure of the universal
4(n−1) budget remain open.

## Reproduce the release checks

```sh
mkdir -p build
python3 -m unittest -q
python3 -S -m unittest -q
python3 -S -O tools/fourth_certificates.py \
  --verify results/fourth-certificates.json \
  --output build/fourth-certificates-checked.json
pandoc --defaults tools/report-pandoc.yaml
```

The certificate tool refuses to overwrite its output.
Detailed notes provide
the bounded experiment commands; no experiment must be
rerun merely to check an existing certificate or word.
