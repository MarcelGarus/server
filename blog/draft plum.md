topics: Plum, programming language design, code

# Plum

## A small, functional, cozy programming language

Well.
I created a new programming language.
Again.
It combines some design and implementation decisions from my previous languages, most notably [Martinaise](/martinaise) and [Candy](/candy).

Candy taught me that performance can't be an afterthought.
Our most recent attempt had interesting language semantics that enabled some really cool tooling (see [my other article](/candy)), but was unusably slow for everyday tasks.

With Martinaise, I gained some experience compiling statically typed code into efficient machine code.
This included lots of low-level stuff, like memory-layouting data structures, deciding on a calling convention, etc.
While Martinaise as a language is not super exciting, it is actually usable and has become my go-to language for new projects.

My new language, [Plum](https://github.com/MarcelGarus/plum), is my attempt at creating a "higher-level Martinaise".
It's still statically typed and compiles to efficient byte code, but has immutable data structures, pure functions, and an optimizing compiler.

The key idea:
Plum itself is not super exciting.
Instead, I want to focus on creating an interesting language _runtime_.

Typically, programming languages can be sorted into two camps:

- *Dynamically typed languages* don't know the types of variables.
  Typically, all values have a uniform representation in memory that not only contains the data, but also describes the structure of the object.
  Sometimes, the code is compiled to machine code just in time for execution.

- *Statically typed languages* know the types of variables up front.
  Values in these languages typically store only the data itself, not metadata describing the structure.
  These languages are often compiled to efficient machine code ahead of time.

If you know which group a language belongs to, you can usually infer how development works as well:

In dynamically typed languages, you can often change and reload code while the program is running.
Just write some more code and it can work on existing data.
This iterative development really appeals to me as everything feels "live".

Compiled languages often heavily rely on crazy optimizations.
Some optimizations, such as [defunctionalization through lambda set specialization](https://dl.acm.org/doi/pdf/10.1145/3591260), assume a closed world – they only work if you know all the code up front.
This heavily favors a workflow where you first write your code and then run the program as a separate step.

What if there was a language with static types, efficient memory layouts for values, and compilation to machine code, but it allows defining and compiling new code during runtime?
Turns out, that exists!
[Scopes](https://sr.ht/~duangle/scopes/) is a Lisp-like language that is compiled to machine code.
The interesting bit:
During runtime, you can use the builtin `scopes:compile` function to compile new functions into machine code (or GPU shaders!) that can seamlessly interoperate with your existing code.

While I like the mixture of explicit compilation and runtime malleability, I'm a bit scared off by how low level Scopes is:
It targets C++ game programmers, has manual memory management, and intends to interoperate with existing C++ code.

What if we took the same concept – having the compiler available as a library – but applied it to a memory-safe, high-level, functional language?

This is what Plum explores.
Check out [the repository](https://github.com/MarcelGarus/plum) for more details.
