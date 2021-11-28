## Exhaustigen

This is a tiny (but delightful!) utility library for exhaustive testing.

It is based (directly) on the idea and code in the following blog post:

https://matklad.github.io//2021/11/07/generate-all-the-things.html

TL;DR: the idea is to write a test that is _similar_ to the sort of test one
writes against a PRNG -- an imperative test that just asks some "generator" to
create scalars, booleans, data vectors, random shuffles, etc. -- but to use a
special generator that has some interesting features:

  - It has a concept of being "done", so you can put it in a do-while loop
  - Every call to it requires an inclusive upper bound, which should be small
  - It _tracks its progress_ through the sequence of bounds
  - It _lazily extends_ the sequence of bounds as it's asked for more data

By threading such a generator through such an imperative test, and putting the
body of that test in a do-while loop, you can write straightforward code with
elaborate value-dependent nesting structure -- eg. generate value K in 0..N and
then value J in 0..K and so forth -- and it will automatically record and
enumerate the space of sequences meaningful to the code, re-running until it has
exhausted all possible paths/values.

## License

MIT + ASL2.0, with permission from Aleksey.