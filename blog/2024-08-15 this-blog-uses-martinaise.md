topics: this blog, Martinaise, code

# This Blog Uses Martinaise

## And why "My article doesn't compile" is now a valid statement

I wrote about this blog's tech stack in [a previous article](developing-this-blog).
Basically, I use the Rust actix framework and I handle SSL certificates, compression schemes, and cache lifetimes manually.
While writing that code was fun, maintaining it is not – especially considering that my entire site could just as easily be a bunch of static HTML files.
So, a re-work was due.

Coincidentally, I just developed [Martinaise](martinaise), my own programming language.
The natural thing to do was to program a static site generator in my newly-born language.
They say that "If all you have is a hammer, everything looks like a nail."
In my experience, it also holds that "If you know how to write a compiler, everything looks like a compilation problem."
For my blog, I wanted Markdown articles to go in, and HTML sites to fall out.
To me, that looks an awful lot like I have to write a compiler.

## A better Markdown

Implementing a Markdown parser opened up some nice opportunities:
Until now, I used the Rust crate `text:comrak` for parsing Markdown, so I had to use weird hacks to encode additional information in the [CommonMark](https://commonmark.org) Markdown syntax.

- *Are images invertible?*
  Most of the images in this blog are black-and-white drawings.
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
  I'd use `markdown:--snip--` to mark the end of an article preview that's shown on the index site and strip out that directive before letting the Markdown parser do its work.

The new site generator in Martinaise parses these ad-hoc extensions directly instead of when inspecting the parsed Markdown after-the-fact.
Some of them even got more beautiful:

```markdown
An invertible image:

!invertible[alt text](url)

The end of the preview is after this sentence at the triple dots.

...

This is not shown in the preview.
```

My version of Markdown is also stricter, making some articles more robust.
For example, one article had the code `text:inc a := ...` in a paragraph.
My pipeline interpreted `text:inc a ` as the programming language and `text:= ...` as the code.
Of course, the client-side JS-library didn't complain.
But now, code without a language or with an unknown language results in a compile error:

```text
Parsing 2023-09-29 candy-compiler-pipeline.md
When parsing markdown:
 241 | 
 242 | As you see, some syntax peculiarieties are also desugured.
 243 | In the CST, the `inc a := ...` definition was parsed as an assignment with a call on the left side.
                                     ^
                                     Unknown language inc a .
```

My motto for articles:
If it compiles, it ~works~ reads.

## A new parsing approach

When developing the compiler for [Candy](https://github.com/candy-lang/candy), an indentation-based language, we settled on a [hand-written recursive descent parser](candy-compiler-pipeline) where the individual `rust:parse` functions took an indentation as a parameter.
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
Instead of an `rust:input: &str` or a parser `mar:struct` that is passed around, there's a parser `mar:enum`:

```mar
enum Parser {
  root: &RootParser,
  indent: &IndentParser,
  quote: &QuoteParser,
}
struct RootParser { input: Str, cursor: Int }
struct IndentParser { parent: Parser, indent: Int, is_at_start: Bool }
struct QuoteParser { parent: Parser, is_at_start: Bool }
```

Only the `mar:RootParser` stores the input, all other parsers ask their parent parser for content.
This allows parsers to decide which characters to omit and which to forward to their children.

For example, the `mar:QuoteParser` can ensure that there's a `markdown:> ` after each newline.
It consumes these characters when the inner parser asks for more input.
If a line doesn't start with `markdown:> `, it just tells the inner parser that the input ended.
This way, parsers are self-contained.

Using a parser is as simple as wrapping the existing parser:

```mar
fun parse_quote(parser: Parser): Result[Maybe[Markdown], Str] {
  parser.consume("> ") or return ok[Maybe[Markdown], Str](none[Markdown]())
  var quoted = Parser.quote(QuoteParser {
    parent = parser, is_at_start = false
  }.put_on_heap()).parse()?
  ok[Maybe[Markdown], Str](some(quoted))
}
```

## Server-side syntax highlighting

Another aspect that bothered me was syntax highlighting.
Until now, I used `text:prism.js`.
It's fine, but client-side highlighting feels hacky – why make compute-restrained client devices take all the heavy lifting instead of highlighting the syntax once on the server?

Cue me, implementing crude syntax highlighters for Bash, C, Candy, Dart, HTML, JSON, Lisp, Markdown, Martinaise, Mehl, Python, Rust, and Zig.
Long live the yak shave!

## Wow!

I have developed quite a few programming languages already, but Martinaise is the first language that I used to write an actual, big project in.
There's the self-hosted compiler of course, but now I can add a blog to that list.
All the code is [on GitHub](https://github.com/MarcelGarus/server).
Good night!
