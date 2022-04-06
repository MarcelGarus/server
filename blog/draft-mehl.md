# Mehl: A Syntax Experiment

As an experiment, I decided to create my own programming language called **Mehl**.
Here's an example Mehl program:

```mehl
(1, 2, 3) List.toIterable (., Int.Number) Iterable.sum
```

```mehl
(1, 2, 3) List[Int].sum
```

```mehl
(1, 2, 3) print
```

In this article, I'll describe the design of the language in more detail – why the language is the way it is.

--snip--

## Bottom-up syntax is useful

Let's take a look at this small lisp program:

```lisp
(filter (lambda (x) (x > 3))
        (map (lambda (x) (* x 2))
             '(1 2 3)))
```

To be honest, I really don't like this syntax.
Why? Because it's completely top-down – you do a `lisp:filter` of a `lisp:map` of a list.
Intuitively, most people will think about this piece of code in a bottom-up fashion:
We take a list of numbers, *then* map that list and *then* filter it – and the syntax doesn't reflect that at all.

What would a programming language with a bottom-up syntax look like?

--snip--

Instead of processing values by wrapping them, we should *chain functions together*.
Turns out, lots of languages do already work like this.
For example, Java code looks like this: `java:something.foo().bar()`.
Shell scripts using pipes also have a similar syntax: `ps aux | grep foo`
Heck, even some functional languages like F# have a custom operator that chains functions together sequentially instead of wrapping them.

If we turn this concept up to eleven and **only allow bottom-up syntax**, we don't even need arguments behind the function name.
If a function takes multiple arguments, you could just pass it a tuple:

```mehl
# Multiple 2 and 3, then double them
(2 3) * double
```

Function signatures become very easy: They take *one input* and produce *one output* – pretty elegant.



Note that each function simply takes one arguments as the input and emits another one as the output.


```mehl
{
  :name, :int.+,
  :in, int? list?,
  :out, int?,
  :doc, "This does something",
  :mayPanic, :true,
  :alwaysPanics, :true,
  :code, [do stuff],
} fun

(:int.+, int? list?, int?, "Adds numbers.", [:add-ints magic-primitive]) fun

3 seconds wait

"Hello, world!" print

42 double prime?
```
