This is a work in progress.

---

This is my personal server, which will be available at [mgar.us](https://mgar.us).

The goal for this server is to offer several services:

* `mgar.us`: A page with general information about me.
* `mgar.us/blog`: An overview of articles that I wrote.
* `mgar.us/contact`: Options to contact me.
* `mgar.us/pay`: Redirects to PayPal, calculates result of path (e.g. mgar.us/pay?amount=13/3).
* `mgar.us/<article-id>`: Each article has a custom URL.
* `mgar.us/<file-id>`: A file I made publicly available.
* `mgar.us/go/<shortcut-id>`: A shortcut to another website.
* `mgar.us/api/...`: APIs are available here.

Other domains redirect here:

* `marcelgarus.de` -> redirect to `mgar.us`
* `marcelgarus.dev` -> redirect to `mgar.us`
* `schreib.marcel.jetzt` -> redirect to `mgar.us/contact`
* `bezahl.marcel.jetzt` -> redirect to `mgar.us/pay`

For information on how to configure the server, the [server setup guide](server-setup.md) might be interesting.

TODOs in no particular order:

* HTTPS
  * support HTTPS
  * redirect HTTP to HTTPS
* beautiful error page
* app
  * visits
  * statistics about which pages were visited how often
  * shortcuts
* contact
* use proper mime types
* files
* blog
  * estimate read time
  * add `link` tags to previous and next article
  * don't show all articles on the main page, but the recent ones
  * add article overview page with all articles
* pay
  * redirect to PayPal
  * calculate amount
* make shortcut previews in social messenges beautiful
