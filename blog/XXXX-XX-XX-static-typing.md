# A Critical Look at Static Typing

Nowadays, there are soooo many programming languages.
One critical differentiation point among them is how their type systems work.
Broadly speaking, you can sort programming languages into two camps:

- **Statically typed languages** determine what sort of values can be saved in variables just by looking at the program.
- **Dynamically typed languages** can't statically infer what sort of values can be saved in varibles. They have to actually run the program to find out.

If given the choice, I personally prefer statically typed languages. Because they know the types of variables, they usually offer better autocompletion and catch many errors while you're writing the code, so they immediately give you feedback on its validity.

That being said, I do believe that most type systems limit what you can express in a language.
Types act like a rigid meta-language that sits on top of your actual code.
This is quite a contrast to the more flexible, dynamic languages.

--snip--

## Types are only a rough approximation of sets

To better understand the appeal of this flexibility, let's take a step back and look at mathematics:
Ask a mathematician what a function is, and you'll get the answer that a function `f : X -> Y` is just a relation from an input set `X` to an output set `Y`.
Alright, so what's a set? It's actually one of the fundamental definitions of mathematics that is not defined using other math-terms, but using natural language instead:

> A set is a thing where each value is either in that thing or not.

At first glance this seems kind of wishy-washy, so here are two examples:
The *set of even numbers* contains the number 4, but not the number 1 or the banana emoji 🍌.
The *set of positive numbers* contains the numbers 1 and 3, but not the number -9.

Static programming languages model sets using **types.** These are often just some primitive types (like `Bool` or `String`), or a combination of primitive types. (For example, the *and* combination of types are classes or structs that contain fields and represent that a variable always contains all those values; the *or* combination of types are enumerations or unions that represent that a variable is one of several types.)
Critically, this means that you can *only* use these ways to create new types. Because types are only a rough abstraction over sets, there are a whole lot of sets that can't be represented as types! If you want to write a function that only accepts numbers larger than zero, or only strings that are palindromes, you're out of luck.

So, what happens in these cases? Well, we're back to runtime errors. Even in static languages like Java or Rust, the compiler won't warn you if you call the logarithm function with a negative value. Instead, you'll get a crash, exception, panic, or whatever else represents a runtime error in those languages.

## Is there a better way?

I think so! Here's a two-step plan:

1. Remove the hard distinction between compile-time and runtime errors. Basically, turn back to how dynamic languages handle errors.
2. Try to find most of those errors with **fuzzing,** which is the process of trying out lots and lots of inputs until one breaks the code.

Together with [Jonas](https://wanke.dev), I'm actually working on a new programming language called **Candy,** which uses fuzzing as a fundamental part of the developer experience. It's nowhere near finished, but take a look at the following function definition:

```candy
foo a b = add a b
```

This code defines a function called `foo` that takes the two arguments `a` and `b` and returns their sum.
As soon as you write this into your editor, the IDE will try out different values and discover that the code fails for `a = ""` and `b = []` (because you can't add `String`s and `List`s). Instead of getting an abstract error like "`String` does not implement addition for `List`", the editor will tell you the concrete error case.

To fix this error, you'll have to add `needs` assertions. Those accept a `Bool` and panic if it's `False`. The cool thing is that they panic in a way that signals to the language runtime that the failure is the fault of the *caller* of the function.
Here's a fixed version of the code:

```candy
foo a b =
  needs (isInt a)
  needs (isInt b)
  add a b
```

Now, if the compiler tries `a = ""` and `b = []`, the code still crashes, but the compiler gets told that *it* did something wrong. Because the compiler can't find inputs that crash the function in any other way, it will report no errors.

I do admit that this looks similar to types in static languages. The difference is that you can put any regular code after the `needs` instead of having a complex meta-language for defining types. As long as it evaluates to a `Bool`, you're good to go.
You can write functions that only accept even numbers, palindromes, valid JSON strings, you name it.
Even better, you'll get error reports for cases you haven't thought about. Whether writing an `average` function that doesn't handle empty lists, or a `sort` function with an off-by-one error, your editor will warn you about it as soon as you type.

I'd even go as far as claiming that this is better than writing unit tests.
Unit tests get you to think creatively about what values could be given to your program, but fuzzing will report the tiniest edge cases – and you don't have to write a single line of extra code to get the benefit.

## Performance

First off: **Fuzzing** has been around for quite some time now, although the use case has primarily been to find bugs in existing pieces of software. There are data centers that do nothing but fuzz complex software like browsers, operating systems, and compilers. This means that there has been years of research and optimizations happening on that front. Even on desktop computers, you can easily test thousands of inputs per second.

Fuzzers also don't just randomly change inputs, but instead try to be clever about it in a way so that all code paths in the program are used. For example, fuzzers for compilers quickly "learn" to generate valid programs that activate some specific parts of the compiler.

Integrating the fuzzing into the editor also opens up performance-benefits because it can focus on recently edited functions instead of primarily fuzzing your dependencies.

## Is this an original idea?

I believe at least some parts of it are.
There are some type systems with dependent types or refinement types that try to enable more flexibilty when dealing with your types.
But even in those languages, *you* still have to prove to the compiler that your program is correct instead of the compiler trying to disprove the correctness of your program.

As mentioned, fuzzing is also a well-established concept.
The only innovation is to use it as a fundamental part of a programming language design and developer experience.

Let's hope something fancy comes out of this!
I'm already excited for the result.
