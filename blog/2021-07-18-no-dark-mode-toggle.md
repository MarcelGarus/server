# Why This Website Has No Dark-Mode Toggle

Maybe you're wondering why there's no dark-mode toggle on this website.
Well, I have a short story for you:

I recently streamed music from my laptop to a bluetooth speaker and wanted to increase the volume.
Naturally, I tried increasing the volume on the computer. Nope, it's already at the maximum.
The bluetooth speaker has volume buttons! Gotta try those! But except from the music being interrupted by one of these awful beeps, nothing happens.
Oh, I know! Windows has an option to adjust the volume of individual programs that I use quite frequently.
I must have lowered the volume there. But nada.
Finally, I checked *in the music app itself* and fair enough, there's the volume slider.

What happened here?

--snip--

In the end, this comes down to misaligned incentives.
The crazy thing is that all these settings make sense individually, but result in a worse UX overall:

* As a music app, you don't want to force users to dig into OS menus to adjust the music volume without changing the volume of other apps.
  Lots of users are probably not familiar with these settings and some operating systems might not even have the option.
  If users are already in your app and want their music to be a little louder, why not simply add the option to do so right there?
* As an OS, it certainly makes sense to allow users to adjust the operating system volume.
  If you want your whole computer to be louder, that should definitely be possible to do – so, add a volume slider.
* Some apps don't have individual volume controls and might even be hostile towards the user: If you're on a videocall with friends and an ad interrupts the music you listen by the side, it should certainly possible for users to control the volume of the music program, even if the program doesn't want to allow that itself.
* And finally, speakers certainly should have volume buttons, that's such a fundamental part of playing music there.

Putting it all together, basic functionality gets re-implementated in a number of layers.
I believe there should be *one way* to do these basic operations.

Identifying misaligned incentives and calling it a day is the easy way out.
But is there a way we software developers can actually achieve good UX in the long term?

Yes!
Just ask yourself: Are you a part of a larger system or *are* you the system?

If you *are* the system, let users make choices on a per-app basis.
In turn, this enables these apps to omit the controls completely.

As a part of the larger system, it shouldn't be your job to offer the controls.
I know there sometimes are business incentives to offer the controls, because "why not".
But if you can afford it, *please* don't offer the controls.
Or at least only offer it on platforms that don't have this functionality natively.
Yes, some users will complain. Make them learn how to do it in the system, or open issues on systems that don't provide that functionality.

By the way, applying this principle results in some interesting solutions:

* Music apps shouldn't have volume controls. That's the task of the OS.
* Speakers shouldn't have volume controls. That's the task of the connected device.
* Browsers should have the option to apply dark-mode only to certain sites.
* And websites shouldn't have a dark-mode toggle.
