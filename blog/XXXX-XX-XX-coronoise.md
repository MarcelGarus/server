# Making People Aware of Increased Infection Risk

As you might have noticed, we are currently in a pandemic.
Luckily, there are several steps you can take to protect yourself and others: You can get vaccinated, wear a mask, avoid large groups of people, regularly ventilate when you're indoors with others, etc.
Some of those measures are easier to follow than others – for example, getting vaccinated is a one-off action, but remembering to open the window every few minutes is a lot harder if you're also concentrating on something else, such as working or studying together with others.

In the context of the Sonic Thinking and the Neurodesign lectures at our university, I developed an app that continuously evaluates the current situation with regards to the infection risk and turns the risk into sound.

![architecture](...)

--snip--

The app takes three inputs: the incidence rate in the area, the number of people nearby, and the CO2 level.
You can adjust each of these inputs manually using a slider, or you can check the box on the left to make the app automatically try to choose an appropriate value.

... scenarios

## Incidence Rate

In Germany, the Robert Koch Institute is the most reliable source for infection data.
ArcGIS offers a pretty straightforward API for gathering incidence data.
For example, the URL `...` answers with a JSON object that containing the name, update time, number of habitants, and reported COVID-19 infections for each district.
You can use these information to calculate the incidence rate.

```json
{
  "GEN": "Potsdam",
  "last_update": "05.02.2022, 00:00 Uhr",
  "EWZ": 182112,     // number of people ("Einwohnerzahl")
  "cases7_lk": 3298  // COVID-19 cases in the last 7 days in the district ("Landkreis")
}
```

In this example, the incidence rate in Potsdam is `txt:100000 * cases7_lk / EWZ = 100000 * 3298 / 182112 = 1810.97`.

Theoretically, by setting `returnGeometry=true` in the URL, you also get location information: For every district, you'll also get a list of points that form the outline of that district.
This bumps the API response up to ... MB (larger than the app itself), and deciding in which district you are will be quite a compute-intensive problem to solve, so in the long term, it would probably make sense to outsource this onto a more powerful server that handles all the heavy number-crunching and just allows devices to ask for the incidence rate at a specific location.
For this project, this was definitely out of scope.

## Exposure Notifications

In 2020, Apple and Google published a whitepaper for an Exposure Notification framework. [3]
This is a tool that allows smartphones to keep track of which other smartphones they encounter, so that people can proactively get notified if they had contact with a person that was later found to be infected.

The framework uses Bluetooth Low Energy to discover nearby devices – a technology that is already common smart home devices or Apple Air Tags ....
Although the framework is designed to log which other people you saw, it works mostly anonymous: Rather than exchanging unique, static device IDs, devices generate a completely random ID every 10 – 20 minutes and then broadcast this ID every 200 – 270 milliseconds.
Devices remember which IDs they sent and received in the last two weeks.
If you are tested positive for COVID-19, you can donate your IDs to the public health services, which will publish them so that other people's Exposure Notification apps can compare those against the IDs they saw and alert their users if they find a match.

Exposure Notification apps such as the Corona-Warn-App in Germany let the operating system take care of the whole scanning process:
They just tell the operating system a few IDs to broadcast for the next few minutes and the operating systems takes care of actually broadcasting the IDs and recording the IDs of other devices, reporting them back to the app after some time.
A benefit of the operating system taking care of the low-level work is that the app doesn't have to run in the background all the time, preserving battery life.

For my use case of estimating the number of people around you right now, this approach won't work – instead of a long-term qualitative measurement that is used to answer questions such as "Did I see this person in the last two weeks?", we want to perform time-sensitive quantitative measurements that determine "How many people are there around me right now?"
That's why my app doesn't talk with the operating system's Exposure Notification framework, but instead uses Bluetooth directly to find Exposure Notifications.
Roughly every second, it scans all Bluetooth devices nearby and filters for those that are Exposure Notifications – according to the Exposure Notification specification, these contain a special _service UUID_ of `text:0xFd6F` that is reserved for this purpose.

Finally, I evaluated if the scanning works.
I went to the local supermarket, stayed at home, and took a walk in the forest.
Here are the results:

![graph](...) with horror sound

Don't be fooled: This is a line chart, not a bar chart.
The number of Exposure Notifications scanned really goes up and down like that.
I suppose this is an artifact of Android rate limiting the app or trying to be clever and not actually re-scanning for devices every time the app tells it to, which leads to either the same number of Exposure Notifications being reported multiple consecutive times or none at all being reported.

The ... data series is me shopping in the local supermarket.
If you're wondering about the gap in the data, that's just me turning the phone into standby mode while putting items on the cash register tape – for now, the app has to run in the foreground in order to record data.
That being said, you can see  in the end of the data series that compared to browsing the supermarket, me waiting in line with multiple people actually leads to a higher number of exposure notifications.

The measurements at home report a mostly constant number of notifications.
The small noise could result from some people changing the room that they're in, people walking by on the outside, or just the corruption of some Bluetooth packets that get sent threw a few walls.
This data series recorded at home also shows that the app isn't perfect for measuring the number of people: There's only me in my flat, but because I live in a buzzing student residence, about ... people are reported.
In some cases, it makes sense to not use the automatic estimatino of the number of people: If I'm in a room with a fixed number of people for a long time, I could just manually enter the number of people in the app and benefit from much more accurate data.

For the measurements in the forest, I took a long walk, listened to podcasts, and opened the app in the foreground.
I enabled the sonification, so that I'll hear a sound when some Exposure Notifications are detected.
I met the occasional hiker, but the higher bump was interesting: While I was walking down a narrow path, I heard the sound of Exposure Notifications being recorded. I looked around and, fair enough, two bikers were pulling up from behind. I stepped aside and let them pass through. To me, this showed that even a very simple sonification – like turning Exposure Notifications to piano blimps – can be useful in everyday life: If I hadn't used the app, I wouldn't have heard the bikers approaching and they'd have to ring or yell. 

## CO2 levels

You might be wondering: Why use CO2 levels as an indicator?
Here's a hint:

![humans make co2](...)

Inside, CO2 levels are usually at ...+ ppm; outside they are about ... ppm.
Here, ppm stands for *parts per million*, so a value of 400 ppm corresponds to 400 / 1000000 = ... % of the air being CO2.
Because that extra CO2 mostly comes from exhaled air, lower CO2 levels naturally correlate with measures that protect against COVID-19 infections: increased ventilation, less people, and less kinetic strength (a measure of how much movement occurs). [4]
So, CO2 levels are a great indicator for the airborne COVID-19 infection risk.

This information can be helpful for people:
A group of Japanese researchers visualized a graph of the CO2 concentration during a concert and made about half of the audience feel "much safer."
They strongly expect for this visualization to become part of the standard operational precedure of concerts. [5]

While I do have a CO2 sensor you can manually read, in the context of the lectures, I didn't manage to integrate another CO2 sensor.
For now, the CO2 value needs to be manually entered into the app.

## Sound generation

Let's shift from inputs to outputs!
While visual indicators are nice, they don't proactively notify you of increased infection risks: Although I have the CO2 set up above my computer display, I rarely look at it if I'm deeply focused on some programming task.
Instead, I'm often surprised if I look up and see how high the levels are.

Sound, on the other hand, is the only sense that proactively comes to us and that can't be easily ignored – there's a reason alarm clocks make sound.
My idea was to play some scary, spooky sound that emotionally conveys the dangerousness of infections.
Forgetting to open the window for some time would lead to ominous music playing, just like you're in a horror film!

... Heimerdinger investigated the music of horror movies and found some common factors for creating suspense in sound: sustained high tones, deep drones, annoying repetitive motives, and specific noises such as the howling wind. [6]
I took all of those inspirations, and composed a song with multiple voices.
For that, I connected my electric piano's MIDI output to my iPad and played different instruments through GarageBand:

![screenshot](...)

Here are some single voices:

[listening example](...)

To programatically combine those instruments, I initially tried to have one control loop that turns instruments on and off. This became complicated pretty fast – you have to track which instruments are playing, when they started (some should only end at some pre-defined beats or have a minimum amount of time between consecutive activations).
In the end, I settled on an architecture where instruments have access to some ambient values such as the current beat number and the inputs, but could do whenever they want. This allows me to implement many different types of effects: fade in and out, start and end at predefined beats, start on some beats and then fade out, or randomly start.

Here's how a composed horror music might sound like:

[listening example](...)

> One other tidbit: First, you only hear one piano note, but as soon as the second instrument starts, you realize the first one was actually played off-beat. That makes the music even more disorienting.

## Study

Many papers investigate potential triggers for positive behavior changes in response to the pandemic (such as practicing social distancing or improving hand hygiene).
Among 324 international participants, Harper et al. looked at potential correlations with fear of the virus, moral foundations, and political orientation. They found that the fear of the virus is the primary factor for these behavior changes. [1]
Notably, ... et al. found that people actually comply less when they fear the authorities. [2] This indicates that **fear of the virus itself** primarily motivates people to be careful.

... et al. indentifies **empathy with those most vulnerable to the virus** as another factor for physical distancing and wearing a face mask. Especially above a certain baseline, inspiring empathy in people can motivate them to follow these two important measures, while only giving them plain information had a negligible effect on their behavior. [7]

My hypothesis for this app is as follows:

> While working, employees/students empathize more with the decisions of olleagues to take protective measures against the corona pandemic, if they are acutely made aware of the infection risk via sonic feedback.

To investigate this, I spent a day at the Neurodesign office at the HPI.
After introducing participants to the project, I silently recorded data indicating increased risks (like, the CO2 levels, the number of people in the room, whether they wore masks) and asked them to fill out a questionnaire.
Then, I activated the sonification of the risk level and continued recording risk-relevant data.
Finally, I asked them to fill out the survey again.

Here's a timeline of what happened:

Timeline

Results

Careful interpretation

## Conclusion

This project is long from being finished.
Here are some ideas for future work:

- Improve the inputs: The incidence isn't yet based on the phone's position, but instead hardcoded to Potsdam. The CO2 levels still need to be manually entered.
- Add more sonifications: The risk can span an exponential scale, so there's still lots of room for different sounds and noises to be incorporated. Some instruments could sonify the rate of risk, warning users of rapid risk increases.
- Improve music generation: Sometimes there are small breaks in the music or voices are slightly misaligned relative to each other. This kind of adds to the horrow, but could be better handled by switching to a lower-level sound library.

Working on this project was fun! During the seminar lectures, I learned a lot about Neuroscience, empathy, and sound.

## Sources

[^1]: **Functional Fear Predicts Public Health Compliance in the COVID-19 Pandemic** paper from April 2020 by Craig A. Harper, Liam P. Stchell, Dean Fido & Robert D. Latzman
[^2]: Compliance with COVID-19 Mitigation Measures in the United States
[^3]: Exposure Notification Framework
[^4]: Predictive and retrospective modelling of airborne infection risk using monitored carbon dioxide
[^5]: CO2 concentration visualization for COVID-19 infection prevention in concert halls
[^6]: Music and sound in the horror film & why some modern and avant-garde music lends itself to it so well
[^7]: The emotional path to action: Empathy promotes physical distancing and wearing of face masks during the COVID-19 pandemic

## Afterwards

- title case
- CO2
- Grammarly
- sources
