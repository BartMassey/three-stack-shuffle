# Source documents

These are the working write-ups that `../PAPER.md` synthesizes. They are kept
for their full proofs, empirical tables, and provenance. **`../PAPER.md` is the
current entry point**; where a source disagrees with it, `PAPER.md` wins.

| file | cost model | status |
|------|-----------|--------|
| `operation-count-theory.md` | operation count | most complete; structural theory, merge sorters, literature placement. Supersedes the `4(n−1)`-diameter idea. |
| `SORTING-BOUNDS.md` | operation count | the merge-family algorithms and their bounds |
| `HEURISTIC-BOUNDS.md` | operation count | the admissible-heuristic / IDA* program; **its `M(n)=4(n−1)` conjecture is retracted** (see `operation-count-theory.md` §7 and `PAPER.md` Part III) |
| `cycle-model-theory.md` | whole cycle | the permutation-distance theory |
| `original-notes.md` | whole cycle | the project's seed notes, with the FIFO/LIFO/flip pitfalls |

Two distinct cost models of the same machine appear here — operation count vs
whole cycles — and their numbers are on different scales. Do not mix them.
