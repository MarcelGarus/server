topics: programming language design, Martinaise, code

# Or

## An underrated language construct

In [my programming language Martinaise](/martinaise), I don't have a predetermined set of operators that are built into the language.
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
In most popular languages, `c:foo() && bar()` will first call `c:foo()` and _only if `c:foo` returns `c:true`_, will `c:bar` be called.
Otherwise, the entire expression directly evaluates to `c:false`.
This "short-circuiting" behavior is useful in lots of situations.
Consider this example written in C:

```c
if (index < buffer_len && buffer[index] == 'f') {
  ...
}
```

If the `c:&&` operator was not short-circuiting, an `c:index` that's bigger than `c:buffer_len` would still access `c:buffer[index]`, potentially causing segmentation faults.
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

## Short-circuiting `text:&&` and `text:||`

I could change `mar:&&` and `mar:||` to be short-circuiting, but that would turn them into control flow construct.
For example, in Rust, you can use `rust:&&` instead of `rust:if`s:

```rust
if condition { ... }

// equivalent:

condition && { ... }
```

Many languages behave this way:
C, Rust, JavaScript, Java, Dart, any more.
This is weird.
All other control flow constructs have keywords – `mar:if`, `mar:switch`, `mar:loop`.
It feels like those languages are in denial about the fact that `rust:&&` and `rust:||` are full-blown control flow constructs.
They instead try to make them appear like operators, even though they behaves different from all the other operators.

## Short-circuiting `text:and` and `text:or`

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

This means you can use `python:or` to also select default values:

```python
foo = my_map[key] or "default"
```

## `mar:and` and `mar:or` in Martinaise

I tried to unify `mar:and`, `mar:or`, and default values into one concept, and make it extensible to custom types.
If you write `mar:foo() and bar()` in your code several things happen.

1.  The left side (`mar:foo()`) gets evaluated.
2.  An `mar:and` function is called with the result of the left expression.
    This function must return a `mar:ControlFlow`:
    
    ```mar
    enum ControlFlow[S, M] {
      short_circuit: S,  | indicates to evaluate to S immediate
      evaluate_more: M,  | indicates to evaluate the right side, passing M
    }
    ```
    
    Function overloading is used to find the correct `mar:and` type-specific behavior.
    For example, `mar:Bool`s
    
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

, the left side gets evaluated first.
Then, 


There is now a `mar:ControlFlow` struct:


Some types define `mar:and` and `mar:or` methods, which return such a `mar:ControlFlow`:


`mar:and` just calls `mar:and()` on the left side value!

 becomes a `mar:switch` while parsing the code!

```mar
if foo() and bar() then ...

# same:

if switch foo().or()
```



The nullish coalescing operator can be seen as a special case of the logical OR (||) operator. The latter returns the right-hand side operand if the left operand is any falsy value, not only null or undefined. In other words, if you use || to provide some default value to another variable foo, you may encounter unexpected behaviors if you consider some falsy values as usable (e.g., '' or 0). See below for more examples.

compare to ?? in
Zig: orelse, catch


