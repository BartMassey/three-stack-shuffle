# Structural bounds for arbitrary residual states

4 October 2026. This extends the endpoint theorem in
REPORT section 5.4 to states with cards already on A or B.
The runtime Python solver is unchanged. The research C++
search has an optional switch; its default is unchanged.

## Conventions and mandatory charges

All sequences below are top to bottom. Relabel each card
by its zero-based position in the target, so the goal is
all cards on D in increasing order. Let C be the longest
bottom suffix of current D already equal to the goal's
bottom suffix. Delete C when forming the active state.
Write a,b,d for the active counts on A,B,D, respectively.

Every initial side card makes an odd number of transfers,
at least one. Every active initial D card makes a positive
even number, at least two. To see the latter, if it never
leaves D, no card initially below it can leave either.
Their entire unchanged bottom segment must be a goal
suffix, contradicting the maximality of C.

Thus the mandatory charge is

    M = a + b + 2d.

Cards in C are assigned zero. They may move in the actual
plan; ignoring their cost can only weaken a lower bound.
This proof does not assume that an optimum preserves C.

## Minimal-cost cards have a joint order constraint

Call an active card ordinary if it makes exactly its
mandatory number of moves: one for an initial side card,
two for an initial D card. Each other active card costs
at least two extra moves, by the individual parity rule.

Project any complete physical plan onto its ordinary
cards, deleting all other cards and their transfers.
Deletion preserves legality. The resulting plan contains
one departure and return per initial D card, and one
return per initial side card.

All initial D departures precede every return. A returned
ordinary card can never leave D again, so it would block
every ordinary D card not yet departed. Initial D cards
depart in their initial top-to-bottom order.

The projected plan is feasible exactly when the following
conditions hold for its chosen cards and side assignments:

1. The initial D cards assigned to each side form an
   increasing subsequence in target ranks.
2. The chosen initial cards on each side form a decreasing
   subsequence in target ranks, read top to bottom.
3. Each chosen D card assigned to a side has larger target
   rank than every chosen initial card on that side.

For necessity, after all departures the D cards occupy
the top of their assigned side, in reversed departure
order, above its retained initial cards. Each side must
return its cards in decreasing target-rank order.
This gives all three conditions.

For sufficiency, depart all selected D cards in order,
using their assigned sides. Each resulting side is
decreasing: its reversed D subsequence, followed by its
decreasing retained initial cards, with the strict rank
separation at their boundary. Repeatedly return the
larger exposed rank. The final D is increasing.

Let K be the maximum total number of chosen active cards
meeting these conditions. The actual ordinary set has
size at most K, hence the residual structural bound is

    R(A,D,B) = M + 2(a+b+d-K).

This is an admissible arbitrary-state lower bound.
Its parity already equals a+b modulo two, the parity of
every remaining physical path. Taking its maximum with
the existing PDB-plus-omitted-charge bound remains sound.
No sum of these overlapping bounds is used.

When A and B are empty, K is exactly I2 of the active D
order in target ranks, recovering the endpoint theorem.
When D is empty, K is the sum of the two longest decreasing
subsequence lengths on the sides.

Treating arbitrary side cards as initial D cards would
be unsound. For A=(2,1,0), D=B=(), and goal=(0,1,2),
the residual optimum is three returns. Applying the
endpoint reversal bound to A's sequence would claim eight.

## Computing K

For each side S and threshold t, let L(S,t) be the longest
decreasing subsequence of S using only ranks at most t.
Use thresholds -1 and every rank present on S. Threshold
-1 represents choosing no initial side cards.

For thresholds tA,tB, run a two-increasing-subsequence DP
over active D, initializing the two tails to tA,tB. An
entry can be omitted or appended to either chain when
its rank exceeds that chain's tail. If F(D;tA,tB) is the
maximum retained count, then

    K = max[L(A,tA)+L(B,tB)+F(D;tA,tB)].

Every term describes a feasible chosen set. Conversely,
the maximum chosen initial rank on each nonempty side
provides one of the enumerated thresholds for any feasible
set, so its count cannot exceed this maximum.

The simpler independent relaxation replaces K by

    LDS(A) + I2(D) + LDS(B).

It is admissible and no stronger than R. It omits only
the constraints between initial side cards and D cards.

The implementations combine all threshold pairs into a
single weighted DP. Initialize the tail-pair entry
(tA,tB) with score L(A,tA)+L(B,tB), then scan D once,
adding one to the score whenever a card extends a chain.
Candidates with equal tail pairs have identical possible
future extensions, so retaining their largest score is
exact. The final largest score is K.

Python uses sparse tail-pair maps. C++ uses bounded
standard arrays with checked indexing. For the search's
supported n<=20, its 625 entries cover all (n+1)^2 tail
pairs. Its worst-case time is O(n^3): side-threshold LDS
preprocessing and a single O(d*n^2) scan of D.

## Exhaustive validation

Independent tuple-state BFS checks every state through
n=6: 23,115 states in total. Identity goals cover arbitrary
targets by rank relabeling. No residual bound exceeds an
exact distance. No tested edge violates consistency.
The latter is a finite observation, not a general proof.

The C++ function agrees with the Python function on every
one of those states and independently passes the exact
distance comparison. The raw C++ validation file uses the
existing permutation-rank/cut encoding, and Python looks
up each independently generated tuple state in it.

At n=6, averaged uniformly over all 20,160 full states:

| Bound | Mean |
|---|---:|
| Mandatory moves | 7.706746 |
| Independent structural | 10.697520 |
| Joint structural R | 11.337401 |
| All four-card PDBs plus charges | 11.870437 |
| Maximum of PDB and R | 12.039583 |
| Exact distance | 12.899504 |

R improves the independent structural bound on 5,834
states and the four-card PDB bound on 1,674 states.
Its latter gains are two moves on 1,643 states and four
on 31 states. R is exact on 9,466 states. A full six-card
PDB is already exact, so there can be no improvement over
that database; the smaller-pattern comparison tests the
same omitted-card mechanism used above eight cards.

Example: A=(2), D=(0,4,3), B=(1), goal=(0,1,2,3,4).
Mandatory and four-card PDB bounds are eight; the joint
bound is ten, equal to the exact distance.

## Eight-card PDB comparisons above the database size

The existing nearly reversed n=14,16,18 targets were used
without changing their frozen results. Patterns are chosen
exactly as in the existing search: score all eight-card
subsets initially, keep the top 32, break ties by mask.
The new bound is enabled only after that choice, so it
does not change the selected patterns.

For each target, compare all states along its existing
verified upper word and 128 deterministic random walks
from the initial state. The latter avoid immediate inverse
moves and use seed 20261004. They are heuristic samples,
not draws from a specified uniform distribution.

| n | Initial PDB | Initial R | Mean witness gain |
|---|---:|---:|---:|
| 14 | 40 | 48 | 3.294118 |
| 16 | 44 | 56 | 4.576271 |
| 18 | 48 | 64 | 6.477612 |

The gain means max(PDB,R)-PDB, including zero gains.
On random-walk states, mean gains are 6.109375,9.656250,
13.625000, respectively. R wins on 126,128,128 of the
128 states. Every witness-state bound remains below its
verified remaining path cost.

Python joint evaluation averages approximately 87,159,
173 microseconds per random-walk state, versus 136,183,
137 microseconds for the 32-pattern PDB lookup. These are
small-sample host timings, not C++ search timing claims.

## Optional C++ search integration

The final optional argument RESIDUAL_STRUCTURAL is 0 or 1.
Omitting it preserves the original heuristic. Enabling it
takes the maximum with R after evaluating all fixed PDBs.
Parity rounding, the exact transposition key, inverse-move
pruning, threshold accounting, and interruption accounting
retain their existing definitions.

Pattern selection occurs before enabling R, avoiding the
unintended change that a dominating common initial bound
would otherwise make to the subset ranking.

With ten seconds of search allowed for each paired run:

| n | Old interval | New interval | New search nodes |
|---|---|---|---:|
| 14 | [48,50] | [50,50] | 671,869 |
| 16 | [56,58] | [56,58] | 1,347,584 |
| 18 | [64,66] | [64,66] | 1,236,992 |

The n=14 run completely exhausts threshold 48 in about
4.48 seconds after setup. Combined with the independently
replayed existing 50-move word, this proves optimum 50.
It does not return a new shorter word: its proof comes
from the exhausted lower threshold and retained upper.
The baseline exhausts no threshold in its ten seconds.

Both n=16 and n=18 remain interrupted at their initial
thresholds in these ten-second runs. The new
heuristic is substantially stronger on measured states,
but these runs still establish a practical search limit.
No claim is made that fewer visited nodes means less work
unless the threshold actually completes.

The n=14 completion justified one further bounded gate:
give n=16 the original campaign's 45-second allowance.
The optimized run completely exhausts threshold 56 after
5,924,262 nodes in approximately 43.66 search seconds
(44.05 seconds including setup). It retains one million
transposition entries and reports 364 transposition hits.
The existing 58-transfer upper word was explicitly
replayed again. Therefore this target's exact cost is 58,
improving its frozen interval [56,58] to [58,58].

The certificate is residual-search-n16-45.json; both the
old campaign result and new ten-second pilot are preserved.
The old 45-second baseline had exhausted no threshold.
The new run completed close to its deadline, so no n=18
expansion was undertaken. Its interval remains [64,66].
These are target-specific optima, not maxima at n=14 or
n=16; reversal still costs 4(n-1) at those sizes.

An earlier unoptimized threshold-pair implementation is
preserved in residual-search-pilot-naive.json. It times
out on all three cases and is superseded by the weighted
DP implementation. Frozen campaign results are preserved.

## Reproduction and artifacts

```sh
python tools/residual_bounds.py --through 6 \
  --output results/residual-validation.json
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic \
  tools/residual_bounds_pdb.cpp -o /tmp/residual_bounds_pdb
python tools/residual_bounds.py \
  --cpp-bound-binary /tmp/residual_bounds_pdb \
  --output results/residual-cpp-validation.json
/tmp/residual_bounds_pdb /tmp/residual-bounds-pdb8.bin
python tools/residual_bounds.py \
  --pdb-bytes /tmp/residual-bounds-pdb8.bin \
  --output results/residual-campaign-samples.json
g++ -O3 -std=c++20 -Wall -Wextra -Wpedantic \
  tools/campaign_exact.cpp -o /tmp/campaign_exact_residual
python tools/residual_bounds.py \
  --search-binary /tmp/campaign_exact_residual \
  --seconds 10 \
  --output results/residual-search-pilot.json
python tools/residual_bounds.py \
  --search-binary /tmp/campaign_exact_residual \
  --search-n 16 --seconds 45 \
  --output results/residual-search-n16-45.json
```

The Python pure functions residual_bound and
independent_bound accept A,D,B,target sequences, all
top to bottom, and relabel arbitrary distinct card IDs.
Results retain explicit states, thresholds, interruption
flags, and replayed upper words where appropriate.
No full-state BFS above eight cards is used.
