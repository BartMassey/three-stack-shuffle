# Sharp leading coefficient for the endpoint recurrence

Fourth campaign, 4 October 2026. The coefficient
`c = 2/log2(3) = 1.261859507142915...` is optimal for
the numeric recurrence defined below, even when every
integer split is available at every recursive size.
Both endpoint costs equal `c n log2(n) + O(n)`.

This closes the restricted recurrence question left in
[the ternary analysis](third-ternary.md). The lower bound
is a subsolution induction, so it permits arbitrary
size-dependent splits and oscillating linear terms. It
does not assume that an optimizer has constant linear
terms in its asymptotic expansion.

It is not a lower bound for the three-stack machine,
actual emitted words, target-dependent recursion, or the
planner after cancellation. The recurrence adds charges;
the planner can cancel moves between its children and
merge operations. Even the frozen leaves below are
certified numeric budgets, not exact worst-case costs
for every endpoint and size.

## Precisely scoped model

There are two models, with and without the optional
reparking charge. Their frozen leaf pairs `(U(k),P(k))`
for `0 <= k <= 64` are exactly the first two components
of `oriented_bounds(k)` at this campaign's start. The
complete integer table is embedded in
[the reproducer](../tools/fourth_recurrence.py) and
[the result](../results/fourth-recurrence.json).
[Tests](../test_fourth_recurrence.py) compare all 65
pairs with the production bounds.

In particular, the first nine pairs are determined by

```text
U(k) = 4 max(0,k-1),                0 <= k <= 8
P(0..8) = (0,1,4,7,12,15,20,23,28).
```

For `9 <= k <= 64`, the leaf table itself was computed
by the all-split recurrence and the credited repark
option below. These are now fixed terminal charges.
Examples include `(U(24),P(24))=(132,132)`,
`(U(52),P(52))=(352,368)`, and
`(U(64),P(64))=(464,468)`.

For every `n > 64`, define numeric equalities

```text
U(n) = min [P(a)+P(n-a)+n]
       1 <= a < n

Q(n) = min [P(a)+U(n-a)+n+a]
       1 <= a < n

P(n) = Q(n)                         without repark
P(n) = min(Q(n), U(n)+n-2)           with repark.
```

Each split has two nonempty, strictly smaller children.
The parked split is ordered: its first child pays the
extra `a`. Central and parked roots choose independently.
Compute `U(n)` before `P(n)`; the repark option therefore
creates no circular definition. There are no zero-size
recursive children, no extra candidate operations, and
no reduction of composed words in this numeric model.

The phrase unreduced recurrence refers to these additive
charges. The optional repark model deliberately includes
the existing fixed two-move cancellation credit, and
the finite leaves already include their fixed savings.
It includes no additional boundary cancellations. The
theorem holds for both variants with these conventions.

The usual safeguard `max(n, U(n)+n-2)` is unnecessary
above 64 in this numeric model. Every frozen `P(k)>=k`
and `U(k)>=0`. Inductively the central candidate is at
least `2n`, the parked candidate at least `n`, and the
repark candidate at least `3n-2`. Thus all subsequent
`P(n)>=n` as well. The empty and singleton cases remain
fixed as `(0,0)` and `(0,1)`; in particular the formula
`U(1)+1-2` is never used.

## A lower potential valid for every split

Let

```text
c = 2/log2(3)
delta = c-1
u = -8
p = u+delta = c-9

F_U(n) = c n log2(n) + u n
F_P(n) = c n log2(n) + p n.
```

Set `F_U(0)=F_P(0)=0` explicitly. The exact inequality
`1<c<4/3` follows from `3<4` and `3^2>2^3`.

All frozen leaves satisfy `U(k)>=0` and `P(k)>=k`.
For `1<=k<=64`, their potential values obey

```text
F_U(k)/k <= 6c-8 < 0
F_P(k)/k <= 7c-9 < 1/3 <= 1.
```

Hence both lower inequalities hold at every leaf,
including the singleton. These deliberately loose
constants give a proof independent of numerical logs.

Write `H(t)=-t log2(t)-(1-t)log2(1-t)` for binary
entropy. If `a+b=n` and `x=a/n`, then

```text
F_P(a)+F_P(b)+n-F_U(n)
    = n[1+delta-c H(x)]
    = c n[1-H(x)] >= 0.
```

The last inequality holds for every central split,
since `H(x)<=1`, with equality only at `x=1/2`.

For the parked split, put `y=a/n`. Its surplus is

```text
F_P(a)+F_U(b)+n+a-F_P(n)
    = n[1+y-(1-y)delta-c H(y)]
    = n[2-c(H(y)+1-y)] >= 0.
```

To see the final inequality without assuming a split,
differentiate `G(y)=H(y)+1-y`:

```text
G'(y) = log2((1-y)/y)-1
G''(y) = -1/[ln(2)y(1-y)] < 0.
```

The unique maximum is at `y=1/3`, where
`G(1/3)=log2(3)`. Thus `c G(y)<=2` for every legal
parked split, with equality only at a third.

The optional repark candidate also preserves the
potential:

```text
F_U(n)+n-2-F_P(n) = (2-c)n-2 > 0,   n > 64.
```

Indeed `2-c>2/3`, so this holds even for every `n>=3`.
The cutoff avoids the small exceptions automatically.

Strong induction now proves the result. Suppose both
costs dominate their potentials at all smaller sizes.
Every central candidate dominates `F_U(n)`, so their
minimum does too. Every split parked candidate then
dominates `F_P(n)`. The just-proved central inequality
at size `n` handles the optional repark candidate as
well. Taking the parked minimum completes the step.
Consequently, for all nonnegative integer sizes,

```text
U(n) >= c n log2(n)-8n
P(n) >= c n log2(n)+(c-9)n,
```

with the zero convention above. This proof bounds every
allowed additive derivation tree, not merely a proposed
constant-linear-term supersolution.

## A matching upper bound for the same frozen leaves

Let `(T_U,T_P)` be `ternary_bounds`, which uses exactly
these leaves, a half split for `U`, and a third split
for `P` above 64. Since these choices occur in the
minima, induction gives `U<=T_U` and `P<=T_P` in both
models. For the variant with repark the extra candidate
can only improve these inequalities.

The finite table satisfies, for every `2<=k<=64`,

```text
U(k) <= P(floor(k/2))+P(ceil(k/2))+k,
P(k) <= 8k.
```

These are exact integer checks in the test suite. They
are needed because a central child that falls below the
cutoff is charged its frozen leaf cost. One cannot simply
replace a fixed leaf by a recursive equality.

For `n>64`, put `a=floor(n/3)`, `b=n-a`,
`d=floor(b/2)`, and `e=b-d`. If `b>64`, use its central
equality; otherwise use the verified leaf inequality.
In either case,

```text
T_P(n) <= T_P(a)+T_P(d)+T_P(e)+2n.
```

The three positive child sizes differ by at most one.
Unroll this ternary inequality only while a node exceeds
64. At each depth, node sizes sum to at most `n`; there
are at most `ceil(log3(n))` internal depths. Each final
leaf contributes at most eight times its size, so

```text
T_P(n) <= 2n ceil(log3(n))+8n
       <= c n log2(n)+10n,                 n >= 1.
```

For a central root above 64, insert this parked upper
bound at its two balanced children. Their entropy term
is nonnegative, giving

```text
T_U(n) <= c n log2(n)+11n.
```

The same bound holds directly for central leaves since
their table also satisfies `U(k)<=8k`.

Combining upper and lower bounds yields

```text
U(n) = (2/log2(3)) n log2(n) + O(n)
P(n) = (2/log2(3)) n log2(n) + O(n).
```

In particular both normalized limits equal `2/log2(3)`.
No fixed smaller leading coefficient with an `O(n)`
remainder can bound this optimized recurrence from above.
Allowing nonconstant bounded linear corrections or
size-dependent splits cannot evade the induction.

## Finite checks and reproducibility

The theorem uses `u=-8`, but a much tighter valid shift
is defined exactly by the finite minimum

```text
u_* = min over 1<=k<=64 of
      {U(k)/k-c log2(k),
       P(k)/k-c log2(k)-delta}.
```

With `p_*=u_*+delta`, the same induction applies
unchanged. The definition proves the leaf inequalities
without a floating-point assumption. Numerical evaluation
finds `u_*` approximately `-0.5474380285716607`, attained
by the parked leaf at `k=24`. The numerical minimizer is
an observation; identifying it is not needed for the
all-size theorem.

The reproducer evaluates every integer split through
4096 in each model. It checks both potentials and the
implemented ternary upper bound at every size. Both
models have identical cost arrays throughout this range;
this finite observation is not asserted for all sizes.

| n | Minimum U | Ternary U | Minimum P | Ternary P |
|---|---:|---:|---:|---:|
| 65 | 474 | 474 | 477 | 477 |
| 128 | 1064 | 1064 | 1124 | 1126 |
| 512 | 5608 | 5632 | 5720 | 5722 |
| 4096 | 59992 | 60352 | 61504 | 61532 |

The result stores the exact frozen table, selected
sizes, numerical lower values, and a SHA-256 digest of
each complete pair of integer cost arrays. It excludes
timings and random samples, so repeated runs reproduce
the same artifact. The dynamic program takes `O(N^2)`
time and `O(N)` space and uses exact integer arithmetic
for all recurrence costs. Floating point is confined
to reporting and checking analytic potentials.

```sh
python3 -m unittest test_fourth_recurrence -v
python3 tools/fourth_recurrence.py --maximum 4096 \
  --output results/fourth-recurrence.json
```

Four tests passed. The full numeric check through 4096
passed for both variants. None of these finite checks
is substituted for the all-size induction, and none
provides a lower bound on canceled machine words.
