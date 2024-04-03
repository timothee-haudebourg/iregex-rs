# Submatching

Submatching, or submatch extraction, consists in finding the *boundaries* of
capture groups in a given match.
Once again, boundaries can be ambiguous when repetitions are involved.
However contrarily to rootmatching,
mainstream regex engines do not agree here on the disambiguation strategy
for submatching.
Most engine prefer to capture the right-most group match,
whereas POSIX's engine uses complex backtracking rules to select group matches.

We chose the most common strategy (right-most group match capture) over the
POSIX strategy, as it also the most simple strategy to formalize and implement
using linear-time algorithms.

For any compiled expression $(A, A_{prefix}, A_{suffix}, start, end)$,
the boundaries of a capture group $\lambda$ for a word $w$ is given by
$[i, j[$ where
- $i$ is the maximum value such that $(w, i) \in L_{start(\lambda)}(A_{prefix}, A)$ and
- $j$ is the maximum value such that $(w, j) \in L_{end(\lambda)}(A_{prefix}, A)$