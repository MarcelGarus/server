This repository contains the code of my server, which is available at [marcelgarus.dev](https://marcelgarus.dev).

The goal for this server is to offer several services:

* `marcelgarus.dev`: An overview of articles that I wrote.
* `marcelgarus.dev/<article-id>`: An article I wrote.
* `marcelgarus.dev/<file-id>`: A file I made publicly available.
* `marcelgarus.dev/go/<shortcut-id>`: A shortcut to another website.
* `marcelgarus.dev/contact`: Options to contact me.
* `marcelgarus.dev/pay`: Redirects to PayPal, calculates result of path (e.g. `marcelgarus.dev/pay?amount=13/3`).
* `marcelgarus.dev/api/...`: APIs are available here.

Why use `marcelgarus.dev` as the primary domain?

* It contains my full name (and my usual username), not some cryptic abbreviation that I don't use anywhere else.
* It's easy to say in conversation.
* The `.dev` domain enforces HTTPS. No need to redirect HTTP to HTTPS.
* Because `mgar.us` redirects to the main domain, links can still be short.

Subdomains of `marcelgarus.dev` and other domains redirect to `marcelgarus.dev` without a subdomain.
They keep the current path but change the hostname and may add a path prefix at the beginning.

* no subdomain (`@`) and subdomain `www` redirects to `marcelgarus.dev`
* the `go` subdomain redirects to `marcelgarus.dev/go`
* German verb domains (like `schreib.marcel.jetzt`) redirect to specific pages
  * a trailing `e` is removed (so both `schreib` and `schreibe` works)
  * `schreib` and `folg` redirect to `marcelgarus.dev/me`
  * `bezahl` and `zahl` redirect to `marcelgarus.dev/pay`
* other subdomains just redirect to `marcelgarus.dev`

TODOs in no particular order:

* app
  * configuring shortcuts
  * faulty (non-200) responses in the last 30 days
  * visits in the last 30 days
  * successful (200) visits in the last 30 days
  * URLs in the last 30 days
  * languages in the last 30 days
  * HTTP referers in the last 30 days
  * uptime
  * resource utilization
  * parse and analyze languages
  * how visits map to areas (index, blog articles, shortcuts, etc.)
  * top most popular blog articles
  * publishing dates of past articles
* pay
  * redirect to PayPal
  * calculate amount
* make images placeholders the correct size

# Setting up the server

This chapter describes my server setup, primarily for my future self.
The server runs Ubuntu 18.04 LTS 64bit; it's a small machine hosted by [Strato](https://strato.de).

## Long-running commands

Using the GNU `screen` utility, you can connect to the server multiple times while retaining the same terminal state.

`screen -S <name>` starts a new named screen session.
Detach from a screen using ctrl+a ctrl+d.

`screen -list` lists all screens in the form `<pid>.<name>`

Screens can be re-connected to using `screen -d -r <id>`.

## Firewall?

No Firewall is needed; only a few programs are listening on ports, so it's easy to get an overview.
To see which programs listen on ports, do:

```bash
ss -tunlp
```

## Setup the repo

```bash
apt install curl git nano build-essential pkg-config libssl-dev
curl https://sh.rustup.rs -sSf | sh
```

Then enter 1 for "proceed with installation"

Because the code uses `#[feature]` flags, you need Rust nightly:

```bash
rustup default nightly
```

To setup rust in the currently running shell:

```bash
source $HOME/.cargo/env
```

```bash
git clone https://github.com/MarcelGarus/server.git
```

Then, add a `Config.toml`:

```toml
address   = "0.0.0.0:80"
admin_key = "the-admin-key"

[https]
redirect_from_address = "0.0.0.0:80"
certificate_chain     = "/etc/letsencrypt/live/marcelgarus.dev/fullchain.pem"
private_key           = "/etc/letsencrypt/live/marcelgarus.dev/privkey.pem"
```

Finally, start the server:

```bash
cargo run
```

Later on, you can apply updates like this:

```bash
git pull && cargo run
```

## Run the server across restarts

List services via

```bash
systemctl list-units --type=service
```

Compile the server into an optimized executable:

```bash
cargo build --release
```

This repo contains a `server.service` file, which is a systemd service description.
Copy it to the system service directory:

```bash
cp server.service /etc/systemd/system
```

Then, reload the available services and enable our server service:

```bash
systemctl daemon-reload
systemctl enable server.service
```

Finally, start the service:

```bash
systemctl start server.service
systemctl status server.service
```

Viewing logs works like this:

```bash
journalctl -f -u server.service
```

## Setup DynDNS to route marcelgarus.dev traffic here (DynDNS via Namecheap)

```bash
apt install ddclient
```

This will automatically start a wizard, where you can enter random values.
Configuring is instead done using the configuration file:

```bash
nano /etc/ddclient.conf
```

The content should be this:

```bash
## Update every 300 seconds.
daemon=300
## Log stuff to these files.
cache=/tmp/ddclient.cache
pid=/var/run/ddclient.pid
## Get the public IP address via dyndns.org
use=web, web=checkip.dyndns.org
# Update using Namecheap.
protocol=namecheap
server=dynamicdns.park-your-domain.com

login=marcelgarus.dev
password='the-namecheap-dyn-dns-password'
ssl=yes
@.marcelgarus.dev, *.marcelgarus.dev

login=marcelgarus.de
password='the-namecheap-dyn-dns-password'
ssl=yes
@.marcelgarus.de, *.marcelgarus.de

login=mgar.us
password='the-namecheap-dyn-dns-password'
ssl=yes
@.mgar.us, *.mgar.us

login=marcel.jetzt
password='the-namecheap-dyn-dns-password'
ssl=yes
@.marcel.jetzt, *.marcel.jetzt
```

To test if it works:

```bash
ddclient -daemon=0 -noquiet -debug
```

The cache file is at `/tmp/ddclient.cache`, and you might need to delete it if you want to re-set the DynDNS A+ record, although the IP didn't change.

Make `ddclient` start when the system is booted:

```bash
update-rc.d ddclient defaults
update-rc.d ddclient enable
```

## Get HTTPS

Install snap:

```bash
apt install fuse snapd
snap install core; snap refresh core
```

Make sure that the old certbot-auto is not installed:

```bash
apt-get remove certbot
```

Install Certbot:

```bash
snap install --classic certbot
```

Ensure that Certbot can be run:

```bash
ln -s /snap/bin/certbot /usr/bin/certbot
```

Certbot offers two basic authentication options: `standalone` tries to spin up an HTTP web server on port 80 and thereby see if you got control over the domain. DNS-based verification creates a TXT DNS record.

HTTP-based authentication only works for specific subdomains, e.g. `marcelgarus.dev` or `something.marcelgarus.dev`.
You must use DNS validation to use wildcard certificates like `*.marcelgarus.dev`, but that's hard with Namecheap.
So for my server, I use a certificate for most subdomains that people will encounter.

<details>
<summary>DNS validation (not chosen)</sumamry>

Namecheap doesn't natively support Certbot, so we need to do that manually:

```bash
certbot certonly --manual --preferred-challenges dns -d "marcelgarus.dev,*.marcelgarus.dev,marcelgarus.de,*.marcelgarus.de,mgar.us,*.mgar.us,marcel.jetzt,*.marcel.jetzt"
```

This command will create a Certbot-internal private/public key pair and ask you to add the public key as a TXT DNS record for the subdomain `_acme-challenge`.
It may take some time for the record to propagate. After some time, it should be visible in this [Google DNS Toolbox](https://toolbox.googleapps.com/apps/dig/#TXT/_acme-challenge.marcelgarus.dev) or be retrievable by running `nslookup -type=TXT _acme-challenge.marcelgarus.dev`).
Once the record is public, click enter.

To make sure the server is restarted with the new certificate after renewal:

```bash
sh -c 'printf "#!/bin/sh\nsystemctl server restart\n" > /etc/letsencrypt/renewal-hooks/post/server.sh'
chmod 755 /etc/letsencrypt/renewal-hooks/post/server.sh
```

</details>

<details>
<summary>Standalone validation</summary>

To make sure the temporary Certbot server doesn't conflict with our server, create hooks:

```bash
sh -c 'printf "#!/bin/sh\nsystemctl server stop\n" > /etc/letsencrypt/renewal-hooks/pre/server.sh'
sh -c 'printf "#!/bin/sh\nsystemctl server start\n" > /etc/letsencrypt/renewal-hooks/post/server.sh'
chmod 755 /etc/letsencrypt/renewal-hooks/pre/server.sh
chmod 755 /etc/letsencrypt/renewal-hooks/post/server.sh
```

Then just run:

```bash
certbot certonly -d "marcelgarus.dev,www.marcelgarus.dev,marcelgarus.de,www.marcelgarus.de,mgar.us,www.mgar.us,marcel.jetzt,www.marcel.jetzt,schreib.marcel.jetzt,schreibe.marcel.jetzt,folg.marcel.jetzt,folge.marcel.jetzt,bezahl.marcel.jetzt,bezahle.marcel.jetzt,zahl.marcel.jetzt,zahle.marcel.jetzt"
```

</details>

The command will also output the paths of the certificates, for example:

```
Certificate is saved at: /etc/letsencrypt/live/marcelgarus.dev/fullchain.pem
Key is saved at:         /etc/letsencrypt/live/marcelgarus.dev/privkey.pem
```

Ensure there's an `[https]` section in the `Config.toml` file that links to these files (like in the example file above).

To renew certificates, just do `cerbot renew`.
