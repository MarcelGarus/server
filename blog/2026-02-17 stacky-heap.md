# Stacky Heap

## A new memory management approach

I recently implemented garbage collection for [Orchard](/orchard), a self-hosted programming system with immutable values.

A straightforward solution would have been reference counting:
You store a counter at every object of how many references point to it and you free the memory when that counter reaches zero.
Reference counting is a good fit for languages with immutable values and eager evaluation; because values can only refer to previously created values, there are no cycles, avoiding the major weakness of reference counting.
Even better, some languages like Koka use the reference count to [opportunistically mutate values while keeping immutable semantics](https://www.microsoft.com/en-us/research/wp-content/uploads/2020/11/perceus-tr-v4.pdf) if the count is 1.

However, reference counting has downsides:
The operations for adjusting the counter can be expensive.
And exception handling becomes complicated because if you only executed part of some function, the reference counters might be in an inconsistent state.

I want to support exceptions in Orchard, so I took another approach:
A stacky heap.

In the beginning, I allocate a big area of memory.
Whenever you need memory for a new value, I give out some of that memory, from left to right.
This way of giving out memory is well-established and usually called "bump allocation" or "arena allocation".

Now comes the cool part:
Because the immutable objects can only refer to previously created objects, pointers inside the heap only ever point to the left.
This enables partial garbage collection!
When you call a function that does allocation-heavy work, but produces a small value, you can garbage-collect only the values allocated by that function.

The way this is surfaced in the language is through a built-in `text:collect_garbage` function.
It's a higher-order function that takes a lambda without arguments.
It just runs the lambda and returns its value, but it frees all intermediary objects that are not referenced by the return value.

```plum
input = ...
output = (collect_garbage (\ () (expensive_operation input)))
```

## Partial garbage collection

Let's say some values are allocated on the heap:

```embed
<style>
rect { fill: var(--box-bg); stroke: var(--fg); stroke-width: 2px; }
text { font: 14px sans-serif; text-anchor: middle; fill: var(--fg); }
.arrow { stroke: var(--pink); stroke-width: 2px; fill: none; marker-end="url(#arrow)" }
</style>
<svg xmlns="http://www.w3.org/2000/svg" width="304" height="67" viewBox="-2 -25 304 67">
    <!-- https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/marker -->
    <defs>
        <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" style="fill: var(--pink);" />
        </marker>
    </defs>
    <rect x="0" width="40" height="20"></rect><text x="20" y="15">a</text>
    <rect x="40" width="20" height="20"></rect><text x="50" y="15">b</text>
    <rect x="60" width="60" height="20"></rect><text x="90" y="15">c</text>
    <rect x="120" width="20" height="20"></rect><text x="130" y="15">d</text>
    <path class="arrow" marker-end="url(#arrow)" d="M 50 0 L 50 -20 L 20 -20 L 20 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 90 20 L 90 40 L 20 40 L 20 20" />
</svg>
```

Let's say we want to call a function with `text:a` and `text:c` and garbage collect all its intermediary allocations.
To do that, we remember the current size of the heap.
Then, we just call the function, which may allocate lots of objects:

```embed
<style>
rect { fill: var(--box-bg); stroke: var(--fg); stroke-width: 2px; }
text { font: 14px sans-serif; text-anchor: middle; fill: var(--fg); }
.arrow { stroke: var(--pink); stroke-width: 2px; fill: none; marker-end="url(#arrow)" }
</style>
<svg xmlns="http://www.w3.org/2000/svg" width="385" height="87" viewBox="-2 -25 385 87">
    <!-- https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/marker -->
    <defs>
        <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" style="fill: var(--pink);" />
        </marker>
    </defs>
    <line x1="140" y1="-15" x2="140" y2="35" style="stroke:blue; stroke-width:2px;" stroke-dasharray="4 1" />
    <rect x="0" width="40" height="20"></rect><text x="20" y="15">a</text>
    <rect x="40" width="20" height="20"></rect><text x="50" y="15">b</text>
    <rect x="60" width="60" height="20"></rect><text x="90" y="15">c</text>
    <rect x="120" width="20" height="20"></rect><text x="130" y="15">d</text>
    <rect x="140" width="40" height="20"></rect><text x="160" y="15">e</text>
    <rect x="180" width="40" height="20"></rect><text x="200" y="15">f</text>
    <rect x="220" width="20" height="20"></rect><text x="230" y="15">g</text>
    <rect x="240" width="60" height="20"></rect><text x="270" y="15">h</text>
    <rect x="300" width="20" height="20"></rect><text x="310" y="15">i</text>
    <rect x="320" width="20" height="20"></rect><text x="330" y="15">j</text>
    <rect x="340" width="40" height="20"></rect><text x="360" y="15">k</text>
    <path class="arrow" marker-end="url(#arrow)" d="M 50 0 L 50 -20 L 20 -20 L 20 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 90 20 L 90 40 L 20 40 L 20 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 160 0 L 160 -20 L 90 -20 L 90 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 230 0 L 230 -20 L 200 -20 L 200 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 310 20 L 310 40 L 270 40 L 270 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 360 20 L 360 50 L 160 50 L 160 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 360 0 L 360 -20 L 270 -20 L 270 0" />
</svg>
```

Let's say `text:k` is the return value.
We can do a standard mark-and-sweep garbage collection on everything after the call boundary:
Starting at `text:k`, we do a depth-first traversal of all dependencies, marking them as needed.
If we cross the call boundary (the blue line), we stop marking.

```embed
<style>
.marked { fill: var(--pink); color: white; }
</style>
<svg xmlns="http://www.w3.org/2000/svg" width="385" height="87" viewBox="-2 -25 385 87">
    <!-- https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/marker -->
    <defs>
        <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" style="fill: var(--pink);" />
        </marker>
    </defs>
    <line x1="140" y1="-15" x2="140" y2="35" style="stroke:blue; stroke-width:2px;" stroke-dasharray="4 1" />
    <rect x="0" width="40" height="20"></rect><text x="20" y="15">a</text>
    <rect x="40" width="20" height="20"></rect><text x="50" y="15">b</text>
    <rect x="60" width="60" height="20"></rect><text x="90" y="15">c</text>
    <rect x="120" width="20" height="20"></rect><text x="130" y="15">d</text>
    <rect x="140" width="40" height="20" class="marked"></rect><text x="160" y="15">e</text>
    <rect x="180" width="40" height="20"></rect><text x="200" y="15">f</text>
    <rect x="220" width="20" height="20"></rect><text x="230" y="15">g</text>
    <rect x="240" width="60" height="20" class="marked"></rect><text x="270" y="15">h</text>
    <rect x="300" width="20" height="20"></rect><text x="310" y="15">i</text>
    <rect x="320" width="20" height="20"></rect><text x="330" y="15">j</text>
    <rect x="340" width="40" height="20" class="marked"></rect><text x="360" y="15">k</text>
    <path class="arrow" marker-end="url(#arrow)" d="M 50 0 L 50 -20 L 20 -20 L 20 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 90 20 L 90 40 L 20 40 L 20 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 160 0 L 160 -20 L 90 -20 L 90 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 230 0 L 230 -20 L 200 -20 L 200 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 310 20 L 310 40 L 270 40 L 270 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 360 20 L 360 50 L 160 50 L 160 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 360 0 L 360 -20 L 270 -20 L 270 0" />
</svg>
```

Then, starting at the call boundary, we go over the objects.
We keep only marked objects, compacting them together and overwriting the other ones.

```embed
<style>
.marked { fill: var(--pink); color: white; }
</style>
<svg xmlns="http://www.w3.org/2000/svg" width="385" height="87" viewBox="-2 -25 385 87">
    <!-- https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Element/marker -->
    <defs>
        <marker id="arrow" viewBox="0 0 10 10" refX="10" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" style="fill: var(--pink);" />
        </marker>
    </defs>
    <line x1="140" y1="-15" x2="140" y2="35" style="stroke:blue; stroke-width:2px;" stroke-dasharray="4 1" />
    <rect x="0" width="40" height="20"></rect><text x="20" y="15">a</text>
    <rect x="40" width="20" height="20"></rect><text x="50" y="15">b</text>
    <rect x="60" width="60" height="20"></rect><text x="90" y="15">c</text>
    <rect x="120" width="20" height="20"></rect><text x="130" y="15">d</text>
    <rect x="140" width="40" height="20"></rect><text x="160" y="15">e</text>
    <rect x="180" width="60" height="20"></rect><text x="210" y="15">h</text>
    <rect x="240" width="40" height="20"></rect><text x="260" y="15">k</text>
    <path class="arrow" marker-end="url(#arrow)" d="M 50 0 L 50 -20 L 20 -20 L 20 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 90 20 L 90 40 L 20 40 L 20 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 160 0 L 160 -20 L 90 -20 L 90 0" />
    <path class="arrow" marker-end="url(#arrow)" d="M 260 20 L 260 50 L 160 50 L 160 20" />
    <path class="arrow" marker-end="url(#arrow)" d="M 260 0 L 260 -20 L 210 -20 L 210 0" />
</svg>
```

## What even is this?

In [Borrow checking, RC, GC, and the Eleven (!) Other Memory Safety Approaches](https://verdagon.dev/grimoire/grimoire), Evan Ovadia describes how several memory safety approaches can be blended together.
My stacky heap combines aspects of several techniques:

- My entire heap is basically a stack of **nested arenas** or **regions**, which is also my preferred memory management strategy in [Zig](/zig).
  Like some parts of **Ada/SPARK**, my pointers can only point left and objects can't outlive dependencies on the left.
- I use **tracing garbage collection** to condense a function call's allocations to its return value.
  **Generational garbage collection** has multiple buckets of objects that can be garbage-collected independently with different frequencies.
  If you squint your eyes a bit, it's like every call of `plum:collect_garbage` starts a new, nested generation that is only collected once.
- Like with **manual memory management**, you decide when you want memory to be collected. You are responsible to call `plum:collect_garbage` at functions that do a lot of internal work, but only return small amounts of data.

So, there you have it:
I created a new memory management approach that combines aspects from arena allocation, tracing garbage collection, and manual memory management.
I'm curious how this adventure continues.
