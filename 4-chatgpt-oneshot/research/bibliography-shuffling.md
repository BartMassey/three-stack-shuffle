# Bibliographic sources for permutation generation and sorting

Source-verification notes for the original sorting study.
The [report bibliography](../REPORT.md#bibliography) is the
consolidated list of references cited by the paper.
Additional campaign sources and model caveats appear in
[the stack-source notes](bibliography-stacks.md).

These references support the report's use of random input permutations and
standard sorting background. They do not establish that the particular
three-stack A-D-B routing problem has a known algorithm, nor whether its
approach is novel.

## Random permutations

* R. A. Fisher and F. Yates, *Statistical Tables for Biological,
  Agricultural and Medical Research* (London: Oliver and Boyd, 1938),
  90 pp. [Google Books record and scan](https://books.google.com/books/about/Statistical_Tables_for_Biological_Agricu.html?id=yhZYAAAAMAAJ).
  This identifies the original 1938 book, its authors, publisher, date,
  and extent. The book is an antecedent for the Fisher–Yates procedure;
  cite it as historical provenance, not as the source of the modern
  in-place loop. I did not verify a page in the scan containing the
  paper-and-pencil procedure, so no page number is given here. The
  bibliographic metadata is from Google's digitized-book record rather
  than a currently hosted publisher page.

* Richard Durstenfeld, “Algorithm 235: Random permutation,”
  *Communications of the ACM* 7, no. 7 (July 1964): 420.
  [ACM Digital Library record](https://dl.acm.org/doi/10.1145/364520.364540),
  DOI [10.1145/364520.364540](https://doi.org/10.1145/364520.364540).
  The ACM record is the publisher source for title, author, journal,
  issue, page, and DOI. The article gives the in-place descending-index
  swap procedure (Algorithm 235) and assumes a uniform random variate.
  This is the direct citation for the implementation used to generate
  uniformly random permutations, assuming unbiased bounded draws.

## Sorting background

* Donald E. Knuth, *The Art of Computer Programming*, vol. 3,
  *Sorting and Searching*, 2nd ed. (Reading, MA: Addison-Wesley, 1998),
  xiv + 780 pp. ISBN 0-201-89685-0.
  [Knuth's official TAOCP page](https://cs.stanford.edu/~knuth/taocp.html).
  Knuth's author-maintained page confirms the exact volume title,
  edition, publisher, year, pagination, and ISBN. This is suitable
  general background for classical sorting methods, including radix
  sorting and merging. For a precise method-specific claim, cite the
  relevant section/page from the edition consulted; this note does not
  assign page numbers.

## Scope and wording

The references above cover permutation generation and sorting
primitives. They are not evidence for a published solution or novelty
claim about routing a permutation through the specified three stacks
in A-D-B order. State any such result as the report's own finding and
support it with its proof or experiments.
