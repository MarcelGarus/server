# Hacking the Gira HomeServer

All lights and shades in our home can be controlled via [KNX](https://www.knx.org).
Our *Gira HomeServer* connects to the physical KNX wires as well as our network and makes it possible to control all devices using an app.
In [the previous article](/gira), we looked at how the Gira HomeServer communicates with the app.
As a reminder, this is how the authentication works:

![comic](https://github.com/marcelgarus/server/raw/main/blog/images/gira-comic.png)

More specifically, the process works like this:

1. The app asks to sign in using a username.
2. The server requests a *hash*, which is just a random string of characters (for example `245013621`).
3. The app calculates a mathematical hash function, passing the salt, username, and password as inputs: `hash(salt, username, password)` Critically, the hash function is irreversible, so having the result doesn't allow you to make predictions about the inputs. In the previous article, we already discovered that the app uses the [insecure MD5 hash function](https://en.wikipedia.org/wiki/MD5) for parts of the password hash.
4. The app sends the result to the server. The result is pretty long (something like `A335F32830EAFB1534DE46EFEECF1A8C`), so it's unlikely that the app gets this right by chance if it doesn't know the password.
5. The server does the same calculation itself and sees if the results match. If they do, the app is authenticated and can control the devices in the home.

Now, what is the most basic, most glaring security bug that can occur?
That' right: Simply ignoring step 5 – regardless of whether the hashes match or not (meaning regardless of whether the password was correct), let the app control the devices! 🤦[^emoji]
In the comic version, exploiting this behavior would look like this:

![comic](...)

[^emoji]: I'm usually not a fan of emojis in blogs, but I believe the facepalm emoji is warranted in this case.

Instead, it's such a simple and preventable bug that I'm alarmed it hasn't been discovered by Gira's quality assurance team.
It's not even a small oversight, but indicates a fundamental flaw in the Gira's security assurance.
Something like this should never happen.

You own a Gira HomeServer and want to see if you're affected? On GitHub, I published [a program written in Dart that toggles all devices off](...).

## How bad is it?

Now that we discovered how to hack the Gira HomeServer, let's step back and look at the impact this has.
To be able to use this hack, a few preconditions need to be met:

* The home needs to have a Gira HomeServer. We have a Gira HomeServer 4, so the hack works at least there. I don't have access to other models, so I'm not aware if the problem exists there as well.
* For the hack to succeed without having access to the network (e.g. by being logged into the home's WiFi), the HomeServer needs to be accessible from the internet. The compancy that set up our HomeServer conveniently also set up port forwarding in our router, so that we can connect to the server while not being at home. More critically, the device was registered at [giradns.com](https://giradns.com) so the app can find it even if our router's IP changes. By scanning the subdomains of `giradns.com`, it's possible to get a list of many Gira HomeServers that you can reach via the public internet.
* As an additional step, you need to know the username to log in. Usually, usernames are less guarded than passwords, so I assume that many are the same as the subdomain or the family's names. I'm pretty certain that performing a [dictionary attack](https://en.wikipedia.org/wiki/Dictionary_attack) using names would be highly effective.

Once these criteria are met, you can control all devices in the home.
That includes not only toggling lights, but also controlling more critical devices, like blinds, thermostats, and alarm systems.
The fact that I can turn off our alarm system without knowing the password is frightening.

To mitigate the risk, I turned off the port forwarding setup company enabled in our router settings.
Until the bug is fixed, having the HomeServer not being reachable from the public internet is a good idea.

## Communicating with Gira

At this point, it's relatively clear that I need to contact Gira about this.
Thankfully, there's a process for reporting security vulnerabilities: [Responsible disclosure](https://en.wikipedia.org/wiki/Responsible_disclosure).
This process works like this:

1. Contact the responsible company privately.
2. After a period of time (hopefully after the vulnerability is fixed), make the vulnerability public so that others can learn from it.

So, in November 2017, I contacted Gira using their contact form: (typography corrected)

> To whom it may concern!
>
> <details>
> <summary>…</summary>
>
> Since we got a Gira Home Server, I thought it would be nice if I could control devices in our house with a simple programming interface.
> Because there are no official information about an API, I decompiled the Gira Home Server app for Android and reengineered the network protocol.
> Eventually, I wrote a Python library that enables to connect to the Home Server, request the state of devices and change it.
> I published the library on GitHub for other hobby programmers that own a Home Server.
> </details>
>
> While testing, I realized you don't need the password hash to change device states.
> If you use a proper username and answer to the password hash request with a random 32 chars long string made up by hexadecimal digits, you are still logged in. In this case, you don't get notified about state changes of devices but you can still change device states.
>
> <details>
> <summary>…</summary>
>
> Especially after the security flaw in WPA2, in my opinion it's a critical issue that an attacker in the network can – after he knows the username, which is transmitted in plain text – access all devices and i.e. deactivate alarm systems.
> </details>
>
> I took the GitHub repository offline, but I wanted to inform you about the problem, so you can fix the flaw.
>
> Sincerely,
> Marcel, a 17 y/o hobby programmer
>
> PS: Exploit as zip: https://drive.google.com/██████████████████████████████

On the next day, I promptly got a response:

> Hi Marcel,
>
> Thank you for this information.
>
> Currently we're working on the next version of HomeServer software. It will provide encrypted communication.
>
> With kind regards
>
> ██████████ █████████████
> Advanced Support Hotline Agent

To 17-year-old me, this sounds promising!
A fix is already in the works, so it's okay to lean back and relax.

---

About half a year later, the bug still existed.
So, in May 2018, I contacted Gira again: (translated from German)

> Hello,
>
> pretty much half a year ago I reported a critical security vulnerability in the Gira HomeServer 4 software (…), which allows attackers to turn off alarm systems if port forwarding is activated in the router's settings (for us, this was enabled by default).
> I have been assured that you are working on new software that supports encryption and closes this gaping security vulnerability.
> Christmas has now passed, I turned 18 and graduated from high school. Since I now have free time again, I wanted to find out how the development of the new software is going. 
>
> Greetings,
> Marcel
>
> <details>
> <summary>Original German message</summary>
>
> Hallo,
>
> Vor ziemlich genau einem halben Jahr habe ich eine kritische Sicherheitslücke beim GiraHomeServer 4 gemeldet (Gira Nachricht ██████), die es jedem Angreifer mit Internetzugriff erlaubt, Alarmanlagen auszuschalten, wenn Port Forwarding beim Router aktiviert ist (dies wurde bei uns standardmäßig eingerichtet).
> Mir wurde versichert, dass an einer neuen Software gearbeitet wird, die Verschlüsselung unterstützt und diese klaffende Sicherheitslücke schließt.
> Inzwischen ist Weihnachten vergangen, ich wurde 18 und habe mein Abitur gemacht. Da ich jetzt wieder freie Zeit habe, wollte ich mich erkundigen, wie es mit der Entwicklung der neuen Software vorangeht.
>
> Viele Grüße
> Marcel
> </details>

The next day, a Gira spokesperson called me (the contact form asked for the phone number) and personally assured me that the software will be published "in next year" (which would mean 2019).

---

Fast-forward to 2021.
So much has happened: I moved to Potsdam, started studying, finished my Bachelor of Science in IT-Systems Engineering.
Recently, I visited my family and while I was at home, I checked if the security flaw is still present – of course it is.
Because of Corona, I have some spare time right now, so let's get to it:

> TODO: Write mail.

Let's see how this goes.
