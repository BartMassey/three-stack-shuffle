# Adaptive Selection Sort on the Three-Stack Card Machine

This note summarizes the progression of selection-sort algorithms for the
three-stack card machine. Gene Welborn contributed the main adaptive and
lookahead ideas, and the later experiments here are direct continuations of
those ideas.

The machine has three stacks in a path:

```text
A - D - B
```

Only adjacent moves are legal:

```text
A -> D
D -> A
D -> B
B -> D
```

Thus a conceptual endpoint transfer such as `A -> B` costs two primitive moves:

```text
A -> D
D -> B
```

All stacks are written top-to-bottom. The input starts on `D`. The goal is:

```text
A = []
D = [1, 2, ..., n]
B = []
```

All algorithms below first perform a free sorted-input check. They also freeze
the largest suffix already in final position. If

```text
D = [p1, ..., pm, m+1, m+2, ..., n]
```

then only the active prefix `[p1, ..., pm]` is moved. If `m = 0`, the cost is
zero.

The common cost convention is:

```text
C = primitive legal move count.
```

When a card is bypassed from one endpoint to the other, it costs two primitive
moves.

---

## 1. Selection Sort

### Description

Selection sort repeatedly sweeps all active cards from one endpoint to the
other. It selects cards in descending value order:

```text
m, m-1, ..., 1
```

The sorted output is built from the bottom upward on `D`. During a sweep, when
the next required card is exposed, it is moved to `D`; all other cards are
bypassed to the opposite endpoint.

### Pseudocode

```text
SELECTION_SORT(D):
    m := active prefix length after freezing the final suffix

    if m = 0:
        return

    move m cards D -> A

    source := A
    destination := B
    next := m

    while next > 0:
        while source is not empty:
            if top(source) = next:
                source -> D
                next := next - 1
            else:
                source -> D
                D -> destination

        swap(source, destination)
```

A single sweep can place several consecutive required cards if they appear in
the right order.

### Example

Input:

```text
D = [2, 1, 3]
```

The suffix `[3]` is already final, so `m = 2`. Move `[2, 1]` to `A`:

```text
A = [1, 2]
D = [3]
B = []
```

The first sweep bypasses `1` and places `2`; the second sweep places `1`.

### Cost

Let `Q` be the number of endpoint-to-endpoint bypasses. For active prefix
length `m`:

```text
C = 2m + 2Q.
```

The first `m` moves set up the active prefix on an endpoint. The second `m`
moves place the active cards on `D`. Each bypass costs two moves.

Worst case:

```text
Q_max = m(m-1)/2
C_max(m) = m^2 + m
```

For `n = 52`:

```text
best:                         0
expected without freezing:   1855
expected with freezing:      1854.9607689164
worst:                        2756
```

---

## 2. Adaptive Selection Sort

### Description

Ordinary selection sort finishes a sweep even after finding the required card.
Adaptive selection turns around immediately. After placing `next`, it moves
directly toward `next - 1`, whichever endpoint contains it.

This is Gene Welborn's adaptive selection algorithm.

The algorithm uses complete knowledge of the current state. Finding which
endpoint contains `next` is free bookkeeping; it is not an online search
restriction.

### Pseudocode

```text
ADAPTIVE_SELECTION_SORT(D):
    m := active prefix length after freezing the final suffix

    if m = 0:
        return

    move m cards D -> A

    next := m

    while next > 0:
        source := endpoint containing next
        destination := the other endpoint

        while top(source) != next:
            source -> D
            D -> destination

        source -> D
        next := next - 1
```

### Example

Suppose the active cards have already been split as:

```text
A = [4, 1]
B = [2, 3]
D = [5, 6, ...]
```

The next required card is `4`, so it is placed immediately:

```text
A -> D
```

The next required card is `3`, which is on `B`. The algorithm turns around and
works from `B` rather than finishing a sweep of `A`.

### Cost

Again let `Q` count endpoint-to-endpoint bypasses. For active prefix length
`m`:

```text
C = 2m + 2Q.
```

The worst case is unchanged from selection sort:

```text
C_worst(n) = n^2 + n.
```

The average case is much better. For `n = 52`:

```text
Gene's expected bypass count Q:       425
expected legal cost, no freezing:     954
expected legal cost, with freezing:   952.9803844582
best:                                 0
worst:                                2756
```

Gene's count of `425` is exactly the expected number of bypasses. In this
machine model each bypass costs two primitive moves, and setup and final
placement also count.

---

## 3. Binary-Presort Adaptive Selection Sort

### Description

Gene's next idea was to presort the values into two piles. For `n = 52` the
two value ranges are:

```text
low:   1..26
high: 27..52
```

The algorithm first partitions by value, then adaptively extracts the high
bucket, then adaptively extracts the low bucket. The low bucket remains
protected below any temporary high cards placed on its endpoint.

### Pseudocode

```text
BINARY_PRESORT_ADAPTIVE_SELECTION(D):
    a := floor(n/2)

    while D is not empty:
        if top(D) <= a:
            D -> A
        else:
            D -> B

    next := n
    while next > a:
        source := endpoint containing next
        destination := the other endpoint

        while top(source) != next:
            source -> D
            D -> destination

        source -> D
        next := next - 1

    next := a
    while next > 0:
        source := endpoint containing next
        destination := the other endpoint

        while top(source) != next:
            source -> D
            D -> destination

        source -> D
        next := next - 1
```

### Cost

Let

```text
a = floor(n/2)
b = ceil(n/2)
```

Partitioning costs `n`; final placement costs another `n`.

The exact worst bypass count is:

```text
Q_max = a(a-1)/2 + b(b-1)/2
```

so:

```text
C_worst = 2n + a(a-1) + b(b-1).
```

For `n = 52`:

```text
a = b = 26
best with free sorted detection:  0
baseline best:                    104
expected bypasses:                225
expected legal cost:              554
exact worst:                      1404
```

The measured random mean in the implementation is about `553.895`, matching
this formula within sampling error.

---

## 4. Lookahead Selection Sort

### Description

Lookahead selection is another Gene Welborn algorithm. While searching for
`current`, it recognizes consecutive future targets:

```text
current-1, current-2, ...
```

Instead of bypassing those cards to the other endpoint immediately, it stages
them temporarily on `D`. After `current` is exposed, the staged block is moved
to the other endpoint. This leaves the next target exposed there.

The important point is order. The move is not cheaper than a bypass; it still
costs two primitive moves per staged card. It can avoid later traversals
because it changes the endpoint order.

### Pseudocode

```text
LOOKAHEAD_SELECTION_SORT(D):
    m := active prefix length after freezing the final suffix

    if m = 0:
        return

    move m cards D -> A

    current := m

    while current > 0:
        source := endpoint containing current
        destination := the other endpoint
        lookahead := current - 1
        held := 0

        while top(source) != current:
            if top(source) = lookahead:
                source -> D
                lookahead := lookahead - 1
                held := held + 1
            else:
                source -> D
                D -> destination

        repeat held times:
            D -> destination

        source -> D
        current := current - 1
```

### Example

Suppose `current = 6` and the source endpoint begins:

```text
source = [5, 2, 4, 6, ...]
```

The card `5` is the immediate future target, so it is staged on `D`. The card
`2` is not the next lookahead target and is bypassed. The card `4` is now the
next lookahead target, so it is staged too. After `6` is exposed, the staged
block is moved to the other endpoint before `6` is finalized.

### Cost and status

Let `Q` count all nonfinal endpoint relocations, including staged lookahead
cards. For active prefix length `m`:

```text
C = 2m + 2Q.
```

Certified bound:

```text
C <= m^2 + m.
```

The exact expected cost and exact worst case remain open.

For `n = 52`, a deterministic benchmark over 20,000 random permutations with
seed `24301` measured:

```text
mean:             810.5857
standard error:     0.6167
minimum:           506
maximum:          1184
```

On the same inputs:

```text
adaptive selection:          952.5079
binary-presort adaptive:     553.8948
lookahead selection:         810.5857
```

So lookahead is a large improvement over adaptive selection, but one binary
value partition is still better on random 52-card inputs.

---

## 5. 2K-Partition Lookahead Selection Sort

### Description

This algorithm combines Gene's value partitioning with Gene's lookahead
selection. The parameter `K` selects `2K` balanced value buckets. For example:

```text
K = 1  -> 2 buckets
K = 2  -> 4 buckets
K = 3  -> 6 buckets
K = 4  -> 8 buckets
```

Only the active prefix is partitioned. The recursive partition tree processes
higher value intervals before lower value intervals. At each leaf bucket, the
algorithm extracts that bucket in descending order with lookahead selection.

### Pseudocode

```text
TWO_K_PARTITION_LOOKAHEAD_SELECTION(D, K):
    m := active prefix length after freezing the final suffix

    if m = 0:
        return

    bucket_count := min(2K, m)

    recursively divide [1, m] into bucket_count balanced value intervals:
        split the current interval into lower and upper groups
        partition its cards by value onto A and B
        process the upper group recursively
        process the lower group recursively

    at each one-bucket leaf:
        extract the interval in descending order with LOOKAHEAD_SELECTION
```

At the root, partitioning starts from `D` and costs one move per card. At
deeper nodes, repartitioning moves a group from an endpoint through `D` and
back to the endpoints, costing two moves per card.

### Example: K = 2, n = 52

Four buckets have size 13:

```text
1..13, 14..26, 27..39, 40..52
```

The root split separates `1..26` from `27..52`. The high half is refined and
completed first. Only after the high half is done is the low half refined and
completed.

### Cost and bounds

Let `P(m,K)` be the fixed partition-tree and final-placement cost. Let `Q`
count nonfinal endpoint relocations, including staged lookahead cards.

```text
C = P(m,K) + 2Q.
```

For `K = 1`, with `a = floor(m/2)` and `b = ceil(m/2)`:

```text
C <= 2m + a(a-1) + b(b-1).
```

In general, if the balanced leaf bucket sizes are `s1, ..., sb`, then:

```text
C <= P(m,K) + sum(si(si-1)).
```

### Measurements

The following deterministic benchmark used 20,000 random 52-card permutations
with seed `24301`.

| Algorithm | Mean moves | Standard error | Minimum | Maximum |
|---|---:|---:|---:|---:|
| Adaptive selection | 952.5079 | -- | -- | -- |
| Lookahead selection | 810.5857 | 0.6167 | 506 | 1184 |
| Binary-presort adaptive | 553.8948 | -- | -- | -- |
| 2K partition, K=1 | 457.6273 | 0.3074 | 294 | 644 |
| 2K partition, K=2 | 385.3420 | 0.1531 | 308 | 472 |
| 2K partition, K=3 | 394.4010 | -- | -- | -- |
| 2K partition, K=4 | 401.0680 | -- | -- | -- |

The `--` entries were not recorded in the source benchmark summary.

For the same benchmark, the partitioned variants have:

| K | Buckets | Fixed partition and placement cost | Mean relocations | Mean moves | Certified bound |
|---:|---:|---:|---:|---:|---:|
| 1 | 2 | 104 | 176.814 | 457.627 | 1404 |
| 2 | 4 | 208 | 88.671 | 385.342 | 832 |
| 3 | 6 | 276 | 59.200 | 394.401 | 676 |
| 4 | 8 | 312 | 44.534 | 401.068 | 600 |

For random 52-card inputs, `K = 2` was best among these consecutive-lookahead
configurations. Higher `K` reduces relocations, but the extra partitioning
cost eventually dominates.

---

## 6. Receding-Horizon Lookahead

### Description

Consecutive lookahead is sufficient but not forced. When extracting `current`,
suppose the source endpoint contains blockers above it:

```text
source = X ++ [current] ++ tail
```

where `X` is the top-to-bottom list of blockers. Each blocker can either be:

```text
staged temporarily on D
```

or:

```text
bypassed directly to the other endpoint
```

Every such mask has the same immediate cost for this pass. The choice matters
only through the endpoint order it creates for later passes.

The full dynamic program over all such choices can grow quickly. The practical
algorithm uses a receding horizon: try every mask for the current pass, score
each candidate by finishing the rest of the current bucket with ordinary
consecutive lookahead, commit only the best first pass, and then replan at the
next target.

### Pseudocode

```text
ROLLOUT_LOOKAHEAD_PASS(current, bucket_low):
    source := endpoint containing current
    X := blockers above current on source

    best_mask := consecutive-lookahead mask
    best_score := score(best_mask)

    for each mask over X:
        trial := copy of current machine state
        apply this one pass using mask
        finish current-1 down to bucket_low with ordinary lookahead
        score := trial primitive move count

        if score < best_score:
            best_score := score
            best_mask := mask

    apply best_mask to the real machine
```

The implementation limits one pass to at most 16 blockers. For `n = 52`,
`K = 2` has 13-card leaves, so the limit is satisfied.

### Measurements

The following deterministic benchmark used 2,000 random 52-card permutations
with seed `24301`.

| K | Buckets | Consecutive mean | Rollout mean | Mean reduction | Rollout time |
|---:|---:|---:|---:|---:|---:|
| 2 | 4 | 385.620 | 343.931 | 41.689 | 8.20 s |
| 3 | 6 | 394.347 | 376.051 | 18.296 | 0.69 s |
| 4 | 8 | 401.046 | 392.375 | 8.671 | 0.26 s |
| 5 | 10 | 422.986 | 418.789 | 4.197 | 0.16 s |
| 6 | 12 | 439.258 | 437.339 | 1.919 | 0.11 s |

The benefit shrinks as buckets get smaller. `K = 2` remains the best measured
configuration in this rollout family.

---


## 7. Incremental Receding-Horizon Lookahead

### Description

Incremental RHL is algorithmically identical to the receding-horizon rollout
in the previous section. It selects the same mask at every target and therefore
has the same primitive move count. It avoids repeatedly simulating the same
ordinary-lookahead suffixes.

For each active bucket state `S`, memoize the exact remaining cost `V(S)` of
ordinary consecutive lookahead. Candidate masks are scored as:

```text
2|X| + 1 + V(successor(S, mask)).
```

After the best mask is committed, its successor is the next planning root.
The memoized rollout DAG is retained and extended rather than discarded.

### Pseudocode

```text
INCREMENTAL_RHL_BUCKET(bucket_low, bucket_high):
    memo := empty
    current := bucket_high

    while current >= bucket_low:
        place every consecutively exposed target on D

        if current < bucket_low:
            stop

        X := blockers above current
        best_mask := consecutive mask
        best_score := infinity

        enumerate distinct mask outcomes for X:
            successor := construct algebraically
            score :=
                2|X| + 1
                + BASE_COST(successor, current-1, bucket_low, memo)

            retain the least score, using the ordinary RHL tie rule

        execute the retained mask
        current := current - 1
```

`BASE_COST` follows ordinary consecutive lookahead, memoizing each normalized
state. Normalization removes exposed target chains, projects and relabels the
active bucket, and identifies states differing only by interchange of `A` and
`B`.

### Mask-count bound

For `m>0` blockers, toggling the bottommost blocker gives the same successor,
and this is the only mask duplication. Thus there are `2^(m-1)` distinct mask
outcomes.

Across a complete `b`-card bucket, the total number of distinct outcomes
enumerated is at most:

```text
2^(b-1).
```

For a 26-card `K=1` leaf:

```text
2^25 = 33,554,432.
```

This is the intended next experiment.

### Status

The implementation is complete as a separate experimental variant. Exhaustive
small-leaf checks through six cards match brute-force scores and masks, and
random `K=2` through `K=4` runs match complete primitive move sequences.

For 2,000 random 52-card permutations with seed `24301`, `K=2` incremental RHL
retained the brute-force mean of `343.931` moves and reduced solver/replay time
from `8.133` seconds to `4.795` seconds (`1.70x`, or `41.0%` faster).

---

## 8. Depth-Limited Receding-Horizon Lookahead

### Description

Depth-limited RHL plans over individual blocker choices rather than complete
capture masks.

At each blocker, the two choices are:

```text
STAGE:   source -> D
BYPASS:  source -> D -> destination
```

The algorithm searches the next `d` binary choices, evaluates each frontier
state by completing the bucket with ordinary consecutive lookahead, commits
only the first choice, reroots the retained search tree, and repeats.

The terminal evaluation is the exact cost of a known greedy completion policy.
It need not be admissible because this is rollout policy improvement, not A*.

### Pseudocode

```text
DEPTH_VALUE(state, d):
    forced_cost, state := perform forced target placements

    if bucket is complete:
        return forced_cost

    if d = 0:
        return forced_cost + GREEDY_COMPLETION_COST(state)

    greedy_action :=
        STAGE if top(source) = next_capture
        else BYPASS

    return forced_cost
           + minimum over STAGE and BYPASS of:
                 immediate action cost
                 + DEPTH_VALUE(child state, d-1)
           breaking ties toward greedy_action
```

```text
DEPTH_LIMITED_RHL_BUCKET(low, high, depth):
    initialize the partial-pass state

    while bucket is unfinished:
        perform forced target placements

        if bucket is complete:
            stop

        compare STAGE and BYPASS using DEPTH_VALUE at depth-1
        break ties toward the greedy action
        execute only the selected first action
        reroot and extend the retained search tree
```

Depth counts only blocker decisions. Flushing staged cards and placing exposed
targets are forced and do not consume depth.

### Relationship to existing policies

```text
depth 0 = ordinary consecutive lookahead
depth 1 = one binary choice followed by greedy completion
```

Existing full-mask RHL is different: it searches every remaining blocker choice
for the current target and commits the whole mask.

Because greedy completion is an available continuation at every node, the
depth-limited policy is guaranteed not to exceed the ordinary greedy policy's
cost. Larger depth gives a nonincreasing rollout estimate, although realized
receding-horizon move counts are not guaranteed to be monotone in depth.

### Intended measurements

Benchmark depths:

```text
0, 1, 2, 4, 6, 8, 10, 12, 14, 16
```

for both `K=2` and `K=1`, recording move count, runtime, expanded binary nodes,
frontier evaluations, cache effectiveness, retained-tree reuse, and peak
memory.

### Initial results

The implementation is complete as a separate experimental variant:

```text
depth-limited-rhl-2k-partition-lookahead-selection-experimental:K:depth
```

The planner memoizes depth values over full physical partial states. Greedy
terminal evaluation is exact but optimized: it finishes the current partial
pass, then uses the incremental-RHL deterministic suffix cache for the
remaining consecutive-lookahead bucket. It does not yet keep an explicit
rerooted tree object, so retained-tree reuse is not reflected beyond cache
hits.

For 200 random 52-card permutations with seed `24301`, the measured `K=2`
means were as follows. The elapsed times in this table were measured before
the suffix-cache terminal evaluator optimization; the move distributions are
unchanged by that exact optimization.

| depth | mean moves | stderr | min | max | elapsed |
|---:|---:|---:|---:|---:|---:|
| 0 | 387.760 | 1.552 | 318 | 448 | 0.004 s |
| 1 | 350.690 | 1.101 | 304 | 402 | 0.114 s |
| 2 | 348.800 | 1.058 | 302 | 402 | 0.206 s |
| 3 | 346.940 | 0.981 | 302 | 402 | 0.343 s |
| 4 | 345.600 | 0.935 | 302 | 382 | 0.579 s |
| 5 | 344.730 | 0.909 | 302 | 382 | 0.932 s |
| 6 | 344.360 | 0.899 | 302 | 378 | 1.432 s |
| 7 | 343.760 | 0.884 | 302 | 376 | 2.241 s |
| 8 | 343.340 | 0.873 | 302 | 376 | 3.570 s |
| 9 | 342.880 | 0.856 | 302 | 376 | 5.689 s |
| 10 | 342.390 | 0.848 | 302 | 376 | 9.147 s |
| 11 | 341.990 | 0.835 | 302 | 376 | 14.891 s |
| 12 | 341.750 | 0.825 | 302 | 372 | 23.734 s |
| 13 | 341.610 | 0.823 | 302 | 372 | 40.261 s |

The same benchmark for `K=1` measured:

| depth | mean moves | stderr | min | max | elapsed |
|---:|---:|---:|---:|---:|---:|
| 0 | 465.320 | 3.114 | 344 | 584 | 0.004 s |
| 1 | 348.710 | 1.775 | 290 | 430 | 0.528 s |
| 2 | 342.440 | 1.718 | 288 | 418 | 0.898 s |
| 3 | 339.900 | 1.711 | 272 | 404 | 1.656 s |
| 4 | 336.240 | 1.595 | 282 | 396 | 2.685 s |
| 5 | 334.130 | 1.600 | 280 | 396 | 5.088 s |
| 6 | 332.370 | 1.591 | 280 | 394 | 10.072 s |
| 7 | 331.310 | 1.612 | 270 | 400 | 19.745 s |

Most of the benefit appears at short depths. `K=2` improves sharply from depth
`0` to depth `1` and then tapers. `K=1` has much larger leaves and therefore a
worse depth-`0` baseline, but short binary lookahead makes those larger leaves
useful: depth `1` is already competitive with `K=2`, and depth `7` reaches
`331.310` moves. Deeper `K=1` runs were not pursued because runtime climbs
quickly and the measured means were already flattening.

After adding the exact suffix-cache terminal evaluator, 5,000-sample tail
checks with the same seed measured:

| variant | mean moves | stderr | min | max | elapsed |
|---|---:|---:|---:|---:|---:|
| `K=1`, depth 6 | 330.478 | 0.303 | 260 | 406 | 71.363 s |
| `K=1`, depth 7 | 328.829 | 0.298 | 260 | 412 | 126.549 s |
| `K=2`, depth 7 | 342.798 | 0.177 | 294 | 394 | 35.666 s |

The shortcut improved compute time without changing the policy, but the tail
remained expensive. `K=1` kept the better mean but still exceeded 400 in the
larger sample; `K=2` had lower variance but still reached 394.

---

## 9. Consecutive-Target Block Rollout

This experiment replaces arbitrary stage/bypass masks with one explicit
reversal block. While exposing `current`, factor its blockers as:

```text
X = U ++ H ++ P ++ V
```

Bypass `U`, park `H` on `D`, bypass `P` above it, flush `H`, and then bypass
`V`. The other endpoint receives:

```text
reverse(V) ++ H ++ reverse(P) ++ reverse(U)
```

Only outcomes beginning `current-1, current-2, ...` are retained, plus direct
finalization. All candidates have the same immediate cost. The scalable
policy scores them with exact ordinary-lookahead completion and commits one
reversal; a global full-physical-state DP is retained only as a small-deck
reference because it does not scale polynomially in practice.

On all 5,040 seven-card all-on-`A` permutations, the global restricted DP was
exactly optimal 2,329 times and had mean gap `1.982937`; the scalable rollout
had the same exact count and mean gap `1.996032`. On 2,000 random 52-card
permutations with seed `24301`, rollout measured:

| Configuration | Mean | Standard error | Minimum | Maximum |
|---|---:|---:|---:|---:|
| Standalone | 731.477 | 1.671 | 530 | 1034 |
| `K=1` | 428.897 | 0.859 | 310 | 566 |
| `K=2` | 375.640 | 0.456 | 310 | 450 |

This is a real improvement over ordinary lookahead, but not competitive with
depth-limited RHL. See `ALGORITHMS.md` for the full recurrence, diagnostics,
complexity caveat, and command-line names.

---

## 10. Perfect Leaf Selection by A*

### Description

The latest experiment keeps the same `2K` partition tree but replaces each
leaf extractor with exact A* search. The search is exact for the current leaf
interval, not for the entire sorting problem.

For a leaf interval `[low, high]`, cards outside the interval are treated as
barriers. The planner may move interval cards above those barriers, but it may
not pop an out-of-interval card. The local goal is:

```text
D projection = [low, low+1, ..., high]
A projection = []
B projection = []
```

The A* heuristic is the transport lower bound applied to the projected
interval state after relabeling `low..=high` to `1..=high-low+1`. The
heuristic is admissible but inconsistent, so A* allows reopenings.

### Pseudocode

```text
PERFECT_LEAF_SELECTION(low, high):
    start := current full machine state
    goal_projection := ([], [low, low+1, ..., high], [])

    open := priority queue containing start
    best_g[start] := 0
    parent := empty map

    while open is not empty:
        state := pop state with smallest g + h

        if queued g is stale:
            continue

        if projection(state, low, high) = goal_projection:
            reconstruct and execute the plan
            return

        for each legal move from state:
            if top moved card is outside [low, high]:
                continue

            child := state after move
            candidate_g := best_g[state] + 1

            if child has no best_g or candidate_g < best_g[child]:
                best_g[child] := candidate_g
                parent[child] := (state, move)
                push child with priority candidate_g + h(child)
```

### State count

A full `s`-card leaf has:

```text
s! * C(s+2, 2)
```

possible ordered three-stack states. For `s = 13`:

```text
13! * C(15, 2) = 653,837,184,000
```

This is too large for blind search. For `K = 3`, the largest leaves have 9
cards; for `K = 4`, the largest leaves have 7 cards. Those are much more
tractable.

### Measurements

These measurements used 200 random 52-card permutations with seed `24301`.
The transport lower-bound mean was `166.860` for each row, since the same
input decks were used.

| K | Method | Mean moves | Standard error | Minimum | Maximum | Mean / lower bound | Time |
|---:|---|---:|---:|---:|---:|---:|---:|
| 3 | Receding-horizon rollout | 376.730 | 0.747 | 350 | 402 | 2.258 | 0.070 s |
| 3 | Perfect leaf A* | 363.330 | 0.568 | 340 | 380 | 2.177 | 26.444 s |
| 4 | Receding-horizon rollout | 392.990 | 0.658 | 372 | 414 | 2.355 | 0.025 s |
| 4 | Perfect leaf A* | 386.470 | 0.536 | 370 | 404 | 2.316 | 1.709 s |

The A* leaf planner improves the mean, but its cost grows quickly with leaf
size. At `K = 3`, it saves about `13.4` moves on average compared with rollout
and takes about `378` times as long. At `K = 4`, it saves about `6.52` moves
and takes about `68` times as long.

These numbers suggest that receding-horizon rollout captures much of the
available improvement at a much lower runtime. Exact leaf planning remains a
useful reference point for smaller leaves and for measuring how much quality
is left on the table.

---

## 11. Summary

The progression is:

```text
selection sort
    -> adaptive selection
    -> binary-presort adaptive selection
    -> lookahead selection
    -> 2K-partition lookahead selection
    -> receding-horizon lookahead
    -> incremental receding-horizon lookahead
    -> depth-limited receding-horizon lookahead
    -> consecutive-target block rollout
    -> exact A* leaf selection
```

For random 52-card inputs, the main lesson is that value partitioning and
lookahead are complementary. Adaptive selection roughly halves the mean cost
of ordinary selection. Lookahead improves adaptive selection by preserving
useful local order. Balanced value partitioning then reduces the distances
that lookahead must traverse.

The best measured practical configuration so far is the four-bucket
receding-horizon rollout (`K = 2`), with mean about `343.931` moves in the
2,000-sample benchmark. Exact A* leaf planning gives better local plans for
small leaves, but at the tested sizes it is much slower than rollout.
