# Independent ternary construction review

Reviewed `ternary_merge.py`, `research/third-ternary.md`,
the two-line root optimization in `oriented_merge.py`, and
the inherited merge and exact-leaf contracts. The physical
construction and leading coefficient are valid. One
documentation error was found and corrected in the root
integration: P(k)<=4k fails for 56 sizes through 64.

For example, P(9)=39>36 and P(64)=468>256. The largest
P(k)/k among these leaves is 7.3125, at k=64. The simple
replacement P(k)<=8k holds for every finite leaf and
preserves the claimed O(n) remainder. This review makes
no change to the implementation; the original note now
uses the corrected 8k bound.

## Legal composition and protected bases

The central contract sorts the active top segment onto D
in the requested order. The parked contract puts its
reverse top-first order onto A, or onto B by reflection.
An arbitrary unchanged base may lie under each active
stack. Exact leaf words never pop an empty local stack;
placing bases below them therefore preserves legality.

A central parent parks its first child on A. Reflection
parks the second child on B while preserving the first
child as an A base. Both side tops expose the final ranks
in reverse requested order. `_central_merge` transfers
the larger ascending rank first, or the smaller descending
rank first. These transfers build the requested D order
with exactly n merge moves.

A parked parent asks the first child for the opposite
order, then reflects its A output onto B. Its top-first
order is now the requested order. The central second
child leaves its cards on D in that same order while
treating the B cards as a protected base. Either reflected
or unreflected central child is legal: its endpoint stays
D and it preserves both side bases. `_side_merge` takes
the next requested rank from D by DA, or from B by BD DA.
The paired BD DA covers and immediately restores the D
head, and never performs a direct side-to-side transfer.
The resulting A top-first order is reversed as required.

The merge lengths prevent consuming any base. Reflection
preserves every local source-availability condition.
Cancelling adjacent inverse moves preserves the state
after the removed pair, so it preserves all later legal
transfers and the endpoint. These facts establish the
recursive contracts without assumptions about outside
blockers or favorable cancellation.

## Finite and asymptotic bounds

Above the cutoff, the actual central routine uses

```text
U(n) <= P(floor(n/2))+P(ceil(n/2))+n.
```

The actual parked routine uses a=floor(n/3), b=n-a and

```text
P(n) <= P(a)+U(b)+n+a.
```

Set c=floor(b/2), d=ceil(b/2). A balanced central
construction gives U(b)<=P(c)+P(d)+b, including when
b is a finite leaf. For leaf sizes 9 through 64 the
optimized central bound includes that split; for sizes
2 through 8 the stored bounds satisfy it directly.
The macro therefore gives

```text
P(n) <= P(a)+P(c)+P(d)+2n.
```

The three child sizes differ by at most one, since n mod
3 gives respectively (k,k,k), (k,k,k+1), or (k,k+1,k+1).
At each macro level the sum of sizes is at most n and
the depth is at most ceil(log3 n). With finite leaves
bounded by 8k, a conservative implemented bound is

```text
P(n) <= 2n ceil(log3 n)+8n.
```

The balanced central root has two parked children whose
sizes sum to n. It consequently satisfies

```text
U(n) <= 2n log3(n)+O(n)
     = (2/log2(3)) n log2(n)+O(n).
```

The coefficient is approximately 1.2618595. The note's
tighter pure-recurrence formula, with singleton cost one
and the special n=2 cost four, describes a construction
using those leaves. Finite optimization through 64 does
not require its leaves to satisfy P(k)<=4k and does not
alter the leading coefficient. The implemented finite
recurrence adds only legal unreduced move tolls, so every
bound holds before any cancellation is counted.

The recursion partitions its requested intervals into
disjoint children; a parked macro makes three parked
children. The fixed finite cutoff limits local split
search work independently of n. Sorting and copying
materialized words can conservatively cost O(n log² n)
host time, in agreement with the note.

## Root early return preserves every word

For endpoint D, the new return occurs after the complete
central candidate search, at the full root interval only.
Every recursive child interval is strictly shorter than
its parent, so no child can satisfy that root condition.
The original routine's remaining root work computed only
the unused parked word. Its helpers do not mutate the
already selected central word. Thus the returned central
word is identical for all inputs, windows, and cutoffs.
Small roots through eight return before the new condition.
For endpoints A and B the condition is false everywhere,
so every returned parked word is also unchanged.

## Independent checks

`python3 -m unittest test_ternary_merge -v` passed the three
initial tests; root integration also passes the fourth,
covering solver registration. Read-only checks compared the current
oriented function with the complete original function
loaded from HEAD: all three endpoints for all targets
through five cards, and randomized initial/target pairs
at sizes 9, 17, 33, 52, 64, 65, 129, and 257 with windows
zero and two. All 510 complete words were identical.

Fifteen fresh ternary words used randomized initial and
target orders at sizes 65, 70, 191, 300, and 1025, all
three endpoints, window two, and nonidentity integer
labels. Every stack had a two-card guard base. Checks
at every one of 48,910 moves verified that no guard was
moved and all three bases remained unchanged. All final
endpoint orders and finite output bounds passed.

Numeric checks also verified the balanced central and
parked macro inequalities through n=10,000, all finite
leaf bounds P(k)<=8k, and the conservative all-n envelope
through 10,000. These checks support the implementation;
the inductive composition and recurrence arguments above
establish arbitrary-size correctness and the leading term.
