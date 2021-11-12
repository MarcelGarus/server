# Templating engine? No thanks.

[Here](https://github.com/MarcelGarus/server/blob/f0688236abbe9f25ff296a6197130a19e3c6d562/src/templates.rs) is the Rust code that produces the pages you're currently reading (at least the code at the time of writing).
Some time ago, a friend of mine asked me why I used this home-grown templating mechanism instead of some standard like mustache.

Admittedly, this might look very hacky on first impression:

```rust
async fn article_full(article: &Article, suggestion: &Article) -> String {
    fs::read_to_string("assets/article-full.html")
        .await
        .unwrap()
        .fill_in_article(&article)
        .replace("{{suggestion-key}}", &suggestion.key)
        .replace("{{suggestion-title}}", &suggestion.title)
}
```

It's just loading the template file and then replacing some strings!
(If you wonder about `rust:fill_in_article(&article)`, that also does nothing more than replacing some strings.)

--snip--

But I still think it's more elegant than using some [full-blown templating engine like mustache](https://mustache.github.io/mustache.5.html) for two reasons:

1. It's pure Rust. Depending one on library less means you have to understand one library less.
   In fact, you can understand this code just by having a rough grasp of the standard library.
   I'm no Rust magician myself, but I still believe this is the *easiest to understand* version of the code that can possibly exist, given no former templating knowledge.
2. It's type-safe! Instead of having some weird looking syntax like `{{#condition}} stuff {{/condition}}` in the HTML code, I instead take advantage of Rust's full type safety!
   Just take a look at how the main page is constructed:

   ```rust
    pub async fn blog_page(articles: Vec<Article>) -> String {
        let mut teasers = vec![];
        for article in articles {
            teasers.push(article_teaser(&article).await);
        }
        page(
            "Blog",
            &metadata(...),
            &itertools::join(teasers, "\n"),
        )
        .await
    }
   ```

   That's just marvelous! It takes a list of `rust:Article`s, then turns each of them into some small HTML snippet using the `rust:article_teaser` function and then it joins all of them and puts them in the body of a `rust:page`.
   I can just apply my full knowledge of Rust's `rust:Iterator` protocol without doing any weird hacks in HTML.

So, the next time someone offers you a templating library, you might want to step back a bit and think:
What's the benefit of adding this other abstraction?
Is this really making my life easier overall?
