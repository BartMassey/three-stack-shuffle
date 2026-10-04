# Independent residual-bound admissibility review

4 October 2026. Reviewed residual-bounds.md, the Python
formula in tools/residual_bounds.py, and its C++ formula
in tools/campaign_exact.cpp. No admissibility defect
found. This review does not claim a consistency theorem.

The suffix argument is valid for arbitrary side contents.
If an active D card never moves, neither can any card
below it: exposing any such card would require removing
the unmoved card first. This unchanged bottom block must
therefore already match the final target suffix. Removing
the longest correct suffix leaves only D cards forced
to depart and return, each costing at least two moves.
Every initial side card must reach D, costing at least
one. Additional moves come in pairs per card.

Projecting onto minimum-cost active cards deletes the
suffix and all nonordinary cards. Projection preserves
the legality of surviving moves. Each projected D card
departs once and returns once; each projected initial
side card returns once. Every return is permanent, so
none can precede any projected initial D departure.
Otherwise the permanent returned card would block the
still-undeparted D card below it.

Consequently the selected D cards placed on a given side
all lie above its selected original side cards. Since
final returns occur in decreasing target-rank order,
each side's selected D cards form an increasing sequence
in their initial D order, its original cards form a
decreasing sequence in their initial side order, and
every selected D rank exceeds every selected original
side rank. These are precisely the claimed coupling
constraints; no independent-side assumption is used.

The threshold maximization is exact for this relaxation.
Every enumerated term selects side ranks at most its
thresholds and D chain ranks strictly above them.
Conversely, choose thresholds equal to the largest
selected initial rank on each side, or -1 for an empty
selection. Every feasible ordinary subset is represented.
Thus its maximum bounds the actual ordinary-card count
from above, making the resulting move bound admissible.

Python's sparse tail-pair DP implements that maximization.
The C++ version uses the same ranks shifted by one,
retains omission transitions by copying the DP table,
and uses strict append inequalities. Its side-subsequence
DP uses zero entries for threshold-excluded cards; these
can contribute only a length-one restart, which is already
allowed. Checked indexing and the documented supported
deck-size limit cover the DP arrays.

For an independent computational check, explicitly
enumerated all selected side subsets and all three-way
assignments of D cards: omitted, assigned to A, assigned
to B. Direct order and threshold comparisons give the
same maximum as the Python DP on every full state
through n=5, totaling 2,955 states. The resulting move
bound also stayed below exact tuple-state BFS distance
on every state. This separate check took about 0.40
seconds and passed without a mismatch.

The existing recorded through-n=6 Python and C++ checks
cover 23,115 states, reporting no inadmissible values or
cross-language formula mismatches. Those artifacts were
inspected, not regenerated in this review. The independent
subset enumeration above additionally checks the DP's
optimization semantics instead of comparing two versions
of the same recurrence alone.
