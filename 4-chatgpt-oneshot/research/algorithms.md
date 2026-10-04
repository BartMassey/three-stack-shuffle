# Radix constructions for three stacks

Historical construction study, retained with its original
measurements. The runtime now favors exact-leaf merging;
see the [current report](../REPORT.md) and
[parked-endpoint improvement](campaign-execution.md).
The radix proofs remain useful asymptotic baselines.

All lists below describe stack contents from top to bottom.
Physical moves are restricted to `DA`, `AD`, `DB`, and `BD`.
The controller knows the complete initial and target orders.

## Proven baseline: stable binary radix

Give each card its rank in the target order, from zero to
`n - 1`. Process the rank bits from least significant to most
significant. For each bit:

1. Move every card from D to A if its bit is zero, or to B if
   its bit is one.
2. Move all of B to D, then all of A to D.

Each class is reversed when entering its auxiliary stack and
reversed again when returning to D. The pass therefore performs
a stable partition, with zero cards above one cards. The standard
induction on the processed bits proves that the final order is
the target order. The auxiliary stacks are empty after every pass.

There are exactly `2n ceil(log2 n)` moves with complete passes.
For 52 cards this gives 624 moves, for every initial order.
This is an executable O(n log n) construction, not a conjecture.

## Proven pass reductions

If the current bit sequence already consists of zeros followed
by ones, the pass does nothing and can be omitted.

Otherwise, leave the maximal bottom suffix of one cards in D.
Partition and return only the cards above it. The suffix already
belongs below every zero card and its internal order is preserved.
This remains the same stable partition and saves two moves per
retained card.

## Proven monotone labels with a gap

Distinct ranks need not be represented by consecutive integers.
Any strictly increasing integer labels give the same target order.
Let `C = 2^ceil(log2 n)`, `H = C/2`, and `s = n - H`.
For `n > 1`, give ranks below s their ordinary labels. Give all
other ranks labels `rank + C - n`. Thus labels are
`0..s-1` and `H..C-1`.

Before the final bit, the cards are sorted by their lower bits.
The last `C - n` cards then have lower-bit values that exceed
every lower-half label. They form a fixed suffix of one cards
and remain in D during the final pass.

This proves a worst-case bound
`2n ceil(log2 n) - 2(C - n)`.
For 52 cards, use labels `0..19` and `32..63`; the bound is 600.
If n is a power of two, this is the ordinary labeling.

Moving the gap to another split preserves correctness. The
implementation emits all splits whose occupied labels fit into
the two halves, allowing measured portfolio selection.

## Proven repeated labels from inverse-order runs

Write down the initial position of each card in target order.
Break this position sequence wherever it descends. Within each
resulting target block, cards already occur in the desired relative
order in the initial sequence. Give every card in a block the same
label, with increasing labels for consecutive blocks.

A stable sort by those labels reconstructs the target. This follows
because stability retains each block's existing correct internal
order, while sorting places the blocks in target order.

With g blocks, radix needs at most `2n ceil(log2 g)` moves.
The gap labeling applies to block labels as well. Its final retained
suffix consists of the last `2^ceil(log2 g) - g` blocks, so the
savings are twice their total number of cards.

The number of blocks equals one plus the number of descents of
the inverse permutation. Uniform random permutations have expected
block count `(n + 1)/2`: each adjacent pair of initial positions
has probability one half of descending. This expectation alone
does not determine the expected number of bits or moves.

## Flexible assignment of the two side stacks

Each pass may independently exchange the roles of A and B.
This changes neither the stable partition nor its unreduced move
count. It can increase cancellation of immediately inverse moves
between consecutive passes.

The implementation chooses the next first card's destination to
match the preceding pass's final source. A root-level inverse-move
reducer can exploit the resulting cancellations. Correctness is
proved by symmetry of the two side stacks. Optimality of this
greedy orientation choice is not claimed.

## Proven reversal construction

For n at least two:

1. Move `n - 1` cards from D to A.
2. Move the remaining D card to B.
3. Repeat `AD`, `DB` exactly `n - 2` times.
4. Move the remaining A card to D.
5. Move `n - 1` cards from B to D.

The card initially at the top becomes D's bottom card in step 4.
The other cards return in reversed order in step 5. The total is
`(n-1) + 1 + 2(n-2) + 1 + (n-1) = 4(n-1)`.
The construction is proved; a general minimality proof is not
provided here. For zero or one card, no moves are required.

## Executable artifacts and validation

`radix_candidates.py` exposes `radix`, `radix_gap`, `runs_radix`,
`runs_radix_gap`, corresponding `_flexible` variants, a gap candidate
portfolio, and `reversal`. All return lists of physical move strings.

Every permutation for n from zero through eight was sorted
successfully by the four non-flexible radix variants. The construction
checks its final state internally. This covered 46,234 permutations.

On 10,000 uniform random 52-card permutations, seed 2026, raw move
counts before adjacent inverse cancellation were:

| Construction | Mean | Maximum observed |
| --- | ---: | ---: |
| ordinary ranks | 616.5746 | 624 |
| gap ranks | 591.5380 | 600 |
| inverse-order runs | 514.3314 | 624 |
| inverse-order runs with gap | 491.6212 | 528 |

Observed maxima are sample results, not worst-case theorems.
The repeated-label construction and flexible variants remain within
the same O(n log n) guaranteed bound as ordinary radix.

After repeatedly cancelling adjacent inverse moves, on the same
sample:

| Construction | Mean | Maximum observed |
| --- | ---: | ---: |
| ordinary ranks | 608.3460 | 624 |
| ordinary ranks, flexible sides | 601.5074 | 614 |
| gap ranks | 583.2722 | 600 |
| gap ranks, flexible sides | 576.4506 | 590 |
| inverse-order runs | 508.2070 | 620 |
| inverse-order runs, flexible sides | 500.8680 | 614 |
| inverse-order runs with gap | 485.5070 | 520 |
| inverse-order runs with gap, flexible sides | 478.2146 | 514 |
