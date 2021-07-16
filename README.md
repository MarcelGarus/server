This is a work in progress.

---

This is my personal server, which is available at [mgar.us](https://mgar.us).

The goal for this server is to offer several services:

* mgar.us: A page with general information about me.
* mgar.us/blog: An overview of articles that I wrote.
* mgar.us/contact: Options to contact me.
* mgar.us/pay: Redirects to PayPal, calculates result of path (e.g. mgar.us/pay?amount=13/3).
* mgar.us/_article-id_: Each article has a custom URL.
* mgar.us/_file-id_: A file I made publicly available.
* mgar.us/go/_shortcut-id_: A shortcut to another website.
* mgar.us/api/...: APIs are available here.

Other domains redirect here:

* marcelgarus.de -> redirect to mgar.us
* marcelgarus.dev -> redirect to mgar.us
* schreib.marcel.jetzt -> redirect to mgar.us/contact
* bezahl.marcel.jetzt -> redirect to mgar.us/pay?amount=

TODOs in no particular order:

* actually host the server
* switch to main branch
* HTTPS
  * support HTTPS
  * redirect HTTP to HTTPS
* provide favicon.ico
* authenticate admin
* visits
  * write an app for it
  * statistics about which pages were visited how often
* shortcuts
  * write an app for it
  * make previews in social messenges beautiful
* blog
  * blog overview page
  * add `link` tags to previous and next article
* contact
* pay
  * redirect to PayPal
  * calculate amount
* files
* set up uptime monitoring using statuscake
