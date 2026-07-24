# Cerium

A toy virtual machine and assembler for a custom instruction set, implemented in
Rust.

Focused on instruction set design, bytecode execution, memory management, and
low-level systems behavior in an interpreted VM.

This builds on older work done in [a previous project](https://github.com/Nengyi-Jonathan-Jiang/Parser-Lexer-Generators)
which featured a more rudimentary form of this idea.

## Features

- Register-based execution model
- Support for memory operands (instructions can operate directly on memory)
- Built-in memory allocation instructions implemented by the VM
- Assembler for converting assembly code (`.casm`) to bytecode (`.ce`)
- Virtual machine for executing bytecode programs
- Debug mode for instruction-level execution tracing

## Running the project

If you are on 64-bit Windows, you can get the executable directly from the
[releases](https://github.com/Nengyi-Jonathan-Jiang/cerium/releases). Otherwise,
you'll need to build the project yourself.

Requirements:

- Rust toolchain (`cargo`)
  Build

```
git clone https://github.com/Nengyi-Jonathan-Jiang/cerium
cd cerium
cargo build
```

The resulting executable can then be directly run.

## Command Line Usage

### Run binary

```
cerium <input-file>
```

Executes a compiled Cerium binary file.

* `<input-file>`: A `.ce` binary file

---

### `debug`

```
cerium debug <input-file>
```

Executes a compiled Cerium binary file in debug mode.

* `<input-file>`: A `.ce` binary file

---

### `assemble`

```
cerium assemble <input-files> <output-file>
```

Assembles one or more Cerium assembly files into a bytecode file.

* `<input-files>`: One or more assembly source files
* `<output-file>`: Output bytecode file

---

### `run-asm`

```
cerium run-asm <input-files>
```

Assembles and executes one or more assembly files directly, without producing an
output file.

* `<input-files>`: One or more assembly source files

---

### `debug-asm`

```
cerium debug-asm <input-files>
```

Assembles and executes one or more assembly files in debug mode, without
producing an output file.

* `<input-files>`: One or more assembly source files

---

* Commands that accept `<input-files>` support multiple files
* In debug mode, Cerium prints the disassembly of each instruction as it
  executes as well as the state of each register

## ISA documentation

See
[examples/casm.md](https://github.com/Nengyi-Jonathan-Jiang/cerium/blob/master/examples/casm.md)

## Example program

The following Cerium program reads an integer and outputs all even numbers
less than or equal to it:

```asm
  input -> r1           // Read an int from input
  mov i r2 <- 2         // Load some constants into registers
  mov i r3 <- 1
  add i r1 <- r1 + r3
  mov i r4 <- LOOP_BEGIN
  mov i r5 <- LOOP_END
LOOP_BEGIN:
  mod i r6 <- r1 % r2
  sub i r1 <- r1 - r3
  jmp r5 if i r1 <= 0   // conditional jump to loop_end
  jmp r4 if i r6 == 0   // conditional jump to loop_begin
  output <- r1
  jmp r4 always         // unconditional jump to loop_begin
LOOP_END:
  halt                  // Required at the end of programs to stop execution
```

See
the [examples](https://github.com/Nengyi-Jonathan-Jiang/cerium/blob/master/examples/casm.md)
folder for more examples

## Design Decisions

When designing the ISA, I followed the general principle of being as simple as
possible while providing reasonable convenience to the programmer.

- **Memory Addressing**  
  Instructions can operate directly on memory (i.e., support memory operands),
  rather than requiring explicit load/store instructions. This effectively makes
  Cerium closer to a CISC-style model than a strict load/store architecture.

  Since the VM is interpreted (no JIT), register access is not significantly
  faster than memory access. Restricting instructions to registers would
  therefore add complexity without a meaningful performance benefit. Allowing
  direct memory access also reduces the number of instructions required in
  typical programs.

- **Built-in Memory Allocation**  
  Implementing an allocator in Cerium assembly would require substantial code
  and would be significantly less efficient than a native implementation.
  Providing allocation primitives in the VM simplifies program development and
  ensures more predictable performance.

- **Variable-Length Instruction Encoding**  
  Most instructions can be represented in 2–3 bytes, so fixed-width encoding
  would introduce unnecessary padding. Variable-length encoding reduces bytecode
  size while still allowing efficient decoding.

- **Control Flow Model**  
  Cerium provides a single jump instruction that targets an address stored in a
  register.

  This is sufficient to implement higher-level control flow constructs such as
  function calls and returns. The calling convention is not enforced by the VM
  and is left to the program.
- **Syntax**  
  The assembly syntax is designed to be simple and readable, rather than closely
  matching existing architectures (e.g., x86 or ARM).

  The goal is to make programs easier to write and understand, especially given
  that the ISA is custom and not tied to hardware constraints.

## Challenges

- Designed an instruction set with well-defined execution semantics and a
  compact bytecode encoding, balancing simplicity with expressiveness and
  efficient interpretation
- Implemented a best-fit allocator within the VM using a binary tree to track
  free blocks, supporting efficient allocation, splitting, and coalescing
- Profiled the VM with Intel VTune to identify hotspots in instruction dispatch
  and pointer-heavy memory access patterns, and reduced overhead by removing
  unnecessary safety checks in performance-critical paths

## Limitations and Future Work

### Current limitations

- Minimal debugging tools
- No JIT compilation

### Possible improvements

- Add more advanced debugging tools (e.g., step execution, breakpoints)
- Add a higher-level language frontend
