# RegIR Expression

An expression over $(\Sigma, \Lambda)$ is a triple
$(D, D_{prefix}, D_{suffix})$
where
- $D$ is an *alternation* over $(\Sigma, \Lambda)$ (defined below) defining the expression's root.
- $D_{prefix}$ is an alternation over $(\Sigma, \Lambda)$ 
  defining the expression's prefix.
- $D_{suffix}$ is an alternation over $(\Sigma, \Lambda)$ defining the
  expression's suffix.

A capture group label (of $\Lambda$) can appear at most once in an expression.
The set of all expressions over $(\Sigma, \Lambda)$ is written
$Ex(\Sigma, \Lambda)$.

## Alternation

An *alternation* over $(\Sigma, \Lambda)$ is a sequence $(c_1, ..., c_n)$ where
for each $i$, $1 \leq i \leq n$, $c_i$ is a *concatenation* over
$(\Sigma, \Lambda)$.
The set of all alternations over $(\Sigma, \Lambda)$ is written
$Alt(\Sigma, \Lambda)$.

## Concatenation

An *concatenation* over $(\Sigma, \Lambda)$ is a sequence $(a_1, ..., a_n)$
where for each $i$, $1 \leq i \leq n$,
$a_i$ is an *atom* over $(\Sigma, \Lambda)$.
The set of all concatenation over $(\Sigma, \Lambda)$ is written
$Cat(\Sigma, \Lambda)$.

## Atom

An *atom* over $(\Sigma, \Lambda)$ is either:
- A *token set*, item of $P(\Sigma)$;
- A *repetition* over $(\Sigma, \Lambda)$;
- An alternation over $(\Sigma, \Lambda)$;
- A *capture group* over $(\Sigma, \Lambda)$.
The set of all atoms over $(\Sigma, \Lambda)$ is written
$Atm(\Sigma, \Lambda)$.

## Repetition

A repetition over $(\Sigma, \Lambda)$ is a triple
$(d, min, max) \in Alt(\Sigma, \Lambda)$
where:
- $d$ is an alternation over $(\Sigma, \Lambda)$;
- $min \in \N$ is the minimum number of repetitions,
- $max \in \N \cup \set{\infty}$ is the maximum number of repetitions,
  $\infty$ denoting the absence of maximum bound.
  If $max \in \N$, then we must have $min \leq max$.

The set of all repetitions over $(\Sigma, \Lambda)$ is written
$Rep(\Sigma, \Lambda)$.

## Capture group

A *capture group* over $(\Sigma, \Lambda)$ is a pair $(\lambda, d) \in \Lambda \times Alt(\Sigma, \Lambda)$ where
- $\lambda$ is a label for the capture group;
- $d$ is an alternation.

The set of all capture groups over $(\Sigma, \Lambda)$ is written
$Cap(\Sigma, \Lambda)$.

## Semantics

The semantics of an expression is a function $S$ mapping an
expression to a 5-uplet $(A, A_{prefix}, A_{suffix}, start, end)$ where
- $A$ is a Tagged NFA recognizing the expression's root.
- $A_{prefix}$ is an NFA recognizing the expression's prefix.
- $A_{suffix}$ is an NFA recognizing the expression's suffix.
- $start$ is a function mapping each capture group label to an associated tag of
  $A$, corresponding to the capture group's start.
- $end$ is a function mapping each capture group label to an associated tag of
  $A$, corresponding to the capture group's end.