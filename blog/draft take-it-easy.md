# Take It Easy

## Optimizing a board game

Recently, we spent a couple of family evenings playing Take It Easy, a board game, where every player arranges hexagons (aka [bestagons](https://www.youtube.com/watch?v=thOifuHs6eY)) in a big hexagonal board:

```embed
<style>
tile-wrapper {
  position: relative;
  display: inline-block;
  margin: 0;
  line-height: 1;
  filter: drop-shadow(0 5px 0 #00000021);
  vertical-align: top;
}
tile-box {
  position: relative;
  display: inline-block;
  height: 64px;
  aspect-ratio: 1/cos(30deg);
  clip-path: polygon(50% -50%,100% 50%, 50% 150%, 0 50%);
  cursor: grab;
  user-select: none;
  touch-action: none;
  background: #2b1046;
  overflow: hidden;
  font-size: 18px;
}
tile-box:active { cursor: grabbing; }
tile-line {
  display: inline-block;
  position: absolute;
  height: 70px;
  width: 10px;
}
.line-a { top: 0px; left: 32px; }
.line-b { top: -2px; left: 32px; rotate: 60deg; }
.line-c { top: -2px; left: 32px; rotate: -60deg; }
.line-1 { background: #939696; }
.line-2 { background: #b99c70; }
.line-3 { background: #d73e62; }
.line-4 { background: #56c2b7; }
.line-5 { background: #028181; }
.line-6 { background: #c40404; }
.line-7 { background: #199c19; }
.line-8 { background: #ed8c05; }
.line-9 { background: #ceaf08; }
tile-text { position: absolute; color: white; font-weight: bold; }
.tile-text-a { top: 4px; left: 0; width: 74px; text-align: center; }
.tile-text-b { top: 34px; left: 13px; text-align: left; }
.tile-text-c { top: 34px; right: 13px; text-align: right; }
tiles-container { display: block; text-align: center }
tile-container {
  position: relative;
  display: inline-block;
  width: 100px;
  height: 68px;
  text-align: center;
  vertical-align: top;
}
tile-board {
  --bg-1: #f97191; --bg: white; --box-bg: #fffbed; --fg: black;
  position: relative;
  display: inline-block;
  width: 324px;
  height: 352px;
}
tile-hole {
  position: absolute;
  display: inline-block;
  height: 64px;
  aspect-ratio: 1/cos(30deg);
  clip-path: polygon(50% -50%,100% 50%, 50% 150%, 0 50%);
  background: #fff0bb;
  overflow: hidden;
  text-align: center;
  font-size: larger;
  color: #c5a73e;
  line-height: 64px;
}
.hole-0 { left: 0; top: 72px }
.hole-1 { left: 0; top: calc(2*72px) }
.hole-2 { left: 0; top: calc(3*72px) }
.hole-3 { left: calc(72px*0.75/cos(30deg)); top: calc(0.5*72px) }
.hole-4 { left: calc(72px*0.75/cos(30deg)); top: calc(1.5*72px) }
.hole-5 { left: calc(72px*0.75/cos(30deg)); top: calc(2.5*72px) }
.hole-6 { left: calc(72px*0.75/cos(30deg)); top: calc(3.5*72px) }
.hole-7 { left: calc(72px*1.5/cos(30deg)); top: 0 }
.hole-8 { left: calc(72px*1.5/cos(30deg)); top: calc(1*72px) }
.hole-9 { left: calc(72px*1.5/cos(30deg)); top: calc(2*72px) }
.hole-10 { left: calc(72px*1.5/cos(30deg)); top: calc(3*72px) }
.hole-11 { left: calc(72px*1.5/cos(30deg)); top: calc(4*72px) }
.hole-12 { left: calc(72px*2.25/cos(30deg)); top: calc(0.5*72px) }
.hole-13 { left: calc(72px*2.25/cos(30deg)); top: calc(1.5*72px) }
.hole-14 { left: calc(72px*2.25/cos(30deg)); top: calc(2.5*72px) }
.hole-15 { left: calc(72px*2.25/cos(30deg)); top: calc(3.5*72px) }
.hole-16 { left: calc(72px*3/cos(30deg)); top: calc(1*72px) }
.hole-17 { left: calc(72px*3/cos(30deg)); top: calc(2*72px) }
.hole-18 { left: calc(72px*3/cos(30deg)); top: calc(3*72px) }
board-line { position: absolute; width: 10px; }
.line-0-1-2 { top: 64px; left: 32px; height: 224px }
.line-3-4-5-6 { top: 28px; left: 94.5px; height: 298px }
.line-7-8-9-10-11 { top: -10px; left: 156.5px; height: 372px }
.line-12-13-14-15 { top: 28px; left: 219.5px; height: 298px }
.line-16-17-18 { top: 64px; left: 281.5px; height: 224px }
.line-0-3-7 { rotate:60deg; top: -43px; left: 95px; height: 224px }
.line-1-4-8-12 { rotate:60deg; top: -26px; left: 126px; height: 298px }
.line-2-5-9-13-16 { rotate:60deg; top: -9.5px; left: 158px; height: 372px }
.line-6-10-14-17 { rotate:60deg; top: 82px; left: 188px; height: 298px }
.line-11-15-18 { rotate:60deg; top: 173px; left: 219px; height: 224px }
.line-2-6-11 { rotate:-60deg; top: 173px; left: 94px; height: 224px }
.line-1-5-10-15 { rotate:-60deg; top: 82px; left: 125px; height: 298px }
.line-0-4-9-14-18 { rotate:-60deg; top: -9px; left: 156px; height: 372px }
.line-3-8-13-17 { rotate:-60deg; top: -26px; left: 187px; height: 298px }
.line-7-12-16 { rotate:-60deg; top: -43px; left: 218px; height: 224px }
@media (prefers-color-scheme: dark) {
  tile-wrapper { filter: drop-shadow(0 5px 0 #ffffff21) }
  tile-box { background: #695d74 }
  tile-hole { background: #160f19; color: #b26dae }
}
</style>

<script>
function tile(a, b, c) { return { a, b, c } }
function tileFromStr(str) { return tile(str[0] - '0', str[1] - '0', str[2] - '0') }
function tileToStr(tile) { return `${tile.a}${tile.b}${tile.c}` }

var allHoles = [];
// Creates a new tile DOM node. If initialHole is set, makes it a child of that
// hole and returns null. If initial hole is not set, returns the DOM node and
// the caller is responsible to attach it to the DOM.
function createTileElement(tile, initialHole) {
  let owningHole = initialHole;
  const wrapper = document.createElement("tile-wrapper");
  const box = document.createElement("tile-box");
  const aLine = document.createElement(`tile-line`);
  const bLine = document.createElement(`tile-line`);
  const cLine = document.createElement(`tile-line`);
  const aText = document.createElement(`tile-text`);
  const bText = document.createElement(`tile-text`);
  const cText = document.createElement(`tile-text`);
  wrapper.appendChild(box);
  box.appendChild(bLine);
  box.appendChild(cLine);
  box.appendChild(aLine);
  box.appendChild(bText);
  box.appendChild(cText);
  box.appendChild(aText);
  box.addEventListener("pointerdown", (e) => {
    console.log(`Dragging ${tileToStr(tile)}`);
    if (owningHole) {
      owningHole.onLeave(tile);
      owningHole = null;
    }
    const pointerId = e.pointerId;
    box.setPointerCapture(pointerId);
    document.body.style.userSelect = 'none';
    document.body.style.webkitUserSelect = 'none';
    const rect = wrapper.getBoundingClientRect();
    let startX = e.clientX - rect.left;
    let startY = e.clientY - rect.top;
    document.body.appendChild(wrapper);
    wrapper.style.position = 'fixed';
    wrapper.style.margin = '0';
    wrapper.style.left = `${rect.left}px`;
    wrapper.style.top = `${rect.top}px`;
    const move = (e) => {
      wrapper.style.left = `${e.clientX - startX}px`;
      wrapper.style.top = `${e.clientY - startY}px`;
    };
    document.addEventListener("pointermove", move);
    document.addEventListener("pointerup", (e) => {
      console.log(`Dragged ${tileToStr(tile)}`);
      box.releasePointerCapture(pointerId);
      document.body.style.userSelect = 'auto';
      document.body.style.webkitUserSelect = 'auto';
      document.removeEventListener("pointermove", move);
      const x = e.clientX - startX + window.scrollX;
      const y = e.clientY - startY + window.scrollY;
      wrapper.style.position = 'absolute';
      wrapper.style.left = `${x}px`;
      wrapper.style.top = `${y}px`;
      console.log(`Dropped tile at ${x} ${y}. ${allHoles.length} holes exist.`);
      for (const hole of allHoles) {
        const element = hole.element;
        const tileRect = element.getBoundingClientRect();
        const tileX = tileRect.left + window.scrollX;
        const tileY = tileRect.top + window.scrollY;
        if (Math.abs(x - tileX) <= 32 && Math.abs(y - tileY) <= 32) {
          if (hole.onEnter(tile)) {
            element.appendChild(wrapper);
            wrapper.style.left = '0';
            wrapper.style.top = '0';
            owningHole = hole;
          }
        }
      }
    }, { once: true });
  });
  aLine.classList.add("line-a");
  bLine.classList.add("line-b");
  cLine.classList.add("line-c");
  aLine.classList.add(`line-${tile.a}`);
  bLine.classList.add(`line-${tile.b}`);
  cLine.classList.add(`line-${tile.c}`);
  aText.innerHTML = tile.a.toString();
  bText.innerHTML = tile.b.toString();
  cText.innerHTML = tile.c.toString();
  aText.classList.add("tile-text-a");
  bText.classList.add("tile-text-b");
  cText.classList.add("tile-text-c");
  if (owningHole) {
    owningHole.element.appendChild(wrapper);
    wrapper.style.position = 'absolute';
    wrapper.style.left = '0';
    wrapper.style.top = '0';
  } else {
    return wrapper;
  }
}
</script>

<tiles-container>
  <tile-board>
    <board-line id="foo-0-1-2" class="line-0-1-2 line-1"></board-line>
    <board-line id="foo-3-4-5-6" class="line-3-4-5-6 line-9"></board-line>
    <board-line id="foo-7-8-9-10-11" class="line-7-8-9-10-11 line-1"></board-line>
    <board-line id="foo-12-13-14-15" class="line-12-13-14-15 line-5"></board-line>
    <board-line id="foo-16-17-18" class="line-16-17-18 line-9"></board-line>
    <board-line id="foo-0-3-7" class="line-0-3-7 line-7"></board-line>
    <board-line id="foo-1-4-8-12" class="line-1-4-8-12 line-6"></board-line>
    <board-line id="foo-2-5-9-13-16" class="line-2-5-9-13-16 line-2"></board-line>
    <board-line id="foo-6-10-14-17" class="line-6-10-14-17 line-6"></board-line>
    <board-line id="foo-11-15-18" class="line-11-15-18 line-7"></board-line>
    <board-line id="foo-2-6-11" class="line-2-6-11 line-7"></board-line>
    <board-line id="foo-1-5-10-15" class="line-1-5-10-15 line-8"></board-line>
    <board-line id="foo-0-4-9-14-18" class="line-0-4-9-14-18 line-8"></board-line>
    <board-line id="foo-3-8-13-17" class="line-3-8-13-17 line-2"></board-line>
    <board-line id="foo-7-12-16" class="line-7-12-16 line-3"></board-line>
    <tile-hole id="foo0" class="hole-0">A</tile-hole>
    <tile-hole id="foo1" class="hole-1">B</tile-hole>
    <tile-hole id="foo2" class="hole-2">C</tile-hole>
    <tile-hole id="foo3" class="hole-3">D</tile-hole>
    <tile-hole id="foo4" class="hole-4">E</tile-hole>
    <tile-hole id="foo5" class="hole-5">F</tile-hole>
    <tile-hole id="foo6" class="hole-6">G</tile-hole>
    <tile-hole id="foo7" class="hole-7">H</tile-hole>
    <tile-hole id="foo8" class="hole-8">I</tile-hole>
    <tile-hole id="foo9" class="hole-9">J</tile-hole>
    <tile-hole id="foo10" class="hole-10">K</tile-hole>
    <tile-hole id="foo11" class="hole-11">L</tile-hole>
    <tile-hole id="foo12" class="hole-12">M</tile-hole>
    <tile-hole id="foo13" class="hole-13">N</tile-hole>
    <tile-hole id="foo14" class="hole-14">O</tile-hole>
    <tile-hole id="foo15" class="hole-15">P</tile-hole>
    <tile-hole id="foo16" class="hole-16">Q</tile-hole>
    <tile-hole id="foo17" class="hole-17">R</tile-hole>
    <tile-hole id="foo18" class="hole-18">S</tile-hole>
  </tile-board>
</tiles-container>
<noscript>You need JavaScript for this to work.</noscript>
```

Each player starts with an empty grid of holes that they need to fill with small hexagon tiles.
One player randomly picks from a pile of turned-over hexagons and announces its digits.
The other players have to go through their own piles and find the hexagon.
Then, everyone places the tile into their grid.
You are not allowed to rotate tiles.

Your goal is to build lines of digits that go all the way from one end to the other without being interrupted by other digits (the colors are only there to help you identify the digits).
The game ends when the entire grid is filled.
Because there are 27 tiles, but only 19 spots, some tiles will not be chosen.

```embed
<p id="scoresParagraph">
Finally, you calculate scores:
Every digit on a finished line line awards you points.
</p>
```

```embed
<script>
let foo = [
  tileFromStr("178"), tileFromStr("164"), tileFromStr("124"),
  tileFromStr("974"), tileFromStr("923"), tileFromStr("968"), tileFromStr("963"),
  tileFromStr("173"), tileFromStr("564"), tileFromStr("123"), tileFromStr("168"), tileFromStr("174"),
  tileFromStr("528"), tileFromStr("524"), tileFromStr("563"), tileFromStr("578"),
  tileFromStr("973"), tileFromStr("964"), tileFromStr("574"),
];
function updateFoo() {
  let fooLines = [];
  for (const combination of [
    { matters: [1, 5, 9], holes: [0, 1, 2] },
    { matters: [1, 5, 9], holes: [3, 4, 5, 6] },
    { matters: [1, 5, 9], holes: [7, 8, 9, 10, 11] },
    { matters: [1, 5, 9], holes: [12, 13, 14, 15] },
    { matters: [1, 5, 9], holes: [16, 17, 18] },
    { matters: [2, 6, 7], holes: [0, 3, 7] },
    { matters: [2, 6, 7], holes: [1, 4, 8, 12] },
    { matters: [2, 6, 7], holes: [2, 5, 9, 13, 16] },
    { matters: [2, 6, 7], holes: [6, 10, 14, 17] },
    { matters: [2, 6, 7], holes: [11, 15, 18] },
    { matters: [3, 4, 8], holes: [2, 6, 11] },
    { matters: [3, 4, 8], holes: [1, 5, 10, 15] },
    { matters: [3, 4, 8], holes: [0, 4, 9, 14, 18] },
    { matters: [3, 4, 8], holes: [3, 8, 13, 17] },
    { matters: [3, 4, 8], holes: [7, 12, 16] },
  ]) {
    const line = document.getElementById(`foo-${combination.holes.join("-")}`);
    for (let i = 0; i <= 9; i++) line.classList.remove(`line-${i}`);
    for (const digit of combination.matters) {
      let allMatch = true;
      for (const hole of combination.holes) {
        const tile = foo[hole];
        if (tile) {
          if (tile.a == digit || tile.b == digit || tile.c == digit) {
            // matches
          } else {
            allMatch = false;
          }
        } else {
          allMatch = false;
        }
      }
      if (allMatch) {
        line.classList.add(`line-${digit}`);
        fooLines.push({ digit, length: combination.holes.length });
      }
    }
  }
  function amountToStr(amount) {
    return (amount == 3) ? "three" : (amount == 4) ? "four" : "five";
  }
  let text = "Finally, you calculate scores: "
    + "Every digit on a finished line line awards you points. "
    + "In the grid above, ";
  if (fooLines.length == 0) {
    text += "there are no lines, so the score is 0.";
  } else if (fooLines.length == 1) {
    const len = fooLines[0].length;
    const digit = fooLines[0].digit;
    text += `there is a line of ${amountToStr(len)} ${digit}s, resulting in a score of ${len * digit} points.`
  } else {
    let score = 0;
    text += "there are lines of";
    fooLines.sort((a, b) => a.digit - b.digit);
    for (let i = 0; i < fooLines.length; i++) {
      const line = fooLines[i];
      const len = line.length;
      const digit = line.digit;
      text += (i == fooLines.length - 1) ? (fooLines.length == 2 ? " and " : ", and ") : (i > 0) ? ", " : " ";
      if (i > 0 && digit == fooLines[i - 1].digit) text += "another ";
      text += (len == 3) ? "three" : (len == 4) ? "four" : "five";
      text += ` ${digit}s (${digit * len} points)`;
      score += digit * len;
    }
    text += `. This results in a total of ${score} points. The board above is interactive, so you can get a feel for how this works.`;
  }
  scoresParagraph.innerHTML = text;
}
for (let i = 0; i < 19; i++) {
  const hole = {
    element: document.getElementById(`foo${i}`),
    onEnter: (tile) => {
      if (foo[i]) return false;
      console.log(`Tile ${tileToStr(tile)} entered hole ${i}.`);
      foo[i] = tile;
      updateFoo();
      return true;
    },
    onLeave: () => {
      console.log(`Tile left hole ${i}.`);
      foo[i] = null;
      updateFoo();
    },
  };
  createTileElement(foo[i], hole);
  allHoles.push(hole);
}
updateFoo();
</script>
```

Curiously, the game manages to engange players even though there is no back-and-forth, no interaction, and everyone just tries to optimize their board in isolation.
But there is a lot of hoping that particular tiles get chosen, of complaints when they don't.
Naturally, I wondered how computers could help visualize the probabilities of choices.

```zig
const Tile = u16;
const Game = [19]Tile;
```

I decided to model the game in [Zig](zig), representing the board as a 19-tile array.
Every tiles is a 2-byte numbers using a many-hot-encoding:
A tile with a 1, 2, and 4 becomes `text:0000010110` (a number where the bits at indices 1, 2, and 4 are set).
The lowest bit is unused, but that makes the math easier later on.

```embed
<tiles-container>
  <tile-container style="width: 235px; height: 100px">
    <div style="position: absolute; top: 30px;">
      <script>
      document.currentScript.parentNode.appendChild(createTileElement(tileFromStr("928"), null));
      </script>
    </div>
    <div style="position: absolute; top: 9; left: 70px">
      <script>
      document.currentScript.parentNode.appendChild(createTileElement(tileFromStr("978"), null));
      </script>
    </div>
    <div style="position: absolute; top: 10px; left: 160px">
      <script>
      document.currentScript.parentNode.appendChild(createTileElement(tileFromStr("923"), null));
      </script>
    </div>
  </tile-container>
</tiles-container>
```

What math, you wonder?
By bitwise-and-ing tiles together, we get a number that only has 1s in the places that occur in all tiles.
To illustrate how to use that to calculate the score, let's look at a single vertical column:

```embed
<tiles-container>
  <tile-board style="width: 75px; height: 208px">
    <tile-hole id="bar0" style="left: 0; top: 0">0</tile-hole>
    <tile-hole id="bar1" style="left: 0; top: 72px">1</tile-hole>
    <tile-hole id="bar2" style="left: 0; top: calc(2*72px)">2</tile-hole>
  </tile-board>
</tiles-container>
```

We can calculate a mask for this column like this:

```embed
<pre><code id="maskCalculation">
mask = game[0] & game[1] & game[2]
     = 0000101100 (523)
     & 0101100000 (568)
     & 0110100000 (578)
     = 0000100000
</code></pre>
```

Also try replacing the tiles with the 9s above!

We can then test this mask for the numbers that can occur along the vertical axis (1, 5, 9) by shifting it to the right and and-ing with 1 (`zig:mask >> digit & 1`).
This will result in 1 for digits that are present in all tiles, and 0 otherwise.
Multiply that with the digit and the length of the axis and you have the score:

```embed
<pre><code id="scoreCalculation">
score =   (mask >> 1 & 1) * 1 * 3
        + (mask >> 5 & 1) * 5 * 3
        + (mask >> 9 & 1) * 9 * 3
      =   ([mask] >> 1 & 1) * 3
        + ([mask] >> 5 & 1) * 15
        + ([mask] >> 9 & 1) * 27
      =   ([mask >> 1] & 1) * 3
        + ([mask >> 5] & 1) * 15
        + ([mask >> 9] & 1) * 27
      = [mask >> 1 & 1] * 3 + [mask >> 5 & 1] * 15 + [mask >> 9 & 1] * 27
      = [(mask >> 1 & 1) * 3 + (mask >> 5 & 1) * 15 + (mask >> 9 & 1) * 27]
</code></pre>
```

```embed
<script>
let bar = [tileFromStr("564"), tileFromStr("528"), tileFromStr("524")];
function tileToNumber(tile) {
  if (tile) {
    return 0
      + (tile.a == 1 ? (1 << 1) : 0)
      + (tile.b == 2 ? (1 << 2) : 0)
      + (tile.c == 3 ? (1 << 3) : 0)
      + (tile.c == 4 ? (1 << 4) : 0)
      + (tile.a == 5 ? (1 << 5) : 0)
      + (tile.b == 6 ? (1 << 6) : 0)
      + (tile.b == 7 ? (1 << 7) : 0)
      + (tile.c == 8 ? (1 << 8) : 0)
      + (tile.a == 9 ? (1 << 9) : 0);
  } else {
    return 0;
  }
}
function numberToBitStr(number) {
  return "00000"
    + (number >> 9 & 1)
    + (number >> 8 & 1)
    + (number >> 7 & 1)
    + (number >> 6 & 1)
    + (number >> 5 & 1)
    + (number >> 4 & 1)
    + (number >> 3 & 1)
    + (number >> 2 & 1)
    + (number >> 1 & 1)
    + (number >> 0 & 1);
}
function updateBar() {
  const mask = tileToNumber(bar[0]) & tileToNumber(bar[1]) & tileToNumber(bar[2]);
  maskCalculation.innerHTML = ""
    + "mask =   game[0] & game[1] & game[2]\n"
    + `     =   ${numberToBitStr(tileToNumber(bar[0]))} (${(bar[0]) ? tileToStr(bar[0]) : "no tile"})\n`
    + `       & ${numberToBitStr(tileToNumber(bar[1]))} (${(bar[1]) ? tileToStr(bar[1]) : "no tile"})\n`
    + `       & ${numberToBitStr(tileToNumber(bar[2]))} (${(bar[2]) ? tileToStr(bar[2]) : "no tile"})\n`
    + `     =   ${numberToBitStr(mask)}`;
  scoreCalculation.innerHTML = ""
    + `score =   (mask >> 1 & 1) * 1 * 3\n`
    + `        + (mask >> 5 & 1) * 5 * 3\n`
    + `        + (mask >> 9 & 1) * 9 * 3\n`
    + `      =   (${numberToBitStr(mask)} >> 1 & 1) * 3\n`
    + `        + (${numberToBitStr(mask)} >> 5 & 1) * 15\n`
    + `        + (${numberToBitStr(mask)} >> 9 & 1) * 27\n`
    + `      =   (${numberToBitStr(mask >> 1)} & 1) * 3\n`
    + `        + (${numberToBitStr(mask >> 5)} & 1) * 15\n`
    + `        + (${numberToBitStr(mask >> 9)} & 1) * 27\n`
    + `      = ${mask >> 1 & 1} * 3 + ${mask >> 5 & 1} * 15 + ${mask >> 9 & 1} * 27\n`
    + `      = ${(mask >> 1 & 1) * 3 + (mask >> 5 & 1) * 15 + (mask >> 9 & 1) * 27}`;
}
for (let i = 0; i < 3; i++) {
  const hole = {
    element: document.getElementById(`bar${i}`),
    onEnter: (tile) => {
      if (bar[i]) return false;
      console.log(`Tile ${tileToStr(tile)} entered hole ${i}.`);
      bar[i] = tile;
      updateBar();
      return true;
    },
    onLeave: () => {
      console.log(`Tile left hole ${i}.`);
      bar[i] = null;
      updateBar();
    },
  };
  createTileElement(bar[i], hole);
  allHoles.push(hole);
}
updateBar();
</script>
```

This bitwise-shifting and bitwise-and-ing might seem unnecessarily complicated, but a neat property is that it doesn't use any control flow structures (`zig:if`, `zig:for`, etc.).
I can do the entire score calculation as a single expression:

```zig
fn score(game: Game) usize {
    return
          (game[0] & game[1] & game[2] >> 1 & 1) * 3 * 1
        + (game[0] & game[1] & game[2] >> 5 & 1) * 3 * 5
        + (game[0] & game[1] & game[2] >> 9 & 1) * 3 * 9
        + (game[3] & game[4] & game[5] & game[6] >> 1 & 1) * 4 * 1
        + (game[3] & game[4] & game[5] & game[6] >> 5 & 1) * 4 * 5
        + (game[3] & game[4] & game[5] & game[6] >> 9 & 1) * 4 * 9
        + ...;
}
```

That is a lot of code, but it's super fast!
Modern CPUs can get slowed down by branches (`zig:if`s) because it breaks pipelining.
My code is branchless and bitwise operations are really cheap.

To tidy things up, what I actually ended up doing is to use Zig's `zig:inline for`, a compile-time `zig:for` loop that is guaranteed to be unrolled by the compiler:

```zig
const a_axes = .{ .{0, 1, 2}, .{3, 4, 5, 6}, ... };
const b_axes = .{ .{0, 3, 7}, .{1, 4, 8, 12}, ... };
const c_axes = .{ .{7, 12, 16}, .{3, 8, 13, 17}, ... };
const a_digits = .{1, 5, 9};
const b_digits = .{2, 6, 7};
const c_digits = .{3, 4, 8};

fn score(game: Game) usize {
    var total: usize = 0;
    inline for (
        .{a_axes, b_axes, c_axes},
        .{a_digits, b_digits, c_digits},
    ) |axes, digits| {
        inline for (axes) |axis| {
            var mask: Tile = 0b1111111111;
            inline for (axis) |tile| {
                mask &= game[tile];
            }
            inline for (digits) |digit| {
                total += (mask >> digit & 1) * axis.len * digit;
            }
        }
    }
    return total;
}
```

## Computing Probabilities

Now that we can calculate the score really fast, the next step is to find good positions for tiles.
Given a game and a tile, where should we place it?
(You can drag the tile around in your browser. Any tile on this website, really.)

```embed
<script>
function board() {
  document.write("board");
}
board([
  tile("123"),
  tile("124"),
  tile("578"),
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
  null,
])
</script>
```

One approach is to find the best position using brute-force:
Try placing the tile at all three spots, go through every set of tiles that could possibly still be chosen, and note down the score you can get if you place them optimally.
This resulting scores can give you an intuition about the risk.
For example, 
If we place the tile at C, depending on what other tiles we get, we can achieve scores of X, Y, Z.

Ugh, that's a lot of numbers.
Plot it!

```embed
<script>

</script>
```

TODO: plot

## What's one more simulation among friends?

For each possible placement, we get each possible subset of remaining tiles, and then for each possible ordering of those, we compute the score.
Hmm.
That works if a handful spots are free, but the number of positions to check grows exponentially as more spots are free.

But we can approximate that!
I wrote a simple simulation that picks random tiles, places them at positions where they don't actively make things worse (they don't block valuable lines), and uses the try-all-orderings approach once we get to a manageable number of free spots.
By doing that a million times, I can still get a rough estimate of how good a position is for a tile.

## The tool you've been waiting for

May I present, my Take It Easy probability visualizer (tm):

TODO: tool

## A Take It Easy AI

The story doesn't end there.
Oh nonono.

Next, let's automatically place tiles based on probability distributions.






## The human side of it all

I read in [TODO: a book](foo) that a human is not a homo oeconomicus.
You might never consider drinking a bottle of wine worth 100€, but if you got a bottle for cheap and the price rose to 100€ over time, you are more easily tempted to drink it rather than sell it.
We value not losing 100€ more than we value winning 100€.

Maybe this explains the difference between the AI's and human's behavior.
And it makes sense.
A single play-through takes, what, twenty minutes?
You don't want to miserably loose in all three of today's games just because every time there was a 10% chance you won by a huge margin.

Even though the game is about optimizing the score, we don't optimize our score.

