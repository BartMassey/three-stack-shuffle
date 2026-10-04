# Independent oriented-merge review

4 October 2026. Read-only review of oriented_merge.py
and test_oriented_merge.py, including the leaf and join
implementations they call. No correctness defect found
in the reviewed recursive construction.

## Endpoint meaning and legality

The central endpoint U realizes the requested order on
D, top to bottom. The parked endpoint P realizes its
reverse on the selected side, top to bottom. This is
consistent with the leaf library and endpoint tests.

For central output, park the first a cards on A and
the next b cards on B, each in reverse requested order.
Returning the largest remaining rank first builds the
requested central order. The merge costs exactly n,
giving

    U(n) <= P(a)+P(b)+n.

For output on A, park the first a cards on B using the
opposite requested orientation. B now exposes the least
remaining rank first. Sort the other b cards on D in
the requested central orientation, also exposing their
least rank first. Repeatedly choose the smaller exposed
rank. A left choice uses BD,DA and a right choice uses
DA. These operations preserve both remaining input
orders and reverse the merged order onto A. Thus

    P(n) <= P(a)+U(b)+2a+b.

Reversing the rank comparison handles the other
orientation by the same proof. Swapping A and B handles
the other parked endpoint.

Every recursive word is legal with empty temporary
side stacks. Such a word remains legal above arbitrary
protected bases: its standalone stack heights never
underflow, so it never pops a base card. This inductive
property permits the remaining initial segment and the
already parked sibling to serve as protected bases.

Adjacent inverse cancellation in _join preserves the
full machine state and cannot expose a base card: a
cancelled move pair has no effect on the stack state,
and deleting it leaves every later operation in its
original state.

The optional fallback P(n)<=U(n)+n-2 is valid for a
nonempty central word. Reflect it if necessary so its
last return is AD; the first parking move DA cancels
that return. An empty central word needs only n parking
moves. For the recursive sizes n>8 the numerical bound
U(n)+n-2 exceeds n, so this exception is covered.

The independently read recurrence computes

    U(52)=352, central split 23+29;
    P(52)=368, parked split 8+44.

The implementation includes each bound-minimizing split
alongside its local split window. Its returned plans
therefore inherit these numerical upper bounds for
the default recursive size cutoff.

## Leading constant for every size

Use only balanced splits a=floor(n/2), b=ceil(n/2),
with singleton bases U(1)=0 and P(1)=1. The two
recurrences above already prove a universal construction
with leading move constant 4/3.

To handle odd sizes without a powers-of-two assumption,
define adjusted costs

    V_U(n)=U(n),
    V_P(n)=P(n)-n/3.

The central recurrence becomes

    V_U(n) <= V_P(a)+V_P(b)+4n/3.

The parked recurrence becomes

    V_P(n) <= V_P(a)+V_U(b)+4n/3+2(a-b)/3
           <= V_P(a)+V_U(b)+4n/3.

Both endpoint types thus have an adjusted toll at most
4n/3 at every recursive node. The balanced tree has
height ceil(log2 n). At each depth its active block
sizes sum to at most n, and its n singleton leaves
each contribute at most 2/3 in adjusted cost. Hence

    U(n) <= (4/3)n ceil(log2 n)+(2/3)n,
    P(n) <= (4/3)n ceil(log2 n)+n.

In particular both are (4/3)n log2 n+O(n) for all n.
Exact leaves and optimized splits can only improve the
construction bounds.

The reviewed final executable uses balanced oriented
recursion above its default max_n=64 cutoff. It retains
the balanced candidate within smaller optimized calls.
Its exact leaves improve the singleton construction,
so the default implementation inherits the all-size
4/3 leading constant. The earlier hybrid fallback was
replaced before this review concluded.

The optimized numerical recurrence likewise uses
balanced splits above 64. Its stated numerical bounds
apply to the default search cutoff. A caller lowering
max_n below 64 can omit a numerically optimal split;
the all-size asymptotic bound still holds, but the same
optimized finite-size bound need not follow.

## Independent checks

All three test_oriented_merge unittest cases passed.
An additional deterministic seed 5123104 generated five
independent initial/target pairs at each of sizes
0,1,8,9,10,13,17,26,52,64,65. Each was checked for all
three endpoints with window=1: 165 cases total.

These extra cases used nonconsecutive card labels and
distinct protected sentinel cards under all three stacks.
Before every individual move, the source top was checked
to exclude every sentinel. Final endpoint order and
unchanged bases were verified, along with the published
recurrence bound for sizes through 64. All passed in
approximately 0.61 seconds, including the fallback cases
at size 65.

After the fallback changed, all three expanded unit
tests passed, including sizes 65,128,257. Nine further
guarded endpoint cases at those sizes passed with
independent random initial and target orders. The
displayed all-size numerical inequalities were also
checked against oriented_bounds for every n<=10,000;
all passed.

These finite checks support the implementation review;
the inductive arguments establish the construction.
