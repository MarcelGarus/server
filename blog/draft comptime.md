topics: Zig, programming language design, Martinaise, code, Soil, assembly

# Zig's two languages
## A case study of using Zig's comptime

When writing about [my first impressions of the programming language Zig](zig), the `zig:comptime` feature intruiged me.
Now that I've written [a sizable project](https://github.com/MarcelGarus/soil) in Zig, I've seen how `zig:comptime` enables new code designs that aren't possible in other systems programming languages like Rust or C++ (as far as I know).
Assuming you're somewhat proficient in Rust or C++, I'll try to highlight how Zig's take on compile time execution is different.

## A quick recap: What is comptime?

Comptime just stands for compile time.
Executing code at compile time is nothing new – compilers do this all the time to optimize your code.
For example, when using clang with the `bash:-O3` option to compile C++ code, clang replaces `c:2 + 2` with `c:4`.
Of course, not all code can run at compile time – reading files, doing network calls, or generally doing anything that requires calling dynamically linked functions or performing syscalls _has_ to happen at runtime.

This means, you essentially end up with two languages:
The main language (Rust or C++) and the subset of the language that can be executed at compile time.
Recently, Rust and C++ have added `rust:const fn`, a feature that allows you to explicitly mark a piece of code as only being allowed to use the compile time subset of the language.

!invertible[A diagram of Rust's two langugaes](files/two-languages-rust.webp)

Zig is different.
The part of the language that can run at compile time is _not a subset_ of the part that can run at runtime – it's just an overlapping set.

!invertible[A Venn diagram of the Zig language features.](files/two-languages-zig.webp)

Turns out, Zig actually consists of two languages, the *runtime language* and the *compile time language*:

- *The Zig runtime language* is what you usually think of as Zig.
  This language is statically typed, compiles to machine code, and executes efficiently.
  You can do all the I/O you want.
- *The Zig compile time language* only runs during compilation.
  It uses a tree-walking interpreter, similar to how some scripting languages work.
  The interesting thing: You can opt in to dynamic typing!
  If you declare a variable of the type `zig:anytype`, it can store anything, similar to `dart:Object` in Java.
  You can also inspect the types of values, store `zig:type`s directly, and get information about them.

Using the `zig:comptime` keyword, you can influence what runs at compile time and what doesn't.
In this small example, it forces a Fibonacci number calculation to happen during compile time:

```zig
var result = comptime fibonacci(10);
```

Here is code that stores a `zig:type` in a variable and uses the builtin `zig:@typeInfo` function – thing's you can only do at compile time:

```zig
const MyIntType: type = u8;
const foo: MyIntType = 3;
const info = @typeInfo(T);
```

After the compile time interpreter finishes executing, this is left of the program:

```zig
// MyIntType is optimized away because it's not used anymore.
const foo: u8 = 3;
const info: std.builtin.Type = .{ .Int = .{ .signedness = .unsigned, .bits = 8 } };
```

When writing Zig, it feels like you have the flexibility of a scripting language at compile time, while still retaining the efficiency of a systems programming language at runtime.
You can use this to design really flexible APIs with complex compile time checks.
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

This is a generic function, meaning it takes a `zig:type` as a parameter.
Depending on what type it is given, it expects a `zig:buffer` of a different size.
The compiler ensures that the size of the `zig:buffer` exactly matches the size of the requested integer:

```zig
const foo = readInt(u16, bytes[0..2]); // works, u16 takes 2 bytes
const bar = readInt(i64, bytes[0..8]); // works, i64 takes 8 bytes
const bar = readInt(i64, bytes[0..2]); // error: i64 needs 8 bytes
```

## Soil

Let's use `zig:comptime`!
My programming language [Martinaise](/martinaise) compiles to a byte code called [Soil](https://github.com/MarcelGarus/soil).
Soil is pretty low-level – it has a handful of registers and instructions that operate on them.
To get you into the right mindset, here are some Soil instructions:

```soil
moveib a 21      | a = 21
moveib b 2       | b = 2
mul a b          | a = a * b = 42
syscall 0        | exits with 42 (uses the a register as the error code)
```

Soil syscalls allow the byte code to hook into functionality that the VM provides – such as exiting in the snippet above.
However, not all targets support all syscalls.
For example, the server where this blog is hosted, [runs Soil](/this-blog-uses-martinaise).
It doesn't have a display system and therefore doesn't support syscalls related to UI rendering.

I ended up writing a small, reusable Zig library that implements everything of Soil _except_ syscalls – those have to be provided.
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
For example, they have to use the C calling convention, accept a `zig:*soil.Vm` as the first parameter as well as `zig:i64`s (one for each register they want to read).
You pass this `zig:Syscalls` struct to the `zig:soil.run` function, which then runs the binary, calling the appropriate function whenever a syscall instruction is executed.
If you don't implement a syscall, the `zig:not_implemented` function is called instead.

## How does it work?

The `zig:run` function behaves differently based on your CPU architecture.
On x86\_64, it compiles the byte code to x86\_64 machine code.
Otherwise, it uses a slower interpreter.
This conditional compilation can be done using a regular `zig:switch` – Zig guarantees to evaluate it at compile time if the switched-over value is compile-time-known.

```zig
pub fn run(binary: []const u8, alloc: Alloc, Syscalls: type) !void {
    comptime check_struct(Syscalls);

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

The call to `zig:check_struct` checks that the `zig:Syscalls` struct contains a `zig:not_implemented` function and that all functions with names of syscalls have the right signature.
The `zig:comptime` keyword ensures this check happens at compile time.
That means, you can use builtin functions that let you reflect over the structure of types:

- `zig:@TypeOf(anytype)` returns the `zig:type` of the given value. This `zig:type` can then be used where a type annotation is expected.
- `zig:@typeInfo(type)` returns a `zig:std.builtin.Type`, which is an enum with information about the type.
- `zig:@field(anytype, []const u8)` behaves like a field access (such as `zig:foo.bar`), but you can pass a compile-time known string as the field name.
- `zig:@hasDecl(type, []const u8)` checks if a type has a declaration with the given name.
- `zig:@compileError([]const u8)` aborts the compilation with the given error.

Using these functions, the checks are pretty straightforward:

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
        else => @compileError(name ++ " should be a function."),
    };

    if (signature.is_generic)
        @compileError("Syscall " ++ name ++ " is generic.");
    
    ... // many more checks
}
```

Because of these checks, if you try to compile the code with an ill-formed struct, you get an error.
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
The lower part tells why the comptime expression was compiled in the first place (how it's reached from `zig:main`).

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

That function then retrieves the syscall handlers, reflects on its signature, and calls it:

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

The `zig:Syscall.by_number` function is perhaps the weirdest function I've ever written:

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
Soil registers are directly mapped to x86\_64 registers – the `soil:a` register lives in `soil:r10`, the `soil:b` register in `soil:r11`, etc.
This way, most Soil byte code instructions map to a single x86\_64 machine instruction.

I wrote a machine code builder, where you can call methods for emitting instructions.
Here's example code of how to emit instructions that save 42 in the `soil:r10` register:

```zig
try machine_code.emit_mov_soil_word(.a, 21);  // mov r10, 21
try machine_code.emit_mov_soil_word(.b, 2);   // mov r11, 2
try machine_code.emit_imul_soil_soil(.a, .b); // imul r10, r11
```

Compiling a `soil:syscall` instruction starts the same way as it did in the interpreter:
With an inlined switch.
Like in the interpreter, inside the inlined `zig:else` case, we get the corresponding syscall function.

```zig
@setEvalBranchQuota(2000000);
switch (number) {
    inline else => |n| {
        const fun = Syscall.by_number(syscalls, n);

        ...
    },
}
```

Now the cool part:
The `zig:check_struct` code already ensured that the syscall functions all use the C calling convention.
This means we can call them from assembly as long as the stack is aligned to 16 bytes, and the arguments are in the correct registers (`soil:rdi`, `soil:rsi`, `soil:rdx`, etc.)

So for each syscall handler that you write in Zig, the compiler can emit machine code that correctly calls that function!

```zig
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
```

## Conclusion

Zig's `zig:comptime` is such an interesting language decision.
Dynamic ducktyping, reflection, and first-level types at compile time remove the need for macros and generic types.
Zig is a language that lets you just "do what you want" at compile time without the language getting in your way.
I would have never thought that a systems programming language could feel so dynamic.

Zig's dynamicness means function signatures sometimes look pretty different from those of other statically typed languages.
If you see a Rust function signature, you immediately know what types are expected.
Even for generic functions, traits tell you how the types need to behave.
In Zig, a function signature may not immediately tell you all you need.
In our case, the `zig:run` function just expects a `zig:Syscalls: type` – that doesn't tell you anything!
Only when you try to compile the code, you get an error.

Of course, just because you _can_ do arbitrary turing-complete checks at compile time doesn't mean you should.
I feel obliged to say that the vast majority of functions in Zig have readable signatures with simple types.
This blog post focuses on how Zig differs from other languages, so insane function signatures are disproportionately represented.

I think the `zig:comptime` feature aligns very well with Zig's general vibe.
It allows the language itself to be simple and uniform.
If you're open to trying new things, I encourage you to play around with Zig!
The combination of high-level dynamic scripting and low-level systems programming makes Zig just _feel_ different from all the mainstream languages.
Happy coding!
