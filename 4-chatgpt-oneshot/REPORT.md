# Permutation Routing and Uniform Shuffling on Three Stacks

*Research release, 4 October 2026*

## Abstract

We consider a card machine with three stacks arranged
A–D–B. A move transfers the top card of one stack to the
top of an adjacent stack. All cards start and finish on D.
The problem is to realize a specified permutation with
few moves; a uniform shuffle follows by choosing the
target permutation uniformly.

We give a recursive merge algorithm whose leaves use
exhaustively computed optimal plans for at most eight
cards. Distinguishing central and side output contracts,
and merging directly onto the required output stack,
gives at most 352 moves for every 52-card permutation
and $2n\log_3 n+O(n)$ moves in general. The latter
uses an asymmetric split whose two-level expansion is
a balanced ternary recursion.
Its leading coefficient is sharp for the specified
additive endpoint recurrence, not for the machine.
On 100 fresh held-out targets, the fast version averages
284.72 moves, with maximum 300 and approximately 8.8 ms
median planning time. A wider split search averages
273.8 moves, with maximum 288. Its parked-leaf predecessor
averages 318.74 on those same targets.

We also compute exact optimal distances through nine
cards and give counting and structural lower bounds.
At 52 cards, a structural bound implies an optimal uniform
mean of at least 166.87917 moves. The optimal 52-card
costs remain undetermined. Reversal requires exactly
4(n−1) moves for every n. That result is distinct from
the stronger claim that every permutation admits such
a plan.
Conditional card-budget obstructions strengthen bounds
on individual targets. We also give an exact algorithm
polynomial for each fixed excess above the elementary
two-moves-per-active-card bound. The unrestricted
controller's computational hardness remains unresolved.

## 1. Introduction

A physical shuffle and a random permutation are different
objects. A controller can choose a random permutation
cheaply. The machine must then move the cards into that
order. We study the second problem.

The machine, called 3SM, has two restrictions. Cards can
be removed only from the tops of stacks, and transfers
are allowed only between adjacent stacks. The controller
knows the complete initial and target orders and may use
ordinary computer memory. The objective is to minimize
physical transfers, while keeping planning practical.

The case n=52 is of particular interest. Asymptotic
complexity alone is not enough: a construction costing
624 moves and one averaging 323 moves both have
O(n log n) move complexity. We therefore distinguish
proved bounds, exact finite computations, and sample
measurements throughout.

Our main construction combines exact small instances with
recursive merging. The important observation is that a
plan for a small deck remains valid when other cards lie
beneath its working cards. This permits exact plans to be
used as subroutines inside larger decks. Reflection of
the side stacks and cancellation of inverse moves then
reduce the cost of joining those subroutines.

### 1.1. Related work

Sorting through networks of stacks and queues is a
classical problem; Tarjan [1][tarjan] provides an early
general formulation. Albert and Bousquet-Mélou [2][albert]
study permutations obtainable through two stacks in parallel.
Their model has separate input and output streams.
In 3SM, D serves as both initial and final storage, and
transfers across its two incident edges are reversible.
We do not identify these models or transfer their
enumeration results to 3SM.

Radix sorting and merging are standard sorting methods
[3][knuth]. Patience sorting provides useful background on
decomposing sequences into monotone subsequences
[4][aldous-diaconis].
We use these ideas to construct legal transfer sequences;
the move bounds below follow from the stated 3SM rules.

Uniform target generation uses the modern Fisher–Yates
procedure in Durstenfeld's form [5][durstenfeld]. The
contribution of this report is the construction and analysis
for the specified machine, together with reproducible finite
computations. We make no claim that the constructions
are new in the broader stack-sorting literature.

## 2. Model and elementary constructions

### 2.1. States, moves, and costs

A state is a triple (A,D,B) of finite sequences containing
each of n distinct cards exactly once. Sequences are
written top to bottom. The four legal move types are

$$
  AD,\quad DA,\quad DB,\quad BD,
$$

where XY removes the top card of X and places it on Y.
The source must be nonempty. Each move costs one
operation. In particular, transferring a card from A
to B through D costs two operations.

![The three-stack machine.][machine-figure]{width=75%}

*Figure 1.* The initial state of 3SM. All cards are on D.
The final state also has all cards on D, in a prescribed
order. There is no A–B edge.

A *plan* is a finite legal word in these four move types.
Let e=(0,1,...,n−1), and define

$$
\begin{aligned}
 d_n(\pi)
   &=\min\{|w|:w\text{ takes }(\varnothing,e,\varnothing)
                   \text{ to }(\varnothing,\pi,\varnothing)\},\\
 M_n&=\max_{\pi\in S_n}d_n(\pi),\\
 \mu_n&=\frac{1}{n!}\sum_{\pi\in S_n}d_n(\pi).
\end{aligned}
$$

Relabeling cards reduces any initial order to e.
Alternatively, assigning each card its target position
reduces routing to sorting by target rank.

We count only physical transfers in d_n. Planning time
and memory are separate costs. We assume n>=1; the
implementation also accepts the empty deck.

### 2.2. Uniform shuffling

Choose a target with the following array algorithm
[5][durstenfeld], then execute a plan that realizes it.

~~~text
target = copy(initial)
for i = n-1, n-2, ..., 1:
    j = uniform integer from {0, ..., i}
    swap target[i] and target[j]
~~~

**Proposition 1.** If the integer choices are independent
and uniform, and the planner realizes every target, the
final deck is uniform on the n! permutations.

*Proof.* There are n! equally likely choice sequences.
Each gives a different final permutation. Executing a
plan preserves the chosen target. ∎

A planner may spend different amounts of time on different
targets without affecting this argument. It must not
discard targets because they are expensive to realize.
Seeded pseudorandom targets are used for experiments;
the proposition concerns ideal uniform choices.

### 2.3. Reversal

Write $\rho_n=(n-1,\ldots,0)$.

**Proposition 2.** For every n>=1, reversal has a plan
of exactly 4(n−1) moves. Thus
$d_n(\rho_n)\le 4(n-1)$.

*Proof.* For n=1, use the empty plan. For n>=2, use

$$
 (DA)^{\,n-1}\;DB\;(AD\;DB)^{\,n-2}\;AD\;(BD)^{\,n-1}.
$$

After the first two blocks, A contains
(n−2,...,0), B contains (n−1), and D is empty.
The repeated pair transfers n−2,...,1 from A to B
through D. Thus A contains (0), and B contains
(1,...,n−1). Return 0 to D, then drain B into D.
The result is $\rho_n$. The move count is

$$
 (n-1)+1+2(n-2)+1+(n-1)=4(n-1).
$$
∎

For example, at n=4 the successive states are:

| After | A | D | B |
|---|---|---|---|
| start | () | (0,1,2,3) | () |
| DA³ DB | (2,1,0) | () | (3) |
| (AD DB)² | (0) | () | (1,2,3) |
| AD | () | (0) | (1,2,3) |
| BD³ | () | (3,2,1,0) | () |

This construction is valid for every n. The separate
question $M_n\le4(n-1)$ asks whether *every* permutation
can be realized within the same budget. Proposition 2
does not assert that stronger statement.
Section 5.4 proves that the reversal construction is
optimal for every n.

### 2.4. A radix baseline

Let b=ceil(log₂ n). Process the b bits of target ranks,
least significant first. For each bit, drain D into A
for zero and B for one, then return all of B followed
by all of A.

**Proposition 3.** Full binary radix passes realize every
target in exactly 2nb moves.

*Proof.* Each pass reverses each bit class twice and is
therefore a stable partition. After j passes, the deck
is sorted by its lowest j bits. Each full pass transfers
every card out of D and back, at cost 2n. ∎

For example, ranks (3,0,2,1) become (0,2,3,1) after the
low-bit pass, then (0,1,2,3). At n=52, the full algorithm
uses 624 moves.

We implemented several reductions. A pass already
partitioned can be skipped. A bottom suffix of one-bits
can remain on D. Increasing codes need not be consecutive,
and cards whose target relative order already agrees with
their initial order can share a code. These refinements
preserve the stable-sort argument. They provide a useful
baseline for the experiments in Section 6.

## 3. Exact small instances

The complete state space has

$$
  N_n=n!\binom{n+2}{2}
$$

states: choose an ordering of the n cards, then cut it
into the three stack sequences. The state graph is
undirected, since every move has an inverse.

We performed breadth-first search from
$(\varnothing,e,\varnothing)$ through n=9.
Permutation ranks and stack-cut indices give a dense
state encoding. Distances to states with empty A and B
give d_n for every target. These are exact finite
computations, not sampled estimates.

| n | Exact mean $\mu_n$ | Exact maximum $M_n$ |
|---|---:|---:|
| 1 | 0 | 0 |
| 2 | 2 | 4 |
| 3 | 5 | 8 |
| 4 | 97/12 | 12 |
| 5 | 111/10 | 16 |
| 6 | 5081/360 | 20 |
| 7 | 5414/315 | 24 |
| 8 | 410341/20160 | 28 |
| 9 | 4287301/181440 | 32 |

*Table 1.* Exact optimal costs. The uniform mean includes
the identity target at cost zero.

The n=9 graph has 19,958,400 states. The recorded search
took approximately 3.56 seconds and allocated about
120 MB for its queue and distance arrays. A separate
tuple-based Python BFS agreed with all target distances
and state counts through n=5.

For runtime use we retain one shortest plan for every
target through n=8: 46,233 plans in total. The files occupy
less than 1 MB. Arbitrary initial and target orders are
handled by relabeling. All stored plans have been replayed
independently and checked against their optimal distances.

The role of these tables is local. They allow a small
block to be sorted cheaply. The roughly linear costs in
Table 1 do not justify extrapolation to n=52.

## 4. Recursive merging with exact leaves

### 4.1. The embedding invariant

Call the cards handled by a subproblem its *active cards*.
Other cards may lie below them on any of the three stacks.

**Lemma 4 (protected bases).** A plan legal for an isolated
deck remains legal if arbitrary fixed sequences are placed
beneath its cards on D and beneath its initially empty
side stacks. It has the same effect on the active cards
and leaves all three fixed sequences untouched.

*Proof.* Induct over the plan. Every move has a nonempty
source in the isolated execution. Its source in the
embedded execution therefore has the same active top
card above its fixed base. The move transfers that card
and preserves the induction hypothesis. ∎

The lemma concerns the planned sequence of moves.
Execution must use active lengths, rather than draining
a physical stack that also contains a fixed base.

### 4.2. The merge construction

For the basic construction, sort the top k cards of D
as follows. Ranks refer to the desired target order.

~~~text
SORT(k):
    if the active segment is increasing:
        return
    if k <= 8:
        execute an exact table plan
        return

    a = floor(k/2)
    b = k-a
    SORT(a)
    move exactly a cards from D to A
    SORT(b)
    move exactly b cards from D to B
    MERGE(a,b)
~~~

MERGE repeatedly returns the larger exposed active rank
from A or B to D, stopping after exactly a and b cards
have been consumed. An exhausted active part is ignored,
even if its stack contains a fixed base.

**Theorem 5.** SORT realizes the desired order and restores
the original bases on A and B. It uses O(k log k) moves.

*Proof.* The table cases are correct. For a larger segment,
the induction hypothesis sorts its first part. Parking
that part on A reverses it, so its largest rank is exposed.
Lemma 4 permits the second recursive call while that part
remains on A. Parking the second sorted part on B again
exposes its largest rank.

MERGE sends the active cards to D in decreasing order.
Since each is pushed onto D, the final active order is
increasing. The specified lengths protect all bases.
The node adds k parking moves and k merge moves, giving

$$
 T(k)\le T(\lfloor k/2\rfloor)
       +T(\lceil k/2\rceil)+2k,
$$

with bounded-size table cases. ∎

![A merge with two two-card children.][merge-figure]{width=100%}

*Figure 2.* A four-card merge node, shown independently
of the table cutoff. The first sorted part (1,4) is parked
as (4,1) on A; the second, (2,3), is parked as (3,2) on B.
The merge word AD BD BD AD leaves (1,2,3,4) on D.
Gray bases represent untouched cards and may be empty.
Arrows between panels abbreviate sequences of operations.

### 4.3. Reflection and cancellation

Two operations on plans are useful.

First, exchanging A and B throughout a plan preserves its
effect on D. Each child may be reflected independently;
Lemma 4 ensures that unequal fixed bases cause no problem.

Second, adjacent inverse moves can be deleted. They move
one card out and immediately return it, restoring exactly
the preceding state. Repeated deletion therefore preserves
legality and the final state.

At a merge node we try both reflections of each child,
giving four concatenations. We reduce each by inverse
cancellation and retain the shortest. This step changes
neither the recursive invariant nor its unreduced bound.

### 4.4. Choosing splits and direction

Balanced splits are inexpensive, but their boundaries
need not be favorable. The default planner considers
splits within four positions of the midpoint, memoizing
the result for each input interval. A thorough variant
considers every contiguous split. Both compare their
result with the balanced plan.

We also solve the reverse routing problem, from target
to initial, then reverse its move word and invert each
move. This yields another plan for the original problem.
The controller retains the shorter plan.

These searches are heuristic. Storing one shortest plan
per interval does not establish optimal substructure:
a longer child can cancel more with its parent.
Even equally short leaf plans may differ in useful
boundary behavior. The all-splits variant is therefore
not claimed to optimize over all possible merge plans.

For n<=8, the controller uses exact lookup directly.
Identity and reversal have direct plans. An additional
candidate distributes the cards into two increasing
subsequences, when possible, and merges them once.
All candidates preserve the balanced bound because the
controller selects a plan no longer than its baseline.

### 4.5. Solve for the parked endpoint

The parent does not actually need a child sorted on D.
It needs that child in reverse sorted order on A or B.
Solving the former problem and then parking can therefore
miss a shorter route to the endpoint that matters.

We ran BFS from the reversed sorted A endpoint, rather
than from identity on D. Reversibility gives exact
distances and plans from every D-only order. Reflection
provides B plans. The protected-base argument of Section
4.1 applies unchanged, so these are legal subroutines.

The new controller retains the old complete plan and
adds a merge-tree search using these parked leaves.
Each parent parks its children on opposite sides and
merges them onto D in exactly n transfers. Reflected
D-to-D plans followed by parking remain alternatives.
This matters because a locally shortest word need not
give the best cancellation in an enclosing execution.

The finite maximum parked costs are:

| k | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| P(k) | 1 | 4 | 7 | 12 | 15 | 20 | 23 | 28 |

All 46,233 words were replayed above protected bases for
both sides. Independent tuple-state BFS agrees through
n=5. This change is available as `--algorithm parked`.
The faster default is unchanged; both retain exact
D-to-D lookup for decks of at most eight cards.

## 5. Performance analysis

### 5.1. A 52-card upper bound

At n=52 the balanced tree has three merge levels and
eight leaves: four of size six and four of size seven.
Ignoring cancellation, the exact leaf maxima give

$$
  B_0(52)=3(104)+4(20)+4(24)=488.
$$

We can account for some cancellations in advance.
For a selected child plan w, let L(w) be its length and
r(w) the length of its final run of identical returns
to D. Set r=0 for the empty plan.

**Lemma 6 (terminal cancellation).** If a child of size k
is subsequently parked on a side stack, a reflection
of its plan saves at least 2r(w) moves at that boundary.

*Proof.* Reflect the child so that its final return source
is the parking destination. Its final r returns and the
first r parking moves are inverse pairs. Since r<=k,
there are enough parking moves to cancel all of them. ∎

For a table leaf define
$E(k)=\max_w(L(w)-2r(w))$, over the stored plans.
Exhaustive inspection gives:

| k | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| E(k) | 0 | 2 | 4 | 8 | 12 | 16 | 20 | 24 |

For a composite child with length bounded by B(k), every
nonempty plan has r>=1. Its effective parked cost is thus
at most max(0,B(k)−2).

**Theorem 7.** With the stored leaf tables and independent
child reflections, the balanced hybrid uses at most
444 moves for every 52-card target. The original three
controller modes inherit this bound.

*Proof.* Both child-tail cancellations can be made
simultaneously. For a node of size a+b, the resulting
bound is $2(a+b)+E(a)+E(b)$. Hence

$$
\begin{aligned}
 B(13)&=26+16+20=62,\\
 B(26)&=52+2(62-2)=172,\\
 B(52)&=104+2(172-2)=444.
\end{aligned}
$$

The node chooses the shortest of all four reflected
concatenations, so it is no longer than the one used
in this argument. Other controller candidates are
accepted only when no longer than the baseline.
The direct reversal plan costs 204 at n=52. ∎

The finite E(k) values depend on the actual stored plans.
Replacing them with different shortest plans requires
checking their terminal runs again. These constants are
part of the executable verification. Theorem 7 is a
computationally supported upper bound, not a claim that
the implementation attains 444 on some target.

### 5.2. An unreduced expected cost

A useful analytic comparison is the fixed balanced tree
that performs every internal parking and merge operation,
without sorted-segment shortcuts or cancellation.

**Proposition 8.** On a uniformly chosen 52-card target,
this unreduced construction has expected cost

$$
 312+4\mu_6+4\mu_7
 =312+4\frac{5081}{360}+4\frac{5414}{315}
 =437.204761\ldots.
$$

*Proof.* Each fixed leaf sees a uniform relative target
order, so its exact plan has mean cost $\mu_k$.
The three internal levels cost 312. Apply linearity
of expectation; independence of leaf costs is not needed. ∎

This expectation belongs to the specified unreduced
construction. The implemented hybrid has a lower measured
mean. We do not have an exact expectation for the final
heuristic controller.

### 5.3. Counting lower bounds

Let $\Pi$ be uniform on $S_n$, and let
$X=d_n(\Pi)$. Every final distance is even, since a move
changes the number of cards on D by one.

A shortest word contains no adjacent inverse pair.
There are two possible first moves and at most three
choices after each move. Thus the number of candidate
words of positive length L is at most $2\cdot3^{L-1}$.
Exchanging A and B pairs distinct nonempty closed words
with the same final permutation. Consequently,

$$
 |\{\pi:d_n(\pi)\le2k\}|
 \le U(2k):=1+\frac{3(9^k-1)}8.
 \qquad\text{(1)}
$$

This count may include illegal words and repeated
representations. Both only enlarge the upper bound.

**Proposition 9.** Both $M_n$ and $\mu_n$ are at least
$\log_3(n!)-O(1)$. Consequently, their asymptotic order
is $\Theta(n\log n)$.

*Proof.* Equation (1) is $O(3^{2k})$. To cover n!
targets, the maximum distance must therefore be at least
$\log_3(n!)-O(1)$. For the mean, use

$$
 \mu_n
 =2\sum_{k\ge0}\Pr(X>2k)
 \ge\frac{2}{n!}\sum_{k\ge0}
           \max(0,n!-U(2k)).
 \qquad\text{(2)}
$$

Sum up to the largest even threshold below
$\log_3(n!)$. The sum of the subtracted geometric
terms is O(1) after division by n!, yielding the claimed
mean bound. Stirling's formula gives $\Omega(n\log n)$;
Proposition 3 supplies the matching order above. ∎

A finite-state count gives a better numerical bound.
Track $(a,b,\ell)$, the heights of A and B and the
previous move. Permit only transitions with nonnegative
heights, a+b<=n, and no immediate inverse. Let c_n(L)
count length-L words returning to a=b=0. Legality depends
only on these heights, so this counts closed legal words
exactly, although it still overcounts target permutations.

Replace U in (2) by

$$
 U_n(2k)=1+\frac12\sum_{j=1}^{k}c_n(2j).
 \qquad\text{(3)}
$$

The computation at n=52 gives

$$
  M_{52}\ge156,\qquad
  \mu_{52}\ge154.9453.
$$

These are bounds on optimal costs, not predictions of
the implemented planner's mean.

### 5.4. A structural lower bound

Let s be the length of the longest common bottom suffix
of e and the target, and put m=n−s. Let q be the target
restricted to the first m initial cards. Define $I_2(q)$
as the maximum number of entries covered by two increasing
subsequences, with $I_2(\varnothing)=0$.

**Theorem 10.** Every target satisfies

$$
 d_n(\pi)\ge 4m-2I_2(q)
             =4n-2I_2(\pi)-2s.
 \qquad\text{(4)}
$$

*Proof.* Deleting a set of cards from every intermediate
state, and omitting their moves, preserves legality.
Indeed, every retained card that was exposed is still
exposed after deletion.

If a card never leaves D, neither can any card initially
below it. Such cards form a common bottom suffix.
Thus all m cards of the active prefix must leave D
and return, each making a positive even number of moves.

Project an arbitrary plan onto those active cards moved
exactly twice. Each makes one excursion to A or B.
All their departures precede all their returns: the first
returned card can never leave again and would block any
remaining departure. Within either side stack, the two
reversals preserve initial relative order. These cards
therefore form a union of two increasing subsequences
of q. At most $I_2(q)$ cards cost two moves; every other
active card costs at least four. This proves the first
inequality.

The common suffix consists of the s largest labels in
increasing order. Appending it increases $I_2$ by exactly
s: all suffix entries can extend one subsequence, and
deleting them from any cover removes at most s entries.
This gives the second expression. ∎

For reversal with n>=2, s=0 and $I_2(\rho_n)=2$.
Theorem 10 and Proposition 2 give
$d_n(\rho_n)=4(n-1)$. For n=1 both sides are zero.
In particular, $M_{52}\ge204$.

For example, (1,0,3,2) has m=4 and $I_2=4$.
The bound is eight moves, attained by distributing the
increasing subsequences (1,3) and (0,2) and merging them.
The structural bound is exact for every target through
n=4, but not for all larger targets.

The uniform expectation can be computed without sampling.
Under the Robinson–Schensted correspondence, a permutation
has a partition shape $\lambda\vdash n$. Greene's theorem
[7][greene] gives $I_2=\lambda_1+\lambda_2$, taking missing
rows as zero. The number of permutations of this shape
is $(f^\lambda)^2$, where the hook-length formula gives
$f^\lambda=n!/\prod_{c\in\lambda}h(c)$; see [8][dukes].
Also $\Pr(s\ge j)=(n-j)!/n!$. Taking expectations in (4),

$$
 \mu_n\ge 4n
 -\frac{2}{n!}\sum_{\lambda\vdash n}
       (\lambda_1+\lambda_2)(f^\lambda)^2
 -\frac{2}{n!}\sum_{j=1}^n(n-j)!.
 \qquad\text{(5)}
$$

Exact integer evaluation over the 281,589 partitions of
52 gives a rational bound whose decimal expansion starts
166.8791708235294. Thus $\mu_{52}\ge166.87917$.
The calculation checks $\sum_\lambda(f^\lambda)^2=52!$.
The instance inequality and ensemble sums were checked
against every target through n=9; an independent
two-subsequence dynamic program checks $I_2$ through n=6.

This bound is linear in n. Counting remains necessary
for the asymptotically stronger $\Omega(n\log n)$ result.
Nor does (4) alone locate the first target requiring
more than 4(n−1) moves.

### 5.5. Reversal and the all-permutations bound

The distinction from Section 2.3 matters here.
For every n, Proposition 2 supplies a reversal plan of
4(n−1) moves. That construction remains valid at every
deck size and, by Theorem 10, is optimal for reversal.

The stronger statement

$$
  M_n\le4(n-1)
  \qquad\text{(6)}
$$

concerns all target permutations. It holds in our exact
computations through n=9. It cannot hold for all n:
at n=212, equation (1) gives

$$
 U(844)<212!,
$$

so some target requires at least 846 moves. Reversal
still has its 844-move plan.

If $n_*=\min\{n:M_n>4(n-1)\}$, the current results give
$10\le n_*\le212$. They do not determine n_* or establish
a phase transition in a probabilistic sense. In particular,
these results do not decide whether $M_{52}\le204$.

### 5.6. Host computation

Physical moves are not the planner's running time.
The Python balanced implementation repeatedly copies
partial move lists. Conservative bounds are
O(n log² n) planning time and O(n log n) storage.

The full interval search examines O(n³) splits in total.
Each evaluation manipulates plans of length O(n log n),
giving safe bounds of O(n⁴ log n) time and
O(n³ log n) storage. Counting split evaluations alone
would understate its host cost.

The implementation limits split search to n<=64.
Above that size it uses balanced recursion, preserving
the large-n bounds. The cutoff is an engineering choice,
not a property of the optimal state graph.

### 5.7. The parked-leaf upper bound

Let U(n) bound the D-to-D construction and P(n) its
reversed sorted parked endpoint. For n<=8 use the
parked table above and U(n)=4 max(0,n−1). A split into
k and n−k proves

$$
 U(n)\le n+P(k)+P(n-k),\qquad
 P(n)\le\max\{n,U(n)+n-2\}.
$$

The first inequality counts the child plans and the
parent's n returns. For the second, reflect a nonempty
D plan so that its final return cancels the first
parking move. An empty plan needs n parking moves.

**Theorem 11.** The parked controller uses at most
410 moves for every 52-card target.

*Proof.* Use the finite leaf maxima and choose the split
minimizing P(k)+P(n−k). A sufficient tree has four
seven-card leaves and three eight-card leaves:

$$
\begin{aligned}
 P(14)&\le14+2(23)+14-2=72,\\
 P(16)&\le16+2(28)+16-2=86,\\
 P(22)&\le22+28+72+22-2=142,\\
 P(30)&\le30+72+86+30-2=216,\\
 U(52)&\le52+142+216=410.
\end{aligned}
$$

The controller includes each minimizing split and its
reflection, and retains the required parked alternatives
at every child. Other choices can only shorten the plan.
The special identity, reversal, and exact-small cases
also satisfy the bound. ∎

Balanced splits alone give 420. The implementation uses
these searches through 64 cards and the old balanced
fallback above that size, preserving O(n log n) moves
and conservative O(n log² n) host time asymptotically.
The new 410 guarantee belongs to the parked mode; the
three original modes retain their 444 guarantee.

### 5.8. Conditional excursion obstructions

The first positive gaps in Theorem 10 occur at n=5.
They reveal a joint constraint. Keeping the maximum
possible number of cards at two moves can force a
remaining card to make six or eight moves, even when
another optimum moves every card at most four times.

For each pattern below, let T be its four nonexceptional
cards. If all cards of T move exactly twice, the stated
minimum for the exceptional card is forced:

| Relative target | Exceptional rank | Minimum moves |
|---|---:|---:|
| (2,1,4,3,0) | 0 | 6 |
| (2,4,1,3,0) | 0 | 8 |
| (4,1,0,3,2) | 4 | 6 |
| (4,2,0,3,1) | 4 | 8 |

The six-move implications follow already from the exact
distance 14 of each pattern. The eight-move implications
were established by a separate finite BFS: track the
physical state and the four selected cards' move counts,
each capped at two; leave the exceptional card unlimited.
The shortest compatible plan has length 16. An independent
bounded oracle rejects six exceptional moves and accepts
eight. Deletion transfers these implications to any
occurrence in a larger target without changing retained
cards' individual move counts.

Work on the active prefix q of length m. For a candidate
twice-moved set T, define F6(T) as the cards forced to
at least six moves by any occurrence whose four selected
cards lie in T; let F8(T) be those forced to eight.
The latter is a subset of the former. Count each card
once in each set, regardless of how many occurrences
force it. Let the candidate family consist of all sets
covered by two increasing subsequences of q.

**Proposition 12.** A valid instance lower bound is

$$
 \min_T\{4m-2|T|+2|F6(T)|+2|F8(T)|\}.
$$

*Proof.* The actual twice-moved set belongs to the
candidate family by Theorem 10. Its cards cost two;
other active cards cost at least four. Forced six-move
cards cost two more, and forced eight-move cards another
two. Deletion proves each charge, and set union prevents
double charging. Minimize over the larger candidate
family. ∎

We enumerate candidate sets with greedy two-chain tails
and a suffix maximum-cardinality DP. To certify B+2h,
only set sizes I2,I2−1,...,I2−h+1 need inspection; smaller
sets already cost enough. Accumulated forced charges
are monotone under extension and permit early pruning.
An interrupted layer supplies no new certificate; bounds
from fully completed layers are retained.

Independent brute-subset enumeration agrees on all
5,913 targets through n=7, never exceeding exact distance.
The bound is exact through n=5. On 100 held-out 52-card
targets, a three-second per-target search increased the
mean certified bound from 166.86 to 172.56. One separate
target was certified at 178 instead of 166. These are
instance certificates and sample summaries, not a new
exact uniform-mean theorem.

This particular relaxation has a useful ceiling: choosing
any two cards as T triggers no four-card implication and
gives 4m−4. It therefore cannot establish the first
failure of the universal 4(n−1) budget, regardless of
how long its enumeration runs. A stronger model must
constrain low-cardinality twice-moved sets as well.

### 5.9. Direct side-output merging

The parked recurrence in section 5.7 still makes a detour:
it merges a composite child onto D before moving it to
its required side. We can avoid that detour recursively.
Let U(n) bound a central-output plan in a requested order,
and P(n) bound a side-output plan in the reverse order.
Either contract may request either orientation.

To produce decreasing output on A, park the first a
cards on B in increasing order, then sort the next b
cards on D in increasing order. The first action uses
P(a) with its requested orientation reversed. Merge by
taking the smaller exposed rank: use DA for a D card,
and BD DA for a B card. The intermediate BD does not
lose access to the D head, since DA immediately removes
the transferred card. This costs 2a+b transfers.

For example, B=(1,4) and D=(2,3), both top first, merge
onto A using BD DA, DA, DA, BD DA. The final A order
is (4,3,2,1). Protected bases are untouched throughout,
so the embedding invariant applies to both contracts.

Consequently, for a+b=n,

$$
 U(n)\le P(a)+P(b)+n,\qquad
 P(n)\le P(a)+U(b)+2a+b.
$$

Use the same exact leaves through eight cards and retain
the old parking alternative. Optimizing the two fixed
split recurrences independently gives P(23)<=125 and
P(29)<=175. Thus every 52-card target has a construction
of at most 52+125+175=352 moves. The implementation
includes these certified splits even when a bounded
search considers additional choices.

There is also a better asymptotic constant. Choose
a=floor(n/2), b=ceil(n/2), and adjusted costs
V_U(n)=U(n), V_P(n)=P(n)-n/3. In the U recurrence the
adjusted toll is 4n/3. In the P recurrence it is
4n/3+(2/3)(a-b)<=4n/3. Summing over balanced levels
and using singleton adjusted cost at most 2/3 proves

$$
 U(n)\le \tfrac43 n\lceil\log_2 n\rceil+\tfrac23 n
       =\tfrac43 n\log_2 n+O(n).
$$

Exact leaves and bounded split improvements can only
reduce this cost. Above 64 cards the controller uses
balanced recursion with these same oriented contracts.
[The construction note](research/oriented-merge.md)
gives the full endpoint proof and independent review.

### 5.10. Residual structural and group bounds

The endpoint lower bound also extends to arbitrary states.
Delete the longest correct bottom suffix of D, and let
a,b,d be the remaining counts on A,B,D. The mandatory
cost is M=a+b+2d. Call a card ordinary if it costs only
its mandatory one or two moves. Every other card costs
at least two more moves, by parity.

Project onto ordinary cards. All D departures precede
all returns. D cards assigned to either side form an
increasing subsequence in initial D order. Retained
initial-side cards form decreasing subsequences on their
sides. Each retained D rank assigned to a side must
exceed every retained initial-side rank there. These
conditions are necessary and sufficient for the projected
ordinary execution: depart the D cards, then merge the
two decreasing sides back onto D.

If K is the maximum cardinality satisfying these coupled
conditions, then the residual distance is at least
M+2(a+b+d-K). A weighted tail-pair dynamic program computes
K in O(n³). Its maximum with the existing pattern-database
bound is admissible. Exhaustive cross-language checks
cover all 23,115 states through six cards; an independent
subset enumerator checks the relaxation through five.
See [the residual proof](research/residual-bounds.md).

Write this residual bound as $R=4m-s-2K$, where m is
the active count and s counts active side cards. It is
consistent: along any legal edge, $|R(x)-R(y)|\le1$.
For a D departure, an ordinary successor set is feasible
before the move. For a side return not extending the
correct suffix, deleting that card from a successor set
gives a predecessor set, so K increases by at most one.
If a return extends the suffix, m,s,K each decrease by
one and R decreases by exactly one. Reverse moves give
the other inequality. Thus pathmax cannot improve R.

A genuinely stronger bound partitions the cards into
fixed disjoint sets P,Q and adds an exact pattern
distance on P to R on Q. Deletion preserves legality,
and the two terms charge disjoint moves. Taking the
maximum over fixed partitions preserves consistency.
Selecting just one partition adaptively remains
admissible but need not be consistent. The
[third residual study](research/third-residual.md)
gives the complete argument and implementation checks.

Separately, finite budget exhaustion gives group clauses:
if a specified pair or triple of cards moves only twice,
some card in a specified complementary group must move
at least six times. Charging disjoint groups avoids
double counting. The new five-card clauses close the
previous six-card gap, and the full catalog through six
cards matches 5032 of 5040 seven-card exact distances.
However, all pair triggers found are decreasing, so an
increasing two-card candidate still evades every clause.
This relaxation retains the 4m-4 ceiling. Details and
selected 52-card improvements are in the
[obstruction note](research/excursion-obstructions.md).

### 5.11. An asymmetric split and a ternary recursion

The side-output contract need not use the same split
as the central contract. Set $a=\lfloor n/3\rfloor$,
$b=n-a$, and split the central child into
$c=\lfloor b/2\rfloor$, $d=\lceil b/2\rceil$.
Substitution in the two endpoint recurrences gives

$$
\begin{aligned}
 P(n)&\le P(a)+U(b)+n+a\\
     &\le P(a)+P(c)+P(d)+2n.
\end{aligned}
$$

The three child sizes differ by at most one. These are
ordinary binary machine routines composed over two
levels, not a new three-way machine operation.

**Theorem 13.** There is a controller using
$2n\log_3 n+O(n)$ moves for every n-card target.
It retains the 352-move guarantee at n=52.

*Proof.* In the parked macro recursion, each level has
total active size at most n and toll at most 2n. Its
depth is at most $\lceil\log_3 n\rceil$. The implemented
finite leaves through 64 satisfy P(k)<=8k, giving
$P(n)\le2n\lceil\log_3 n\rceil+8n$. A balanced central
root has two parked children of total size n and toll n,
so has the stated leading term. All compositions preserve
protected bases. The finite controller through 64 is
unchanged, so its 52-card bound remains 352. ∎

The coefficient of $n\log_2 n$ is $2/\log_2 3$,
approximately 1.26186, instead of 4/3. To see why thirds
are natural, try supersolutions
$U(n)=cn\log_2 n+un$, $P(n)=cn\log_2 n+pn$.
With $\delta=p-u$ and binary entropy H, split fractions
x for U and y for P require

$$
 cH(x)\ge1+\delta,\qquad
 cH(y)\ge1+y-(1-y)\delta.
$$

Since H(x)<=1, these imply $c[H(y)+1-y]\ge2$.
The bracket is maximized at y=1/3, with value
$\log_2 3$; take x=1/2. This derivation explains the
construction. It proves no lower bound on the machine,
nor optimality outside this restricted supersolution
ansatz. The [construction note](research/third-ternary.md)
and [independent review](research/third-ternary-review.md)
give implementation details and protected-base checks.

The fourth campaign closes the remaining recurrence
question. Fix the numeric leaf charges U(k),P(k) through
64 as above. For larger sizes, define the numeric costs
by minimizing each displayed endpoint recurrence over
all positive splits. One may also allow P(n) to choose
U(n)+n−2. No other cancellation credits are included.

**Proposition 14.** Both optimized numeric costs have
leading term $cn\log_2 n$, where $c=2/\log_2 3$.

*Proof.* Set $\delta=c-1$ and define lower potentials

$$
 \Phi_U(n)=cn\log_2 n-8n,\qquad
 \Phi_P(n)=\Phi_U(n)+\delta n,
$$

with both zero at n=0. Since c<4/3, the finite leaves
satisfy these bounds using only U(k)>=0 and P(k)>=k.
For a central split of fraction x, the sum of child
potentials and toll exceeds $\Phi_U(n)$ by
$cn[1-H(x)]\ge0$. For a parked split of fraction y,
the corresponding excess over $\Phi_P(n)$ is
$n[2-c(H(y)+1-y)]\ge0$. Thus every split preserves
the lower bounds, not just the thirds construction.
The optional repark candidate has excess
$(2-c)n-2>0$ for n>64 and also preserves them.
Induction proves both numeric lower bounds. The balanced
ternary construction supplies matching upper bounds with
an O(n) remainder, proving the leading term. ∎

This is sharpness of a specified cost recurrence, not
a lower bound on executions. The finite charges are
upper guarantees, and the model omits cross-boundary
cancellation. Better algorithms or richer endpoint
contracts can escape it. Exact all-split minimization
through 4096 checks the numeric model; the proof is
inductive for all sizes. See the
[recurrence study](research/fourth-recurrence.md).

## 6. Computational experiments

### 6.1. Method

The primary comparison uses 1,000 targets at n=52,
generated with Python's seeded pseudorandom generator,
seed 20261003. Each algorithm receives the same targets.
Every emitted move sequence is replayed by a separate
simulator, checking legal sources, exact final order,
and empty side stacks.

Planning times include plan generation and cancellation
but exclude simulation. They are indicative measurements
from one Linux x86-64 host running Python 3.13.5, not
isolated or portable timing guarantees.

The principal timing records precede the final
two-subsequence shortcut. That addition changes none of
the 1,000 target plans but adds a small amount of host work.
These original tables precede the bounded campaigns;
later subsections report their separate samples.

### 6.2. Random 52-card targets

| Planner | Mean moves | Maximum seen | Time/plan |
|---|---:|---:|---:|
| Radix, simplified | 608.506 | 624 | 0.17 ms |
| Adaptive radix | 478.088 | 518 | 0.16 ms |
| Patience merge | 437.528 | 510 | 0.48 ms |
| Balanced hybrid | 363.164 | 402 | 0.18 ms |
| Fast controller | 357.254 | 386 | 0.34 ms |
| Default controller | 322.838 | 350 | 53.24 ms |

*Table 2.* Shared 1,000-target comparison. Every maximum
in this table is a sample maximum.

The fast controller compares the two routing directions.
The default adds the bounded split search of Section 4.4.
Its sample mean standard error is 0.308 moves. This
quantifies sampling variation under the sampling model;
it does not bound the worst case or prove optimality.

The sequence of improvements has a simple interpretation.
Stable passes give a reliable O(n log n) baseline.
Exact leaves reduce the cost of small subproblems.
Reflection and cancellation make their composition
cheaper. Split search then spends host time selecting
more favorable compositions.

For the first 50 targets from the same sample:

| Controller | Mean moves | Maximum seen | Time/plan |
|---|---:|---:|---:|
| Default | 324.04 | 342 | 55.38 ms |
| Thorough | 310.68 | 328 | 571.73 ms |

*Table 3.* Paired comparison on 50 targets. The thorough
mean should be compared with this default mean, rather
than with the 1,000-target mean in Table 2.

Thorough saves 13.36 moves per target on this sample at
approximately ten times the planning cost. Whether that
tradeoff is useful depends on the physical move time.

### 6.3. Structured targets and scaling

Selected 52-card targets give:

| Target | Default moves | Thorough moves |
|---|---:|---:|
| Identity | 0 | 0 |
| Reversal | 204 | 204 |
| Rotation by one | 104 | 104 |
| Rotation by 26 | 104 | 104 |
| 26 disjoint adjacent swaps | 104 | 104 |
| Interleaved halves | 102 | 102 |
| Reversed blocks of at most eight | 220 | 180 |

*Table 4.* Structured examples. Interleaving means
(0,26,1,27,...,25,51). Block reversal reverses each
successive block of eight cards, with a final block of
four. These counts are not asserted to be optimal.

Scaling checks used 20 targets per size. Above 64,
the controller uses balanced planning in both directions.

| n | Mean moves | Mean planning time |
|---|---:|---:|
| 128 | 1,180.1 | 0.98 ms |
| 1,024 | 15,578.4 | 11.55 ms |
| 4,096 | 78,734.1 | 57.23 ms |

*Table 5.* Large-n samples. The change in behavior between
searched n=64 and balanced n=65 reflects the controller
cutoff and should not be interpreted as a machine phase
transition.

### 6.4. Autonomous campaign and held-out comparisons

The parked method was frozen after 100 development
targets, before inspecting the new seed 2026100401.
On 1,000 held-out targets generated by repeated
`Random(seed).sample(range(52), 52)`, it gave:

| Controller | Mean | Sample max | Median planning |
|---|---:|---:|---:|
| Original recommended | 322.896 | 350 | 53.5 ms |
| Parked portfolio | 318.874 | 340 | 104.2 ms |

The paired mean saving is 4.022 moves, with an approximate
normal 95% interval [3.800,4.244]. There are 762 wins,
238 ties, and no losses. The last fact also follows from
retaining the original plan. All outputs were replayed.
The observed maximum 340 does not replace the 410 proof.

A separate lower-bound holdout uses the same seed but
repeated `shuffle`, hence different targets. Its 100
targets have mean structural bound 166.86 and conditional
bound 172.56: 87 improve by six, 11 by four, and two by
two. Two interrupted searches retain completed-layer
bounds. The stored target lists, not just their seeds,
are part of the reproduction record.

Running the parked controller on those actual 100 targets
gives mean upper cost 317.92 and sample maximum 342.
Their mean certified interval is [172.56,317.92], of width
145.36. Every recorded per-instance upper/lower ratio is
below 1.989. Neither these ratios nor the sample lower
mean assert a guarantee for unobserved targets.

Other bounded experiments delimit the next steps:

- Canonical-word counting removes immediate inverses,
  duplicate top-swap words, and doubled swaps, and fixes
  the first side of every closed component. Product-state
  BFS verifies shortest-word coverage through n=6.
  Its counting mean is 156.1873229. Combining its exact
  cumulative bound with the exact distribution of B
  gives no improvement over 166.8791708.
- Adaptive size-nine pattern LPs yield rational dual
  certificates. On 100 development targets, their maximum
  with B improves the sample mean from 166.98 to 167.26.
  The ordinary fractional formulation has a ceiling
  32n/9, or 186 at n=52 after even rounding.
- Six two- or three-color abstractions at n=8 improve
  B on 351 of 40,320 targets, but increase its mean only
  from 19.79182 to 19.809375. Direct scaling is unattractive.
- Choosing an additional shortest parked leaf with a
  maximal terminal parking run saved no moves on a
  20-target pilot. This option remains disabled.

Full-state eight-card pattern databases, combined with
mandatory moves of omitted cards, support target-specific
IDA*. Six selected nonreversal targets at n=10,...,12
were solved exactly, improving existing plans by two to
six moves. The largest completed search used 2,370 nodes.
Validation includes exact small tables, witness replay,
and an explicit interruption check.

The same method stalled on nearly reversed structured
targets at n=14,16,18. After 45 seconds each, their
certified intervals remained [48,50], [56,58], [64,66].
No lower-bound improvement follows from these timeouts.
Scaling was stopped; the residual heuristic, rather
than the small width of an endpoint interval, controls
the search difficulty.

### 6.5. Direct-output merge holdout

The new endpoint construction was developed on 20 targets
from the original development sample. Two settings were
then frozen: midpoint plus certified splits, and those
splits with an additional window of four. A fresh sample
of 100 targets uses seed 2026100402 and repeated
`sample(range(52),52)`.

| Controller | Mean | Sample max | Median planning |
|---|---:|---:|---:|
| Previous parked controller | 318.74 | 340 | about 103 ms |
| Direct-output, fast | 284.72 | 300 | 8.8 ms |
| Direct-output, window four | 273.80 | 288 | 134.9 ms |

Both new settings improve every target in this sample.
Every word was replayed; every cost is below the proved
352 bound. Times are indicative measurements on a shared
research host, not isolated timing guarantees. Neither
the sample maximum 288 nor mean 273.8 is an optimality
theorem. The earlier 1,000-target study and this fresh
100-target study must not be combined as a paired sample.

The residual structural bound also improves exact search.
It exhausts the old n=14 threshold 48 in about 4.5 search
seconds, proving optimum 50 with the existing replayed
upper word. A 45-second follow-up at n=16 exhausts 56
in about 43.66 seconds, proving optimum 58. The n=18
ten-second pilot retains [64,66]; no completed threshold
or new certificate is inferred from that timeout.

On the original lower-bound holdout, the new window
controller gives mean certified interval [172.56,271.76],
width 99.2, with every recorded upper/lower ratio below
1.695. This is a paired instance statement, not a new
uniform-mean theorem.

### 6.6. Third campaign: bounds and execution

The third campaign kept the exact uniform-mean theorem
at 166.87917. It also extracted a statistical conclusion
from the earlier 100-target lower-bound sample. These
are different kinds of statements.

Let B be the structural bound and define the fixed
latent statistic $G(\pi)=\min\{6,d(\pi)-B(\pi)\}$.
Certified gains underestimate G, even if their search
times are chosen adaptively. The sample has 100,98,87
certified gains reaching thresholds 2,4,6 respectively.
One-sided exact binomial inversion [12][clopper-pearson],
with error 1/300 at each threshold, gives simultaneous
lower probability bounds 0.944558,0.905985,0.754564.
Since $E[G]=2\sum_{j=1}^3\Pr(G\ge2j)$, adding their
contribution to the exact E[B] gives the 99% lower
confidence bound

$$
 \mu_{52}\ge172.08938.
$$

This coverage statement assumes IID-uniform targets;
the recorded seeded generator alone does not establish
that ideal premise. It is not a deterministic replacement
for (5). No independence between B and G is needed.
An alternative Hoeffding calculation [13][hoeffding]
is recorded separately, not selected after comparison
without accounting for multiple testing.

An elementary fractional relaxation has a severe limit.
Let $t_i\in[0,1]$ indicate a twice-moved card and
$e_i\ge0$ its extra excursions beyond four moves.
Minimize $4m-2\sum t_i+2\sum e_i$, subject to total
twice-card capacity I2, decreasing-triple constraints,
and the catalog's linearized group implications.
Whenever I2<=m/2, the assignment $t_i=I2/m,e_i=0$
is feasible and attains B exactly. Exact shape/suffix
counting puts 99.83144% of uniform 52-card targets in
this class. Even granting the remaining class the
relaxation's 4m−4 ceiling, its possible improvement in
uniform mean is at most 0.084995 moves. This limitation
does not apply to stronger chain constraints.

Adding every decreasing-chain inequality
$\sum_{i\in C}t_i\le2$ escapes this argument. On one
52-card pilot, a rational dual with only 17 nonzero rows
(eight group rows and nine chain rows) proves
$\sum_i(t_i-e_i)\le16.5$. Therefore d>=175, hence
d>=176 by parity. This matches an existing integer
certificate; it is not a new instance bound. Discovery
took about nine seconds. Verification reconstructs
the rows and checks rational loads without an LP solver.
The [ensemble note](research/third-ensemble.md) states
both relaxations precisely and provides the certificate.

The increasing-pair search found no obstruction among
371 distinct selected seven-card target/pair cases:
each admits a plan moving both selected cards twice
and every other card at most four times. These cover
94 targets, not all seven-card cases. The residual
implementation became about 2.7 times faster in a
ten-second n=18 pilot, but a 45-second follow-up still
did not complete threshold 64. The certified interval
remains [64,66]; no optimality is inferred from timeout.

The ternary planner computes only requested endpoints
above 64. A separate root optimization skips an unused
parked calculation without changing any selected word.
On three paired targets at each size, the new results
are as follows; times are median bidirectional planning.

| n | Oriented mean | Ternary mean | Oriented time | Ternary time |
|---|---:|---:|---:|---:|
| 52 | 284.00 | 284.00 | 4.24 ms | 4.19 ms |
| 512 | 5054.00 | 4930.67 | 110.71 ms | 41.30 ms |
| 4096 | 56854.67 | 55114.00 | 1007.48 ms | 333.03 ms |

These small paired checks support scaling comparisons,
not population estimates. Earlier timing tables remain
historical measurements before the root optimization.
The [third-campaign summary](research/third-campaign-summary.md)
links the frozen data and all stopping decisions.

### 6.7. Fourth campaign and release decision

A final bounded campaign tested three remaining leads.
The first gave Proposition 14: a matching lower potential
settles the coefficient of the numeric recurrence even
with arbitrary size-dependent splits.

The second tested rational certificates on 20 fresh
52-card targets, seed 2026100404. The structural mean
bound on this sample is 165.3. The earlier integer
method, capped at B+6 with three seconds per target,
gives mean 171.2; a ten-second chain/group LP pipeline
gives 171.9. Taking the maximum gives 172.3. All 20
LP outputs have independently verified rational duals.
Mean discovery time is approximately 0.18 seconds for
the capped integer method and 5.76 seconds for the LP.
These are sample bounds with different budgets, not
new uniform-mean theorems or an equal-budget comparison.

The LP has nine wins against that capped method. To
test whether this is chiefly a cap effect, the first
five winning targets receive integer search with a
ten-second budget and B+12 cap. Four apparent LP wins
reverse; one remains at 176 versus 174. This selected
subset does not estimate population performance. The
methods are complementary on these cases, but the LP
does not justify replacing the faster integer method.
All 20 rational proofs verify without SciPy in about
0.016 seconds total on this host. See the
[certificate follow-up](research/fourth-certificates.md).

The third experiment retains two or four endpoint plans
per interval instead of one. Representatives have distinct
six-operation prefix/suffix signatures; parents compare
their combinations after cancellation. The signatures
are heuristic, not a contextual dominance criterion.
An explicit complete-plan fallback to oriented_window
preserves its pointwise performance and 352 guarantee.

On 20 separate fresh targets, seed 2026100405:

| Controller | Mean moves | Sample maximum | Median seconds |
|---|---:|---:|---:|
| oriented_window | 274.8 | 284 | 0.102 |
| Two endpoint representatives | 274.3 | 284 | 0.475 |
| Four endpoint representatives | 273.0 | 284 | 0.754 |

Every search finishes within its budget, and all 60
words replay. The small savings do not justify the
extra host cost as a default. The implementation remains
an experimental library, not a registered CLI mode.

There is a concrete reason not to retain only shortest
children. Sort initial order (5,2,4,3,8,6,7,1,0) with
a four/five split. The left parked child costs eight
moves and the final merge costs nine. One right parked
child costs 13; the resulting 30-move concatenation
cancels one inverse pair, leaving 28. Another right
child costs 15; its 32-move concatenation cancels three
pairs, leaving 26. All endpoints and protected bases
are identical. This is a local composition example,
not a claim that 26 is the global optimum. The
[portfolio study](research/fourth-portfolio.md) gives
the complete words and their independent replay.

These experiments close the planned campaign. Improving
the recurrence's finite search choices alone cannot
improve its leading coefficient. The tested portfolios
buy few moves, and stronger lower-bound discovery remains
the main mathematical opportunity. The
[release summary](research/fourth-campaign-summary.md)
records the stopping decision and reproduction checks.

### 6.8. A light endpoint follow-up

One subsequent pilot changes only portfolio selection.
After retaining the shortest representative, rank others
by length minus twice the larger initial or final run
of identical moves. This estimates possible cancellation;
it is not a dominance rule. The default selection policy
and full-plan fallback are unchanged.

On ten fresh 52-card targets, seed 2026100406, the
width-four boundary policy averages 271.2 moves versus
273.4 for shortest selection and 274.2 for oriented_window.
It wins seven comparisons against shortest and ties three.
Sample maxima are 284,288,288; median planning times are
0.807,0.760,0.102 seconds respectively. All 30 plans
replay, with no timeouts. This small result supports an
optional research policy, not a default change or a
population claim. No further search follows this pilot.
See [the endpoint follow-up](research/fifth-endpoints.md).

## 7. Controller complexity

The decision problem asks whether d(pi)<=K for an
explicit permutation and a binary move budget, with n
variable. It is in NP: radix supplies a universal
polynomial upper bound on certificate length. No
NP-hardness theorem for these exact rules is established
here. Nearby stack-network results change the number
of buffers, connectivity, input/output, or cost model
[9][felsner], [10][koenig], [11][mihalak].

An NP-hardness proof would already use polynomially
bounded numerical parameters and would exclude an FPTAS
unless P=NP: take approximation error below the inverse
polynomial upper bound and use integrality. It would
not by itself exclude a PTAS. That requires a separate
constant relative gap or approximation-preserving
reduction.

There is a positive parameterized result. An excursion
is a departure from D and the next return of that card.
For a proposed chronological excursion schedule, simulate
D alone, checking every departure and the final order.
Form the graph whose vertices are excursions and whose
edges join intervals with interleaving endpoints.

**Lemma 15.** Such a schedule is realizable if and only
if its D simulation is legal and its interval-crossing
graph is bipartite.

*Proof.* Two crossing intervals cannot use the same
side: the later departure would block the earlier return.
Conversely, color a bipartite graph by A and B. Intervals
on either side are nested or disjoint, so every return
is exposed. The D check supplies the remaining legality
conditions. ∎

**Theorem 16.** After suffix trimming to m active cards,
the decision problem is in XP for the parameter
$r=\lfloor(K-2m)/2\rfloor$, when K>=2m.

*Proof.* There are at most r exceptional cards making
more than one excursion. Together they contribute at
most 4r departure/return events. All other cards depart
in initial order and then return in reverse target
order; their projected event sequence is fixed.

Enumerate exceptional identities, allocations of extra
excursions, event orders, and interleavings with the
ordinary sequence. For each, apply Lemma 15 in quadratic
time. Every legal plan within budget appears in this
enumeration, and every accepted schedule supplies a plan.
A crude time bound is $f(r)(m+1)^{5r+2}$, polynomial for
each fixed r. Enumerate all excesses at most r to handle
at-most budgets. ∎

This is XP, not an FPT claim in r. At r=0 it reduces
to the two-increasing-subsequence characterization.
The entire excess-one algorithm agrees with all 873
targets through n=6, checking 2,310,697 schedules.

The second campaign sharpens the exponent to 2r+2.
In every plan, the first departures of all active cards
occur in initial order, followed by their final returns
in reverse target order. No final return can precede
an active card's first departure: it would permanently
block that card. This fixes a 2m-event skeleton.

Every other return is temporary and must be matched by
a later departure. While any such temporary card is on
D, neither kind of skeleton event is possible. Extra
events therefore form balanced, possibly nested blocks
within gaps of the skeleton. For j<=r extra excursions,
enumerate a balanced word with j pairs, m^j opening-card
labels, and at most (2m+1)^j gap placements. Apply the
same D simulation and bipartiteness test. Every legal
plan occurs, yielding time $f(r)(m+1)^{2r+2}$.
Complete checks recover every optimum through n=6 using
r<=4. The [detailed proof](research/parameter-complexity.md)
also explains why nested blocks cannot simply be replaced
by independent side-to-side transfers.

For fixed r, this exponent is constant; for unrestricted
inputs it is not. Membership in P would require one
constant exponent for all inputs. Binary search over K
still takes only O(log m) decision calls, since the
universal upper bound is polynomial. Its difficulty is
the cost of those calls when r grows, not an excessive
number of search steps. Even a linear upper bound 4m
would leave r as large as m. An FPT bound g(r)m^c would
improve the parameter dependence, but would not itself
make all growing-r instances polynomial-time either.

A tempting stronger normal form is false. Target
(2,3,1,4,0) has optimum 12, but requiring D to drain
completely before any return raises the optimum to 14.
An optimal word is

~~~text
DB DB DA DA BD DA DA BD AD AD AD AD
~~~

Its fifth move returns a card while initial card 4 is
still on D. This five-card counterexample rules out a
direct identification with mandatory-loading variants
such as [11][mihalak]; it does not prove hardness of 3SM.

## 8. Discussion

The exact small-n results leave an interesting gap.
Reversal is the unique hardest target for n=2,...,8.
At n=9, the non-reversal target

$$
  (6,8,4,7,2,5,0,3,1)
$$

also requires 32 moves. The maximum still equals
4(n−1), but the set of extremizers has changed.
Neither this observation nor the counting obstruction
at n=212 locates the first failure of (6).

At n=52, the optimal mean lower bound is 166.87917,
whereas direct-output merging averages about 274 in its
window-search holdout. The optimal maximum lies between
204 and 352. The gap
between lower bounds and constructions is substantial.

A remaining direction is to retain richer sets of plans
per interval, distinguished by boundary behavior. The
terminal-run tie heuristic gave no gain; the later
width-four portfolio gave a modest gain at substantial
host cost. Neither exhausts this possibility. A sound
exact dominance rule must account for cancellation
exposing interiors; short boundary summaries alone do
not suffice.

Other approaches were less effective in the experiments.
Natural merging and patience-style distribution supplied
useful intermediate algorithms. A bounded local BFS
rewriting experiment saved no moves on 100 sampled
natural-merge plans and cost about 0.23 seconds per plan.
That result concerns the tested rewrite procedure and
window sizes, not all possible local optimization.

A sharper counting bound would also be useful. The
height-state computation counts many words for the same
permutation. Controlling this multiplicity could improve
the lower bounds without enumerating the full state graph.
The accompanying [research plan](RESEARCH_PLAN.md) gives
priorities, proof obligations, and bounded experiments.

## 9. Conclusion

Exact small-deck plans can be composed into a practical
general algorithm because stack bases remain protected.
Oriented central/side-output merging gives
$2n\log_3 n+O(n)$ moves and a verified 352-move
upper bound at n=52. The fast controller averages 284.72
moves on 100 fresh targets; wider split search gives
273.8 on those same targets.
Conditional excursion constraints improve certified
instance bounds, though not the exact uniform-mean
bound of 166.87917.
The leading coefficient is optimal for the stated
additive recurrence, and the final bounded experiments
support retaining the current practical controllers.

The asymptotic order is settled by a matching counting
lower bound. The constants, optimal 52-card costs, and
the first failure of the all-permutations bound
$M_n\le4(n-1)$ remain open. The reversal construction
is optimal at exactly 4(n−1) moves for every n.

## Appendix A. Artifacts and reproduction

Source, tables, and experimental records accompany this
paper [6][artifacts]. The runtime solver uses only the Python
standard library. A C++20 compiler is needed to regenerate
exact tables or run target-specific exact search.
Matplotlib is used only
to regenerate the paper's figures.

The test suite has 67 test groups, including replay of
all 46,233 stored exact plans, optimal-distance checks,
protected-base cases, recursive composition, arbitrary
initial orders, and enumeration of the small
Fisher–Yates choice space.
With third-party site packages disabled, all tests still
pass except two optional SciPy discovery tests, which
are skipped. Exact rational verification needs no SciPy.
The campaign adds parked endpoint and protected-base
checks, an independent distance-DAG check, and conditional
bound tests against complete small-instance tables.

~~~sh
python3 shuffle.py --n 52 --summary
python3 shuffle.py --n 52 --algorithm thorough --summary
python3 shuffle.py --n 52 --algorithm parked --summary
python3 shuffle.py --n 52 --algorithm oriented --summary
python3 shuffle.py --n 52 --algorithm oriented_window --summary
python3 shuffle.py --n 512 --algorithm ternary --summary
python3 -m unittest -v
python3 benchmark.py --n 52 --samples 1000 \
  --algorithms radix runs_radix_flexible patience \
  hybrid fast recommended \
  --output results/comparison-52.json
~~~

The core construction is in
[hybrid_merge.py](hybrid_merge.py), split selection in
[hybrid_variants.py](hybrid_variants.py), and controller
choices in [solver.py](solver.py). Exact lookup is in
[exact_solver.py](exact_solver.py).

The principal data are
[comparison-52.json](results/comparison-52.json),
[thorough-52.json](results/thorough-52.json),
[structures-52.json](results/structures-52.json), and
[scaling.json](results/scaling.json). Full exact distances
are in the files results/exact-n1.json through
results/exact-n9.json.

The finite upper-bound calculation is recorded in
[hybrid-bounds.json](results/hybrid-bounds.json).
The exact integer counts and rational lower bound are in
[lower-bounds.json](results/lower-bounds.json).
The structural mean bound and exhaustive checks are in
[structural-lower-bound-52.json]
(results/structural-lower-bound-52.json), reproduced by:

~~~sh
python3 tools/structural_bound.py --n 52 --check-through 9
~~~

[README.md](README.md) gives the remaining reproduction
commands.

Figures are original diagrams of the stated constructions.
Their generator is
[report_figures.py](tools/report_figures.py).
[Figure 1 as PNG](figures/3sm-machine.png) and
[Figure 2 as PNG](figures/3sm-merge.png) are supplied for
viewers that do not display embedded SVG.

The executed campaign is indexed in
[campaign-summary.md](research/campaign-summary.md).
It links the conditional certificates, exact searches,
parked endpoint tables, holdout measurements, and
complexity proofs. Runtime code remains standard-library
Python; SciPy is needed only for the experimental LP
and MILP scripts, not for the integer conditional bound.
The [second-campaign summary](research/second-campaign-summary.md)
indexes direct-output merging, group obstructions, residual
certificates, the improved parameterized algorithm, and
their independent reviews and fresh holdout records.
The [third campaign](research/third-campaign-summary.md)
adds ternary recursion, residual cost partitioning,
increasing-pair witnesses, statistical coverage, and
portable rational lower-bound certificates.
The [fourth campaign](research/fourth-campaign-summary.md)
completes the recurrence analysis, tests certificates
on fresh targets, and records the endpoint-portfolio
experiment and final release checks.

## Bibliography

[1] Robert E. Tarjan.
“Sorting Using Networks of Queues and Stacks.”
*Journal of the ACM* **19**(2), 341–346, 1972.
[doi:10.1145/321694.321704][tarjan].

[2] Michael Albert and Mireille Bousquet-Mélou.
“Permutations sortable by two stacks in parallel and
quarter plane walks.”
*Discrete Mathematics & Theoretical Computer Science
Proceedings*, vol. AT, FPSAC 2014, 585–596, 2014.
[doi:10.46298/dmtcs.2425][albert].
[Publisher's full text](https://dmtcs.episciences.org/2425/pdf).

[3] Donald E. Knuth.
*The Art of Computer Programming, Volume 3:
Sorting and Searching*. Second edition.
Addison-Wesley, Reading, Massachusetts, 1998.
[Author's bibliographic record][knuth].

[4] David Aldous and Persi Diaconis.
“Longest Increasing Subsequences: From Patience Sorting
to the Baik–Deift–Johansson Theorem.”
*Bulletin of the American Mathematical Society*
**36**(4), 413–432, 1999.
[doi:10.1090/S0273-0979-99-00796-X][aldous-diaconis].

[5] Richard Durstenfeld.
“Algorithm 235: Random permutation.”
*Communications of the ACM* **7**(7), 420, 1964.
[doi:10.1145/364520.364540][durstenfeld].

[6] *3SM computational artifacts*.
Source code, exact-plan tables, and experimental data
accompanying this report. Local distribution, 2026.
[Artifact documentation](README.md).

[7] Curtis Greene.
“An Extension of Schensted's Theorem.”
*Advances in Mathematics* **14**(2), 254–265, 1974.
[Author's bibliographic record][greene].

[8] Mark Dukes and Andrew Mullins.
“A Bump Statistic on Permutations Resulting from the
Robinson–Schensted Correspondence.”
*Annals of Combinatorics* **29**, 375–394, 2025.
Theorems 3 and 4 recall Greene's theorem and the
hook-length formula.
[doi:10.1007/s00026-024-00708-z][dukes].

[9] Stefan Felsner and Martin Pergel.
“The Complexity of Sorting with Networks of Stacks
and Queues.” *ESA 2008*, 417–429, 2008.
[Author manuscript][felsner].

[10] Felix G. König and Marco E. Lübbecke.
“Sorting with Complete Networks of Stacks.”
*ISAAC 2008*, LNCS **5369**, 895–906, 2008.
[Author manuscript][koenig].

[11] Matúš Mihalák and Marc Pont.
“On Sorting with a Network of Two Stacks.”
*ATMOS 2019*, OASIcs **75**, 3:1–3:12, 2019.
[doi:10.4230/OASIcs.ATMOS.2019.3][mihalak].

[12] C. J. Clopper and E. S. Pearson.
“The Use of Confidence or Fiducial Limits Illustrated
in the Case of the Binomial.”
*Biometrika* **26**(4), 404–413, 1934.
[doi:10.1093/biomet/26.4.404][clopper-pearson].

[13] Wassily Hoeffding.
“Probability Inequalities for Sums of Bounded Random
Variables.” *Journal of the American Statistical
Association* **58**(301), 13–30, 1963.
[doi:10.1080/01621459.1963.10500830][hoeffding].

[clopper-pearson]: https://doi.org/10.1093/biomet/26.4.404
[hoeffding]: https://doi.org/10.1080/01621459.1963.10500830

[felsner]: https://page.math.tu-berlin.de/~felsner/Paper/sqsort.pdf
[koenig]: https://or.rwth-aachen.de/files/research/publications/stack-sorting.pdf
[mihalak]: https://doi.org/10.4230/OASIcs.ATMOS.2019.3

[greene]: https://scholarship.haverford.edu/mathematics_facpubs/172/
[dukes]: https://doi.org/10.1007/s00026-024-00708-z
[knuth]: https://cs.stanford.edu/~knuth/taocp.html
[aldous-diaconis]: https://doi.org/10.1090/S0273-0979-99-00796-X
[tarjan]: https://doi.org/10.1145/321694.321704
[albert]: https://doi.org/10.46298/dmtcs.2425
[durstenfeld]: https://doi.org/10.1145/364520.364540
[artifacts]: README.md

[machine-figure]: figures/3sm-machine.svg
[merge-figure]: figures/3sm-merge.svg
