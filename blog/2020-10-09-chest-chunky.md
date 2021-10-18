# Chunky: A Database Layer

Chest's lowest abstraction layer is called *Chunky*.
In the end, all data should somehow be stored in a file, something like `🌮.chest` (yes, the taco emoji is a great name for a database). The Chunky framework will take care of managing access to that file.

A key question is how to deal with **mutating data**:
If we need to insert some data "in the middle" of the database, we don't want to re-write everything that comes after it.
Files are a linear stream of bytes and that doesn't quite fit our use case. So, the Chunky layer offers an abstraction from that.

--snip--

![Chest abstracts files](https://github.com/MarcelGarus/server/raw/main/blog/images/chest-chunky-layers.png)

Also, writing to the file might fail for various reasons – whether the OS kills our program, the user plugs out the storage medium, the power supply vanishes, or a black hole consumes the earth. Chunky also ensures that we handle such cases gracefully by fulfilling the four ACID goals:

- **Atomicity**: If you do a change, it's either fully written to the database file or not at all – partially written changes should never occur.
- **Consistency**: The database should always be in a consistent state, going from one consistent state into another.
- **Isolation**: To all clients it should look like they are the only client of the database.
- **Durability**: Changes written to the database should be persistent.

Before going into how Chunky achieves these goals internally, let's just give a little API overview:

## The API

Chunky divides the file into chunks of a fixed size – that's where it gets its name from.
To do anything with those chunks, you need to start a transaction, during which you can read and change chunks.
At the end of the transaction, all the changed chunks are actually written to the file.

Here's a schematic diagram of how the file looks like:

![Chunks are placed in the file one after another](https://github.com/marcelgarus/server/raw/main/blog/images/chest-chunky-chunks.png)

And here's how a usage might look like in actual code:

```dart
final chunky = Chunky('🌮.chest');
print(chunky.numberOfChunks);

// Only using a transaction, you can interact with the chunks.
chunky.transaction((transaction) {
  // Read the first chunk.
  final chunk = await transaction[0];
  // Change the first byte to 42.
  chunk.setUint8(0, 42);
  // Create a new chunk.
  final newChunk = transaction.reserve();
  print('New chunk reserved at ${newChunk.index}');
}); // At the end of the transaction, the changed chunk is written to disk.
```

## So, how does it actually work?

When you call `dart:Chunky('🌮.chest')`, Chunky looks for a file named `🌮.chest` and opens it.

Calling `dart:chunky.transaction(...)`

1. waits for all former transactions to finish and then
2. actually starts the transaction by creating a `dart:Transaction` and calling the callback.

A `dart:Transaction` buffers all the chunks accessed during that transaction – that's both the original chunk as well as the current version of the chunk.
Accessing chunks loads the original chunk from disk and saves a snapshot of it. After that, it creates a copy, wraps it into a `dart:TransactionChunk` and then returns that.

A `dart:TransactionChunk` is used to track dirtiness: It contains an `dart:isDirty` property and if any `dart:set...` method is called, for example `dart:setUint8(0, 42)`, the `dart:isDirty` property is set to `dart:true`.

When a transaction is over, the accessed chunks are diffed with the original version.
Here's the code snippet doing just that:

```dart
// _newChunks and _originalChunks are both Maps mapping chunk indizes to chunks.
final differentChunks = _newChunks.entries
  .whereKeyValue((index, chunk) => !_originalChunks.containsKey(index) || chunk.isDirty)
  .whereKeyValue((index, chunk) => chunk._data != _originalChunks[index])
  .toList();
```

First, the chunks are filtered to

- the ones that were newly added by calling `dart:reserve()` on the transaction and
- the dirty ones (as in, a `dart:set...` method has been called).

Then, those chunks are compared with the original chunks – after all, if `dart:set...` is called with the same value that's already stored or it's called multiple times, the value might be the same as the beginning of the transaction.

## Okay. So, how are the ACID goals achieved?

Because only one transaction is running at a time, the isolation goal is automatically fulfilled.

Regarding atomicity, the only guarantees that the operating system gives us is that creating and removing files is atomic as well as changing a single bit in a file.
That's why Chunky uses a **transaction file**:

1. When a transaction is finished, a separate file is created, the naming scheme being something like `🌮.chest.transaction`. It contains only a single byte that acts like a single bit – only differentiating between zero and non-zero. Initially, it's set to zero.
2. The file is flushed to disk (the changes are actually written to disk).
3. All changed chunks are appended to the transaction file.
4. The file is flushed again. Notably, this flushing doesn't affect the first byte still set to zero.
5. The first byte is set to one and the file is flushed a third time. The first byte being non-zero indicates that the transaction file is complete and contains all changed chunks.
6. Then, the chunks in the actual `🌮.chest` file are changed.
7. Afterwards, the transaction file is deleted.

If the program gets killed at any point, a consistent state can always be restored:

- Does a transaction file exist?
  - No: The changes are consistent: We're either before step 1 (old state) or after step 7 (new state).
  - Yes: Is the first bit of the transaction file non-zero?
    - Yes: We're after step 5 and can just copy all changed chunks from the transaction file to the original one (new state).
    - No: We're before step 5 and can just delete the transaction file (old state).

Because we either revert to the old state or the new one in all of the cases, the transactions are atomic.  
The transaction byte being set to `1` guarantees that the changes will actually be written to disk, either right now or during recovery in case the program crashes.

## Conclusion

I hope you got a general idea about how the Chunky framework works internally and how it ensures the ACID goals.
Given file transactions, we can now go on to plan what to actually store in those chunks.
More about that in the next article of this series.
