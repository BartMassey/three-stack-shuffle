# Counting lower bounds

Historical word-counting bounds. The current exact
uniform-mean lower theorem at n=52 is 166.87917, and
reversal proves a maximum of at least 204; see
[the structural derivation](next-lower-bounds.md).
The [counting campaign](campaign-counting.md) improves
word counting to 156.1873 but not the structural theorem.
[Conditional rules](campaign-structure.md) strengthen
individual certificates, not the exact ensemble mean.

`tools/counting_bound.py` computes rigorous counting lower
bounds; `results/lower-bounds.json` stores exact integer
counts and rational mean bounds. The entire calculation
completed in 0.36 seconds. Only n=52 used the improved
height-state DP. Larger n used a cheap closed-form bound.

## Why the counts bound optimal distance

Let X be the optimal number of moves for a uniformly chosen
final permutation, with all cards initially and finally in
D. Every such distance is even: each move changes the
number of cards in D by one. A shortest word never has a
move immediately followed by its inverse, because removing
that pair preserves every stack's exact contents.

There are initially only two legal moves, DA and DB.
Subsequently at most three of the four move types remain
after excluding the previous move's inverse. Therefore at
most 2*3^(L-1) nonbacktracking words have length L>0. This
is an upper bound; many counted words have empty sources
or fail to return all cards to D.

Swapping A and B maps each nonempty closed legal word to a
distinct word with the same final D permutation. It has no
fixed words, because the first move DA becomes DB or vice
versa. Consequently each nonidentity permutation realized
within a length budget has at least two counted words.
Including the identity separately gives the cumulative
target upper bound

```
U(2k) = 1 + sum(j=1..k) 3^(2j-1)
      = 1 + 3*(9^k - 1)/8.
```

Words realizing the identity more than once, or multiple
words realizing other permutations, increase this upper
bound. No assumption of one word per target is needed.

If U(2k)<n!, some target requires at least 2k+2 moves.
For the mean, even-valued nonnegative distance gives

```
E[X] = 2 * sum(k>=0) P(X > 2k)
     >= (2/n!) * sum(k>=0) max(0, n! - U(2k)).
```

Only finitely many terms on the right are positive.
The code computes the fraction with exact Python integers
and reduces it using `Fraction`. Floats are supplemental.

## Improved height-state count

The DP state is `(a,b,last)`, where a and b are the numbers
of cards in A and B and D has n-a-b cards. Starting from
`(0,0,none)`, it counts every legal word without adjacent
inverse moves. The four transitions are:

| move | new a | new b | legality |
|------|-------|-------|----------|
| AD | a-1 | b | a>0 |
| DA | a+1 | b | a+b<n |
| DB | a | b+1 | a+b<n |
| BD | a | b-1 | b>0 |

At each length L, summing states with a=b=0 counts closed
legal nonbacktracking words exactly. Although the DP does
not store card order, word legality depends only on stack
heights, so this omission does not affect the word count.
Replacing 2*3^(L-1) by this exact closed-word count gives
a stronger cumulative target upper bound. The same
mirror pairing and tail-sum proof continue to apply.

The n=52 DP was run through L=204. Its cumulative upper
bound first exceeds 52! at L=156. At L=154 it covers at
most 0.46643529 of the target count; thus at least one
target requires 156 moves. The exact rational mean bound
is stored in JSON; its decimal value is 154.9453333327.

These lower bounds concern optimal target-aware plans.
They do not predict the mean of the implemented planner.
The maximum lower bound is weaker than any independently
proved 204-move reversal lower bound. At L=204 the counting
capacity is much larger than 52!, so this argument gives
no obstruction to a universal 204-move bound at n=52.

## Basic obstruction threshold

| n | max lower bound | mean lower bound |
|---|-----------------|------------------|
| 52 | 144 | 143.4100929808 |
| 100 | 332 | 331.7451854700 |
| 200 | 788 | 786.8866509178 |
| 211 | 840 | 839.7459260317 |
| 212 | 846 | 845.1263211186 |

The first n for which the basic word bound alone rules
out a universal 4(n-1) move guarantee is 212. All integer
comparisons were exact. At n=211 the budget is 840 and
U(840)/211! is approximately 1.0162958731, so counting
does not yet forbid coverage. At n=212 the budget is 844
and U(844)/212! is approximately 0.3883017251. Hence some
212-card target requires at least 846 moves.

This is the first obstruction from this particular bound.
It does not identify the true deck size where reversal
ceases to determine the worst case. A stronger counting
argument or a specific permutation might give an earlier
obstruction. The improved DP was not run near n=212.

Asymptotically, the same counting argument implies optimal
worst-case and mean distances at least
`log_3(n!) - O(1)`, hence growth of order n log n.
Thus a universal 4(n-1) upper bound for all n is impossible,
even though the exact small-n maximum follows that formula.
This does not settle the optimal 52-card maximum or mean.

## Verification and reproduction

```sh
python3 tools/counting_bound.py
```

The CLI defaults to n=52 for the height DP and writes the
machine-readable file above. `--height-n` accepts 1..52.
`--output` selects another results destination. Integers
in JSON are stored as decimal strings to preserve exact
values in consumers with limited numeric precision.

An independent recursive enumeration of actual legal
move words matched the DP's closed-word counts at every
length 0..10 for n=1,2,3,4. This reference enumeration
computed each source height explicitly and checked the
forbidden inverse independently of the DP transitions.
