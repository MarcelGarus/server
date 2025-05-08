topics: Plum, programming language design, code

# Types in Plum

## Some type algorithms and why the compiler models types as strings

[Plum](/plum) is a programming language with structural typing:
Types don't have an identity.
Values of these two types can be used interchangably:

```plum
Foo = & x: Int y: Int
Bar = & x: Int y: Int
```

Essentially, named types are just aliases for the structure of the type.
This is not new or groundbreaking, but it's the first time I implemented a compiler with a structural type systems.
One aspect made this a lot more challenging than I initially thought: Recursion.

Consider this type:

```plum
LinkedList t =
  | empty
    more: (& item: t rest: (LinkedList t))
```

It's an enum with two variants – `plum:empty` and `plum:more`.
The `plum:more` variant contains a struct with the `plum:item` and the `plum:rest` of the linked list, which is another linked list.

You can create linked list instances like this:

```plum
| more: (& item: 1 rest: (| more: (& item: 2 rest: (| empty))))
```

This `plum:LinkedList Int` contains the items 1 and 2.

## Recursive types as recursive data structures

So, how does the compiler internally represent types?
Because names shouldn't affect type checking in any way, so you could model types like this:

```mar
enum Type {
  int,
  struct_: Map[String, Type],
  enum_: Map[String, Type],
}
```

> Note:
> This is a simplification.
> Plum has more primitives (bytes, arrays), special types (never) and composite types (lambdas).

Here, recursive types would result in recursive type data structures in the compiler:

**TODO: picture**

> Note: In Plum, an enum variant without a payload actually has an empty struct as the payload type.

However, there are some serious downsides to this approach.
Representing types as potentially recursive data structures in the compiler require a tremendous level of care when working with them.
Simple actions such as debug-printing or hashing types can lead to infite traversals and hanging the compiler if you're not careful.
To prevent that, you have to compare type identities (the type objects' addresses).

## Types as trees

To work around these problems, I decided to model types as (finite) trees rather than recursive graphs.
I added a `plum:recursive` variant in the enum that tells us how many levels up in the type tree to continue:

```mar
enum Type {
  int,
  struct_: Map[String, Type],
  enum_: Map[String, Type],
  recursive: Int,
}
```

Using this, the linked list type is represented like this in the compiler:

**TODO: picture**

The recursive type tells us that we should start two layers further up in the tree – at the original enum.

Formatting, hashing, and generally traversing types now no longer requires us to be careful about running into infinite recursions.
Simple enough right?

Well.
Take a look at this Plum function, which switches over a linked list enum:

```plum
length list: (LinkedList t) -> Int =
  list
  % empty -> 0
    more: node ->
      length (node.rest) .+ 1
```

We switch on the `plum:list` of this enum type:

**TODO: picture**

In the `plum:more` case, we unpack the enum variant's payload, making it available as the `plum:node` variable.
What is the type of of a single `plum:node`?

If we would naively extract the payload's type from the internal `mar:Map[String, Type]`, that would leave us with this type:

**TODO: picture**

Oh no!
This type is no longer self-contained:
The recursive type references a type two levels up in the type tree, but that part of the tree has been discarded when figuring out the payload type!

What we want to happen instead is for the type to "wrap around" like this:

**TODO: picture**

I had a version of the compiler that worked like this.
However, it was quite finnicky to use:

- Whenever you traverse into a type, you had to remember to first call `mar:extend_one_level()` on it.
  This would extend the recursive types at the bottom so that they no longer reference the root node.
- Whenever you create a new type, you had to remember to call `canonicalize()` on it.
  This would simplfy the type.

If you forget one of these steps, invalid types might sneak into the compiler pipeline.

## Types as strings

At some point, I wondered if things were easier if I represented types as strings.
Turns out, it works surprisingly well.
Currently, the compiler represents the linked list type as this string:

```plum
(| empty: (&) more: (& item: (Int) rest: (^2)))
```

Note that the string has a very strict structure to it – every type has parentheses around it and even the `plum:(&)` type for the `plum:empty` variant is explicit.

This has several benefits:

- Strings are incredibly information-dense and many common type operations become very efficient.
  For example, comparing two types is as simple as comparing two strings.
  Hashing a type is a simple as hashing the string.
  Computers are made for these kinds of dense, linear memory accesses.
  Compare that with iterating over hash maps and following pointer indirections, which is what you would have to do for the enum-based approach.

- The type algorithms themselves now have to deal with strings.
  This sounds unsafe or cumbersome, but most of the algorithms adapted nicely and some even got easier.

- The code that just wants to use types can no longer accidentally deconstruct or construct invalid types.
  You have to use functions for inspecting

Implementing that turned out to be quite easy and made a lot of the type algorithms easier:

Well.
I thought about this a long time.

Well.

So...

because

This is the type of the`plum:list`:

What type does it have?

It's this one:
