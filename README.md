This repository contains the code of my server, which is available at [marcelgarus.dev](https://marcelgarus.dev).

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
netstat -tulpn
```

## Setup the repo

1.  Install Git
    ```bash
    apt install curl git nano build-essential pkg-config libssl-dev
    ```
2.  [Install Caddy](https://caddyserver.com/docs/install)
3.  Clone this repo
    ```bash
    git clone https://github.com/MarcelGarus/server.git
    cd server
    ```
4.  Make the production Caddyfile the system Caddyfile:
    ```bash
    cp production.Caddyfile /etc/caddy/Caddyfile
    ```
5.  Enable Caddy:
    ```bash
    systemctl enable caddy
    ```

Done! The server will run, even when the system restarts.

Later on, you can apply updates like this:

```bash
git pull && cargo run
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
