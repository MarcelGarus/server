topics: Zig, programming language design, Martinaise, code

# Zig's two languages
## How Soil uses comptime

In my [article about my first impressions of the programming language Zig](zig), I wrote about how the `zig:comptime` feature can simpilify code.
Now that I've written a compiler in Zig, I've learned how to use `zig:comptime` to make amazing things that aren't even possible in other systems programming languages like Rust or C (as far as I know).
This article contains highlights of that.

## A quick recap: What is comptime?

Comptime just stands for compile time.
Turns out, Zig is actually two languages:

- *The Zig compile time language* uses a tree-walking interpreter, similar to how some scripting languages work.
  It's not statically typed (for example, variables of the type `zig:anytype` can contain anything and you can inspect their types).
  However, you can't do everything at compile time – certain operations such allocating memory, reading files, or doing network calls only work during runtime.
  In particular, syscalls or calls of externally linked functions can only happen at runtime.
- *The Zig runtime language* is what you usually think of as Zig.
  This language compiles to machine code and executes efficiently.
  It's statically typed, so you can't have variables of the type `zig:type` or `zig:anytype`.
  But you can do I/O.

For most code it doesn't matter whether it runs at compile time or runtime: The compiler can optimize `zig:2 + 2` to `zig:4`, or not, depending on its mood.
But there are some parts of the language that _have_ to run at compile time, and some that _have_ to run at runtime.
This is different from how `rust:const fn`s work in C++ or Rust.
In Zig, the part of the langauge that can run at compile time is not a _subset_ of the language – it's an overlapping, but different language.

![TODO: Image]()

Using the `zig:comptime` keyword, you can influence what runs at compile time and what doesn't.
For example, in this case, it forces a Fibonacci number calculation during compile time:

```zig
var result = comptime fibonacci(10);
```

At compile time, you can even do things you can't do at runtime.
Here, we store a type and call the builtin function `zig:@typeInfo`:

```zig
const MyIntType = u8;
const foo: MyIntType = 3;
const info = @typeInfo(T);
```

After the compile time interpreter finishes executing, this is left of the program:

```zig
// MyIntType is optimized away because it's not used anymore.
const foo: u8 = 3;
const info: std.builtin.Type = .{ .Int = .{ .signedness = .unsigned, .bits = 8 } };
```

That's right – `zig:@typeInfo` is a builtin function that takes a `zig:comptime T: type` and gives you information about it.
The `zig:std.builtin.Type` it returns is just a normal struct, so you can inspect it during runtime.

When writing Zig, it feels like you have the flexibility of a scripting language with reflection at compile time, while still retaining the efficiency of a systems programming language at runtime.
You can use this to design really flexible APIs with complex checks at compile time.
Take the `zig:std.mem.readInt` function:

```zig
fn readInt(
    comptime T: type,
    buffer: *const [@divExact(@typeInfo(T).Int.bits, 8)]u8,
    endian: Endian
) T {
    const value: T = @bitCast(buffer.*);
    return if (endian == native_endian) value else @byteSwap(value);
}
```

The compiler ensures that the size of the `zig:buffer` exactly matches the size of the requested integer.
It takes the bit length of the passed integer type, divides it by 8, and requires that as the buffer size.

```zig
const foo = readInt(u16, bytes[0..2]); // works, u16 takes 2 bytes
const bar = readInt(i64, bytes[0..8]); // works, i64 takes 8 bytes
const bar = readInt(i64, bytes[0..2]); // error: i64 needs 8 bytes
```

## Soil

Back to my project.
My programming language Martinaise compiles to a byte code called Soil.
Soil is pretty low-level – it has registers `soil:a` to `soil:f` and instructions that operate on them.
To get you into the right mindset, here's an example of Soil instructions (they are quite low level):

```soil
moveib a 21      | a = 21
moveib b 2       | b = 2
mul a b          | a = a * b = 42
syscall 0        | exits the process with error code 42
```

When writing an interpreter for Soil in Zig, by far the biggest effort went into implementing Soil syscalls.
Code can use these syscalls to hook into functionality that the VM provides – such as exiting in the snippet above.
However, not all targets support all syscalls.
For example, the server where this blog is hosted doesn't have a display system and therefore doesn't support syscalls related to UI rendering.

I ended up writing a small, reusable Zig library that implements everything of Soil except syscalls – those have to be provided.
You use the library like this:

```zig
const std = @import("std");
const soil = @import("soil");

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    const alloc = gpa.allocator();

    const binary = try std.fs.cwd().readFileAlloc(alloc, "my_file.soil", 1000000000);
    try soil.run(binary, alloc, Syscalls);
}

const Syscalls = struct {
    pub fn not_implemented(_: *soil.Vm) callconv(.C) void {
        std.debug.print("Syscall not implemented", .{});
        std.process.exit(1);
    }

    pub fn exit(_: *soil.Vm, status: i64) callconv(.C) void {
        std.process.exit(@intCast(status));
    }

    pub fn print(vm: *soil.Vm, msg_data: i64, msg_len: i64) callconv(.C) void {
        const msg = vm.memory[@intCast(msg_data)..][0..@intCast(msg_len)];
        std.io.getStdOut().writer().print("{s}", .{msg}) catch {};
    }

    ...
};
```

What's going on here?

You create a struct with all the syscall functions.
Those have to follow a few criteria.
For example, they have to use the C calling convention, accept a `zig:Vm` as the first argument as well as `zig:i64`s (one for each register you need to read).
You pass this struct to the `zig:soil.run` function, which then runs the binary, calling appropriate an function whenever a syscall instruction is executed.
If you don't implement a syscall, the `zig:not_implemented` function is called instead.

## How does it work?

The `zig:run` function behaves differently based on your CPU architecture.
On x86\_64, it compiles the byte code to x86\_64 machine code.
Otherwise, it uses a slower interpreter.

```zig
pub fn run(binary: []const u8, alloc: Alloc, Syscalls: type) !void {
    comptime @import("syscall.zig").check_struct(Syscalls);

    const file = try parse_file(binary, alloc);

    switch (builtin.cpu.arch) {
        .x86_64 => {
            const compile = @import("x86_64/compiler.zig").compile;
            var vm = try compile(alloc, file, Syscalls);
            try vm.run();
        },
        else => {
            const Vm = @import("interpreter/vm.zig");
            var vm = try Vm.init(alloc, file);
            try vm.run(Syscalls);
        },
    }
}
```

The call `zig:comptime @import("syscall.zig").check_struct(Syscalls);` checks that the `zig:Syscalls` struct has the right structure – it should contain a `zig:not_implemented` function and all functions with names of syscalls should have the right signature.
The `zig:comptime` keyword ensures this happens all at compile time.

The checking code itself is pretty straightforward and uses `zig:@typeInfo` a lot to reflect about the structure of the type and the functions.

```zig
pub fn check_struct(Syscalls: type) void {
    switch (@typeInfo(Syscalls)) {
        .Struct => {},
        else => @compileError("Syscall struct has to be a struct."),
    }

    // The syscall struct should contain a not_implemented function that takes
    // a *Vm.
    const not_implemented = "not_implemented";
    if (!@hasDecl(Syscalls, not_implemented))
        @compileError("The Syscall struct doesn't contain a not_implemented function.");
    check_syscall_signature(Syscalls, not_implemented);

    // All syscalls need to have good signatures.
    for (0..256) |number| {
        const option_name = name_by_number(number);
        if (option_name == null) continue; // unknown syscall, that's fine
        const name = option_name.?;
        if (!@hasDecl(Syscalls, name)) continue; // not implemented, that's fine
        check_syscall_signature(Syscalls, name);
    }
}

pub fn check_syscall_signature(Syscalls: type, name: []const u8) void {
    const signature = switch (@typeInfo(@TypeOf(@field(Syscalls, name)))) {
        .Fn => |f| f,
        else => return,
    };

    if (signature.is_generic)
        @compileError("Syscall " ++ name ++ " is generic.");
    
    ... // many more checks
}
```

Because of these checks, if you compile the code with an invalid struct, you get an error.
For example, if one of your syscall functions takes a parameter of an unexpected type, the compiler will yell at you:

```text
src/syscall.zig:74:17: error: All except the first syscall argument must be i64 (the content of a register). For the print syscall, an argument is main.Foo.
                @compileError("All except the first syscall argument must be i64 (the content of a register). For the " ++ name ++ " syscall, an argument is " ++ @typeName(param_type) ++ ".");
                ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
src/syscall.zig:45:32: note: called from here
        check_syscall_signature(Syscalls, name);
        ~~~~~~~~~~~~~~~~~~~~~~~^~~~~~~~~~~~~~~~
src/root.zig:12:49: note: called from here
    comptime @import("syscall.zig").check_struct(Syscalls);
             ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^~~~~~~~~~
referenced by:
    main: src/main.zig:46:17
    callMain: /snap/zig/11625/lib/std/start.zig:524:32
    callMainWithArgs: /snap/zig/11625/lib/std/start.zig:482:12
    main: /snap/zig/11625/lib/std/start.zig:497:12
```

Note that the stack trace consists of two parts:
The upper part is the ~runtime~ compile-runtime stack trace of the comptime evaluation.
The lower part contains information about why the comptime expression was compiled in the first place (how it's reached from `zig:main`).

## The Interpreter

The interpreter basically consists of a tight loop with a `zig:switch` over the instruction kind.
The case for syscalls is interesting:

```zig
.syscall => |number| {
    @setEvalBranchQuota(2000000);
    switch (number) {
        inline else => |n| run_syscall(vm, Syscalls, n),
    }
},
```

It takes the syscall number (a `zig:u8`) and then switches over it.
Instead of handling cases manually, it uses an `zig:inline else` to make the compiler duplicate the `zig:else` branch 256 times.
Inside the `zig:else` branch, it calls `zig:run_syscall` with a compile-time-known `zig:n`.

That function then gets the correct function for handling the syscall, reflects on its signature, and calls it:

```zig
inline fn run_syscall(vm: *Vm, Syscalls: type, comptime n: u8) void {
    const fun = comptime Syscall.by_number(Syscalls, n);
    const signature = @typeInfo(@TypeOf(fun)).Fn;

    const result = switch (signature.params.len) {
        1 => fun(vm),
        2 => fun(vm, vm.get_int(.a)),
        3 => fun(vm, vm.get_int(.a), vm.get_int(.b)),
        4 => fun(vm, vm.get_int(.a), vm.get_int(.b), vm.get_int(.c)),
        5 => fun(vm, vm.get_int(.a), vm.get_int(.b), vm.get_int(.c), vm.get_int(.d)),
        else => @compileError("handle syscalls with more params"),
    };

    // Move the return value into the correct registers.
    switch (@TypeOf(result)) {
        void => {},
        i64 => vm.set_int(.a, result),
        else => @compileError("syscalls can only return void or i64"),
    }
}
```

The `zig:Syscall.by_number` function is perhaps the most unhinged function I've ever written:

```zig
pub fn by_number(Syscalls: type, comptime n: u8) TypeOfSyscall(Syscalls, n) {
    const name = name_by_number(n) orelse return Syscalls.not_implemented;
    if (!@hasDecl(Syscalls, name)) return Syscalls.not_implemented;
    return @field(Syscalls, name);
}

// Duplicates the logic from above but all returns are wrapped with @TypeOf.
fn TypeOfSyscall(Syscalls: type, comptime n: u8) type {
    const name = name_by_number(n) orelse return @TypeOf(Syscalls.not_implemented);
    if (!@hasDecl(Syscalls, name)) return @TypeOf(Syscalls.not_implemented);
    return @TypeOf(@field(Syscalls, name));
}
```

Because functions can't return `zig:anytype`, duck typing on return values is not possible.
If someone ever finds a more elegant way to write this code, please contact me.

## The Compiler

On x86\_64, the byte code is compiled to machine code.
I wrote a machine code builder, where you can call methods for emitting instructions.
Internally, it will append the correct bytes to a buffer.

In the resulting machine code, the Soil registers are directly mapped to x86\_64 registers.
The `soil:a` register lives in `soil:r10`, the `soil:b` register in `soil:r11`, etc.
This way, most Soil byte code instructions map to a single x86\_64 machine instruction.

Compiling a `soil:syscall` instruction starts the same way as it did in the interpreter:
With an inlined switch.
Like in the interpreter, inside the inlined `zig:else` case, we get the corresponding syscall function.

Now the cool part:
Because we ensured before that the syscall functions all follow the C calling convention, the compiler can just instructions that shuffle the Soil registers into the correct registers according to the C calling convention.

```zig
@setEvalBranchQuota(2000000);
switch (number) {
    inline else => |n| {
        const fun = Syscall.by_number(syscalls, n);

        // Save all the Soil register contents on the stack.
        try machine_code.emit_push_soil(.a);
        try machine_code.emit_push_soil(.b);
        try machine_code.emit_push_soil(.c);
        ...

        // Align the stack to 16 bytes.
        ...

        // Move args into the correct registers for the C ABI.
        // Soil        C ABI
        // Vm (rbx) -> arg 1 (rdi)
        // a (r10)  -> arg 2 (rsi)
        // b (r11)  -> arg 3 (rdx)
        // c (r12)  -> arg 4 (rcx)
        // d (r13)  -> arg 5 (r8)
        // e (r14)  -> arg 6 (r9)
        const signature = @typeInfo(@TypeOf(fun)).Fn;
        const num_args = signature.params.len;
        if (num_args >= 1) try machine_code.emit_mov_rdi_rbx();
        if (num_args >= 2) try machine_code.emit_mov_rsi_r10();
        if (num_args >= 3) try machine_code.emit_mov_rdx_r11();
        if (num_args >= 4) try machine_code.emit_mov_rcx_r12();
        if (num_args >= 5) try machine_code.emit_mov_soil_soil(.sp, .d);
        if (num_args >= 5) try machine_code.emit_mov_soil_soil(.st, .e);

        // Call the syscall implementation.
        try machine_code.emit_call_comptime(@intFromPtr(&fun));

        // Unalign the stack.
        ...

        // Restore Soil register contents.
        ...
        try machine_code.emit_pop_soil(.c);
        try machine_code.emit_pop_soil(.b);
        try machine_code.emit_pop_soil(.a);

        // Move the return value into the correct registers.
        switch (signature.return_type.?) {
            void => {},
            i64 => try machine_code.emit_mov_soil_rax(.a),
            else => unreachable,
        }
    },
}
```

## Conclusion

Zig's `zig:comptime` is such an interesting language decision.
Dynamic ducktyping, reflection, and first-level types at compile time remove the need for macros and generic types.
Zig is a language that lets you just "do what you want" at compile time without the language getting in your way.

Functions can impose arbitrary turing-complete limitations on the types of parameters.
Those compile time checks can result in a stark contrast between Zig function signatures and those of other system programming languages.
If you see a Rust function signature, you immediately know what types are expected.
Even for generic functions, traits tell you how the types need to behave.
In Zig, a function signature may not immediately tell you all you need.
In our case, the `zig:run` function just expects a `zig:Syscalls: type` – that doesn't tell you anything!
Only when you try compile the code, you get an error.

Of course, just because you _can_ do anything at compile time doesn't mean you should.
The more dynamic your type checks get, the more you should document requirements in comments.
I feel obliged to say that the vast majority of functions in Zig have readable signatures with simple types.
This blog post focuses on how Zig differs from other languages, so insane function signatures are disproportionately represented.

The `zig:comptime` feature aligns very well with Zig's general vibe.
It allows the language itself to be simple and uniform.
If you can, play arounnd with it!
Zig will give you a new perspective on statically-typed programming just because of how different and unique it feels compared to the mainstream languages.
Happy coding!
