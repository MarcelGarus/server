topics: Plum, code

# Memory Layouting

While creating my programming language [Plum](/plum), I wondered how to find efficient memory layouts for structs and enums.
For example, consider this struct with the three fields a, b, and c:

```plum
& a: Byte b: Int c: Byte
```

I want to use the statically available type information to choose efficient memory layouts for values so that they don't need to store information about the structure at runtime.
As `plum:Int`s in Plum always take up 64 bits, these are the fundamental pieces of data that we somehow need to arrange in memory:

TODO: a bbbbbbbb c

If you don't know about memory alignment, you might think putting the field next to each other is a good idea:

TODO: abbbbbbbbc

However, on modern computers, memory accesses should be _aligned_:
When loading and storing values from and to memory, the absolute memory address should be a multiple of the value's size.
When I first learned about this behavior, it felt weird, quirky, and unintuitive.
But apparently, this makes things easier for the hardware and it's here to stay:
TODO
ARM, x86

For my use case, that means:
In order to efficiently calculate with `plum:Int`s, we need to be able to load them, so they should be stored at a multiple of 64 bits, or 8 bytes.
That's why compilers put padding (unused space) between struct fields.
This reduces the information density, but usually makes the code more efficient.
Here's the memory layout that C would use for a struct:

TODO

Other languages enable more efficient layouts.
For example, because the Rust language gives no guarantees about the langauge, the compiler is free to reorder fields.
The Rust compiler chooses the following layout:

TODO

However, Rust has a somewhat weird restriction:
Type's

A restriction of Rust is that a type's size has to be a multiple of its alignment.
This allows the compiler to always choose an optimal layout – in the worst case, it can always sort fields by decreasing size so that fields with a bigger size (and alignment) are at the front.

However, for
An additional restriction of Rust is that
This is reasonable for enums,

So far, so good.

Surely, that's a common problem that many programming languages struggle with!

In particular,

```rust
enum Inner { A: i64, B: i64 }
struct Outer { inner: Inner, extra: u8 }
```

- Haskell
- LLVM
- Zig
-
