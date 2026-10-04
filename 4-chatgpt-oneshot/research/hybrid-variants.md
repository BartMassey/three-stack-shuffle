# Bounded hybrid split search

This describes the original split-search study and its
measurements. These variants still underpin the default.
The later [execution campaign](campaign-execution.md) adds
parked endpoints and prompt cache release; it does not
replace the historical measurements below.

Implementation: `hybrid_variants.py`. This builds on
`hybrid_merge._combine`, exact leaf plans from `optimal`,
and the balanced recursive `hybrid_merge` baseline.
No changes were made to those shared modules.

Every public variant returns the shorter of its searched
plan and the balanced baseline. Consequently the balanced
baseline's proved operation bound is preserved exactly.
The default search cutoff is n<=64. Above that cutoff,
the public variants return the balanced plan unchanged.
Thus these bounded searches do not worsen asymptotic
behavior at larger deck sizes.

## Concrete variants

`hybrid_window` tries every split within a configurable
distance of the segment midpoint. It memoizes each
interval's shortest plan. Default window=1; windows 2,
3, 4, and 6 were also tested. Zero gives balanced splits.
Each interval retains its shortest locally composed plan.
This does not prove globally optimal recursive plans,
because boundary cancellations depend on plan details.

`hybrid_aligned` tries the midpoint and the nearest lower
and upper multiples of the leaf limit. This favors exact
leaves that fill the eight-card lookup limit, while still
including balanced splits. It also memoizes intervals.

`hybrid_root` tries alternative splits only at the root,
with balanced recursion underneath. Default radius=10
tests split lengths 16 through 36 at n52.

`hybrid_packed` tries two predetermined trees whose leaf
counts are balanced and whose leaf sizes fill the exact
limit, with the partial leaf at either edge. At n52 and
leaf_limit8, this uses seven leaves instead of the eight
leaves produced by splitting card counts in half.

## Measurements

All rows used the same 500 permutations of range(52),
from Python Random seed27391. Each plan was executed in
the independent `three_stack.Machine` simulator; legality,
empty side stacks, and the exact target were verified.
Times include plan generation and simulation on this
environment; they are approximate local measurements.
Maximums are observed sample maxima, not guarantees.

| Variant | Mean ops | Maximum ops | Seconds / 500 |
|---|---:|---:|---:|
| Balanced, leaf4 | 388.468 | 430 | 0.149 |
| Balanced, leaf5 | 388.468 | 430 | 0.141 |
| Balanced, leaf6 | 375.880 | 412 | 0.123 |
| Balanced, leaf7 | 362.852 | 392 | 0.109 |
| Balanced, leaf8 | 362.852 | 392 | 0.108 |
| Packed leaf8 | 354.624 | 386 | 0.249 |
| Aligned leaf8 | 350.192 | 378 | 0.670 |
| Root radius10 | 350.412 | 378 | 1.678 |
| Midpoint window1 | 344.756 | 380 | 0.951 |
| Midpoint window2 | 337.784 | 368 | 3.716 |
| Midpoint window3 | 331.512 | 364 | 7.968 |
| Midpoint window4 | 328.028 | 356 | 13.474 |
| Midpoint window6 | 323.344 | 350 | 26.077 |

Window2 gives a substantial improvement at about 7.4ms
per sampled deck, compared with 0.22ms for the balanced
baseline. Window6 gives better counts at about 52ms per
deck. Packed leaf8 is a cheaper improvement at about
0.50ms per deck. The root-only radius10 search is slower
and slightly worse on mean than aligned leaf8 here.
Full interval DP was deliberately not benchmarked in this
batch; the root agent is measuring that independently.

The leaf-limit experiment favors eight-card exact leaves
on this sample. At n52 the balanced leaf7 and leaf8 trees
are identical, as are the leaf4 and leaf5 trees. These
equalities follow from that specific tree's segment sizes
and should not be extrapolated to other n.

## Cross-size validation

All four public variants were also tested on five random
permutations each at n=0,1,2,7,8,9,16,17,31,32,52,63,64,
65,100. This independently simulated 300 returned plans.
Each result was compared against balanced operation count.
For n65 and n100 the exact equality of returned plans to
the balanced fallback was also verified. All passed.
