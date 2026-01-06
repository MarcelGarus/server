topics: Plum, code

# Memory Layouting

While creating my programming language [Plum](/plum), I wondered how to find efficient memory layouts for structs and enums.
For example, consider this struct with the three fields a, b, and c:

```plum
& a: Byte b: Int c: Byte
```

I want to use the statically available type information to choose efficient memory layouts for values so that I don't need to use runtime memory to store information about the structure.
As `plum:Int`s in Plum always take up 64 bits, these are the fundamental pieces of data that we somehow need to arrange in memory:

TODO: a bbbbbbbb c

If you don't know about memory alignment, you might think putting the field next to each other is a good idea:

TODO: abbbbbbbbc

However, on modern computers, memory accesses should be _aligned_:
When moving values between registers and memory, the absolute memory address should be a multiple of the value's size.
When I first learned about this behavior, it felt weird, quirky, and unintuitive.
But apparently, this makes things easier for the hardware and it's here to stay:
TODO: ARM, x86

For my use case, that means:
In order to efficiently load a 64-bit `plum:Int` from memory into a register to do calculations, it should be stored at an address that is a multiple of 64 bits, or 8 bytes.
That's why compilers put padding (unused space) between struct fields.
This reduces the information density, but usually makes the code faster:
It trades space and time efficiency.
Here's the memory layout that C would use for a struct:

TODO: a.......bbbbbbbbc.......

Note that it doesn't just add padding after the a field, but also after c, so that if you have an array of these structs, all the b fields are aligned to 8 bytes.

Other languages enable more efficient layouts.
For example, because the Rust language gives no guarantees about the memory layout of types, the compiler is free to reorder fields.
The current Rust compiler chooses the following layout:

TODO: find out

However, Rust has a similar restriction to C:
In order to support storing slices of data types, the alignment (the )

However, Rust has restriction:
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
