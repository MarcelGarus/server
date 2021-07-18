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
nano adminkey.txt
```

Then, paste the admin key.

This should start the server:

```bash
cargo run
```

Later on, updates can be applied like this:

```bash
git pull && cargo run
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

Make ddclient start when the system is booted:

```bash
sudo update-rc.d ddclient defaults
sudo update-rc.d ddclient enable
```
