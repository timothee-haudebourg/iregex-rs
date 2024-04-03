# Rootmatching

Rootmatching consists in deciding if an expression recognizes a given word,
and what part of the word is the *prefix*, the *root*, and the *suffix*.

Of course for the same word, multiple solutions are possible.
The most common strategy shared by mainstream regex engines,
and the one adopted here,
is to minimize the prefix length, and then minimize the suffix length.
In other word, we want the *left-most* and *longest* root.