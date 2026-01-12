topics: Martinaise, code

# Stride Size

## An exploration of size and alignment

Many programming languages have product types, often called structs.
For example, this is a struct in my language [Martinaise](/martinaise):

```mar
struct Foo { a: Byte, b: Int, c: Byte }
```

In Martinaise, `mar:Int`s take up 8 bytes (64 bits) and `mar:Byte`s take up 1 byte (who would have thought).
So, for an instance of `mar:Foo`, these are the pieces of data that we somehow need to store in memory:

```embed
<svg xmlns="http://www.w3.org/2000/svg" width="454" height="64" viewBox="-2 -2 454 64">
  <style>
    rect { fill: white; stroke: black; stroke-width: 2px; }
    text { font: 20px sans-serif; text-anchor: middle; }
    line { stroke: var(--pink); stroke-width: 2px; }
  </style>
  <rect x="0" width="40" height="40" /><text x="20" y = "27">a</text>
  <rect x="60" y="20" width="320" height="40" /><text x="200" y = "47">b</text>
  <rect x="410" y ="10" width="40" height="40" /><text x="430" y = "37">c</text>
</svg>
```

The most straightforward way is to store the fields next to each other:

```embed
<div id="fieldsNextToEachOther"></div>
<script>
function buildSvg(parts) {
  const size = parts.reduce((sum, part) => sum + part.size, 0);
  const grid = 20;
  let out = `<svg xmlns="http://www.w3.org/2000/svg"
      width="${size * grid + 4}" height="${17 + grid}"
      viewBox="-2 -15 ${size * grid + 4} ${17 + grid}">
    <style>
      rect { fill: white; stroke: black; stroke-width: 2px; }
      text { font: 14px sans-serif; text-anchor: middle; }
      .grid { stroke: var(--pink); stroke-width: 2px; }
      .padding { stroke: black; stroke-width: 1px; }
    </style>`;
  for (let i = 0; i <= size; i++)
    out += `<line class="grid" x1="${i * grid}" y1="${(i % 8 == 0) ? -15 : -7}" x2="${i * grid}" y2="${grid}" />`;
  var usedSoFar = 0;
  for (const part of parts) {
    out += `<rect x="${usedSoFar * grid}" width="${part.size * grid}" height="${grid}" />`;
    if (part.kind === "field") out += `<text x="${(usedSoFar + part.size / 2) * grid}" y="15">${part.name}</text>`;
    if (part.kind === "padding") {
      for (let i = 0; i < part.size; i++) {
        out += `<line class="padding" x1="${(usedSoFar + i) * grid}" y1="${grid}" x2="${(usedSoFar + i + 1) * grid}" y2="0" />`;
      }
    }
    usedSoFar += part.size;
  }
  out += "</svg>";
  return out;
}
fieldsNextToEachOther.innerHTML = buildSvg([
  { kind: "field", name: "a", size: 1 },
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "c", size: 1 },
]);
</script>
```

However, on modern computers, memory accesses should be _aligned_:
When moving values between registers and memory, the absolute memory address should be a multiple of the value's size.
While x86 CPUs are forgiving (unaligned memory access is allowed, but may be slower than aligned one), ARM CPUs are stricter and some instructions cause "alignment faults" if you load data from an unaligned address.
So, in order to be able to efficiently load an 8-byte `mar:Int` from memory into a register to do calculations, it should be stored at an address that is a multiple of 8.

When I first learned about alignment, it felt weird, quirky, and unintuitive.
But apparently, this makes things easier for the hardware and it's here to stay.
Compilers deal with that by put padding (unused space) between struct fields.
Here's the memory layout that a C compiler would use for our struct:

```embed
<div id="cLayout"></div>
<script>
cLayout.innerHTML = buildSvg([
  { kind: "field", name: "a", size: 1 },
  { kind: "padding", size: 7 },
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "c", size: 1 },
  { kind: "padding", size: 7 },
]);
</script>
```

If we place such a struct at an address that is a multiple of 8, the b field will also be at an address that is a multiple of 8, so we can load the `mar:Int` directly into a register.
Note that the C compiler doesn't just add padding after the a field, but also after c, so that if you have an array of these structs, all of them are aligned to 8 bytes.

Other languages enable more efficient layouts.
For example, the Rust language gives no guarantees about the order of fields in memory (unless you use a special annotation, `rust:#[repr(C)]`).
This allows the Rust compiler to reorder fields, and that's exactly what it does.

```embed
<div id="rustLayout"></div>
<script>
rustLayout.innerHTML = buildSvg([
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "a", size: 1 },
  { kind: "field", name: "c", size: 1 },
  { kind: "padding", size: 6 },
]);
</script>
```

However, like C, Rust requires the size of a type to be a multiple of its alignment, documented in [`rust:std::mem::size_of`](doc.rust-lang.org/std/mem/fn.size_of.html).

In Martinaise, I decided to go a different route:
The size of a value doesn't have to be a multiple of its alignment.
The `mar:Foo` struct has a size of just 10 bytes and an alignment of 8:

```embed
<div id="martinaiseLayout"></div>
<script>
martinaiseLayout.innerHTML = buildSvg([
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "a", size: 1 },
  { kind: "field", name: "c", size: 1 },
]);
</script>
```

## Slices

What happens when we want to put multiple `mar:Foo`s next to each other in memory?
In C and Rust, that automatically works.
In Martinaise, the `mar:Slice` type from the standard library has to be careful to ensure proper alignment:
For each item, it reserves space according to the "stride size", which is just the size rounded up to the alignment.

```mar
struct Slice[T] { data: Address, len: Int }

fun get_ref[T](slice: Slice[T], index: Int): &T {
  {slice.data + {index * stride_size_of[T]()}}.to_reference[T]()
}

fun stride_size_of[T](): Int {
  size_of[T]().round_up_to_multiple_of(alignment_of[T]())
}

| implemented by the compiler
fun size_of[T](): Int { ... }
fun alignment_of[T](): Int { ... }
```

## Struct Layouting

If a type's size is always a multiple of its alignment, layouting a struct is pretty easy:
By sorting the fields according to decreasing alignment, there is no padding between them.
This is optimal, but only because we view fields as opaque memory blobs, when they might actually contain padding at the end.

In Martinaise, things are more complicated:

- Sorting by decreasing alignment is not optimal.
- Sorting by increasing alignment is not optimal.
- Sorting by decreasing size is not optimal.
- Sorting by increasing size is not optimal.


In another language of mine, Plum, I layout fields from regularly-shaped ones to irregularly-shaped ones – first, fields where the size is a multiple of 8, then fields where the size is a multiple of 4, then 2, then 1.
I put fields at the end (possibly requiring padding for proper alignment), unless the field fits into an existing gap of padding.

## Tighter Memory Layouts

Compared to the C/Rust approach, my more nuanced take on memory layouts adds some complexity to the language and the compiler.
However, this complexity is localized to the implementation of slices and struct layouting.

The upside is that all the structs that aren't in arrays/slices (my gut feeling says this is the majority) get more efficient layouts:

```embed
<div style="margin-left:0; margin-right: 0;">
<textarea id="codeInput" rows="6" cols="80"
    style="
      border: 0; width=100%; background:var(--box-bg); padding:var(--padding); border-radius: var(--box-border-radius);
      font-family: monospace; font-size: 0.9em; text-align: left; line-height: 1.5;
      white-space: pre; word-spacing: normal; word-break: normal; word-wrap: normal;
      -moz-tab-size: 2; -o-tab-size: 2; tab-size: 2;
      -webkit-hyphens: none; -moz-hyphens: none; -ms-hyphens: none; hyphens: none;
    ">
opaque Byte = size 1, alignment 1
opaque CInt = size 4, alignment 4
opaque Int = size 8, alignment 8
struct Foo { int: CInt, b: Byte }
struct Bar { foo1: Foo, foo2: Foo, x: Byte }
</textarea>
</div>
<div id="errors" style="color: var(--pink); font-weight: bold;"></div>
<b>In C:</b><div id="outputC"></div>
<b>In Rust:</b><div id="outputRust"></div>
<b>In Martinaise:</b><div id="outputMartinaise"></div>
<script>
function updateMemoryLayouts() {
  const outdatedDisclaimer = "The diagrams below are outdated until you fix this.";
  let parsed; try { parsed = parse(codeInput.value); } catch (e) { errors.innerHTML = e.message + "<br>" + outdatedDisclaimer; return; }
  try { validate(parsed); } catch (e) { errors.innerHTML = e.message + "<br>" + outdatedDisclaimer; return; }
  errors.innerHTML = "";
  for (const [layouter, outputId] of [
    [layoutInC, "outputC"],
    [layoutInRust, "outputRust"],
    [layoutInMartinaise, "outputMartinaise"],
  ]) {
    let layoutsToRender = [];
    let layouts = new Map();
    for (const def of parsed) {
      if (def.kind === "opaque") {
        layouts[def.name] = { size: def.size, alignment: def.alignment };
      } else if (def.kind === "struct") {
        let fields = [];
        for (const field of def.fields) {
          const fieldLayout = layouts[field.type];
          fields.push({ name: field.name, size: fieldLayout.size, alignment: fieldLayout.alignment });
        }
        const layout = layouter(fields);
        layout.size = layout.parts.reduce((sum, part) => sum + part.size, 0);
        layouts[def.name] = layout;
        layoutsToRender.push({ name: def.name, layout });
      }
    }
    let out = "<table>";
    for (const layout of layoutsToRender) {
      out += "";
      out += `<tr>
          <td>${layout.name} (<i>${layout.layout.size}B</i>)</td>
          <td>${buildSvg(layout.layout.parts)} </td>
        </tr>";
    }
    out += "</table>";
    document.getElementById(outputId).innerHTML = out;
  }
}

function parse(source) {
  class Parser { constructor(source) { this.source = source; this.cursor = 0; } }
  function consumeWhitespace(parser) {
    while (parser.cursor < parser.source.length) {
      const char = parser.source[parser.cursor];
      if (char === ' ' || char === '\t' || char === '\n' || char === '\r')
        parser.cursor++;
      else
        break;
    }
  }
  function consumeKeyword(parser, keyword) {
    consumeWhitespace(parser);
    if (parser.source.startsWith(keyword, parser.cursor)) {
      const nextCharIndex = parser.cursor + keyword.length;
      if (nextCharIndex < parser.source.length) {
        const nextChar = parser.source[nextCharIndex];
        if (/[a-zA-Z0-9_]/.test(nextChar)) return false;
      }
      parser.cursor += keyword.length;
      return true;
    }
    return false;
  }
  function consumeSymbol(parser, symbol) {
    consumeWhitespace(parser);
    if (parser.source.startsWith(symbol, parser.cursor)) {
      parser.cursor += symbol.length;
      return true;
    }
    return false;
  }
  function parseIdentifier(parser) {
    consumeWhitespace(parser);
    const start = parser.cursor;
    while (parser.cursor < parser.source.length) {
      const char = parser.source[parser.cursor];
      if (/[a-zA-Z0-9_]/.test(char)) parser.cursor++; else break;
    }
    if (parser.cursor > start) return parser.source.slice(start, parser.cursor);
    return null;
  }
  function parseInteger(parser) {
    consumeWhitespace(parser);
    const start = parser.cursor;
    while (parser.cursor < parser.source.length) {
      const char = parser.source[parser.cursor];
      if (/[0-9]/.test(char)) parser.cursor++; else break;
    }
    if (parser.cursor > start) return parseInt(parser.source.slice(start, parser.cursor), 10);
    return null;
  }
  function parseOpaque(parser) {
    if (!consumeKeyword(parser, "opaque")) return null;
    const name = parseIdentifier(parser); if (!name) throw new Error('An opaque type needs a name after the "opaque" keyword.');
    if (!consumeSymbol(parser, "=")) throw new Error(`${name} should have the form "opaque ${name} = size n, alignment m".`);
    if (!consumeKeyword(parser, "size")) throw new Error(`${name} should have the form "opaque ${name} = size n, alignment m".`);
    const size = parseInteger(parser); if (size === null) throw new Error(`${name} should have the form "opaque ${name} = size n, alignment m".`);
    if (!consumeSymbol(parser, ",")) throw new Error(`${name} should have the form "opaque ${name} = size ${size}, alignment m".`);
    if (!consumeKeyword(parser, "alignment")) throw new Error(`${name} should have the form "opaque ${name} = size ${size}, alignment m".`);
    const alignment = parseInteger(parser); if (alignment === null) throw new Error(`${name} should have the form "opaque ${name} = size ${size}, alignment m".`);
    return { name: name, kind: "opaque", size: size, alignment: alignment };
  }
  function parseStruct(parser) {
    if (!consumeKeyword(parser, "struct")) return null;
    const name = parseIdentifier(parser); if (!name) throw new Error('A struct type needs a name after the "struct" keyword.');
    if (!consumeSymbol(parser, "{")) throw new Error(`${name} should have a "{" after its name.`);
    const fields = [];
    while (true) {
      if (consumeSymbol(parser, "}")) break;
      const fieldName = parseIdentifier(parser); if (!fieldName) throw new Error(`${name} contains something weird that doesn't look like a field.`);
      if (!consumeSymbol(parser, ":")) throw new Error(`${name}'s field ${fieldName} needs a ":" after the field name.`);
      const type = parseIdentifier(parser); if (!type) throw new Error(`${name}'s field ${fieldName} needs a type name after the ":".`);
      fields.push({ name: fieldName, type });
      const hasComma = consumeSymbol(parser, ",");
      if (!hasComma) {
        if (!consumeSymbol(parser, "}")) throw new Error(`${name} contains something weird that doesn't look like a field.`);
        break;
      }
    }
    return { name: name, kind: "struct", fields: fields };
  }

  const parser = new Parser(source);
  const defs = [];
  while (true) {
    consumeWhitespace(parser);
    if (parser.cursor >= parser.source.length) break;
    let def = parseOpaque(parser);
    if (def) { defs.push(def); continue; }
    def = parseStruct(parser);
    if (def) { defs.push(def); continue; }
    throw new Error(`The code should just contain top-level "opaque" or "struct" definitions; got "${parser.source.slice(parser.cursor, parser.cursor + 10)}..."`);
  }
  return defs;
}

function validate(defs) {
  const knownTypes = new Set();
  for (const def of defs) {
    if (knownTypes.has(def.name)) throw new Error(`Type '${def.name}' is defined multiple times. That's not allowed.`);
    if (def.kind === "opaque") {
      if (def.size < 0) throw new Error(`${def.name} has a negative size.`);
      if (def.alignment != 8 && def.alignment != 4 && def.alignment != 2 && def.alignment != 1)
          throw new Error(`${def.name} has a weird alignment. Only 1, 2, 4, or 8 are allowed.`);
    }
    if (def.kind === "struct") {
      for (const field of def.fields) {
        if (!knownTypes.has(field.type)) {
          if (field.type === def.name)
            throw new Error(`${def.name} refers to itself. For simplicity, that's not allowed in this demo (feel free to define an opaque Box type).`);
          else
            throw new Error(`${def.name}'s field ${field.name} refers to type ${field.type}, but that hasn't been defined before.`);
        }
      }
    }
    knownTypes.add(def.name);
  }
}

function paddingNeededFor(current, alignment) {
  return (current % alignment == 0) ? 0 : (alignment - (current % alignment));
}

function layoutInC(fields) {
  let parts = [];
  let usedSoFar = 0;
  for (const field of fields) {
    const padding = paddingNeededFor(usedSoFar, field.alignment);
    parts.push({ kind: "padding", size: padding });
    parts.push({ kind: "field", name: field.name, size: field.size });
    usedSoFar += padding + field.size;
  }
  const maxAlignment = Math.max(...fields.map((f) => f.alignment));
  parts.push({ kind: "padding", size: paddingNeededFor(usedSoFar, maxAlignment) });
  return { alignment: maxAlignment, parts };
}

function layoutInRust(fields) {
  let sortedFields = [];
  for (const field of fields) if (field.alignment == 8) sortedFields.push(field);
  for (const field of fields) if (field.alignment == 4) sortedFields.push(field);
  for (const field of fields) if (field.alignment == 2) sortedFields.push(field);
  for (const field of fields) if (field.alignment == 1) sortedFields.push(field);
  return layoutInC(sortedFields);
}

function layoutInMartinaise(fields) {
  let structAlignment = Math.max(...fields.map((f) => f.alignment));
  const alignedFields = [];
  const remainingFields = [];
  for (const field of fields) (field.size % structAlignment === 0 ? alignedFields : remainingFields).push(field);
  const remainingIndices = remainingFields.map((_, i) => i);
  let bestPermutation = remainingIndices;
  let minSize = Infinity;
  let count = 0;
  for (const perm of permutations(remainingIndices.length)) {
    if (count >= 1000) break;
    count++;
    let currentSize = 0;
    for (const index of perm) {
      const field = remainingFields[index];
      const padding = paddingNeededFor(currentSize, field.alignment);
      currentSize += padding + field.size;
    }
    if (currentSize < minSize) { minSize = currentSize; bestPermutation = perm; }
  }
  const parts = [];
  let currentOffset = 0;
  for (const field of alignedFields) {
    parts.push({ kind: "field", name: field.name, size: field.size });
    currentOffset += field.size;
  }
  for (const index of bestPermutation) {
    const field = remainingFields[index];
    const padding = paddingNeededFor(currentOffset, field.alignment);
    parts.push({ kind: "padding", size: padding });
    currentOffset += padding;
    parts.push({ kind: "field", name: field.name, size: field.size });
    currentOffset += field.size;
  }
  return { alignment: structAlignment, parts };
}
function* permutations(n) {
  const indices = Array.from({ length: n }, (_, i) => i);
  yield* generate(indices, []);
  function* generate(current, used) {
    if (used.length === current.length) { yield used; return; }
    for (let i = 0; i < current.length; i++)
      if (!used.includes(current[i])) yield* generate(current, [...used, current[i]]);
  }
}

codeInput.addEventListener("keyup", updateMemoryLayouts);
codeInput.addEventListener("change", updateMemoryLayouts);
updateMemoryLayouts()
</script>
```

You should be able to edit the code and see the memory layout update (it only supports opaque types and structs).

So, was it worth it?
At least for Martinaise, definitely.
While I didn't cover it in this article, Martinaise's enums are represented by the payload followed by a tag, so many types have odd shapes – for example, a `mar:Maybe[Int]` takes up 9 bytes.
Also, following this rabbit hole was fun.
I can sleep well knowing that my language sometimes chooses better layouts than the Rust compiler.
