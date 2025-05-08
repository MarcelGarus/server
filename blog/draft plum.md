topics: Plum, programming language design, code

# Plum

## A small, functional, cozy programming language

I recently invented yet another programming language, [Plum](https://github.com/MarcelGarus/plum).
It combines some design and implementation decisions from my previous languages, most notably [Martinaise](/martinaise) and [Candy](/candy).

Candy taught me that performance can't be an afterthought.
Our most recent attempt had interesting language semantics that enabled some really cool tooling (see [my other article](/candy)), but was unusably slow for everyday tasks.

With Martinaise, I gained some experience compiling statically typed code into efficient machine code.
This included lots of low-level fun, like memory-layouting data structures, deciding on a calling convention, etc.
While Martinaise as a language is not super exciting, it is actually usable and has become my go-to language for new projects.

Plum is my attempt at creating a "higher-level Martinaise".
It's still statically typed and compiles to efficient byte code, but has immutable data structures, pure functions, and an optimizing compiler.
Check out [the Plum repository](https://github.com/MarcelGarus/plum) for more details.
