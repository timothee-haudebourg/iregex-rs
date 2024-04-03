# Tagged Finite Automata (TFA)

The language of a tagged automaton w.r.t to a given tag $t$, written $L_t(A)$
is a set of pairs $(w, n)$ where
$w$ is a word of $L(A)$ and
$n \in \N$ is a position such that there exists a path in $A$,
$$q_1 \rightarrow_{l_1} ... \rightarrow_{l_{k-1}} q_k \rightarrow_{\epsilon} q_{k+1} \rightarrow_{l_{k+1}} ... \rightarrow_{l_{f-1}} q_f$$
recognizing $w$ such that
$q_n \rightarrow_\epsilon q_{n+1}$ is tagged by $t$ and
$|l_1...l_{k-1}| = n$.

## Prefixed TFA

If $A$ is a TFA and $P$ a regular finite automaton,
we write
$L_t(P, A)$ the *$P$-prefixed* $t$-tagged language of $A$,
defined as all $(p.w, |p|+n)$ where
$p \in L(P)$ and $(w, n) \in L_t(A)$.