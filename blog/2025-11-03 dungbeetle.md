topics: programming language design, code

# Dungbeetle

## A tiny computer

```embed
<div id="dungbeetle" style="font-family: monospace; line-height: 1.2">Loading...</div>
<br>
<button onclick="loadExample('intro')">intro.dung</button>
<button onclick="loadExample('edit')">edit.dung</button>
<button onclick="loadExample('jump')">jump.dung</button>
<button onclick="loadExample('move')">move.dung</button>
<button onclick="loadExample('move2')">move2.dung</button>
<button onclick="loadExample('fibonacci')">fibonacci.dung</button>
<button onclick="loadExample('counter')">counter.dung</button>
<button onclick="loadExample('file')">file.dung</button>
<button onclick="loadExample('numtostring')">numtostring.dung</button>
<script>
  let vm = null;

  fetch('files/dungbeetle.wasm')
    .then((response) => response.arrayBuffer())
    .then((bytes) => WebAssembly.instantiate(bytes, {}))
    .then((module) => {
      vm = module;
      loadExample('intro');
    });

  function render() {
    const memory = new Uint8Array(vm.instance.exports.memory.buffer);
    const ptr = vm.instance.exports.render();

    let end = ptr;
    while (memory[end] !== 0) end++;

    const ui = JSON.parse(
      new TextDecoder().decode(memory.subarray(ptr, end))
    );
    console.log(ui);

    const colors = {
      red: '#b62c2d',
      yellow: '#9e7352',
      green: '#479f6e',
      blue: '#0000ff',
      magenta: '#9b51b2',
      cyan: '#48a0b0',
    };

    let out = '';
    for (let line of ui.cells) {
      for (let cell of line) {
        let content = String.fromCharCode(cell.char);
        if (content == ' ') content = '&nbsp;';
        if (content == '<') content = '&lt;';
        if (content == '>') content = '&gt;';
        if (content == '&') content = '&amp;';
        if (content == "'") content = '&apos;';
        if (content == '"') content = '&quot;';

        let backgroundColor = '#2e0d22';
        let foregroundColor = '#ffffff';
        if (cell.style.color) foregroundColor = colors[cell.style.color];
        if (cell.style.chrome) backgroundColor = colors.yellow;
        if (cell.style.reversed) {
          let tmp = backgroundColor;
          backgroundColor = foregroundColor;
          foregroundColor = tmp;
        }
        let style = `color:${foregroundColor};background-color:${backgroundColor};`;
        if (cell.style.underlined) style += `text-decoration: underline;`;

        out += `<span style="${style}">${content}</span>`;
      }
      out += '<br>';
    }
    document.getElementById('dungbeetle').innerHTML = out;
  }

  document.addEventListener('keydown', (event) => {
    let charCode = 0;
    // The wasm module expects specific codes for special keys,
    // which don't map directly to event.key.
    switch (event.key) {
      case 'ArrowUp':
        charCode = 0x41; // 'A'
        break;
      case 'ArrowDown':
        charCode = 0x42; // 'B'
        break;
      case 'ArrowRight':
        charCode = 0x43; // 'C'
        break;
      case 'ArrowLeft':
        charCode = 0x44; // 'D'
        break;
      case 'Backspace':
        charCode = 0x7f;
        break;
      case 'Tab':
        charCode = 0x09;
        break;
      case 'Enter':
        // The zig code doesn't seem to handle Enter, but it's a common CLI input.
        // I'll map it to space for now, which runs an instruction.
        charCode = ' '.charCodeAt(0);
        break;
      default:
        if (event.key.length === 1) {
          charCode = event.key.charCodeAt(0);
        }
        break;
    }

    if (charCode !== 0 && vm) {
      vm.instance.exports.input(charCode);
      render();
    }

    // Prevent default browser action for keys handled by the widget.
    if (
      [
        'ArrowUp',
        'ArrowDown',
        'ArrowLeft',
        'ArrowRight',
        'Tab',
        ' ',
        'Enter',
      ].includes(event.key)
    ) {
      event.preventDefault();
    }
  });

  const examples = {
    intro: `
      05 d0 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      64 75 6e 67 62 65 65 74  6c 65 2e 00 00 00 00 00
      61 00 74 69 6e 79 00 76  6d 2e 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      32 35 36 00 62 79 74 65  73 00 6f 66 00 00 00 00
      6d 65 6d 6f 72 79 00 63  6f 6e 74 61 69 6e 00 00
      69 6e 73 74 72 75 63 74  69 6f 6e 73 00 61 6e 64
      64 61 74 61 2e 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      73 70 61 63 65 00 74 6f  00 73 74 65 70 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      73 6f 6d 65 00 6a 75 6d  70 73 3a 00 00 00 00 00
      05 de 00 00 00 00 00 05  00 00 00 00 00 00 05 d7
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      6e 65 78 74 3a 00 65 64  69 74 2e 64 75 6e 67 00`,
    edit: `
      01 00 00 00 01 00 00 00  01 00 00 00 01 05 00 00
      61 72 72 6f 77 73 00 74  6f 00 6d 6f 76 65 00 00
      63 75 72 73 6f 72 00 61  72 6f 75 6e 64 2e 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      2b 00 74 6f 00 69 6e 63  72 65 61 73 65 2e 00 00
      2d 00 74 6f 00 64 65 63  72 65 61 73 65 2e 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      74 61 62 00 74 6f 00 63  6f 6e 74 69 6e 75 65 00
      75 6e 74 69 6c 00 6e 65  78 74 00 68 61 6c 74 00
      28 63 61 72 65 66 75 6c  29 2e 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      69 6e 66 6f 00 61 62 6f  75 74 00 74 68 65 00 00
      69 6e 73 74 72 75 63 74  69 6f 6e 00 69 73 00 00
      61 74 00 74 68 65 00 62  6f 74 74 6f 6d 2e 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      6e 65 78 74 3a 00 6a 75  6d 70 2e 64 75 6e 67 00`,
    jump: `
      05 b1 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      6a 75 6d 70 00 64 69 72  65 63 74 6c 79 00 74 6f
      74 68 65 00 68 61 6c 74  21 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 68 65 72  65 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 76  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 01  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 01 05 00
      6e 65 78 74 3a 00 6d 6f  76 65 2e 64 75 6e 67 00`,
    move: `
      05 b1 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      69 6e 73 74 72 75 63 74  69 6f 6e 73 00 00 00 00
      61 6c 77 61 79 73 00 74  61 6b 65 00 00 00 00 00
      74 61 72 67 65 74 73 00  61 73 00 74 68 65 00 00
      66 69 72 73 74 00 61 72  67 75 6d 65 6e 74 3a 00
      6d 6f 76 65 00 74 6f 00  66 72 6f 6d 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      66 69 6e 69 73 68 00 74  68 65 00 00 00 00 00 00
      73 65 71 75 65 6e 63 65  21 00 00 00 00 00 00 00
      02 d6 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 37 00 00 00 00
      00 00 33 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 30 31 32 00 34  35 36 00 38 39 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      6e 65 78 74 3a 00 6d 6f  76 65 32 2e 64 75 6e 67`,
    move2: `
      05 b2 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      69 6e 64 69 72 65 63 74  00 6d 6f 76 65 3f 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 77 68 61 74 00 00 00  00 77 68 65 72 65 00 00
      00 00 74 6f 00 00 00 00  74 6f 00 6d 6f 76 65 00
      00 6d 6f 76 65 00 00 00  00 00 69 74 00 00 00 00
      00 00 76 00 00 00 00 00  00 00 00 76 00 00 00 00
      00 00 78 00 00 00 00 00  00 00 00 79 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 02 3f 72 00 00 00  01 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      05 00 00 00 00 00 00 68  61 76 65 00 66 75 6e 21`,
    fibonacci: `
      00 02 0e 0d 02 0f 0e 06  0d 0e 0f 03 00 00 00 01
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00`,
    counter: `
      02 1f 21 02 08 1f 02 1e  fd 09 1e 89 06 1e 20 02
      13 1f 02 fe 3f 04 1f 05  03 63 6c 65 61 72 00 fd
      02 ff 27 02 2f 21 05 30  73 65 74 75 70 fc ff ff
      01 02 2e 2f 05 40 00 00  00 00 00 00 00 00 00 00
      02 45 2e 02 2d ff 09 2d  5f 06 2d 60 02 50 2e 03
      ff 05 30 00 61 64 64 69  74 69 6f 6e 00 00 00 39
      02 64 2e 02 ff 27 04 2e  02 6c 2e 07 1b 40 02 72
      2e 02 ff 7f 05 30 00 63  61 72 72 79 00 00 00 31
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      61 00 63 6f 75 6e 74 65  72 2e 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      74 61 62 00 74 6f 00 69  6e 63 72 65 6d 65 6e 74
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00`,
    file: `
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 03 1d 00 00 00 01  00 05 00 00 00 05 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00`,
    numtostring: `
      c1 02 0e 00 02 0d 0e 0c  0d 0c 05 10 0a 39 31 30
      08 0d 0f 02 ff 0d 0b 0e  0c 02 0d 0e 0c 0d 0c 08
      0d 0f 02 fe 0d 0b 0e 0c  08 0e 0f 02 fd 0e 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00
      00 00 00 00 00 00 00 00  00 00 00 00 00 31 39 33`,
  };

  function loadExample(name) {
    const bytes = examples[name]
      .replace(/\n/g, ' ')
      .split(' ')
      .filter((str) => str != '')
      .map((hex) => parseInt(hex, 16));
    for (let i = 0; i < 256; i++) vm.instance.exports.set(i, bytes[i]);
    vm.instance.exports.move(0);
    render();
  }
</script>
```

Some time ago, I created a tiny virtual machine called Dungbeetle.
It has 256 bytes of memory, which you can modify directly.
You can also store instructions in memory and tell the VM to run them.

Recently, I ported the terminal version to web assembly, so you play around with Dungbeetle directly on this website.
Try it out!

```embed
<table>
  <tr><td><b>arrow keys</b></td><td>move around</td></tr>
  <tr><td><b>+ and -</b></td><td>modify memory</td></tr>
  <tr><td><b>space</b></td><td>run a single instruction at the cursor position</td></tr>
  <tr><td><b>tab</b></td><td>run instructions until halt</td></tr>
  <tr><td><b>some other keys</b></td><td>literal input</td></tr>
</table>
```

The code is [on GitHub](https://github.com/MarcelGarus/dungbeetle).
