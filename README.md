This is a work in progress.

---

This is my personal server, which is available at [marcelgarus.dev](https://marcelgarus.dev).

The goal for this server is to offer several services:

* `marcelgarus.dev`: An overview of articles that I wrote.
* `marcelgarus.dev/<article-id>`: An article I wrote.
* `marcelgarus.dev/<file-id>`: A file I made publicly available.
* `marcelgarus.dev/go/<shortcut-id>`: A shortcut to another website.
* `marcelgarus.dev/contact`: Options to contact me.
* `marcelgarus.dev/pay`: Redirects to PayPal, calculates result of path (e.g. `marcelgarus.dev/pay?amount=13/3`).
* `marcelgarus.dev/api/...`: APIs are available here.

Other domains redirect here:

* `mgar.us` redirects to `marcelgarus.dev`
* `marcelgarus.de` redirects to `marcelgarus.dev`
* `schreib.marcel.jetzt` redirects to `marcelgarus.dev/contact`
* `bezahl.marcel.jetzt` redirects to `marcelgarus.dev/pay`

Why use `marcelgarus.dev` as the main domain?

* It contains my full name (and my usual username), not some cryptic abbreviation that I don't use anywhere else.
* It's easy to say in conversation.
* The `.dev` domain enforces HTTPS. No need to redirect HTTP to HTTPS.
* Using the short `mgar.us` that redirects to the main domain is still possible, so links don't get shorter.

For information on how to configure the server, the [server setup guide](server-setup.md) might be interesting.

TODOs in no particular order:

* redirect HTTP to HTTPS
* switch to marcelgarus.dev
* app
  * visits
  * statistics about which pages were visited how often
  * shortcuts
* configure caching of content
* pay
  * redirect to PayPal
  * calculate amount
* treat paths as normalized
* add compression
* auto-select all of the shortlink in the article footer

# Setting up the server

This document describes my server setup, mostly for my future self.
Got a server with Ubuntu 18.04 LTS 64bit from [Strato](https://strato.de).

## Long-running commands

Using the GNU `screen` utility, you can connect to the server multiple times while retaining the same terminal state.

`screen -S <name>` starts a new named screen session.
Detach from a screens using ctrl+a ctrl+d.

`screen -list` lists all screens in the form `<pid>.<name>`

Screens can be re-connected to using `screen -d -r <id>`.

## Setup the repo

```bash
sudo apt install curl git nano build-essential pkg-config libssl-dev
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
git clone https://github.com/marcelgarus/server.git
```

Then, add a `Config.toml`:

```toml
address = "0.0.0.0:80"
admin_key = "the-admin-key"

[certficate]
cert = "/etc/letsencrypt/live/mgar.us/fullchain.pem"
key  = "/etc/letsencrypt/live/mgar.us/privkey.pem"
```

Finally, start the server:

```bash
cargo run
```

Later on, updates can be applied like this:

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
sudo cp server.service /etc/systemd/system
```

Then, reload the available services and enable our server service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable server.service
```

Finally, start the service:

```bash
sudo systemctl start server.service
sudo systemctl status server.service
```

Viewing logs works like this:

```bash
journalctl -f -u server.service
```

## Setup DynDNS to route mgar.us traffic here (DynDNS via Namecheap)

```bash
sudo apt install ddclient
```

This will automatically start a wizard, where you can enter random values.
Configuring is instead done using the configuration file:

```bash
sudo nano /etc/ddclient.conf
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
login=mgar.us
password='the-namecheap-dyn-dns-password'
ssl=yes
## The addresses of the A+ Dynamic DNS Records to update
@, www
```

To test if it works:

```bash
ddclient -daemon=0 -noquiet -debug
```

Make `ddclient` start when the system is booted:

```bash
sudo update-rc.d ddclient defaults
sudo update-rc.d ddclient enable
```

## Get HTTPS

Install snap:

```bash
sudo apt install fuse snapd
sudo snap install core; sudo snap refresh core
```

Make sure that the old certbot-auto is not installed:

```bash
sudo apt-get remove certbot
```

Install Certbot:

```bash
sudo snap install --classic certbot
```

Ensure that Certbot can be run:

```bash
sudo ln -s /snap/bin/certbot /usr/bin/certbot
```

Temporarily stop the server; Certbot needs port 80.

Then, run the certbot:

```bash
sudo certbot certonly --standalone
```

The command will output the paths of the certificates, for example:

```
Certificate is saved at: /etc/letsencrypt/live/mgar.us/fullchain.pem
Key is saved at:         /etc/letsencrypt/live/mgar.us/privkey.pem
```

The following commands tell Certbot how to temporarily stop and start the server for certificate renewals:

```bash
sudo sh -c 'printf "#!/bin/sh\nsystemctl server stop\n" > /etc/letsencrypt/renewal-hooks/pre/server.sh'
sudo sh -c 'printf "#!/bin/sh\nsystemctl server start\n" > /etc/letsencrypt/renewal-hooks/post/server.sh'
sudo chmod 755 /etc/letsencrypt/renewal-hooks/pre/server.sh
sudo chmod 755 /etc/letsencrypt/renewal-hooks/post/server.sh
```

In the `Config.toml` file, add an `[https]` section.
