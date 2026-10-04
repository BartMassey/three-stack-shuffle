# Permutation Routing and Uniform Shuffling on Three Stacks

*Working paper, 4 October 2026*

## Abstract

We consider a card machine with three stacks arranged
A–D–B. A move transfers the top card of one stack to the
top of an adjacent stack. All cards start and finish on D.
The problem is to realize a specified permutation with
few moves; a uniform shuffle follows by choosing the
target permutation uniformly.

We give a recursive merge algorithm whose leaves use
exhaustively computed optimal plans for at most eight
cards. The algorithm uses O(n log n) moves. Computing
leaves for their actual parked endpoints gives an upper
bound of 410 moves for every 52-card permutation.
The resulting controller averages 318.874 moves on
1,000 held-out targets, with maximum 340 and approximately
104 ms median planning time. Its faster predecessor
averages 322.896 moves on the same targets at 53.5 ms.

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
No new large search was run for this paper.

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

**Lemma 13.** Such a schedule is realizable if and only
if its D simulation is legal and its interval-crossing
graph is bipartite.

*Proof.* Two crossing intervals cannot use the same
side: the later departure would block the earlier return.
Conversely, color a bipartite graph by A and B. Intervals
on either side are nested or disjoint, so every return
is exposed. The D check supplies the remaining legality
conditions. ∎

**Theorem 14.** After suffix trimming to m active cards,
the decision problem is in XP for the parameter
$r=\lfloor(K-2m)/2\rfloor$, when K>=2m.

*Proof.* There are at most r exceptional cards making
more than one excursion. Together they contribute at
most 4r departure/return events. All other cards depart
in initial order and then return in reverse target
order; their projected event sequence is fixed.

Enumerate exceptional identities, allocations of extra
excursions, event orders, and interleavings with the
ordinary sequence. For each, apply Lemma 13 in quadratic
time. Every legal plan within budget appears in this
enumeration, and every accepted schedule supplies a plan.
A crude time bound is $f(r)(m+1)^{5r+2}$, polynomial for
each fixed r. Enumerate all excesses at most r to handle
at-most budgets. ∎

This is XP, not an FPT claim in r. At r=0 it reduces
to the two-increasing-subsequence characterization.
The entire excess-one algorithm agrees with all 873
targets through n=6, checking 2,310,697 schedules.

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
whereas the default's sampled mean is approximately 323.
The optimal maximum lies between 204 and 410. The gap
between lower bounds and constructions is substantial.

A remaining direction is to retain richer sets of plans
per interval, distinguished by boundary behavior. The
tested terminal-run tie heuristic gave no gain, but does
not exhaust this possibility. A sound exact dominance
rule must account for cancellation exposing interiors;
short boundary summaries alone do not suffice.

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
Balanced merging gives O(n log n) moves. Exact parked
leaves, reflection, and cancellation give a verified
410-move upper bound at n=52. The parked controller
averages 318.874 moves on its 1,000-target holdout.
Conditional excursion constraints improve certified
instance bounds, though not the exact uniform-mean
bound of 166.87917.

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

The test suite has 25 test groups, including replay of
all 46,233 stored exact plans, optimal-distance checks,
protected-base cases, recursive composition, arbitrary
initial orders, and enumeration of the small
Fisher–Yates choice space.
The campaign adds parked endpoint and protected-base
checks, an independent distance-DAG check, and conditional
bound tests against complete small-instance tables.

~~~sh
python3 shuffle.py --n 52 --summary
python3 shuffle.py --n 52 --algorithm thorough --summary
python3 shuffle.py --n 52 --algorithm parked --summary
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
