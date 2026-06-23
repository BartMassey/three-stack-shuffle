# Three-Stack Sorting Algorithms

This document is a living specification and analysis of sorting algorithms for
the three-stack card machine. The algorithms are ordered approximately from
simplest and least effective to more adaptive, specialized, or complicated.
That ordering is necessarily a judgment call: some later algorithms have
better behavior only on particular input structures, and some remain
experimental.

## 1. Machine model and accounting conventions

The stacks form a path:

```text
A — D — B
```

A legal machine operation moves the top card between adjacent stacks:

```text
A → D    D → A    D → B    B → D
```

A direct `A ↔ B` transfer is **not** one operation. It is shorthand for two
legal moves through `D`:

```text
A → D → B
```

or symmetrically:

```text
B → D → A
```

For a deck of size `n`, the initial state is

```text
A = []
D = [p1, p2, ..., pn]      # top to bottom
B = []
```

where `p1, ..., pn` is a permutation of `1, ..., n`. The goal is

```text
A = []
D = [1, 2, ..., n]         # top to bottom
B = []
```

Only machine moves are charged. Inspection of the complete permutation,
comparisons, bookkeeping, and arbitrarily expensive offline planning are free.

Consequences of that convention:

- Every algorithm may return immediately on an already sorted input, so the
  global best case is normally zero.
- A fixed-schedule algorithm may still have a nonzero *baseline* cost on a
  sorted input if that free early-exit test is omitted.
- A result is labeled as an **exact algorithm cost**, **certified bound**, or
  **experimental estimate**. These categories must not be conflated.

Notation:

- `lg n = log2 n`.
- Card sequences are written top-to-bottom.
- An ascending sequence increases top-to-bottom on `D`.
- A descending sequence decreases top-to-bottom on `D`.
- For divide-and-conquer recurrences, let

  ```text
  L = ceil(lg n)
  E(n) = nL - (2^L - n).
  ```

  `E(n)` is the minimum total leaf depth of a balanced binary tree with `n`
  leaves. For `n = 52`:

  ```text
  E(52) = 52·6 - (64 - 52) = 300.
  ```

## 2. Common reversal macros

### 2.1 Optimal reversal in `D`

Input:

```text
D = [1, 2, ..., n]
```

Output:

```text
D = [n, n-1, ..., 1]
```

For `n >= 2`:

```text
repeat n-1 times: D → A
D → B

repeat n-2 times:
    A → D
    D → B

A → D
repeat n-1 times: B → D
```

Exact cost:

```text
4n - 4.
```

This is optimal. Every card must leave `D` and return. At most two cards can
make the round trip in two moves without retaining the wrong relative order;
the other `n-2` cards require at least four moves each.

For `n = 52`:

```text
204 moves.
```

### 2.2 Segment-reversal macros

For a top segment of length `k`:

```text
reverse D onto A or B       3k - 2
reverse A or B onto D       3k - 2
reverse A onto B, or B→A    2k
reverse in place on A or B  4k - 2
```

Example: reverse a descending segment from `D` onto `A`:

```text
repeat k-1 times: D → B
D → A
repeat k-1 times:
    B → D
    D → A
```

Example: reverse a descending segment from `A` onto `D`:

```text
repeat k-1 times:
    A → D
    D → B
A → D
repeat k-1 times: B → D
```

Protected cards below the active segments are not disturbed.

---

## 3. SELECTION SORT

### Motivation

This is the simplest complete algorithm. It repeatedly sweeps all unsorted
cards from one endpoint to the other, leaving the next required output card on
`D` whenever it is encountered.

The output is constructed from the bottom upward, so cards are selected in the
order

```text
n, n-1, ..., 1.
```

A useful optimization is to freeze the largest suffix already in its final
position.

### Pseudocode

```text
m := largest value such that
     D = [p1, ..., pm, m+1, m+2, ..., n]

if m = 0:
    stop

move the top m cards D → A

source := A
destination := B
next := m

while next > 0:
    while source contains active cards:
        if top(source) = next:
            source → D
            next := next - 1
        else:
            source → D
            D → destination

    swap(source, destination)
```

A sweep may place several consecutive required cards on `D`.

### Exact cost for a given active prefix

Let `m` be the unfrozen prefix length and let `Q` be the number of bypassed
cards. Each active card is moved once from `D` to `A` and eventually once from
an endpoint to its final position on `D`. Each bypass costs two moves.

```text
C = 2m + 2Q.
```

### Best and worst cases

Best case:

```text
0
```

because a completely sorted deck is entirely frozen.

For fixed `m`, the no-bypass best case is `2m`.

In the worst case, a sweep with `r` remaining cards selects only one and
bypasses the other `r-1`. Therefore

```text
Q_max = (m-1) + (m-2) + ... + 1
      = m(m-1)/2
```

and

```text
C_max(m) = m^2 + m.
```

The global exact worst case is therefore

```text
C_worst(n) = n^2 + n.
```

### Expected case

Without suffix freezing, let `x1, ..., xn` be the original positions of cards

```text
n, n-1, ..., 1.
```

A new sweep is required before `x2` with probability `1/2`. Thereafter, a new
sweep occurs when `xi` is a local extremum of three random distinct positions,
which has probability `2/3`. The exact expected bypass count is

```text
E[Q] = (n-1)(2n-1)/6
```

and hence

```text
E[C] = 2n + 2E[Q]
     = (n+1)(2n+1)/3.
```

Suffix freezing changes the expectation only slightly. If `M` is the active
prefix length, then

```text
P(M = 0) = 1/n!
P(M = m) = (m-1)(m-1)! / n!       for 2 <= m <= n.
```

Conditioned on `M = m`, the expected cost is

```text
H(m) = (2m^3 + m^2 + 2m - 6) / (3(m-1)).
```

Thus the frozen-suffix expectation is the exact finite sum

```text
E[C_frozen]
  = sum from m=2 to n of P(M=m) H(m).
```

### `n = 52`

```text
best:                         0
expected without freezing:   1855
expected with freezing:      1854.9607689164
worst:                        2756
```

---

## 4. ADAPTIVE SELECTION SORT

### Motivation

Ordinary SELECTION SORT finishes each sweep even after finding a required card.
ADAPTIVE SELECTION SORT turns around immediately. The unsorted cards remain
split between `A` and `B`, and the algorithm moves directly toward the next
required card.

This is Gene Welborn's adaptive selection algorithm.

### Pseudocode

```text
freeze the maximal correct suffix
move the active prefix D → A

next := largest active value

while next > 0:
    source := the endpoint stack containing card next
    destination := the other endpoint

    while top(source) != next:
        source → D
        D → destination

    source → D
    next := next - 1
```

The location of every card is known from free offline planning. This is not an
online search algorithm.

### Exact cost

Let `Q` be the number of cards bypassed between successive required cards.
Then

```text
C = 2m + 2Q
```

for an active prefix of length `m`.

Gene's original count treated each direct `A ↔ B` transfer as cost one and
often counted only `Q`. On this machine, every such transfer costs two legal
moves.

### Best and worst cases

Best case with suffix freezing:

```text
0.
```

Without freezing, the no-bypass best case is `2n`.

Successive required cards can alternate between opposite ends of the
remaining order, giving

```text
Q_max = n(n-1)/2
```

and the exact global worst case

```text
C_worst(n) = n^2 + n.
```

Thus adaptivity greatly improves the average case but not the worst case.

### Expected case

When `m` cards remain, the ranks `R` and `S` of two successive required cards
are uniformly random distinct positions. The expected number of cards strictly
between them is

```text
E[|R-S|-1] = (m-2)/3.
```

Without suffix freezing:

```text
E[Q] = sum from m=2 to n of (m-2)/3
     = (n-1)(n-2)/6
```

and

```text
E[C] = 2n + (n-1)(n-2)/3.
```

With suffix freezing, use the same distribution of `M` as for SELECTION SORT.
Conditioned on `M=m`, the exact expected cost is

```text
A(m) = m(m^2 + 2m - 2) / (3(m-1)).
```

Therefore

```text
E[C_frozen]
  = sum from m=2 to n of P(M=m) A(m).
```

### `n = 52`

```text
Gene's expected bypass count Q:       425
expected legal cost, no freezing:     954
expected legal cost, with freezing:   952.9803844582
best:                                 0
worst:                                2756
```

The reported value `425` is therefore exactly explained by counting a direct
endpoint-to-endpoint bypass as one step and omitting setup and final placement.

---

## 5. LOOKAHEAD SELECTION SORT

### Motivation

This algorithm is due to **Gene Welborn**. It improves adaptive selection by
recognizing consecutive future targets while searching for the current target.
Those cards are staged temporarily on `D` and then moved together to the other
endpoint. The staging reverses their order on the destination, leaving the
next target exposed instead of buried.

As with all algorithms in this document, first perform the free sorted-input
test. Also freeze the maximal correct suffix, exactly as in SELECTION SORT and
ADAPTIVE SELECTION SORT. Thus an already sorted input costs zero, and only the
active prefix of length `m` participates in the algorithm.

### Pseudocode

```text
m := largest value such that
     D = [p1, ..., pm, m+1, m+2, ..., n]

if m = 0:
    stop

move the top m cards D → A

current := m

while current > 0:
    source := the endpoint containing current
    destination := the other endpoint
    lookahead := current - 1
    held := 0

    while top(source) != current:
        if top(source) = lookahead:
            source → D
            lookahead := lookahead - 1
            held := held + 1
        else:
            source → D
            D → destination

    repeat held times:
        D → destination

    source → D
    current := current - 1
```

The cards counted by `held` are precisely the consecutive future targets
`current-1, current-2, ...` encountered during the search. They temporarily
sit above the frozen suffix and previously finalized cards on `D`. Moving the
held block to `destination` exposes `current-1` there, so the next selection
may be immediate.

The complete state is known, so locating the endpoint containing `current` is
free offline bookkeeping, as it is for ADAPTIVE SELECTION SORT.

### Legal-move accounting

Staging a lookahead card is not a free or one-move endpoint transfer. Its two
steps are separated in time:

```text
source → D
...
D → destination
```

and therefore cost two legal moves, exactly like an ordinary endpoint bypass.
Let `Q` count all nonfinal endpoint-to-endpoint relocations, including staged
lookahead cards. The exact cost for an active prefix is

```text
C = 2m + 2Q.
```

The first `m` pays for setup, the second `m` pays for final placement, and
every relocation in `Q` pays two adjacent-stack moves.

### Bounds and status

Best case with suffix freezing:

```text
0.
```

For a fixed active size `m`, the no-relocation best case is `2m`. During the
selection of one target, each other active card can be relocated at most once.
Consequently,

```text
Q <= m(m-1)/2
C <= m^2 + m.
```

This is a certified bound, not a claim that the bound is attained for every
`m`. The exact random-input expectation and exact worst case remain open.

For `n=52`, a deterministic benchmark over 20,000 random permutations with
seed `24301` measured:

```text
mean:             810.5857
standard error:     0.6167
minimum:           506
maximum:          1184
```

This is an experimental estimate, not a calculated expectation. On the same
inputs, ADAPTIVE SELECTION SORT measured `952.5079`, confirming that lookahead
provides a substantial standalone average-case improvement. BINARY-PRESORT
ADAPTIVE SELECTION SORT measured `553.8948`, so standalone lookahead does not
supersede value presorting.

The important distinction from ADAPTIVE SELECTION SORT is order, not a cheaper
move convention: batching consecutive future targets reverses their order on
the destination and can avoid later traversal. This lookahead rule can also be
combined with value presorting; in that variant the value buckets should
remain on separate endpoints rather than being consolidated.

---

## 6. `2K`-PARTITION LOOKAHEAD SELECTION SORT

### Motivation

This single parameterized algorithm combines two ideas due to **Gene
Welborn**: balanced value partitioning and staging consecutive future targets
while searching for the current target. Its positive integer parameter `K`
selects `2K` value buckets. The former 2-, 4-, 6-, and 8-partition algorithms
are exactly the configurations `K=1`, `K=2`, `K=3`, and `K=4`.

First perform the free sorted-input test and freeze the maximal correct suffix.
Only the active prefix of length `m` is partitioned or moved.

### Pseudocode

```text
m := largest value such that
     D = [p1, ..., pm, m+1, m+2, ..., n]

if m = 0:
    stop

buckets := min(2K, m)

recursively divide [1, m] into `buckets` balanced value intervals:
    split the current interval's buckets into lower and upper groups
    partition its cards by value onto A and B
    process the upper group recursively
    process the lower group recursively

at each one-bucket leaf:
    extract its interval in descending order with LOOKAHEAD SELECTION SORT
```

The root partition starts on `D` and costs one move per card. Every deeper
partition moves its group from an endpoint through `D` and back to the two
endpoints, costing two moves per card. Upper groups are completed before lower
groups. Higher cards temporarily moved onto a protected lower group remain
above it and are removed before that group is refined.

### Legal-move accounting and bounds

Let `P(m,K)` be the fixed partition-tree and final-placement cost, and let `Q`
count all nonfinal endpoint-to-endpoint relocations, including staged
lookahead cards. The exact cost is

```text
C = P(m,K) + 2Q.
```

For `K=1`, only the root partition is needed, so `P(m,1)=2m`. Let

```text
a = floor(m/2)
b = ceil(m/2).
```

No low card must be traversed while selecting the high bucket, and the high
cards are finalized before low extraction begins. Therefore a certified bound
is

```text
Q <= a(a-1)/2 + b(b-1)/2
C <= 2m + a(a-1) + b(b-1)                 (K=1).
```

In general, set `b=min(2K,m)`. If the balanced leaf bucket sizes are
`s1, ..., sb`, then

```text
C <= P(m,K) + sum(si(si-1)).
```

Best case with suffix freezing is zero. The exact expected cost and exact
worst case remain open.

### `K=1`, `n=52`

A deterministic benchmark over 20,000 random permutations with seed `24301`
measured:

```text
mean:             457.6273
standard error:     0.3074
minimum:           294
maximum:           644
certified bound:  1404
```

On the identical inputs:

```text
LOOKAHEAD SELECTION SORT:                 810.5857
2K-PARTITION LOOKAHEAD SELECTION, K=1:    457.6273
BINARY-PRESORT ADAPTIVE SELECTION SORT:   553.8948
ADAPTIVE SELECTION SORT:                  952.5079
```

The measured values are experimental estimates, not calculated expectations.

---

## 6.1 Choosing `K`

### Motivation and operation

For `K=2`, a second level of value partitioning precedes lookahead selection.
Because only two endpoint stacks are available, four buckets cannot all be
refined independently at once. Instead, refinement is just in time:

1. Partition the active values into low and high halves on `A` and `B`.
2. Move the high half back to `D`, partition it into two quarters, and extract
   all high cards with lookahead.
3. The untouched low half is now exposed on `A`. Move it back to `D`, partition
   it into two quarters, and extract it with lookahead.

Higher-valued cards temporarily placed on `A` remain above the protected low
half. Descending extraction removes all of them before low-half refinement.

### Accounting and bound

For an active prefix of length `m`, the first partition costs `m`, the two
second-level partitions together cost `2m`, and final placement costs `m`.
If `Q` counts nonfinal endpoint relocations, including staged lookahead cards,
the exact cost is

```text
C = 4m + 2Q.
```

Let `k1, ..., k4` be the four bucket sizes, differing by at most one. Cards in
different buckets never need to be traversed together, giving the certified
bound

```text
C <= 4m + sum(ki(ki-1)).
```

For `m=52`, all four buckets contain 13 cards, so the certified bound is 832.

A deterministic benchmark over 20,000 random permutations with seed `24301`
measured:

```text
mean:             385.3420
standard error:     0.1531
minimum:           308
maximum:           472
certified bound:   832
```

The corresponding mean relocation count is about `88.671`, compared with
`176.814` for the `K=1` configuration. The extra partition
level costs 104 moves but saves about 176.285 relocation moves, a net mean
improvement of about 72.285 moves.

The binary partition tree works for every positive `K`; `2K` need not be a
power of two. For 52 cards, six buckets have sizes
`9, 9, 9, 9, 8, 8`; eight have sizes `7, 7, 7, 7, 6, 6, 6, 6`.

On the same 20,000 permutations, deeper partitioning did not improve on four
buckets:

| Buckets | Fixed partition and placement cost | Mean relocations | Mean moves | Certified bound |
|---:|---:|---:|---:|---:|
| `K=1` (2) | 104 | 176.814 | 457.627 | 1404 |
| `K=2` (4) | 208 | 88.671 | 385.342 | 832 |
| `K=3` (6) | 276 | 59.200 | 394.401 | 676 |
| `K=4` (8) | 312 | 44.534 | 401.068 | 600 |

The smaller buckets continue to reduce both relocations and the certified
bound, but their extra endpoint-to-`D`-to-endpoint partition levels cost more
than they save on average. Four buckets are therefore the measured optimum
among the one-, two-, four-, six-, and eight-bucket configurations tested
here (where one bucket is standalone LOOKAHEAD SELECTION SORT).

### 6.2 Experimental optimized lookahead pass

Consecutive lookahead is sufficient but not necessary. While uncovering
`current`, any subset of the blockers may be left temporarily on `D`; every
other blocker crosses immediately to the other endpoint. If the blockers in
top-to-bottom encounter order are `X`, a capture subset `H` and its complement
`P` produce

```text
new source      = cards below current
new destination = H ++ reverse(P) ++ old destination
```

where `H` and `P` retain their encounter order before the displayed reversal.
Every mask has the same immediate cost, `2|X| + 1`: two primitive moves per
blocker and one to finalize `current`. The choice matters only through the
states presented to later passes.

An exact dynamic program over this selection family is:

```text
OPT(current, A, B):
    if current < bucket.low:
        return 0

    canonicalize (A, B) under endpoint symmetry
    if memo contains (current, A, B):
        return memo[current, A, B]

    source := endpoint containing current
    destination := the other endpoint
    write source = X ++ [current] ++ tail

    best := infinity
    for each subset H of X:
        P := X with the cards of H removed
        new source := tail
        new destination := H ++ reverse(P) ++ destination
        cost := 2|X| + 1 + OPT(current - 1, new A, new B)
        best := min(best, cost)

    memo[current, A, B] := best
    return best
```

The `2^|X|` masks are only the branching factor at one state; the number of
reachable endpoint states can still grow factorially. The first practical
experiment therefore uses a receding-horizon rollout rather than the full
state DP:

```text
ROLLOUT LOOKAHEAD PASS(current):
    source := endpoint containing current
    X := blockers above current on source

    best mask := the consecutive-lookahead mask
    best score := infinity

    for each subset H of X:
        simulate this pass with exactly H staged on D
        finish the current value bucket with consecutive lookahead
        score := simulated primitive moves
        retain H if score is strictly smaller

    execute the best mask
```

Ties retain the consecutive rule. The experiment repeats this optimization at
every target, so each committed pass receives a new full-bucket rollout. The
implementation limits a pass to 16 blockers to keep exhaustive enumeration
deliberately small. For `n=52`, the four-bucket configuration (`K=2`) has
13-card leaves and therefore satisfies this limit.

A deterministic initial benchmark over 2,000 random 52-card permutations with
seed `24301` measured:

| `K` | Buckets | Consecutive mean | Rollout mean | Mean reduction | Rollout time |
|---:|---:|---:|---:|---:|---:|
| 2 | 4 | 385.620 | 343.931 | 41.689 | 8.20 s |
| 3 | 6 | 394.347 | 376.051 | 18.296 | 0.69 s |
| 4 | 8 | 401.046 | 392.375 | 8.671 | 0.26 s |
| 5 | 10 | 422.986 | 418.789 | 4.197 | 0.16 s |
| 6 | 12 | 439.258 | 437.339 | 1.919 | 0.11 s |

These are experimental sample means, not calculated expectations. Rollout
improves every tested bucket count, with rapidly diminishing benefit as the
leaves shrink. `K=2` remains the best tested configuration and reduces the
mean by about 10.8% relative to consecutive lookahead at the cost of roughly
240 times the solver runtime in this benchmark. `K=1` at `n=52` has 26-card
leaves and is intentionally rejected by the 17-card leaf limit.

---

## 7. BINARY-PRESORT ADAPTIVE SELECTION SORT

### Motivation

One value-partition pass reduces the distances traversed by ADAPTIVE SELECTION
SORT. This is Gene's “presort into two piles” idea.

For `n = 52`, the buckets are

```text
low:   1..26
high: 27..52
```

corresponding to Gene's zero-based `0..25` and `26..51`.

### Pseudocode

Let

```text
a := floor(n/2)
```

Partition once:

```text
while D is nonempty:
    if top(D) <= a:
        D → A
    else:
        D → B
```

Then run adaptive extraction on the high bucket:

```text
next := n

while next > a:
    source := endpoint containing next
    destination := other endpoint

    while top(source) != next:
        source → D
        D → destination

    source → D
    next := next - 1
```

The low bucket remains protected below any temporary high cards placed on its
endpoint. Once the high bucket is exhausted, run the same adaptive extraction
for

```text
a, a-1, ..., 1.
```

No literal “stack low on high” transfer is needed.

### Exact baseline costs

Let

```text
a = floor(n/2)
b = ceil(n/2).
```

Partitioning costs `n`; final placements cost another `n`.

Best case without a free sorted-input exit:

```text
C_best = 2n.
```

The exact worst bypass count is

```text
Q_max = a(a-1)/2 + b(b-1)/2
```

so

```text
C_worst
  = 2n + a(a-1) + b(b-1).
```

The global best case is zero if the already-sorted input is detected before
partitioning.

### Expected case

The relative order within each bucket is uniformly random. For a bucket of
size `s`, reaching its maximum from the exposed top after partitioning has an
expected `(s-1)/2` bypasses. The subsequent distances between required cards
contribute `(s-1)(s-2)/6` expected bypasses. Therefore

```text
E[Q]
  = (a-1)/2 + (b-1)/2
    + [(a-1)(a-2) + (b-1)(b-2)] / 6
```

and

```text
E[C]
  = 2n
    + (a-1) + (b-1)
    + [(a-1)(a-2) + (b-1)(b-2)] / 3.
```

The earlier value that omitted `(a-1)/2 + (b-1)/2` implicitly assumed that
each bucket maximum was already exposed after partitioning. The literal
partition pseudocode above does not guarantee that orientation.

This remains quadratic, but its leading expected adaptive term is halved.
Recursively adding more value partitions leads toward radix sort.

### `n = 52`

```text
a = b = 26
best with free sorted detection:  0
baseline best:                    104
expected bypasses:                225
expected legal cost:              554
exact worst:                      1404
```

Gene's reported value near `320` is compatible with a different step
convention plus extra pile-stacking overhead; it is not our legal-move count.

---

## 8. MERGE SORT

### Motivation

This is conventional top-down merge sort translated literally to the stack
machine. It is asymptotically good but pays substantial positioning overhead.

### Pseudocode

```text
MERGE_SORT(k):
    if k <= 1:
        return

    a := ceil(k/2)
    b := floor(k/2)

    move top a cards D → A
    move next b cards D → B

    move a cards A → D
    MERGE_SORT(a)
    move a cards D → A

    move b cards B → D
    MERGE_SORT(b)
    move b cards D → B

    while both sorted halves are nonempty:
        move the larger exposed card to D

    move the remaining half to D
```

After a sorted half is moved to an endpoint, its maximum is exposed. Moving
larger cards first to `D` produces an ascending sequence top-to-bottom.

### Exact cost

At an internal node of size `k`:

```text
split:                    k
position recursive halves: 2k
merge:                    k
```

Therefore

```text
T(k)
  = T(ceil(k/2)) + T(floor(k/2)) + 4k
T(0) = T(1) = 0.
```

The exact balanced-tree solution is

```text
T(n) = 4E(n).
```

For powers of two:

```text
T(n) = 4n lg n.
```

The fixed implementation is input-independent. A free sorted-input check gives
best case zero; otherwise best, expected, and worst costs all equal `T(n)`.

### `n = 52`

```text
baseline / expected / worst:  4E(52) = 1200
best with sorted detection:   0
```

---

## 9. MSB RADIX SORT

### Motivation

MSB RADIX SORT partitions by value range rather than by physical position.
Unlike MERGE SORT, the two recursive results need only be concatenated, not
comparison-merged.

### Pseudocode

For an active consecutive value interval of size `k`:

```text
MSB_SORT(low, high):
    k := high - low + 1

    if k <= 1:
        return

    a := floor(k/2)
    split_value := low + a - 1

    while active D segment is nonempty:
        if top(D) <= split_value:
            D → A
        else:
            D → B

    move a-card lower bucket A → D
    MSB_SORT(low, split_value)
    move lower bucket D → A

    move upper bucket B → D
    MSB_SORT(split_value+1, high)

    move sorted lower bucket A → D
```

The lower values finish above the upper values.

### Exact cost

For lower size `a = floor(k/2)` and upper size `b = k-a`:

```text
T(k) = T(a) + T(b) + 2k + 2a.
```

The extra round trip is assigned to the smaller bucket.

For powers of two:

```text
T(n) = 3n lg n.
```

This fixed implementation is input-independent. A sorted-input exit gives best
case zero; otherwise its cost is exact for every input.

### `n = 52`

```text
baseline / expected / worst:  880
best with sorted detection:   0
```

This is a straightforward implementation, not a claim of optimality among all
orientation-aware MSB radix schemes.

---

## 10. LSB RADIX SORT

### Motivation

LSB RADIX SORT is one of the cleanest reliable `O(n lg n)` algorithms on this
machine. It performs stable binary bucket passes from the least significant
bit upward.

### Pseudocode

Sort by the bits of `card-1`:

```text
for bit := 0 to ceil(lg n)-1:
    while D is nonempty:
        if bit of (top(D)-1) is 0:
            D → A
        else:
            D → B

    while B is nonempty:
        B → D

    while A is nonempty:
        A → D
```

Each bucket is reversed going out and reversed again coming back, preserving
its internal order. Returning `B` before `A` leaves the zero bucket above the
one bucket.

### Correctness invariant

After pass `j`, the deck is stably sorted by the low `j+1` bits of `card-1`.

### Exact cost

Each pass moves every card out and back:

```text
2n moves per pass.
```

For the fixed schedule:

```text
T(n) = 2n ceil(lg n).
```

This is exact and input-independent. A free sorted-input check changes the
global best case to zero and changes the random-input expectation only by the
negligible factor `1 - 1/n!`.

### `n = 52`

```text
baseline / expected / worst:  624
best with sorted detection:   0
```

---

## 11. NATURAL SORT

### Motivation

NATURAL SORT exploits ascending runs already present in the input. It retains
the same worst-case bound as LSB RADIX SORT but is substantially better on
typical random inputs and on nearly sorted decks.

### Pseudocode

At the start of a pass, regard `D` as a concatenation of maximal ascending
runs.

```text
while D contains more than one ascending run:
    move whole runs alternately D → A and D → B

    while A and B both contain runs:
        merge the topmost run from A
        with the topmost run from B onto D:
            repeatedly move the larger exposed card
            then move the remainder

    if one endpoint has an unmatched run:
        move that run to D
```

Moving an ascending run to an endpoint exposes its maximum. The merge emits
cards in decreasing order onto `D`, producing one ascending run.

For a deliberately simple full-pass implementation, every pass moves every
card out of `D` and back once.

### Exact cost in terms of initial runs

Let `R` be the number of maximal ascending runs initially. Each pass replaces
at most two runs by one, so the full-pass implementation uses exactly

```text
ceil(lg R)
```

passes and

```text
C(n,R) = 2n ceil(lg R).
```

Thus:

```text
best:   0                  when R = 1
worst:  2n ceil(lg n).
```

The decreasing deck has `R=n` and attains the worst case.

### Expected case

For a uniformly random permutation,

```text
R = 1 + number of descents.
```

If `A(n,k)` is the Eulerian number counting permutations with exactly `k`
descents, then the exact expected cost is

```text
E[C]
  = (2n / n!)
    · sum from k=0 to n-1 of
      A(n,k) ceil(lg(k+1)).
```

This expectation is inexpensive to compute exactly using the Eulerian
recurrence.

### `n = 52`

```text
best:       0
expected:   520.1955606296
worst:      624
reversal:   624
```

---

## 12. HU–TUCKER NATURAL MERGE SORT

### Motivation

NATURAL SORT discovers useful ascending runs, but then merges them in uniform
full passes. A card in a long run consequently pays for every pass even when
that run should be kept near the root of the merge schedule.

HU–TUCKER NATURAL MERGE SORT retains the same maximal ascending runs and chooses
a minimum-cost binary merge tree subject to preserving their physical order.
This is an **alphabetic merge tree**: its leaves, read left-to-right, remain in
the original top-to-bottom run order. The name describes the optimization
problem; the implementation solves it directly with an interval dynamic
program rather than the asymptotically faster Hu–Tucker construction.

### Tree construction

Let the initial ascending-run lengths be

```text
s1, s2, ..., sR
```

and let `C(i,j)` be the minimum weighted path length of an alphabetic binary
tree spanning runs `i` through `j`. Then

```text
C(i,i) = 0

C(i,j)
  = (si + ... + sj)
    + min over i <= k < j of
      (C(i,k) + C(k+1,j)).
```

Recording the minimizing split reconstructs an optimal tree. The direct
implementation uses `O(R^3)` time and `O(R^2)` memory; `R <= n <= 52` makes
that cost negligible under the machine accounting convention.

### Machine realization

For each internal node, let `upper` and `lower` be its two consecutive child
segments in top-to-bottom order on `D`:

```text
REALIZE(node):
    if node is one ascending-run leaf:
        return

    REALIZE(upper)
    move the sorted upper segment D → A

    REALIZE(lower)
    move the sorted lower segment D → B

    while both endpoint segments are nonempty:
        move the larger exposed card to D

    move the remaining endpoint segment to D
```

Moving an ascending segment from `D` to an endpoint exposes its maximum.
Returning the larger exposed card first therefore constructs one ascending
segment on `D`. Exact child sizes delimit every operation, so segments parked
below the active node are never consumed.

Correctness follows by induction on the tree. Leaves are ascending initially;
an internal node merges two correctly sorted consecutive children into one
ascending segment; the root covers the complete deck.

### Exact cost for a given input

At an internal node containing `s` cards, moving both children to the endpoints
costs `s`, and merging them back costs another `s`. Thus the node costs exactly

```text
2s.
```

If leaf `i` has depth `di`, the complete exact algorithm cost is

```text
C = 2W
W = sum from i=1 to R of si di.
```

The dynamic program minimizes `W` over every order-preserving binary merge
tree. Consequently this algorithm is never more expensive than a balanced
full-pass merge tree over the same runs, although its cost remains an upper
bound on the unrestricted machine optimum rather than a claim of global
optimality.

### Best and worst cases

One ascending run requires no moves, so the best case is zero.

Refining runs into singleton leaves cannot reduce the minimum alphabetic-tree
cost. The maximum is therefore attained by the decreasing input, whose `n`
unit-weight leaves have minimum total depth `E(n)`. Hence

```text
worst = 2E(n).
```

For powers of two this is `2n lg n`, improving NATURAL SORT only on inputs
whose run lengths permit a better unbalanced schedule; for non-powers of two
it also improves the certified worst-case constant term.

No closed form for the random-permutation expectation is established here.

### `n = 52`

```text
best:                         0
measured random mean:         484.1695
measured standard error:      0.0893
samples / seed:               20,000 / 0x5eed
worst:                        2E(52) = 600
reversal:                     600
```

The measured value is an experimental estimate, not a calculated expectation.

---

## 13. SIGNED NATURAL SORT — EXPERIMENTAL

### Motivation

Ordinary NATURAL SORT recognizes only ascending runs. SIGNED NATURAL SORT also
recognizes descending runs and reverses them before merging. Its principal
motivation is to avoid treating a reversed deck as `n` singleton runs.

### Conceptual pseudocode

```text
repeat until D is sorted:
    decompose the active deck into maximal monotone runs

    for each run, in order:
        if run is ascending:
            move it directly to the assigned endpoint
        else:
            reverse it onto the assigned endpoint

    merge adjacent normalized ascending runs back onto D
```

A descending run of length `k` can be reversed from `D` onto an endpoint in

```text
3k - 2
```

moves. Ascending runs cost `k` to transfer.

### What is established

- Correct normalization and merge macros exist.
- A completely reversed deck can be handled by the optimal central reversal
  primitive in exactly

  ```text
  4n - 4
  ```

  moves.
- For `n=52`, that score is `204`.

### What remains unresolved

A fully satisfactory phase rule must specify how maximal monotone runs interact
with ascending sequences produced by earlier phases, and its exact worst and
expected cases have not been proved.

A safe implementation may always compare its planned signed treatment with
ordinary NATURAL SORT and execute the cheaper complete plan; that hybrid has
the certified NATURAL SORT worst-case bound but is not the pure algorithm
whose optimal structure remains under study.

### `n = 52`

```text
best:               0
reversal:           204
expected:           unknown
pure worst case:    unknown
safe-hybrid bound:  at most 624
```

---

## 14. SPLIT-MERGE SORT

### Motivation

NATURAL SORT respects existing contiguous runs. SPLIT-MERGE SORT searches for
a longer prefix that can be partitioned, while preserving card order, into two
ascending subsequences. This can combine structure that is interleaved rather
than contiguous.

### Phase blocks

At the beginning of a phase, identify the maximal ascending sequences on `D`.
These are the **phase blocks**. A split prefix must contain whole phase blocks;
it may not stop inside one.

### Pseudocode

```text
while D has more than one phase block:
    identify the ascending phase blocks

    while unprocessed cards remain in D:
        among prefixes made of whole phase blocks, find the longest
        prefix partitionable into:

            ↑
            ↑ ↑

        prefer one sequence when prefix lengths tie
        use a fixed deterministic tie-break thereafter

        if one sequence is selected:
            move it to the endpoint with fewer recorded sequences
        else:
            split the prefix between A and B, preserving order

    repeatedly merge the topmost A sequence
    with the topmost B sequence onto D

    if one endpoint has one unmatched sequence:
        move it onto D last
```

Every split output is ascending logically and therefore merge-ready on its
endpoint.

### Correctness and phase bound

A one-sequence selection that leaves another phase block cannot be maximal:
the next complete ascending block could serve as a second sequence. Therefore
every nonfinal split iteration emits two sequences.

A two-sequence iteration consumes at least two phase blocks. If it consumed
only one ascending block, the preferred one-sequence decomposition would use
the same prefix.

Thus a phase beginning with `r` blocks emits at most `r` sequences. Pairwise
merging leaves at most

```text
ceil(r/2)
```

new blocks. The algorithm terminates in at most `ceil(lg r)` phases.

### Exact costs

Every phase moves each card from `D` to an endpoint once and back to `D` once:

```text
2n moves per phase.
```

Therefore:

```text
best:   0
worst:  2n ceil(lg n).
```

The reversed deck realizes the worst case: at every level it is a descending
sequence of ascending value intervals, and two ascending subsequences can
cover at most two such intervals.

### `n = 52`

```text
best:       0
worst:      624
reversal:   624
expected:   unknown
```

The expectation is plausibly below NATURAL SORT's, but no exact distribution
has yet been derived.

---

## 15. REVERSING SPLIT-MERGE SORT — EXPERIMENTAL

### Motivation

This extends SPLIT-MERGE SORT by allowing selected subsequences to be
descending and then normalizing them. It is designed to exploit long reversed
structure without giving up the logarithmic phase bound.

### Phase blocks and legal candidate forms

As in SPLIT-MERGE SORT, a selected prefix must consist of whole ascending phase
blocks.

For each such prefix, consider decompositions preserving card order into one
of:

```text
↑
↓
↑ ↑
↑ ↓
↓ ↑
↓ ↓
```

A singleton is classified as ascending. A genuine descending subsequence has
length at least two.

### Candidate selection rule

Among all legal candidates, choose lexicographically by:

1. maximum prefix length;
2. minimum exact split-and-normalization cost;
3. minimum number of output sequences;
4. minimum number of cards assigned to descending subsequences;
5. a fixed lexicographic membership-bitstring tie-break.

This removes the obsolete fixed minimum descending length of five. Whether a
short descending subsequence is worthwhile is decided by its exact cost and by
how much prefix it enables the phase to consume.

### Pseudocode

```text
while D has more than one phase block:
    identify the ascending phase blocks

    while unprocessed cards remain:
        enumerate legal candidates of the six forms
        whose prefix consists of whole phase blocks

        choose by the lexicographic rule above
        execute the corresponding split/normalization macro
        record the resulting ascending endpoint sequence(s)

    merge endpoint sequences pairwise back onto D
    move one unmatched sequence onto D last, if present
```

### Exact split-and-normalization costs

The costs below include removing the selected prefix from `D` and leaving its
output sequences ascending on the endpoints. They exclude the later merge.

| Case | Exact cost |
|---|---:|
| `↑a` | `a` |
| `↓a` | `3a - 2` |
| `↑a ↑b` | `a + b` |
| `↑a ↓b` | `a + 5b - 2` |
| `↓a ↑b` | `5a + b - 2` |
| `↓m ↓M`, `m <= M` | `5m + 3M - 2` |

For `↓↓`, put the shorter descending sequence on `A` and the longer on `B`:

```text
reverse A → D
reverse B → A
move the protected D segment → B
```

### Correctness and phase bound

The whole-phase-block restriction gives the same counting argument as
SPLIT-MERGE SORT:

- one output consumes at least one phase block;
- two outputs consume at least two phase blocks;
- the split emits no more sequences than the number of input blocks;
- pairwise merging at least halves the block count.

Hence there are at most

```text
ceil(lg n)
```

phases.

Every split case costs at most five moves per selected card, and the merge
costs `n`. Therefore a simple certified worst-case upper bound is

```text
C(n) <= 6n ceil(lg n).
```

This is only a coarse bound, not a claim about the true worst case.

### Reversal score

For a fully reversed deck, the maximum prefix is the entire deck as one
descending sequence. Reversing `D` onto an endpoint costs `3n-2`; moving the
unmatched ascending sequence back to `D` costs `n`.

```text
C_reversal(n) = 4n - 2.
```

For `n=52`:

```text
206 moves.
```

The standalone optimal reversal primitive is two moves better, but reversal is
handled naturally rather than as a special case.

### `n = 52`

```text
best:                         0
reversal:                     206
expected:                     unknown
certified worst-case bound:   1872
true worst case:              unknown
```

---

## 16. General lower bounds

Let `M(n)` be the maximum, over all input permutations, of the minimum legal
move count needed to sort that permutation.

### Counting bound

Moves are reversible. A shortest program never immediately undoes its previous
move. The first move has at most two choices and each later reduced move has at
most three choices, so programs of length at most `m` are bounded by `3^m` up
to an inessential constant.

Therefore:

```text
M(n) >= ceil(log_3(n!)).
```

For `n=52`:

```text
ceil(log_3(52!)) = 143.
```

### Reversal bound

Reversal requires exactly `4n-4`, so:

```text
M(n) >= 4n - 4.
```

For `n=52`:

```text
M(52) >= 204.
```

The current elementary bound is therefore:

```text
M(52) >= max(143, 204) = 204.
```

Asymptotically, the counting argument gives:

```text
M(n) = Omega(n lg n).
```

---

## 17. `n = 52` comparison

All figures count legal adjacent-stack moves.

| Algorithm | Best | Expected random input | Worst / certified bound | Special structured score |
|---|---:|---:|---:|---:|
| SELECTION SORT | 0 | 1854.961 | 2756 exact | — |
| ADAPTIVE SELECTION SORT | 0 | 952.980 | 2756 exact | Gene's bypass mean: 425 |
| LOOKAHEAD SELECTION SORT | 0 | unknown; measured 810.586 | <=2756 certified | Gene Welborn's algorithm |
| `2K`-PARTITION LOOKAHEAD SELECTION SORT | 0 | measured 457.627 (`K=1`), 385.342 (`K=2`), 394.401 (`K=3`), 401.068 (`K=4`) | <=1404, 832, 676, 600 certified | Gene Welborn's combined ideas |
| BINARY-PRESORT ADAPTIVE SELECTION SORT | 0 | 554 baseline | 1404 exact | — |
| MERGE SORT | 0 | 1200 baseline | 1200 exact | — |
| MSB RADIX SORT | 0 | 880 baseline | 880 exact | — |
| LSB RADIX SORT | 0 | 624 baseline | 624 exact | — |
| NATURAL SORT | 0 | 520.196 | 624 exact | reversal: 624 |
| HU–TUCKER NATURAL MERGE SORT | 0 | unknown; measured 484.170 | 600 exact | reversal: 600 |
| SIGNED NATURAL SORT | 0 | unknown | pure worst unknown; safe hybrid <=624 | reversal: 204 |
| SPLIT-MERGE SORT | 0 | unknown | 624 exact | reversal: 624 |
| REVERSING SPLIT-MERGE SORT | 0 | unknown | certified <=1872 | reversal: 206 |

Reference values:

```text
optimal reversal:                  204
general counting lower bound:      143
current general lower bound:       204
```

The table is not a total ranking. For example, fixed LSB RADIX SORT has a
better certified worst case than the current bound for REVERSING SPLIT-MERGE
SORT, while the latter is dramatically better on reversed or highly
reversible structure.

---

---

## 18. A* PLANNING: TRANSPORT HEURISTIC

The first per-instance planner will search the complete machine state graph
with A*. A state is

```text
S = (A, D, B)
```

with every stack represented top-to-bottom. Every legal edge has cost one.

TRANSPORT HEURISTIC combines three unavoidable-movement bounds:

1. baseline transportation to and from the three stacks;
2. endpoint cards that cannot reach the goal in one move;
3. active `D` cards that cannot reach the goal in exactly two moves.

### 16.1 Frozen suffix

The **frozen suffix** is the longest bottom segment of `D` equal to the
corresponding suffix of the goal:

```text
[k, k+1, ..., n].
```

Let

```text
D = X ++ F
```

where `F` is the frozen suffix and `X` is the active prefix.

Any card initially in `X` must leave `D` and later return. If a card in `D`
never moves, every card below it also remains in place, so all unmoved cards
form a bottom suffix. For that suffix to occur in the goal, it must be
contained in `F`.

Hence the baseline transportation bound is

```text
h0(S) = 2|X| + |A| + |B|.
```

### 16.2 Endpoint surcharge

A card initially on `A` or `B` needs at least one move to enter `D`. If it
cannot finish in one move, it needs at least three, so the next possible cost
adds two moves.

Consider the cards on `A` that finish with one move each. They leave `A` in
current top-to-bottom order and are pushed onto `D`, reversing that order.
Consequently their current values must form a decreasing subsequence.
Therefore at most

```text
LDS(A)
```

cards on `A` can finish in one move. The same applies to `B`.

The endpoint surcharge is

```text
2(|A| - LDS(A)) + 2(|B| - LDS(B)).
```

### 16.3 Active-`D` surcharge

A card initially in `X` has baseline cost two:

```text
D → endpoint → D.
```

Partition the cards that actually attain this two-move cost according to
whether they visit `A` or `B`.

Within either endpoint class, their order in `X` must be increasing. They
leave `D` in top-to-bottom order, are reversed on the endpoint stack, and are
reversed again when returned to `D`, restoring their original relative order.
That relative order must agree with the ascending goal.

Thus all exactly-two-move cards in `X` must be coverable by two increasing
subsequences. Define

```text
L2(X)
```

as the maximum number of cards in a subsequence of `X` that can be partitioned
into at most two increasing subsequences.

At least

```text
|X| - L2(X)
```

active `D` cards require four or more moves, adding two moves each.

### 16.4 Complete heuristic

Stacks are sequences written top-to-bottom. Standard notation such as `|S|`,
`S[i]`, prefixes, map lookup, and map update is used directly.

```text
TRANSPORT_HEURISTIC(A, D, B):
    n := |A| + |D| + |B|

    frozen := FROZEN_SUFFIX_LENGTH(D, n)
    X := D[0 .. |D|-frozen)       # active prefix of D

    base :=
        2|X| + |A| + |B|

    endpoint_extra :=
        2(|A| - LDS_LENGTH(A))
        + 2(|B| - LDS_LENGTH(B))

    center_extra :=
        2(|X| - MAX_TWO_INCREASING_COVER(X))

    return base + endpoint_extra + center_extra
```

### 16.5 Supporting pseudocode

Only the nontrivial helpers are defined explicitly.

#### Frozen suffix

```text
FROZEN_SUFFIX_LENGTH(D, n):
    expected := n
    i := |D| - 1

    while i >= 0 and D[i] = expected:
        i := i - 1
        expected := expected - 1

    return |D| - 1 - i
```

#### Longest decreasing subsequence

A simple quadratic dynamic program is adequate for `n <= 52`.

```text
LDS_LENGTH(S):
    if |S| = 0:
        return 0

    for i := 0 to |S|-1:
        best[i] := 1

        for j := 0 to i-1:
            if S[j] > S[i]:
                best[i] := max(best[i], best[j] + 1)

    return max_i best[i]
```

This uses `O(|S|^2)` time and `O(|S|)` memory.

#### Maximum cover by two increasing subsequences

A state `(u,v)` records the final values of two increasing subsequences,
canonically ordered so that `u <= v`. Zero denotes an empty subsequence,
since all card labels are positive. The map value is the greatest number of
selected cards achieving those tails.

```text
MAX_TWO_INCREASING_COVER(X):
    states := {(0,0) ↦ 0}

    for x in X:
        next := copy(states)       # skipping x changes nothing

        for ((u,v), count) in states:
            if x > u:
                pair := sorted pair (x,v)
                next[pair] :=
                    max(next.get(pair, -∞), count + 1)

            if x > v:
                pair := (u,x)
                next[pair] :=
                    max(next.get(pair, -∞), count + 1)

        states := next

    return max(states.values)
```

The straightforward implementation has `O(|X|^2)` possible tail pairs and
therefore uses `O(|X|^3)` time and `O(|X|^2)` memory.

### 16.6 Admissibility proof

The cards initially in `A`, `B`, `X`, and the frozen suffix are disjoint.

- Every card in `A` or `B` costs at least one move.
- Every card in `X` costs at least two moves.
- At least `|A|-LDS(A)` cards initially in `A` cost at least three moves.
- At least `|B|-LDS(B)` cards initially in `B` cost at least three moves.
- At least `|X|-L2(X)` cards initially in `X` cost at least four moves.

The surcharge terms add two only to cards already charged their smaller
baseline cost. Because the four starting regions are disjoint, none of these
charges overlap improperly. Therefore TRANSPORT HEURISTIC never exceeds the
true remaining cost and is admissible.

For a completely reversed deck:

```text
A = []
D = [n, n-1, ..., 1]
B = []
```

the frozen suffix is empty and `L2(D)=2`, so

```text
h = 2n + 2(n-2) = 4n - 4.
```

The heuristic exactly recognizes the optimal reversal cost.

### 16.7 Consistency: disproved

A unit-cost heuristic is consistent when every legal move `S → S'` satisfies

```text
h(S) <= 1 + h(S').
```

TRANSPORT HEURISTIC is not consistent. The smallest counterexample has three
cards:

```text
S:
    A = []
    D = [3, 2, 1]
    B = []
```

There is no frozen suffix. Since a decreasing sequence of length three has
`L2=2`:

```text
h(S)
  = 2·3 + 2(3-2)
  = 8.
```

Make the legal move

```text
D → A
```

moving card `3`:

```text
S':
    A = [3]
    D = [2, 1]
    B = []
```

Now:

```text
LDS(A) = 1
L2(D)  = 2
```

so

```text
h(S')
  = 2·2 + 1
  = 5.
```

Therefore:

```text
h(S) = 8 > 1 + 5 = 6.
```

One legal move decreases the heuristic by three. The disappearing
two-increasing-cover surcharge causes the violation.

A* must consequently either:

- reopen a state when a cheaper path to it is found; or
- apply pathmax along generated edges:

  ```text
  h(child) := max(h(child), h(parent) - 1).
  ```

Pathmax does not make the raw heuristic globally consistent, but it prevents
the `f=g+h` value from decreasing along the current search path.

---

## 19. Open questions and bookkeeping rules

1. Strengthen TRANSPORT HEURISTIC while retaining admissibility.
2. Find a useful consistent relaxation, or evaluate reopenings versus pathmax.
3. Add disjoint additive pattern databases for exact small-card abstractions.
4. Determine the exact worst and expected costs of LOOKAHEAD SELECTION SORT
   and `2K`-PARTITION LOOKAHEAD SELECTION SORT.
5. Determine the exact worst and expected costs of SPLIT-MERGE SORT.
6. Determine or tightly approximate the random-input expectation of HU–TUCKER
   NATURAL MERGE SORT.
7. Complete and analyze a pure SIGNED NATURAL SORT phase rule.
8. Tighten the worst-case analysis of REVERSING SPLIT-MERGE SORT.
9. Audit whether the mixed and double-descending normalization macros can be
   fused further.
10. Improve the orientation-aware implementation of MSB RADIX SORT.
11. Search optimal programs for small `n` and use them as block macros or
   pattern databases.
12. Improve the lower bound beyond

    ```text
    max(ceil(log_3(n!)), 4n-4).
    ```

13. Continue to distinguish rigorously among:
    - exact optimal costs;
    - exact costs of a specified algorithm;
    - certified upper bounds;
    - expected values;
    - heuristic or experimental estimates.
