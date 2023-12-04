topics = [ "this blog", "Rust", "code" ]

--start--

# On Developing This Blog

I created the blogging server that hosts this page with my bare hands.
Well, the [HTTP request handling software](https://github.com/MarcelGarus/server) at least.
And I have to say; I thoroughly enjoyed it!

--snip--

I head to recreate every feature that other blogging frameworks give you for free:

* providing HTTP to HTTPS redirects
* make the browser cache pages
* provide an RSS feed
* provide correct MIME types for all served files
* setting up HTTPS

As a result of this endeavor, I have complete control over every part of the server.
Every header.
Every response code.
Every URL.
No magic.

If you want to learn more about how the web works, I recommend developing your server by hand.
I used [`actix-web`](https://crates.io/crates/actix-web) and the [source code of my server](https://github.com/MarcelGarus/server) is publicly available.
