topics: programming language design, Martinaise, code

# Or

## An underrated language construct

In [my programming language Martinaise](/martinaise), I don't have a fixed set of operators that are built into the language.
Instead, you can define your own operators by writing a function:

```mar
fun ==(a: Bool, b: Bool): Bool {
  if a then b else not(b)
}

fun !=[T](a: T, b: T): Bool {
  not(a == b)
}

fun main() {
  println(true != false)
}
```

This is also how I initially implemented boolean operators (because `mar:|` is already the character for comments, I use `mar:/` for or):

```mar
fun &(a: Bool, b: Bool): Bool { if a then b else false }  | and
fun /(a: Bool, b: Bool): Bool { if a then true else b }   | or
fun ^(a: Bool, b: Bool): Bool { if a then not(b) else b } | xor
```

However, there's a subtle difference between how these operators behave compared to those of most mainstream languages:
In Martinaise, `mar:foo() & bar()` first calls `mar:foo()` and `mar:bar()`, and then calls the `mar:&` operator.
In most popular languages, equivalent code will first call `c:foo()` and _only if `c:foo` returns `c:true`_, will `c:bar` be called.
Otherwise, the entire expression directly evaluates to `c:false`.
This "short-circuiting" behavior is useful in lots of situations.
Consider this example written in C:

```c
if (index < buffer_len && buffer[index] == 'f') {
  ...
}
```

If the `c:&&` operator was not short-circuiting, an `c:index` that's bigger than `c:buffer_len` would still access `c:buffer[index]`, potentially accessing memory outside of the buffer (and thereby triggering undefined behavior).
Naively translating the C code into Martinaise code will fail exactly in this way.
Instead, you have to nest `mar:if`s:

```mar
if index < buffer.len then
  if buffer.get(index) == #f then
    ...
```

Well, you'd probably rather check if `mar:buffer.get_maybe(index) == some(#f)`, but the point still stands:
For complex condition checks, I want to have short circuiting behavior.
So, I looked at what other programming languages do.

## Short-circuiting `mar:&&` and `mar:||`

I could change `mar:&&` and `mar:||` to be short-circuiting, similar to how they work in C, Rust, JavaScript, Java, Dart, and many more languages.
This would allow you to do this:

```mar
if condition then { ... }

// equivalent:

condition && { ... }
```

This works in the languages mentioned above and I think this behavior is weird.
It feels like they are in denial about the fact that `rust:&&` and `rust:||` are full-blown control flow constructs (which usually have keywords).
Instead, they try to conceal them as operators, even though they behave different from all the other operators.

## Short-circuiting `mar:and` and `mar:or`

Some languages (notably Python and Zig) go another way:
They acknowledge that short-circuiting boolean operations influence the program flow, so they give it just as much visibility as other control flow constructs.
They give it a keyword.

```python
if foo() or bar():
    ...
```

In this Python code, you intuitively understand that `python:or` may affect the control flow just as much as `python:if`.

## Using short-circuiting for more than Bools

I got curious how `mar:and` and `mar:or` behave for non-boolean objects in dynamically typed languages.
In Python, [the docs](https://docs.python.org/3/library/stdtypes.html#truth-value-testing) say this:

> By default, an object is considered true unless its class defines either a `python:__bool__()` method that returns `python:False` or a `python:__len__()` method that returns zero, when called with the object.
> Here are most of the built-in objects considered false:
> 
> - constants defined to be false: `python:None` and `python:False`
> - zero of any numeric type: `python:0`, `python:0.0`, `python:0j`, `python:Decimal(0)`, `python:Fraction(0, 1)`
> - empty sequences and collections: `python:''`, `python:()`, `python:[]`, `python:{}`, `python:set()`, `python:range(0)`

The docs also contain this [nice handy table](https://docs.python.org/3/library/stdtypes.html#boolean-operations-and-or-not):

<blockquote>
<center>
<table>
<tr>
  <td><strong>Operation</strong></td>
  <td><strong>Result</strong></td>
</tr>
<tr>
  <td><code>x or y</code></td>
  <td>if x is true, then x, else y</td>
</tr>
<tr>
  <td><code>x and y</code></td>
  <td>if x is false, then x, else y</td>
</tr>
</table>
</center>
</blockquote>

This means you can use `python:or` to select default values:

```python
foo = my_map[key] or "default"
```

## `mar:and` and `mar:or` in Martinaise

I tried to unify `mar:and`, `mar:or`, and default values into one concept, and make it extensible to custom types.
If you write `mar:foo() and bar()` in your code several things happen.

1.  The left side, `mar:foo()`, gets evaluated.
2.  An `mar:and` function is called with the result of the left expression.
    This function must return a `mar:ControlFlow`:
    
    ```mar
    enum ControlFlow[S, M] {
      short_circuit: S,  | evaluate to S immediately
      evaluate_more: M,  | evaluate to the right side, passing M as a binding
    }
    ```
    
    Function overloading is used to find type-specific behavior for `mar:and`.
    For example, these are the `mar:and` and `mar:or` functions for `mar:Bool`s:
    
    ```mar
    fun and(bool: Bool): ControlFlow[Bool, Nothing] {
      if bool
      then ControlFlow[Bool, Nothing].evaluate_alternative
      else ControlFlow[Bool, Nothing].short_circuit(false)
    }
    
    fun or(bool: Bool): ControlFlow[Bool, Nothing] {
      if bool
      then ControlFlow[Bool, Nothing].short_circuit(true)
      else ControlFlow[Bool, Nothing].evaluate_alternative
    }
    ```
3.  Depending on the variant of the `mar:ControlFlow`, the right behavior is chosen.

If you're wondering about the `mar:M` binding, this is useful for handling errors of results:

```mar
var content = read_file("hello.txt") or(error) panic("Couldn't open file: {error}")
```

Essentially, `mar:or` becomes a `mar:switch`:

```mar
# equivalent to the above:
var content =
  switch read_file("hello.txt").or()
  case short_circuit(content) content
  case evaluate_more(error) panic("Couldn't open file: {error}")
```

## Using `mar:and` and `mar:or`

Once you acknowledge that `mar:and` and `mar:or` are control flow constructs just as much as `mar:if`s, they can make your code a lot clearer.
Unlike `mar:if`, an `mar:or` allows you to first state the expected expression (usually a `mar:Bool`, `mar:Maybe`, or `mar:Result`) and handle the exceptional case after the fact.

Let's look at a few real-world use cases!
I won't explain much about the code – the details are pretty irrelevant.
I'll just let you admire the `mar:or` keyword in the wild.

Comparing two slices:

```mar
fun <=>[T](a: Slice[T], b: Slice[T]): Ordering {
  var i = 0
  loop {
    if i == a.len and i == b.len then return Ordering.equal
    if i == a.len then return Ordering.less
    if i == b.len then return Ordering.greater
    var ord = a.get(i) <=> b.get(i)
    ord is equal or return ord
    i = i + 1
  }
}
```

Parsing imports in the Martinaise compiler:

```mar
fun parse_imports(parser: &Parser): Result[Vec[AstStr], Str] {
  var imports = vec[AstStr]()
  loop imports.&.push(parser.parse_import()? or break)
  ok[Vec[AstStr], Str](imports)
}
```

Copying slices:

```mar
fun copy_to[T](from: Slice[T], to: Slice[T]) {
  from.len == to.len or
    panic("copy_to slice lens don't match ({from.len} and {to.len})")
  memcopy(from.data, to.data, from.len * stride_size_of[T]())
}
```

## Conclusion

If you're creating a programming language, consider making `mar:and` and `mar:or` keywords.
Also, allowing developers to customize their behavior for types is a huge opportunity for improving ergonomics.
Happy coding!
