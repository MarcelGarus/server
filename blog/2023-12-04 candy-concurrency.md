topics: Candy, programming language design, code

# Concurrency In Candy
## How Candy uses structured concurrency

> As of 2023-08-10, concurrency in Candy works differently. See [this article](candy-slim-runtime) for the reasoning behind that.

Many programming languages have the option to spawn a new thread.
A downside of this is that the control-flow becomes non-local:
If you call a function, you can't see if it starts another thread that then runs in the background.

```rust
sin(2) // may spawn a thread? who knows?
```

Candy is a functional language and we want functions to be a perfect unit of abstraction – you should know be able to reason about a function call without looking into the function's source code.
That's why we opted to use structured concurrency.

!invertible[concurrency with threads vs. structured concurrency](files/structured-concurrency.webp)

...

The idea of structured concurrency is to enforce that spawned control flows are joined eventually.
In Candy, that happens with a new kind of scope, a *parallel scope*.
Inside, you can use the given *nursery* to spawn new threads of execution, called *fibers*.
Only once all fibers spawned on the nursery completed, does the parallel section itself end.

```candy
parallel { nursery ->
  async nursery { print "Banana" }
  async nursery { print "Grapefruit" }
}
print "Kiwi"
```

Here, `candy:parallel` starts a new parallel scope.
Inside, two fibers are spawned using the `candy:async` function with the `candy:nursery`.
There's no guarantee about the order of execution, so this code prints "Banana" and "Grapefruit" in any order.
Only once both fruits are printed, does the `candy:parallel` section end and the code print "Kiwi".

There's also an `candy:await` function, the counterpart to `candy:async`.
That way, you can model dependencies between spawned fibers.

```candy
parallel { nursery ->
  willBeOne = async nursery {
    print "One"
    1
  }
  async nursery { print "Two" }
  one = await willBeOne

  # async and await cancel each other out
  three = await async { 3 }
}
```

## Channels

Concurrently executing fibers are only one part of the equation.
They can communicate using *channels*.
Channels can store a number of values – you can send values to them on one end and receive them from the other end.

!invertible[channels](files/channel.webp)

```candy
c = channel.create 4  # capacity of 4
tx = c.sendPort
rx = c.receivePort

send tx 1
send tx 2
send tx 3

one = receive rx
```

## How does it work?

The Candy runtime maintains a list of fibers and a list of channels.
It then chooses a random running fiber and continues executing instructions.
If a fiber starts a parallel section or interacts with a channel, it's paused.

!invertible[fibers](files/candy-fibers.webp)

In this example, fiber 1 started a parallel section and spawned three children.
One of the fibers called `candy:receive` on a channel.
Because there are no values, it's waiting for items to be sent to the channel.
The runtime will randomly choose between fiber 2 and 4 and run either of them.

## Well, but…

Actually, this concurrency approach is a thing of the past.
As of PR X, we removed concurrency from the Candy runtime.
It's no longer a builtin concept, but may be implemented in the underlying platform where Candy is embedded.
For more information about the reasoning behind that change, see [this article](candy-slim-runtime).
