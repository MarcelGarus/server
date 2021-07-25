# Why This Website Has No Dark-Mode Toggle

Maybe you're wondering why there's no dark-mode toggle on this website.

**Short answer**:
Misaligned incentives between layers of software result in an overall subpar experience.
I want to put pressure on the browser offering to control dark mode of individual websites.

**Long answer**:
I recently streamed music from my laptop to a bluetooth speaker and wanted to increase the volume.
Naturally, I tried increasing the volume on the computer. Nope, it's already at the maximum.
The bluetooth speaker has volume buttons! Gotta try those! But except from the music being interrupted by one of these awful beeps, nothing happens.
Oh, I know! Windows has an option to adjust the volume of individual programs that I use quite frequently.
I must have lowered the volume there. But nada.
Finally, I checked **in the music app itself** and fair enough, there's the volume slider, its knob almost on the left.

What happened here?

--snip--

The crazy thing is that all these options to change the volume make sense individually, but result in a worse UX overall:

* As a music app, you don't want to force users to dig into OS menus to adjust the music volume without changing the volume of other apps.
  Lots of users are probably not familiar with these settings and some operating systems might not even have the option.
  If users are already in your app and want their music to be a little louder, why not simply add the option to do so right there?
* As an OS, it definitely makes sense to allow users to adjust the operating system volume.
  If you want your whole computer to be louder, that should be possible to do – so, add a volume slider.
* Some apps don't have individual volume controls and might even be hostile towards the user: If you're on a videocall with friends and an ad interrupts the music you listen to by the side, you should be able to control the volume of the music program, even if it doesn't want to allow that itself.
* And finally, shouldn't speakers have volume buttons? Changing the volume seems like such a fundamental part of listening to music on them.

Putting it all together, basic functionality gets re-implementated in a number of layers.
I believe there should be **one way** to do these basic operations.

Identifying misaligned incentives and calling it a day is the easy way out.
But is there a way we software developers can actually **achieve good UX in the long term**?

Yes!
Just ask yourself: **Are you a part of a larger system or *are* you the system?**

If you *are* the system, let users make choices on a per-app basis.
In turn, this enables these apps to omit the controls completely.

As a part of the larger system, it shouldn't be your job to offer the controls.
I know there sometimes are business incentives to offer the controls, because it helps some users while not having a direct negative effect on others.
But if you can afford it, *please* don't offer the controls.
Or at least only offer it on platforms that don't have this functionality natively.
Yes, some users will complain. Make them learn how to do it in the system, or open issues on systems that don't provide that functionality.

If you're both part of a larger system as well as a system itself, don't offer the choice to change all apps, but only to individual ones.
For example, a web browser could have the address bar and tabs UI as well as the default dark-mode of websites be controlled by the system settings, but offer a separate dark-mode setting override for individual websites.

Applying this principle results in some interesting solutions:

* Music apps shouldn't have volume controls. That's the task of the OS.
* Speakers shouldn't have volume buttons. Controlling the volume is the task of the connected device.
* Operatings systems should offer dark-mode for only some of the apps.
* Browsers should have the option to apply dark-mode only to some sites.
* And websites shouldn't have a dark-mode toggle.
