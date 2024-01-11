
# Exploring Zig

I've been keeping an eye on [Zig](https://ziglang.org) for some time now.
Zig is a low-level language that has a similar abstraction level to C.
From how Zig was talked about, I expected to use a lot of `comptime` (a Zig keyword that forces expressions to be evaluated at compile-time).
Instead, I was primarily blown away by how it handles memory allocations.

Zig requires manual memory management, but all data structures from the standard library accept an allocator in the constructor.
That makes it super easy to use arena allocators:
When running `martinaise watch` (a command that watches a file for changes and recompiles and executes the program every time it changes), all the memory needed for a single compilation is allocated in an arena, a contiguous chunk of memory.
When the pipeline ran through, it frees the entire arena at once.

To me, this felt deliberating.
I thought less about memory management than I do with Rust.
The compiler just allocates things all over the place, and shares immutable data between compilation stages and between values in lots of places.
For example, 

When writing our Candy compiler in Rust,

In the compiler, I share a lot of data between stages.

In our Rust compiler, we would often


