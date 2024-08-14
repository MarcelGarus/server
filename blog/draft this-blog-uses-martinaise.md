topics: this blog, Martinaise, code

# This Blog Uses Martinaise
## Why "My article doesn't compile" is now a valid statement

I wrote about this blog's tech stack in [a previous article](developing-this-blog).
Basically, I use the Rust actix framework and I handle SSL certificates, compression schemes, and cache lifetimes manually.
While writing that code was fun, maintaining it is not – especially considering that my entire site could just as easily be a bunch of static HTML files.
So, a re-work was due.

Coincidentally, I just developed [Martinaise](martinaise), my own programming language.
The natural thing to do was to program a static site generator in my newly-born language.
There's this quote "If all you have is a hammer, everything looks like a nail."
In my experience, it also holds that "If you know how to write a compiler, everything looks like a compilation problem."
For my blog I wanted Markdown articles to go in, and HTML sites to fall out.
To me, that looks an awful lot like I have to write a compiler.

## A better Markdown

Implementing a Markdown parser opened up some nice opportunities:
Until now, I used the Rust library `comrak` for parsing Markdown, so I had to use some weird hacks to encode additional information in the Markdown syntax.

- *Are images invertible?*
  Most of the images in this blog are simple digital black-and-white drawings.
  If you view this website in dark mode, those images will be inverted.
  To mark images as invertible, I'd use `text:invert:` at the beginning of the alt text:
  
  ```markdown
  ![invert:alt text](url)
  ```
- *What's the language of inline code?*
  It still baffles me that there is syntax for specifying the programming language of multi-line code blocks, but not of inline code.
  So, I'd use something like `text:rust:` at the beginning of inline code snippets to enable syntax highlighting:
  
  ```markdown
  A `rust:Vec<Value>` represents the solutions to the Kakuro.
  ```
- *Where's the end of the preview?*
  I'd use `--snip--` to mark the end of an article preview that's shown on the index site and strip out that directive before letting the Markdown parser do its work.

The new site generator in Martinaise parses these ad-hoc extensions directly instead of when inspecting the parsed Markdown after-the-fact.
Some of them even got more beautiful:

```markdown
An invertible image:

!invertible[alt text](url)

The end of the preview is after this sentence at the triple dots.

...

This is not shown in the preview.
```

My version of Markdown is also stricter.
This is partly to catch errors (such as an emphasis that isn't closed) and partly to make my life as a developer easier.

TODO
`inc a := ...`

Parsing 2023-09-29 candy-compiler-pipeline.md
When parsing markdown:
 221 | That makes the next stages easier to implement.
 222 | 
 223 | ```
 224 | assignment: struct
       ^
       Code block without language.


## A new parsing approach

When developing the compiler for [Candy](https://github.com/candy-lang/candy), an indentation-based language, we settled on a [hand-written recursive descent parser](candy-compiler-pipeline) where the individual `parse` functions took an indentation as a parameter.
The functions were not supposed to parse any line that doesn't start with the given indentation.
For example, the function for parsing a parenthesized expression has the following signature:

```rust
fn parenthesized(input: &str, indentation: usize) -> Option<(&str, Rcst)> {
  ...
}
```

That approach fails for Markdown.
In Markdown, there's not a single type of indentation you can pass to functions.
For example, in quotes, each line needs to start with a `markdown:>`.
And you can nest those structures arbitrarily:

```markdown
> Quote.
>
> - List
>   > Quote in list
```

If you parse a newline in such a nested structure, you need to ensure that the next line starts with `markdown:>   >` or you stop parsing.
So, I opted for nested parsers.
Instead of an `rust:input: &str` or a parser `martinaise:struct` that is passed around, there's a parser `martinaise:enum`:

```martinaise
enum Parser {
  root: &RootParser,
  indent: &IndentParser,
  quote: &QuoteParser,
}
struct RootParser { input: Str, cursor: Int }
struct IndentParser { parent: Parser, indent: Int, is_at_start: Bool }
struct QuoteParser { parent: Parser, is_at_start: Bool }
```

Only the `martinaise:RootParser` stores the input, all other parsers ask their parent parser for content.
This allows parsers to decide which characters to omit and which to forward to their children.

For example, the `martinaise:QuoteParser` can ensure that there's a `markdown:> ` after each newline.
It consumes these characters when the inner parser asks for more input.
If a line doesn't start with `markdown:> `, it just tells the inner parser that the input ended.
This way, parsers are self-contained.

Using a parser is as simple as wrapping the existing parser:

```martinaise
fun parse_quote(parser: Parser): Result[Maybe[Markdown], Str] {
  parser.consume("> ") or return ok[Maybe[Markdown], Str](none[Markdown]())
  var quoted = Parser.quote(QuoteParser {
    parent = parser, is_at_start = false
  }.put_on_heap()).parse()?
  ok[Maybe[Markdown], Str](some(quoted))
}
```

## Server-side syntax highlighting

Another tidbit that bothered me was syntax highlighting.
Until now, I used `prism.js`

TODO

## 

add this blog to that list


I've already written some code in my programming language Martinaise – mostly the compiler itself.
Now, I can add this blog to that list.
