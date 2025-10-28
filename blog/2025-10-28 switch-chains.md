topics: Plum, programming language design, code

# Switch Chains

## Separating decision making from actions

My programming language [Plum](/plum) has postfix syntax for switching on enums:

```plum
number =
  a_bool
  % true -> 2
    false -> 3
```

It also has structural typing and infers types of expressions:

```plum
value =
  a_bool
  % true -> | foo: 2
    false -> | bar: "Hi"
```

Here, the value of `plum:| foo: 2` is `plum:| foo: Int` and the type of `plum:| bar: "Hi"` is `plum:| bar: String`.
The type of a switch expression is the union of its cases, so `plum:| foo: Int bar: String` (which you can read as "either it's `plum:foo` with an `plum:Int` payload, or `plum:bar` with a `plum:String` payload").

Turns out, structural typing and postfix switches compose really well!

In other languages, I'd often write code like this:

```plum
if condition {
  // action 1
  ...
} else {
  var value = computation()
  if other_condition(value) {
    // action 2
    ...
  } else {
    // action 3
    ...
  }
}
```

The action comments would be a high-level description of what I'm doing.
But note how I interleave the information gathering / decision making with actually doing the action?
In Plum, I can cleanly separate them using this pattern:

```plum
condition
% true -> | action_1
  false ->
    value = computation
    other_condition value
    % true -> | action_2
      false -> | action_3
% action_1 -> ...
  action_2 -> ...
  action_3 -> ...
```

Here, I construct a value of the type `plum:| action_1 action_2 action_3` and then immediately switch on it.
The comments are gone!
The code is now fully self-documenting.

My first big data structure written in Plum was the standard library's hash map.
It uses a typical implementation:
A giant array stores the entries and you do linear searches to put entries into a free place and to find them.
A key's hash determines where you start the search, so it works in an ammortized runtime of O(1).

The code for putting an item into the map uses the pattern I described above:

```plum
raw_put
  buckets: (Array (Bucket k v)) index: Int key: k value: v
  -> (Array (Bucket k v))
= index = index .mod (buckets.length)
  buckets .get index
  % filled    -> | keep_searching
    empty     -> | use_this_bucket
    tombstone -> | use_this_bucket
  % use_this_bucket -> buckets .set index (| filled: (& key value))
    keep_searching  -> raw_put buckets (index .+ 1) key value
```

You don't need to understand every detail, but the gist is that I'm creating an enum `plum:| keep_searching use_this_bucket`, and then switch on it to do the corresponding action.
Here's another function that uses switch chains:

```plum
raw_get_maybe
  buckets: (Array (Bucket k v)) index: Int key: k equals: (\ k k -> Bool)
  -> (Maybe v)
= index = index .mod (buckets.length)
  buckets .get index
  % filled: entry ->
      equals (entry.key) key
      % true  -> | found_it: entry.value
        false -> | keep_searching
    tombstone -> | keep_searching
    empty     -> | not_in_map
  % found_it: value -> | some: value
    not_in_map      -> | none
    keep_searching  -> buckets .raw_get_maybe (index .+ 1) key equals
```

Beautiful!
