# A Critical Look at Static Typing

Nowadays, there are soooo many programming languages.
One critical differentiation point among them is how their type systems work.
You can sort programming languages into two camps:

- **Statically-typed languages** determine what sort of values can be stored in variables just by looking at the program.
- **Dynamically-typed languages** can't statically infer what sort of values can be stored in variables. They have to run the program to find out.

If given a choice, I prefer statically-typed languages. Because they know the types of variables, they can offer clever suggestions. Additionally, the compiler catches many errors while you write the code, so you get immediate feedback.

On the other hand, most type systems limit what you can express in a language.
In comparison, dynamic languages feel more flexible.

--snip--

## What's wrong with types?

To better understand the appeal of this flexibility, let's take a step back and look at mathematics:
Ask a mathematician what a function is, and you'll get the answer that a function `text:f : X -> Y` is just a relation from an input set `X` to an output set `Y`.
Alright, so what's a set? Interestingly, it's one of the fundamental definitions of mathematics that is not defined using other math terms but using natural language instead:

> A set is a thing where each value is either in that thing or not.

At first glance, this seems wishy-washy, so here are two examples:
The *set of even numbers* contains the number 4, but not the number 1 or the banana emoji 🍌.
The *set of positive numbers* contains the numbers 1 and 3, but not the number -9.

Static programming languages model sets using **types.** Typically, there are only a limited number of types available:

* A primitive type is built into language. Examples are `Bool` or `String` in most languages.
* A composite type combines multiple other types into one new type. Most languages have an *and* combination called *struct* or *class*, some languages even have an *or* combination called *enum*.

Critically, because you can only create new types in these pre-defined ways, **you can't represent all sets as types!** If you want to write a function that only accepts positive numbers or only strings that are palindromes, you're out of luck.

So, what happens in these cases? Well, we're back to runtime errors. Even in static languages like Java or Rust, the compiler won't warn you if you call the logarithm function with a negative input. Instead, you'll get a crash, exception, panic, or whatever else represents a runtime error in those languages.

## Is there a better way?

I think so! Here's a two-step plan:

1. Remove the rigid distinction between compile-time and runtime errors. Embrace how dynamic languages handle errors.
2. Try to find most of those errors with **fuzzing,** the process of trying out lots and lots of inputs until one breaks the code.

Together with [Jonas](https://wanke.dev), I'm working on a new programming language called [Candy](https://github.com/candy-lang/candy), which uses fuzzing as a fundamental part of the developer experience. It'll still take some time to be useful, but take a look at the following function definition:

```candy
foo a b = add a b
```

This code defines a function called `foo` that takes the arguments `a` and `b` and returns their sum.
As soon as you write this into your editor, it will try out different values and discover that the code fails for `a = ""` and `b = []` (because you can't add a `String` and a `List`). The editor will tell you the exact error case instead of reporting an abstract error like "`String` does not implement addition for `List`."

To fix this error, you'll have to specify the function's `needs`: Candy provides a `needs` function that takes a `Bool`. If this argument is `False`, the program will crash in a way that signals "The program just crashed here. But it's not the fault of this function. Instead, whoever *called* this function gave it a wrong input."


Using the `needs` function, here's a fixed version of the code:

```candy
foo a b =
  needs (isInt a)
  needs (isInt b)
  add a b
```

Now, if the compiler tries `a = ""` and `b = []`, the code still crashes, but the compiler knows that *it* did something wrong. Because the compiler can't find inputs that crash the function in any other way, it will report no errors.

I do admit that this looks similar to types in static languages. The difference is that you can put any regular code after the `needs` instead of having a complex meta-language for defining types. As long as it evaluates to a `Bool`, you're good to go.
You can write functions that only accept even numbers, palindromes, valid JSON strings; you name it.

Even better, you'll get error reports for cases you haven't considered yet. If you write an `average` function that doesn't handle empty lists, your editor will warn you about it while you still type.
It could also discover invalid uses of stateful APIs:

```candy
file = File.open "some-file.txt"
File.write file "Hello!"
File.close file
File.close file # error: the file is already closed here
```

I'd even go as far as claiming that this is **better than writing unit tests.**
Unit tests get you to think creatively about what values your program needs to handle.
Fuzzing makes *the computer* come up with such values and it will report the tiniest edge cases – and you don't have to write a single line of extra code to get the benefit.

## Performance

First off: **Fuzzing** has been around for quite some time now, and fuzzers have been researched and optimized for years to find bugs in existing pieces of software. Some data centers do nothing but fuzz complex software like browsers, operating systems, and compilers. You can quickly test thousands of inputs per second, even on desktop computers.

Fuzzers also don't just randomly change inputs. Instead, they try to be clever about finding inputs to execute all code paths in the program. For example, fuzzers for compilers quickly "learn" to generate valid programs that activate some specific parts of the compiler.

Integrating the fuzzing into the editor also offers performance benefits because it can focus on recently edited functions instead of primarily fuzzing your dependencies.

## Is this an original idea?

I believe at least some parts of it are.
Languages with so-called refinement types also enable some flexibility in this direction, but you still have to prove to the compiler that your program is correct instead of the computer trying to disprove the correctness.

As mentioned, fuzzing is also a well-established concept.
The only innovation is to use it as a fundamental part of programming language design and developer experience.

Let's hope something fancy comes out of this!
I'm already excited about the result.
