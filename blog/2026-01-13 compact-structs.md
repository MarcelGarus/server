# Compact Structs

## Memory layouting is fucking hard

Many programming languages have product types, often called structs.
For example, this is a struct in my language [Martinaise](/martinaise):

```mar
struct Foo { a: Byte, b: Int, c: Byte }
```

In Martinaise, `mar:Int`s take up 8 bytes (64 bits) and `mar:Byte`s take up 1 byte (surprising, I know).
So, for an instance of `mar:Foo`, these are the pieces of data that we somehow need to store in memory:

```embed
<style>
rect { fill: var(--box-bg); stroke: var(--fg); stroke-width: 2px; }
text { font: 14px sans-serif; text-anchor: middle; fill: var(--fg); }
.grid { stroke: var(--pink); stroke-width: 2px; }
.padding { stroke: var(--fg); stroke-width: 1px; }
</style>
<svg xmlns="http://www.w3.org/2000/svg" width="304" height="67" viewBox="-2 -15 304 67">
    <rect x="0" width="20" height="20"></rect><text x="10" y="15">a</text>
    <rect x="80" y="30" width="160" height="20"></rect><text x="160" y="45">b</text>
    <rect x="280" y="10" width="20" height="20"></rect><text x="290" y="25">c</text>
</svg>
```

The most straightforward way is to store the fields next to each other:

```embed
<script>
function buildSvg(parts) {
  const size = parts.reduce((sum, part) => sum + part.size, 0);
  const grid = 20;
  let out = `<svg xmlns="http://www.w3.org/2000/svg"
      width="${size * grid + 4}" height="${17 + grid}"
      viewBox="-2 -15 ${size * grid + 4} ${17 + grid}">`;
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
</script>
<script id="fieldsNextToEachOther">
fieldsNextToEachOther.outerHTML = buildSvg([
  { kind: "field", name: "a", size: 1 },
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "c", size: 1 },
]);
</script>
```

However, on modern computers, memory accesses should be _aligned_:
When moving values between registers and memory, the absolute memory address should be a multiple of the value's size.
While x86 CPUs are forgiving (unaligned memory access is allowed, but may be slower than aligned one), ARM CPUs are stricter and some instructions cause "alignment faults" if you load data from an unaligned address.
So, in order to be able to efficiently load an 8-byte `mar:Int` from memory into a CPU register to do calculations, it should be stored at an address that is a multiple of 8.

When I first learned about alignment, it felt weird, quirky, and unintuitive.
The _absolute_ address matters?
Whyy?
But apparently, this makes things easier for the hardware and it's here to stay.
Compilers usually deal with that by making types have a size and an alignment, where an alignment of n means that the value can only be placed at addresses that are a multiple of n.
To achieve correct alignment of struct fields, compilers introduce padding (unused space).
Here's the memory layout that a C compiler would use for our `mar:Foo`:

```embed
<script id="cLayout">
cLayout.outerHTML = buildSvg([
  { kind: "field", name: "a", size: 1 },
  { kind: "padding", size: 7 },
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "c", size: 1 },
  { kind: "padding", size: 7 },
]);
</script>
```

If we place such a `mar:Foo` at an address that is a multiple of 8, the `mar:b` field will also be at an address that is a multiple of 8, so we can load the `mar:Int` directly into a register.
Note that the C compiler doesn't just add padding after the `mar:a` field, but also after `mar:c`, so that if you have an array of `mar:Foo`s, all of them are aligned to 8 bytes.

Other languages enable more compact layouts.
For example, the Rust language gives no guarantees about the order of fields in memory (unless you use a special annotation, `rust:#[repr(C)]`).
This allows the Rust compiler to reorder fields, and that's exactly what it does:

```embed
<script id="rustLayout">
rustLayout.outerHTML = buildSvg([
  { kind: "field", name: "b", size: 8 },
  { kind: "field", name: "a", size: 1 },
  { kind: "field", name: "c", size: 1 },
  { kind: "padding", size: 6 },
]);
</script>
```

Compact layouts are generally more efficient because they use less memory, lead to fewer cache misses, etc.
However, like C, Rust requires the size of a type to be a multiple of its alignment, documented in [`rust:std::mem::size_of`](doc.rust-lang.org/std/mem/fn.size_of.html).

In Martinaise, I decided to go a different route:
The size of a value doesn't have to be a multiple of its alignment.
The `mar:Foo` struct has a size of just 10 bytes and an alignment of 8:

```embed
<script id="martinaiseLayout">
martinaiseLayout.outerHTML = buildSvg([
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

Here's the relevant code [from the standard library](https://github.com/MarcelGarus/martinaise/blob/main/stdlib/mem.mar):

```mar
struct Slice[T] { data: Address, len: Int }

| Used internally by get, set, get_ref, etc.
fun get_ref_unchecked[T](slice: Slice[T], index: Int): &T {
  {slice.data + {index * stride_size_of[T]()}}.to_reference[T]()
}

fun stride_size_of[T](): Int {
  size_of[T]().round_up_to_multiple_of(alignment_of[T]())
}

| Magically implemented by the compiler.
fun size_of[T](): Int { ... }
fun alignment_of[T](): Int { ... }
```

## Struct Layouting

If a type's size is always a multiple of its alignment, layouting a struct is pretty easy:
By sorting the fields according to decreasing alignment, there is no padding between them.
This is optimal, but only because we view fields as opaque memory blobs, when they might actually contain padding at the end.

If your sizes are independent of the alignment, things are more complicated and it's difficult to find an optimal strategy.
For every sorting criteria I've come up with, there's a combination of fields that cause the layout to contain unnecessary padding:

- Sorting fields by decreasing alignment is not optimal:
  `embed:<br><script id="decAlignment">decAlignment.outerHTML = buildSvg([{kind: "field", name: "size 9, alignment 8", size: 9}, {kind: "padding", size: 3}, {kind: "field", name: "size 4, al. 4", size: 4}, {kind: "field", name: "size 4, al. 4", size: 4}])</script>`
- Sorting fields by increasing alignment is not optimal:
  `embed:<br><script id="incAlignment">incAlignment.outerHTML = buildSvg([{kind: "field", name: "size 4, al. 4", size: 4}, {kind: "padding", size: 4}, {kind: "field", name: "size 8, alignment 8", size: 8}])</script>`
- Sorting fields by decreasing size is not optimal:
  `embed:<br><script id="decSize">decSize.outerHTML = buildSvg([{kind: "field", name: "size 5, al. 4", size: 5}, {kind: "padding", size: 3}, {kind: "field", name: "size 4, al. 4", size: 4}])</script>`
- Sorting fields by increasing size is not optimal:
  `embed:<br><script id="incSize">incSize.outerHTML = buildSvg([{kind: "field", name: "size 5, al. 4", size: 5}, {kind: "padding", size: 3}, {kind: "field", name: "size 8, alignment 4", size: 8}])</script>`

Even though I haven't proven it, I suspect this problem is NP-complete – it feels a bit similar to [bin-packing](https://en.wikipedia.org/wiki/Bin_packing_problem).
Perhaps it helps that the alignments are always powers of two?

I took the hacky route:
First, I noticed that lots of structs only contain `mar:Int`s and pointers and other structs that only contain `mar:Int`s and pointers – those cause the maximum alignment to be 8 and because their size is a multiple of 8, I can safely move them to front of the struct.
For the (often few) remaining fields, I just bruteforce 1000 permutations and choose the best one.

In another language of mine, [Plum](/plum), I place fields from regularly-shaped ones to irregularly-shaped ones – first, fields where the size is a multiple of 8, then fields where the size is a multiple of 4, then 2, then 1.
I also don't append a field at the end of the struct if it fits in a padding hole before.
This doesn't always find an optimal placement, for example here:

```embed
<script id="badPlumLayout">
badPlumLayout.outerHTML = buildSvg([
  { kind: "field", name: "size 4, al. 2", size: 4 },
  { kind: "padding", size: 4 },
  { kind: "field", name: "size 10, alignment 8", size: 10 },
]);
</script>
```

But the struct layouts do feel very packed together in practice.

## Compact Memory Layouts

Compared to the C/Rust approach, my more nuanced take on memory layouts adds some complexity to the language and the compiler.
However, this complexity is localized to the implementation of slices and struct layouting.

The upside is that all the structs that aren't in arrays/slices (my gut feeling says this is the majority) get more efficient layouts.
To get a feel for that, here is an editable code area, with the memory layouts shown live below:

```embed
<div style="margin-left:0; margin-right: 0;">
<textarea id="codeInput" rows="6" cols="80" style="
  border: 0; width=100%; background:var(--box-bg); padding:var(--padding); border-radius: var(--box-border-radius);
  color: var(--fg);
  font-family: monospace; font-size: 0.9em; text-align: left; line-height: 1.5;
  white-space: pre; word-spacing: normal; word-break: normal; word-wrap: normal;
  -moz-tab-size: 2; -o-tab-size: 2; tab-size: 2;
  -webkit-hyphens: none; -moz-hyphens: none; -ms-hyphens: none; hyphens: none;
">
opaque Byte = size 1, alignment 1
opaque CInt = size 4, alignment 4
opaque Int = size 8, alignment 8

struct Foo { a: Byte, b: CInt, c: Byte }
struct Bar { foo1: Foo, foo2: Foo, x: Byte }</textarea>
</div>
<div id="errors" style="color: var(--pink); font-weight: bold;"></div>
<b>In C:</b><div id="outputC"></div>
<b>In Rust:</b><div id="outputRust"></div>
<b>In Martinaise:</b><div id="outputMartinaise"></div>
<b>In Plum:</b><div id="outputPlum"></div>
<script>
function updateMemoryLayouts() {
  const outdatedDisclaimer = "The diagrams below are outdated until you fix this.";
  let parsed; try { parsed = parse(codeInput.value); } catch (e) { errors.innerHTML = e.message + "<br>" + outdatedDisclaimer; return; }
  try { validate(parsed); } catch (e) { errors.innerHTML = e.message + "<br>" + outdatedDisclaimer; return; }
  errors.innerHTML = "";
  for (const [layouter, output] of [
    [layoutInC, outputC],
    [layoutInRust, outputRust],
    [layoutInMartinaise, outputMartinaise],
    [layoutInPlum, outputPlum],
  ]) {
    let layouts = new Map();
    let out = "<table>";
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
        const size = layout.parts.reduce((sum, part) => sum + part.size, 0);
        layouts[def.name] = { size, alignment: layout.alignment };
        out += `<tr><td>${def.name} (<i>${size}B</i>)</td><td>${buildSvg(layout.parts)} </td></tr>`;
      }
    }
    out += "</table>";
    output.innerHTML = out;
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

function layoutInPlum(fields) {
  let layouts = fields.map((f, i) => ({ name: f.name, size: f.size, alignment: f.alignment, index: i }));
  function getEvenness(size) {
    if (size > 0 && size % 8 === 0) return 8;
    if (size > 0 && size % 4 === 0) return 4;
    if (size > 0 && size % 2 === 0) return 2;
    return 1;
  }
  layouts.sort((a, b) => {
    const evA = getEvenness(a.size);
    const evB = getEvenness(b.size);
    if (evA !== evB) return evB - evA; // Descending evenness
    if (a.size !== b.size) return b.size - a.size; // Descending size
    return a.index - b.index; // Ascending index
  });
  let bytes = []; // -1 is padding, >= 0 is the field index
  for (const field of layouts) {
    let placed = false;
    let offset = 0;
    while (offset + field.size <= bytes.length) {
      let isAllPadding = true;
      for (let i = offset; i < offset + field.size; i++)
        if (bytes[i] !== -1) { isAllPadding = false; break; }
      if (isAllPadding) {
        for (let i = offset; i < offset + field.size; i++) bytes[i] = field.index;
        placed = true;
        break;
      }
      offset += field.alignment;
    }
    if (!placed) {
      const paddingNeeded = paddingNeededFor(bytes.length, field.alignment);
      for (let i = 0; i < paddingNeeded; i++) bytes.push(-1);
      for (let i = 0; i < field.size; i++) bytes.push(field.index);
    }
  }
  const parts = [];
  let i = 0;
  while (i < bytes.length) {
    let value = bytes[i];
    let start = i;
    while (i < bytes.length && bytes[i] === value) i++;
    let len = i - start;
    if (value === -1) {
      parts.push({ kind: "padding", size: len });
    } else {
      parts.push({ kind: "field", name: fields[value].name, index: value, size: len });
    }
  }
  const totalSize = bytes.length;
  let maxAlignment = 1;
  for (const f of fields) maxAlignment = Math.max(maxAlignment, f.alignment);
  return { alignment: maxAlignment, parts: parts };
}

codeInput.addEventListener("keyup", updateMemoryLayouts);
codeInput.addEventListener("change", updateMemoryLayouts);
updateMemoryLayouts()
</script>
```

## Big Picture

There's other stuff you can do to make your programs use less memory.
Admittedly, some of them probably have a bigger impact than being really smart about your struct layouts.

**Niches** tell the compiler that some bit patterns never occur.
Depending on the language, the compiler can ensure that null pointers or certain NaN patterns in floating point numbers never occur and make those bit patterns mean something else.
This is how an `rust:Option<&Foo>` in Rust can be 8 bytes.

**Global type arenas** combined with column-wise storage can make languages really efficient.
For example, if you have a type `mar:struct Point { x: Int, y: Int }` in your program, the compiler could create two global lists containing all x and y positions, respectively.
A `mar:Point` value would actually just be an index into those lists.
Tight loops that operate only on some fields of objects then have really dense memory access patterns, which is good for memory caches.
If you can make assumptions about the maximum number of objects of a type, you don't even need the full 8 bytes for an index, but instead use something smaller, which makes _all other_ data structures more efficient, etc.
Modern game engines use this kind of data-driven design a lot.

Anyways.
Was it worth it?
At least for Martinaise and Plum, definitely.
While I didn't cover it in this article, both languages represent enums as the payload followed by a tag, so many types have odd shapes – for example, a `mar:Maybe[Int]` takes up 9 bytes.
Also, following this rabbit hole was fun.
I can sleep well knowing that my languages sometimes choose better struct layouts than the Rust compiler.
