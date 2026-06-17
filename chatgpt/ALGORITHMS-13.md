# Three-Stack Sorting Algorithms

This is a living record of algorithms and bounds for the three-stack card-sorting problem.

## 1. Model and conventions

The stacks are arranged on a path:

```text
A — D — B
```

A machine operation moves the top card of one stack to an adjacent stack. Thus the legal directed moves are:

```text
A → D    D → A    D → B    B → D
```

A direct move between `A` and `B` is illegal.

For a deck of size `n`, the initial state is

```text
A = []
D = [p1, p2, ..., pn]    # top to bottom
B = []
```

where `p1, ..., pn` is an arbitrary permutation of `1, ..., n`. The goal state is

```text
A = []
D = [1, 2, ..., n]       # top to bottom
B = []
```

Only machine operations are charged. Arbitrarily expensive offline computation, inspection of the entire input permutation, comparisons, and planning are free.

Notation:

- `lg n` means `log2 n`.
- A run is written top-to-bottom.
- “Ascending” means increasing card numbers top-to-bottom.
- A stated move count is an upper bound unless explicitly called exact or optimal.

For non-power-of-two divide-and-conquer trees it is useful to define

```text
L = ceil(lg n)
E(n) = nL - (2^L - n).
```

`E(n)` is the minimum total leaf depth of a binary tree with `n` leaves whose depths differ by at most one. In particular,

```text
E(52) = 52·6 - (64 - 52) = 300.
```

## 2. Reversal primitive

Input:

```text
D = [1, 2, ..., n]
```

Output:

```text
D = [n, n-1, ..., 1].
```

Algorithm, for `n >= 2`:

```text
repeat n-1 times: D → A
D → B

repeat n-2 times:
    A → D
    D → B

A → D
repeat n-1 times: B → D
```

Move count:

```text
(n-1) + 1 + 2(n-2) + 1 + (n-1) = 4n - 4.
```

This is optimal. Every card must leave `D` and return. At most two cards can do so in only two moves—one through each side stack—without preserving the wrong relative order. The remaining `n-2` cards require at least four moves each.

For `n = 52`:

```text
4·52 - 4 = 204 moves.
```

This is not one of the named general sorting algorithms, but it is an important primitive and lower-bound witness.

---

## 3. LSB RADIX SORT

### Idea

Perform stable binary radix sort on `card - 1`, from least significant bit to most significant bit.

For each bit position `j = 0, ..., ceil(lg n)-1`:

```text
while D is nonempty:
    if bit j of (top(D) - 1) is 0:
        D → A
    else:
        D → B

while B is nonempty: B → D
while A is nonempty: A → D
```

Moving a bucket from `D` to a side stack reverses it. Moving it back to `D` reverses it again, so each bucket retains its previous relative order. Collecting `B` before `A` leaves the zero bucket above the one bucket.

### Correctness invariant

After pass `j`, the deck is stably sorted by the low `j+1` bits of `card - 1`. After all passes, it is sorted by the complete key.

### Move count

Each pass moves every card out of `D` once and back into `D` once:

```text
2n moves per pass.
```

Therefore the exact count for the fixed-pass implementation is

```text
C_LSB(n) = 2n ceil(lg n).
```

Complexity:

```text
Theta(n lg n).
```

For `n = 52`:

```text
C_LSB(52) = 2·52·6 = 624.
```

The move count is input-independent unless completed passes or already-sorted cases are explicitly detected and skipped.

---

## 4. NATURAL SORT

### Idea

Split the input into maximal ascending runs, then repeatedly merge adjacent runs. The initial run decomposition is computed offline and costs no machine operations.

A merge pass works as follows:

1. Move whole runs alternately from `D` to `A` and `B`.
2. Each moved run is reversed, so its largest remaining card is exposed.
3. Merge exposed run pairs back into `D`, always moving the larger exposed card first.
4. Pushing the cards onto `D` reverses the decreasing merge output, yielding one ascending run.

An unpaired run is simply returned to `D`.

### Correctness invariant

At the start of each pass, `D` is a concatenation of ascending runs. A pass replaces adjacent pairs by their sorted union, again leaving a concatenation of ascending runs.

### Move count

A full merge pass moves every participating card out of `D` once and back once:

```text
2n moves per full pass.
```

If the initial input has `r` ascending runs, the straightforward balanced-pass implementation uses at most

```text
C_NAT(n, r) <= 2n ceil(lg r).
```

Worst case occurs when `r = n`, giving

```text
C_NAT(n) <= 2n ceil(lg n) = Theta(n lg n).
```

For `n = 52`, the certified straightforward worst-case bound is

```text
C_NAT(52) <= 624.
```

A strictly decreasing deck has `52` singleton ascending runs, so this baseline NATURAL SORT treats reversal as a worst-case instance and uses its full merge schedule.

A more carefully shaped nonuniform merge schedule may reduce the non-power-of-two constant; that optimization is not part of the baseline definition above.

---

## 5. MERGE SORT

### Idea

This is top-down merge sort by physical position.

For a subdeck of size `k` in `D`, let

```text
a = ceil(k/2)
b = floor(k/2).
```

Then:

```text
move the top a cards D → A
move the next b cards D → B

move a cards A → D
MERGE SORT the a-card subdeck
move a cards D → A

move b cards B → D
MERGE SORT the b-card subdeck
move b cards D → B

merge A and B back into D:
    while both current halves are nonempty:
        move the larger exposed card to D
    move the remainder to D
```

After a recursively sorted half is moved back to a side stack, it is reversed; hence its largest card is exposed. Moving larger cards first produces a decreasing stream, which becomes increasing top-to-bottom when pushed onto `D`.

Protected cards belonging to outer recursive calls may remain below the active segment of a side stack. Subproblem sizes delimit these protected blocks.

### Move count

At an internal node of size `k`:

- splitting costs `k`;
- moving both halves into `D` and back costs `2k`;
- merging costs `k`.

Thus

```text
C_MERGE(k)
  = C_MERGE(ceil(k/2))
  + C_MERGE(floor(k/2))
  + 4k,
```

with `C_MERGE(0) = C_MERGE(1) = 0`.

For powers of two:

```text
C_MERGE(n) = 4n lg n.
```

For balanced halving in general:

```text
C_MERGE(n) = 4E(n).
```

Complexity:

```text
Theta(n lg n).
```

For `n = 52`:

```text
C_MERGE(52) = 4E(52) = 4·300 = 1200.
```

This count is for the explicit split–recurse–put-back–merge implementation above. Future stack-oriented variants may fuse phases and improve the constant; such variants should be documented separately rather than silently changing this definition.

---

## 6. MSB RADIX SORT

### Idea

Recursively split by value range rather than by physical position.

For a subproblem containing a consecutive value interval of size `k`, choose a threshold dividing it into:

- a lower bucket of size `a`;
- an upper bucket of size `b = k-a`.

Scan the active subdeck in `D`:

```text
lower-valued cards: D → A
upper-valued cards: D → B
```

Then, in the straightforward stack implementation:

```text
move the lower bucket A → D
recursively MSB RADIX SORT it
move it D → A

move the upper bucket B → D
recursively MSB RADIX SORT it

move the sorted lower bucket A → D
```

The final move places all lower values above the already-sorted upper values, so no comparison merge is required.

### Correctness invariant

Each recursive call sorts exactly one consecutive value interval. Concatenating the sorted lower interval above the sorted upper interval gives the sorted parent interval.

### Move count

For a split into lower size `a` and upper size `b`:

```text
split:                         k
move/sort/park lower bucket:  2a, excluding recursion
move upper bucket to D:       b
restore lower above upper:    a
```

Hence the straightforward recurrence is

```text
C_MSB(k) = C_MSB(a) + C_MSB(b) + 2k + 2a.
```

A balanced implementation may take `a = floor(k/2)` so that the bucket incurring the extra round trip is the smaller one.

For powers of two this gives

```text
C_MSB(n) = 3n lg n.
```

Complexity:

```text
Theta(n lg n).
```

Using `a = floor(k/2)` recursively gives the current straightforward figure

```text
C_MSB(52) = 880.
```

This is not claimed optimal among MSB implementations. In particular, orientation-aware recursion, unequal split trees, or phase fusion may improve the constant. Earlier discussion of a possible `600`-move variable-depth implementation should be regarded as a target requiring a complete low-level construction and proof, not yet an established result.

---

## 7. SIGNED NATURAL SORT

### Status

Proposed extension; some low-level details and the exact worst-case proof remain open.

### Idea

Decompose the deck into monotone runs, allowing both:

- ascending runs;
- descending runs.

Normalize descending runs to ascending orientation, then merge the resulting ascending runs as in NATURAL SORT. The motivation is that ordinary NATURAL SORT views reversal as `n` singleton ascending runs, while SIGNED NATURAL SORT recognizes it as one descending run.

For a fully reversed deck, normalization can use the optimal reversal primitive:

```text
4n - 4 moves.
```

Thus for `n = 52`, reversal costs only

```text
204 moves
```

rather than the `624`-move baseline NATURAL SORT schedule.

### Tentative macro-level analysis

A candidate difficult shape is an alternating sequence of descending pairs, for example

```text
27,1, 28,2, 29,3, ..., 52,26.
```

This has 26 descending runs of length 2 under the intended greedy run decomposition.

A previous macro-level calculation assigned:

```text
104 moves to normalize the 26 pairs
496 moves to merge 26 equal runs
600 moves total.
```

That `600` figure depends on two assumptions that still need a complete machine-level realization:

1. each descending run can be normalized independently at the quoted cost without disturbing inaccessible runs;
2. the chosen nonuniform adjacent-run merge tree can be executed with weighted cost `2(a+b)` per merge and no additional positioning moves.

Until those points are proved, record `600` as a conjectured or macro-model score, not as a certified worst-case bound for the actual machine.

Complexity target:

```text
O(n lg n) worst case,
with substantially lower cost on inputs having long monotone runs.
```

---



### Endpoint-to-center reversal macro

Under the stack-relative terminology, a descending sequence of length `k` on
`A` is physically increasing top-to-bottom. It can be reversed onto `D` in

```text
3k - 2
```

moves:

```text
repeat k-1 times:
    A → D
    D → B

A → D

repeat k-1 times:
    B → D
```

The first `k-1` cards are routed from `A` to `B`, the last card is left on
`D`, and the buffered cards are then returned from `B` to `D`. The resulting
sequence is ascending on `D`. The symmetric macro applies from `B` to `D`.

This improves on the naive composition “reverse `A` onto `B`, then move all
`k` cards `B → D`,” whose cost is `3k`.


## REVERSING SPLIT-MERGE SORT

### Fixed algorithm

A phase consists of a paired split followed by a merge. All logical sequences
presented to the merge are ascending.

Let the unprocessed portion of the deck be the current contents of `D`.

#### Paired split loop

Repeat:

1. **Odd-tail test.** If all remaining cards in `D` form one monotone
   sequence, stop the paired split loop.

   - If the sequence is ascending, leave it in `D`.
   - If it is descending, reverse it optimally *in `D`*, at cost `4k-4` for
     length `k`, and leave the resulting ascending sequence in `D`.

   This sequence is the optional odd sequence for the phase. It remains in
   `D` as a protected base for the merge.

2. Otherwise, among prefixes of the remaining `D`, find the longest prefix
   that can be partitioned, preserving card order, into exactly two nonempty
   monotone subsequences. The allowed orientation patterns are

   ```text
   ascending / ascending
   ascending / descending
   descending / ascending
   descending / descending
   ```

   A descending subsequence used in such a pair must have length at least
   five. Ascending subsequences may have any positive length.

3. Send one subsequence to `A` and the other to `B`. Because cards are pushed
   onto endpoint stacks, an ascending input subsequence is already physically
   reversed and ready for an ascending merge. A descending input subsequence
   is physically ascending on its endpoint stack, so reverse that top segment
   in place at cost `4k-2`. After normalization, both logical subsequences are
   ascending and physically decreasing, with their maxima exposed.

4. Continue with the remaining suffix in `D`.

Every regular iteration emits exactly two logical sequences, one to each side.
Thus `A` and `B` contain equal numbers of sequences. If the split has an odd
number of sequences overall, the unique odd sequence is the normalized tail
left in `D`; it is not moved to a side stack.

The longest-prefix rule is primary. To make the algorithm completely
deterministic, equally long candidates are resolved by minimum total endpoint
reversal cost, followed by a fixed lexicographic tie-break on the two
subsequences.

#### Merge

Treat the optional odd tail already in `D` as a protected base.

While `A` and `B` contain logical sequences:

1. take the topmost sequence from each side;
2. repeatedly move the larger exposed card to `D`;
3. when one sequence is exhausted, move the remainder of the other.

The two decreasing side-stack sequences are thereby emitted in decreasing
order, producing one ascending sequence top-to-bottom in `D`.

Because paired splitting placed the same number of sequences on `A` and `B`,
the merge has no unmatched side-stack sequence. The only possible unmatched
sequence is the odd tail already left in `D`.

Repeat phases until `D` is one ascending sequence.

### Consequences

A fully descending deck is not a separate special case. It is simply a split
phase with zero pairs and one descending odd tail. The tail is reversed in
`D`, so a reversed deck of size `n` costs exactly

```text
4n - 4.
```

For `n=52`, this is `204` moves.

The length-five threshold applies only to descending subsequences selected as
members of a pair, because they require endpoint reversal at cost `4k-2`.
A descending odd tail is reversed centrally at cost `4k-4`; for lengths three
and four this is already better than decomposing the tail into ascending
singletons, and length two ties.


## 8. Current lower bounds

Let `M(n)` be the maximum, over all `n!` input permutations, of the minimum number of legal moves needed to sort that permutation.

### Counting bound

Every move is reversible. If every permutation were sortable in at most `m` moves, reversing the programs would generate all `n!` permutations from the sorted deck.

The first move has at most two choices. In a shortest program, a move is never followed immediately by its inverse, so each later step has at most three choices. Therefore the number of reduced programs of length at most `m` is at most

```text
3^m.
```

Consequently,

```text
M(n) >= ceil(log_3(n!)).
```

Asymptotically,

```text
log_3(n!)
  = n log_3 n - (log_3 e)n + O(log n)
  = Theta(n lg n).
```

For a concrete `n`, use the factorial directly rather than dropping the lower-order term.

For `n = 52`:

```text
ceil(log_3(52!)) = 143.
```

### Reversal bound

The reversal instance requires exactly `4n-4`, so

```text
M(n) >= 4n - 4.
```

For `n = 52` this is `204`, stronger than the counting bound.

Thus the current elementary bound is

```text
M(52) >= max(143, 204) = 204.
```

---

## 9. Current `n = 52` summary

| Algorithm or bound | Status | Moves / bound |
|---|---:|---:|
| Optimal reversal primitive | proved exact for reversal | 204 |
| LSB RADIX SORT | established fixed-pass algorithm | 624 |
| NATURAL SORT | established straightforward worst-case upper bound | 624 |
| MERGE SORT | established for the explicit implementation | 1200 |
| MSB RADIX SORT | established straightforward balanced implementation | 880 |
| SIGNED NATURAL SORT | proposed; reversal score established | 204 on reversal |
| SIGNED NATURAL SORT alternating-pairs score | tentative macro model | 600 |
| General counting lower bound | proved | 143 |
| Current general worst-case lower bound | proved using reversal | 204 |

These figures refer to the precisely documented variants. An improved implementation should receive either a new name or an explicit version note.

---

## 10. Open problems and bookkeeping rules

1. Improve the certified upper bound for `M(52)`.
2. Improve the general lower bound beyond `max(ceil(log_3(n!)), 4n-4)`.
3. Give a complete low-level SIGNED NATURAL SORT implementation and prove its worst case.
4. Determine the best orientation-aware implementation of MSB RADIX SORT.
5. Search exact optimal programs for small `n` and use them as pattern databases or block-sorting macros.
6. Distinguish carefully among:
   - exact optimal costs;
   - exact costs of a specified algorithm;
   - certified upper bounds;
   - heuristic or macro-model estimates.
