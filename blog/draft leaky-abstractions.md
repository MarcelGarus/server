topics: Martinaise, programming language design, code

# In Defense of Leaky Abstractions

## They can be a good thing

In programming, a leaky abstraction refers to an abstraction that leaks implementation details that it is supposed to abstract away.
[Joel Spolsky' Law of Leaky Abstractions](https://www.joelonsoftware.com/2002/11/11/the-law-of-leaky-abstractions/) theorizes the following:

> All non-trivial abstractions, to some degree, are leaky.

For example, the network protocol TCP attempts to abstract unreliable networks by retransmitting messages.
This doesn't work all the time.
If the network is unreliable, TCP operations can have wildly varying performance or may never complete.

Instead of trying to fix leaky abstractions, a technique I've recently come to like is to embrace the leaks and build guardrails around them.
In fact, I believe there are cases where intentionally adding more leakage than strictly necessary can be a good thing.

## Rust's unsafe

While Rust is generally a memory-safe language, sometimes you need to sidestep the compiler's memory-safety checks to implement new data structures or use external functions.
Rust has an `rust:unsafe` keyword for that.

```rust
fn foo() {
    unsafe {
        ...
    }
}
```

Even though `rust:unsafe` increases the complexity of the Rust language, it brings a huge benefit:
Efficient data structures can be implemented directly in Rust.

## Martinaise's Assembly Functions

In Martinaise, you can write functions in [a custom assembly](/soil) instead of the language itself:

```mar
opaque Int = 8 bytes big, 8 bytes aligned

fun +(left: Int, right: Int): Int asm {
  moveib a 8  add a sp load a a | left
  moveib b 16 add b sp load b b | right
  load c sp | return value address
  add a b store c a ret
}
fun -(left: Int, right: Int): Int asm {
  moveib a 8  add a sp load a a | left
  moveib b 16 add b sp load b b | right
  load c sp | return value address
  sub a b store c a ret
}
```

It's a custom assembly variant where each instruction gets compiled to a single byte code instruction.
Interfacing with assembly works seamlessly because types have a deterministic memory layout and Martinaise has a pre-defined calling convention.

Even though Martinaise expliticly leaks the underlying byte code compilation target, the language itself gets simpler:
Instead of many types like `mar:Int` or `mar:Float` being magically implemented by the compiler, the only "magic" are the assembly functions.

## A cozy language feeling

Leaky abstractions gives me a cozy feeling.
When writing assembly functions or unsafe code, it kind of feels like working in the engine room of a boat:
You get to peek behind the curtain of the usual language and work on a lower level.
When diving into the implementation of code, you never hit a wall of compiler-magic – you just get dropped one level lower.

![a cross-section of a boat with an engine room](files/boat.webp)
