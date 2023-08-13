topics = [ "programming language design" ]
description = "How Candy uses structured concurrency."

--start--

# Why Candy Doesn't Have Built-In Concurrency Anymore

Until now, we supported primitives for structured concurrency in our programming language Candy.
You can find the details [in the last article](candy-concurrency), but the short version is that there were built-in functions for operations such as starting multiple parallel execution threads.

We compile Candy code into bytecode for our VM.
Recently, [Clemens](https://tiedt.dev) started working on an LLVM backend as an alternative.
As part of that, we started thinking more about how Candy interacts with the platform it runs on.

![invert:VM and LLVM backends](files/TODO)

--snip--

## The Current State

Until now, Candy code could interact with the environment using [channels](candy-concurrency).



---

- runtime change
- handles
- concurrency via handles



