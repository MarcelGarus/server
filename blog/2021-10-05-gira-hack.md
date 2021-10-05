# Hacking the Gira HomeServer

Our home has a smart electric grid that can be controlled using [KNX](https://www.knx.org).
A *Gira HomeServer* connects to the physical KNX wires as well as to our WiFi network and enables us to control all devices using an app.
In [my previous article](/gira), I looked at how the Gira HomeServer communicates with the app.
As a reminder, this is how the authentication phase works:

![comic](https://github.com/marcelgarus/server/raw/main/blog/images/gira-comic.png)

--snip--

More specifically, the process works like this:

1. The app asks to sign in using a username. (The first three comic boxes.)
2. The server sends a *salt*, which is just a random string of characters (for example `245013621`).
3. The app calculates a mathematical hash function, using the salt, username, and password as inputs: `hash(salt, username, password)` Critically, the hash function is irreversible, so having the result doesn't allow you to make predictions about the inputs. In the previous article, we already discovered that the app uses the [insecure MD5 hash function](https://en.wikipedia.org/wiki/MD5) for parts of the password hash.
  The app sends the result to the server. The result is pretty long (something like `A335F32830EAFB1534DE46EFEECF1A8C`), so it's unlikely that the app gets this right by chance if it doesn't know the password.
4. The server does the same hash calculation itself and checks if the results match. If they do, the app is authenticated and can control the devices in the home.

This process protects against other devices listening in on the communication – if a malicious device wants to sign in, it's given a different random salt and can't calculate the corresponding hash, because it doesn't know the password.

While developing the app and tinkering with the authentication flow, I discovered a glaring security flaw: The server ignores step 4! Regardless of whether the password hash was correct, the server lets the app control all devices! 🤦

Exploiting this behavior looks like this in the comic version:

![comic](https://github.com/marcelgarus/server/raw/main/blog/images/gira-hack-comic.png)

Note that the server doesn't send anything to the app (like which devices are turned on or off), but it still executes commands from the app.
This is such a simple and preventable bug that I'm alarmed Gira didn't discover it.
It's not even a small oversight, but instead indicates a fundamental flaw in Gira's organizational approach to security.
**A working security process doesn't let something like this happen.**

You own a Gira HomeServer and want to see if you're affected?
On GitHub, I published [a program written in Dart that toggles all devices off](https://github.com/marcelgarus/girahomeserver) (including alarm systems).
It goes without saying that you should please only test this in your own home.

## How bad is it?

Let's take a step back and look at the impact of this hack.
To be able to use it maliciously on other people's homes, a few preconditions need to be met:

* The home needs to have a Gira HomeServer that controls the devices. We have a Gira HomeServer 4 and I don't have access to other models, so I can only confirm that the hack works on that particular model.
* The Gira HomeServer needs vulnerable software. At least our HomeServer's software is still vulnerable in 2021.
* For the hack to succeed without having access to the local network (e.g. WiFi), the HomeServer needs to be accessible from the internet. The company that set up our HomeServer conveniently also enabled port forwarding in our router and registered it at [giradns.com](https://giradns.com), so that we can control devices while not being at home. By scanning the subdomains of `giradns.com`, you can get a list of Gira HomeServers reachable via the public internet.
* As an additional step, you need the username to log in. Usually, usernames are less guarded than passwords, so I assume that many are the same as the subdomain or the family's names. I'm pretty certain that performing a [dictionary attack](https://en.wikipedia.org/wiki/Dictionary_attack) using names would be highly effective.

Once these criteria are met, you can control all devices in the home.
That includes not only lights, but also critical devices, like blinds, thermostats, and alarm systems.
The fact that I can turn off our alarm system without knowing the password is frightening.

To mitigate the risk for us, I turned off port forwarding in our router settings.
Until the bug is fixed, having the HomeServer not being reachable from the public internet is a good idea.

## Communicating with Gira

At this point, it's relatively clear that I need to contact Gira about this.
Thankfully, there's a process for reporting security vulnerabilities: [Responsible disclosure](https://en.wikipedia.org/wiki/Responsible_disclosure).
This process works like this:

1. Contact the responsible company privately.
2. After a sufficient period of time (hopefully after the vulnerability is fixed), make the vulnerability public so that others can learn from it.

So, in November 2017, I contacted Gira using their contact form:

> <span class="secondary">(corrected typography and grammar)</span>
>
> To whom it may concern!
>
> Since we got a Gira HomeServer, I thought it would be nice if I could control devices in our house with a simple programming interface.
> <span class="secondary">[…]</span>
> I realized you don't need the password hash to change device states.
> If you use a proper username and answer to the password hash request with a random 32 character long string of hexadecimal digits, you are still logged in. In this case, you don't get notified about state changes of devices but you can still change device states.
> <span class="secondary">[…]</span>
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
> █████ ████████  
> Advanced Support Hotline Agent

To 17-year-old me, this sounded promising!
A fix is already in the works, so it's probably okay to lean back and relax.

---

About half a year later, the bug still existed.
Naturally, in May 2018, I contacted Gira again:

> <span class="secondary">(translated from German)</span>
>
> Hello,
>
> pretty much half a year ago I reported a critical security vulnerability in the Gira HomeServer 4 software (Gira message ██████), which allows attackers to turn off alarm systems if port forwarding is activated in the router's settings (for us, this was enabled by default).
> I have been assured that you are working on new software that supports encryption and closes this gaping security vulnerability.
>
> Christmas has now passed, I turned 18 and graduated from high school. Since I now have free time again, I wanted to find out how the development of the new software is going. 
>
> Greetings,  
> Marcel

The next day, a Gira spokesperson called me (the contact form had asked for a phone number) and personally assured me that the software will be published "in next year" (which would mean 2019).

---

Fast-forward to 2021.
So much has happened in my life: I moved to Potsdam, started studying, and finished my Bachelor.
Recently, I visited my family and while I was at home, I confirmed that the security flaw is still present.
It's been *multiple years* and there are probably still vulnerable buildings out there – that's unacceptable.
So, I published the [vulnerability exploit on GitHub](https://github.com/marcelgarus/girahomeserver).
Because of Corona, I have some spare time right now, so let's get to it:

> <span class="secondary">(translated from German)</span>
>
> Hello again,
>
> In 2017 I reported a critical security vulnerability in the Gira HomeServer. A proof of concept exploit can be found here: [github.com/marcelgarus/girahomeserver](https://github.com/marcelgarus/girahomeserver)
>
> After my last email, I was assured by phone in mid-2018 that a software update was being worked on, which should then be published "in the new year" (2019).
> It's now 2021, I've finished my bachelor's degree and the problem still exists with our Gira HomeServer. What is the status of the software update? In the meantime, I've written a blog article about the vulnerability that explains the problem in more detail: [mgar.us/gira-hack](https://mgar.us/gira-hack)
>
> Best regards,  
> Marcel
