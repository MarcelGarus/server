# On Developing This Blog

## Build stuff manually!

I created the blogging server that hosts this page with my bare hands.
And I have to say; I thoroughly enjoyed it!

...

I had to recreate every feature that other blogging frameworks give you for free:

- providing HTTP to HTTPS redirects
- make the browser cache pages
- provide an RSS feed
- provide correct MIME types for all served files
- setting up HTTPS

As a result of this endeavor, I have complete control over every part of the server.
Every header.
Every response code.
Every URL.
No magic.

If you want to learn more about how the web works, I recommend developing your server by hand.
I used [`text:actix-web`](https://crates.io/crates/actix-web) and the [source code of my server](https://github.com/MarcelGarus/server) is publicly available.
