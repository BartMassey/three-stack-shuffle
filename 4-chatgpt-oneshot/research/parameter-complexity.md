# A smaller exact excess-excursion enumeration

4 October 2026. Exact model: three unbounded stacks
A-D-B, unit top transfers, identity and target on D.
This strengthens the [previous XP bound][previous].
It establishes neither FPT in excess nor unrestricted
polynomial time. Exact-model hardness remains open.

## Result

Trim the common identity bottom suffix and let m be the
remaining size. Identity has distance zero. Otherwise
every active card leaves and returns to D at least once.
For a decision budget K >= 2m put

    r = floor((K - 2m) / 2).

There is a constructive exact decision algorithm in

    f(r) (m + 1)^(2r + 2)

time and polynomial working space in m+r. The previous
enumeration gave f(r)(m+1)^(5r+2). The improvement comes
from identifying a fixed event skeleton for all cards,
including cards with repeated excursions.

The proof does not force a complete initial drain before
any return. Temporary returns can occur throughout the
loading and unloading parts of the skeleton.

## Fixed skeleton lemma

Take any completed legal plan on the active cards. Each
card has a first departure from D and a final return to
D. Delete every other event for the moment.

First departures occur in initial order, 0,...,m-1.
An untouched card remains below all initially earlier
untouched cards, so it cannot leave ahead of them.

Final returns occur in reverse target order. A card's
final return places it permanently on D, and subsequent
final returns build the target above it.

Every first departure precedes every final return.
Suppose a final return happened while an active card had
not yet departed. The returned card would remain forever
above that untouched card, preventing its required first
departure. This is a contradiction.

Thus the projected skeleton has exactly this order:

    out(0), ..., out(m-1),
    in(target[m-1]), ..., in(target[0]).

The lemma uses suffix trimming: without it, a never-moved
bottom suffix need not contribute skeleton events.

## Balanced temporary blocks lemma

A plan of length 2m+2j has j nonfinal returns to D and
j nonfirst departures from D. Call these temporary
returns and repeat departures.

While a temporarily returned card remains on D, no
skeleton event can occur:

- A first departure would require exposing an untouched
  initial card below the temporary card.
- A final return would permanently cover that temporary
  card and prevent its required repeat departure.

The temporary cards therefore form a stack on top of
the current skeleton state of D. A temporary return pushes
onto this stack; a repeat departure pops its top. Each
maximal run of these events is balanced and occurs in
one gap of the fixed skeleton. It can contain nested
pairs involving several cards.

These statements do not require a temporary card to
return to the same side. Nor do they replace a nested
block by individual side-to-side transfers: holding
several cards on D can affect which side tops are exposed.

The proof applies to every legal plan as written. No
exchange argument or cost-increasing normalization is
needed.

## Complete enumeration

For each j from zero through r, do the following.

1. Enumerate every balanced parenthesis word with j
   pairs. An opening means a temporary return to D;
   its matching closing means that card's repeat
   departure. There are at most 4^j such words.
2. Assign a card label to each opening, with repetition
   allowed. There are m^j assignments. Each matching
   closing has the same label as its opening.
3. Split the word into its primitive balanced components.
   If there are q components, then q <= j. Assign their
   chronological skeleton gaps in nondecreasing order.
   Components may share a gap. There are at most
   (2m+1)^q <= (2m+1)^j choices.
4. Merge these events into the fixed skeleton. Simulate
   D and reject a return of a card already present, an
   illegal departure, or a wrong final target.
5. Apply the excursion-crossing bipartiteness test from
   the previous study. Its two colors choose A and B.

The initial and final exterior gaps cannot contain legal
nonempty blocks because both side stacks are empty there.
The implementation omits these two gaps. Allowing them
in the counting bound is harmless.

For j=0 use the two-increasing-subsequence construction.
For j>0 the explicit central simulation also verifies
that each card's chronological events alternate. The
side test consequently receives proper excursions.

Soundness follows from the D simulation and crossing
graph theorem: every accepted coloring yields a physical
plan of exactly 2m+2j moves. For completeness, take any
plan within the budget, extract its fixed skeleton,
balanced temporary word, opening labels, and component
gaps. The enumeration includes that combination. Its
physical side assignment certifies bipartiteness.

There are at most

    sum(j=0..r) 4^j m^j (2m+1)^j

schedules, each requiring O((m+j)^2) time and space for
the existing feasibility test. Absorbing factors depending
only on r yields the claimed bound. Stream candidates
instead of storing them; the graph dominates workspace.

## Completed implementation checks

[The executable enumerator][program] uses exactly this
construction and independently replays every returned
word through the physical three-stack simulator. It
checks decisions and returned optimal lengths against
the existing complete exact tables.

| Domain | Targets | Decisions | Schedules | Seconds |
|---|---:|---:|---:|---:|
| n <= 7, r <= 1 | 5,913 | 11,826 | 350,121 | 2.54 |
| n <= 6, r <= 2 | 873 | 2,619 | 428,878 | 2.04 |
| n <= 6, r <= 4 | 873 | 4,365 | 7,008,211 | 27.42 |

The last run recovers the optimum for every target
through n=6: their excess is at most four. At n=6 the
numbers accepted for r=0,1,2,3,4 are respectively
132,439,675,719,720. All checks passed. Each invocation
had a 50-second internal budget and 512 MiB address-space
limit; these finite checks validate the implementation,
while the preceding proof establishes arbitrary-size
correctness.

The earlier drain-first counterexample (2,3,1,4,0)
is still accepted with 12 moves. One returned word is

```text
DA DB AD DB DA DA DA BD AD BD AD AD
```

Its third move temporarily returns card 0 to D, and its
fourth move sends that card to B, before cards 2,3,4 have
first departed. This is a balanced block in skeleton
gap two. It demonstrates why the new skeleton lemma
does not reinstate the falsified drain-first restriction.

Reproduce from the repository root:

```sh
timeout 55s python tools/parameter_complexity.py \
  --max-n 7 --max-excess 1 --seconds 50 \
  --output results/parameter-excess-one.json
timeout 55s python tools/parameter_complexity.py \
  --max-n 6 --max-excess 2 --seconds 50 \
  --output results/parameter-excess-two.json
timeout 55s python tools/parameter_complexity.py \
  --max-n 6 --max-excess 4 --seconds 50 \
  --output results/parameter-excess-four.json
```

## What this does and does not settle

For each fixed r the exponent is a constant, so that
slice has a polynomial-time exact algorithm. This is
an XP guarantee. An FPT guarantee would instead require
f(r)m^c for a constant c independent of r. The new result
reduces the XP exponent but does not provide that form.

Binary search over K does not change this distinction.
It reduces the number of decision calls to logarithmic
in a known budget range; the calls can still involve r
growing with m. Replacing a polynomial number of costly
calls by logarithmically many does not make each call
polynomial in the unrestricted input size.

For the structural bound B=4m-2L2, where L2 is the largest
two-increasing-subsequence union, a budget K=B+2s gives

    r = m-L2+s.

Thus small excess s above B does not make this algorithm
polynomial with a fixed exponent: m-L2 can grow with m.
For example, a reversed active prefix has L2=2. The
current proof gives no algorithm parameterized solely
by s, and no FPT claim in the deletion quantity m-L2.

[previous]: campaign-complexity.md
[program]: ../tools/parameter_complexity.py
