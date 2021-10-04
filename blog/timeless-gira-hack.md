# Hacking the Gira HomeServer

Our home has a smart electric grid that can be controlled using [KNX](https://www.knx.org).
In our home, a *Gira HomeServer* connects to the physical KNX wires as well as to our WiFi network and enables us to control all devices using an app.
In [the previous article](/gira), we looked at how the Gira HomeServer communicates with the app.
As a reminder, this is how the authentication phase works:

![comic](https://github.com/marcelgarus/server/raw/main/blog/images/gira-comic.png)

More specifically, the process works like this:

1. The app asks to sign in using a username (first three comic boxes).
2. The server sends a *salt*, which is just a random string of characters (for example `245013621`).
3. The app calculates a mathematical hash function, using the salt, username, and password as inputs: `hash(salt, username, password)` Critically, the hash function is irreversible, so having the result doesn't allow you to make predictions about the inputs. In the previous article, we already discovered that the app uses the [insecure MD5 hash function](https://en.wikipedia.org/wiki/MD5) for parts of the password hash.
4. The app sends the result to the server. The result is pretty long (something like `A335F32830EAFB1534DE46EFEECF1A8C`), so it's unlikely that the app gets this right by chance if it doesn't know the password.
5. The server does the same hash calculation itself and checks if the results match. If they do, the app is authenticated and can control the devices in the home.

This process protects against other devices listening to the communication – if a malicious device wants to sign in, it's given a different random salt and can't calculate the corresponding hash, because it doesn't know the password.

While developing the app and tinkering with the authentication flow, I discovered a glaring security flaw: The server ignores step 5! Regardless of whether the password hash was correct, the server lets the app control all devices! 🤦[^emoji]
Exploiting this behavior would look like this in the comic version:

![comic](...)

Note that the server doesn't send any device updates to the app (like which devices are turned on or off), but it still listens to commands from the app.
This is such a simple and preventable bug that I'm alarmed Gira's quality assurance team didn't discover it.
It's not even a small oversight, but instead indicates a fundamental flaw in the Gira's security assurance.
**Something like this should never happen.**

You own a Gira HomeServer and want to see if you're affected?
On GitHub, I published [a program written in Dart that toggles all devices off](...) (including alarm systems).
Please only test this in your home.

[^emoji]: I'm usually not a fan of emojis in blogs, but I believe the facepalm emoji is warranted in this case.

## How bad is it?

Let's take a step back and look at the impact of this hack.
To be able to use it maliciously on other people's homes, a few preconditions need to be met:

* The home needs to have a Gira HomeServer. We have a Gira HomeServer 4 and I don't have access to other models, so I can only confirm that the hack works on that particular model.
* For the hack to succeed without having access to the network (e.g. without being logged into the home's WiFi), the HomeServer needs to be accessible from the internet. The company that set up our HomeServer conveniently also enabled port forwarding in our router, so that we can connect to the server while not being at home. They also registered our router at [giradns.com](https://giradns.com) so the app can find it even if its IP changes. By scanning the subdomains of `giradns.com`, you can get a list of Gira HomeServers reachable via the public internet.
* As an additional step, you need the username to log in. Usually, usernames are less guarded than passwords, so I assume that many are the same as the subdomain or the family's names. I'm pretty certain that performing a [dictionary attack](https://en.wikipedia.org/wiki/Dictionary_attack) using names would be highly effective.

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
> Since we got a Gira Home Server, I thought it would be nice if I could control devices in our house with a simple programming interface.
> […]
> While testing, I realized you don't need the password hash to change device states.
> If you use a proper username and answer to the password hash request with a random 32 chars long string made up by hexadecimal digits, you are still logged in. In this case, you don't get notified about state changes of devices but you can still change device states. […]
>
> I took the GitHub repository offline, but I wanted to inform you about the problem, so you can fix the flaw.
>
> Sincerely,  
> Marcel, a 17 y/o hobby programmer
>
> PS: Exploit as zip: https://drive.google.com/███████

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

To 17-year-old me, this sounded promising!
A fix is already in the works, so it's probably okay to lean back and relax.

---

About half a year later, the bug still existed.
Naturally, in May 2018, I contacted Gira again: (translated from German)

> Hello,
>
> pretty much half a year ago I reported a critical security vulnerability in the Gira HomeServer 4 software (Gira message ██████), which allows attackers to turn off alarm systems if port forwarding is activated in the router's settings (for us, this was enabled by default).
> I have been assured that you are working on new software that supports encryption and closes this gaping security vulnerability.
> Christmas has now passed, I turned 18 and graduated from high school. Since I now have free time again, I wanted to find out how the development of the new software is going. 
>
> Greetings,  
> Marcel

The next day, a Gira spokesperson called me (the contact form asked for the phone number) and personally assured me that the software will be published "in next year" (which would mean 2019).

---

Fast-forward to 2021.
So much has happened: I moved to Potsdam, started studying, finished my Bachelor of Science in IT-Systems Engineering.
Recently, I visited my family and while I was at home, I checked if the security flaw is still present – of course it is.
Because of Corona, I have some spare time right now, so let's get to it:

> TODO: Write mail.

Let's see how this goes.
