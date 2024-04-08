topics = [ "programming language design", "code" ]
description = "TODO"

--start--

# In Defense of Leaky Abstractions

In programming, a leaky abstraction refers to an abstraction that leaks details that it is supposed to abstract away.
In practice, [all non-trivial abstractions, to some degree, are leaky](https://www.joelonsoftware.com/2002/11/11/the-law-of-leaky-abstractions/).
For example, the network protocol TCP attempts to abstract unreliable networks by retransmitting messages, but this behavior can leak by wildly varying performance.

Leaky abstractions are usually negatively connotated, but I argue that intentionally adding more leakage than strictly necessary *can* be a good thing.

![a cross-section of a boat with an engine room](files/boat.webp)

--snip--

Here are two examples where abstractions have intentional holes:

## Rust's unsafe

While Rust is generally a memory-safe language, sometimes you need to sidestep the compiler's memory-safety checks to implement new data structures or use external functions.
Rust has an `rust:unsafe` keyword for that.

```rust
// TODO: real world example
fn foo() {
    unsafe {
        
    }
}
```

## Martinaise's Assembly Functions

In Martinaise, you can write functions in assembly instead of the language itself.



```martinaise
struct Slice[T] { data: &T, len: U64 }

fun copy_to[T](from: Slice[T], to: Slice[T]) {
  assert(from.len == to.len, "copy_to slice lens don't match")
  memcopy(
    from.data.to_address(),
    to.data.to_address(),
    from.len * stride_size_of[T](),
  )
}

fun memcopy(from: Address, to: Address, amount: U64) asm {
  moveib a 16 add a sp load a a | from
  moveib b 24 add b sp load b b | to
  moveib c 32 add c sp load c c | amount
  moveib e 1
  cmp a b isless cjump .right_to_left
  .left_to_right: ..loop:
  move st c isequal cjump .done
  loadb d a storeb b d
  add a e add b e sub c e
  jump ..loop
  .right_to_left:
  add a c add b c sub a e sub b e | make a and b point to the last byte
  ..loop:
  move st c isequal cjump .done
  loadb d a storeb b d
  sub a e sub b e sub c e
  jump ..loop
  .done: ret
}
```

## Conclusion

TODO
