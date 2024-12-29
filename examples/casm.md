# How the Cerium VM works

## Locations
Operations on the Cerium VM act on *locations*, which can be a 
register or the memory pointed to by a register. Cerium provides 
8 registers (`sp`, `r1`, `r2`, `r3`, `r4`, `r5`, `r6`, `r7`). To
access the memory pointed to by a register, add an `@` before the
register name (for example, the memory pointed to by `r1` would 
be `@r1`). 

Note: Although `sp` seems to have a special name, it 
works just like the other registers; using it as some sort of 
stack pointer is just convention.

### Memory
Cerium has infinite memory. This is implemented as two growable 
buffers, one for positive memory values and one for negative 
memory values (yes, negative memory values!). Positive memory 
values will be used for "stack" space and negative values for
"heap" space. Do not cross the boundary between positive and 
negative when reading a value.

## Endian-ness
CeriumVM is big-endian. That means that a value like `0x1A2B3C4D` 
is stored in memory as
```
Index: | i + 0 | i + 1 | i + 2 | i + 3 |
Value: |  1A   |  2B   |  3C   |  4D   |
```

## Types
Most operations will have one or more *types*, which indicate 
the type (`f` for 32-bit float, `i` for 32-bit signed integer, 
`s` for 16-bit signed integer, `b` for 8-bit signed integer) of 
the operands and/or result of the operation. When accessing a 
location as a given type, the first *x* bytes are read. This means
that you can't use "type punning" to convert between integer types;
you have to actually do an explicit conversion (explained later)

## Instructions
Instructions are typically composed a name and some *locations* and 
*types*.

### Lod
The `lod` instruction loads a constant value into a location. It is written as
```
lod [location] <- [type] [constant]
```
For example, to load the 8-bit integer `-46` into the `r1` register,
one would write
```
lod r1 <- b -46
```
Integral constants can also be written in hexidecimal notation with 
the prefix `0x`. Float constants can be anything parseable by Rust's
`parse::<f32>()` function.

### Mov
The `mov` instruction copies a value from one location to another, possibly
with a type conversion. It is the only instruction that can perform type 
conversions. It is written as
```
mov [type 1] [location 1] <- [type 2] [location 2]
```
For example, to move the value at the memory pointed to by `r1` into `r2`, 
converting it from a 32-bit float to a 16-bit int, one would write
```
mov s r2 <- f @r1 
```

All conversions are valid. They are implemented as primitive type casts in 
Rust. There are no restrictions on the locations moved to and from. 

### Arithmetic and bitwise operations
Arithmetic and bitwise operations with two operands are written as 
follows:
```
[name] [type] [target] <- [operand 1] [symbol] [operand 2]   
```
For example, to add `r1` to the memory pointed to by `r2` and store
the result in `r3`, interpreting both operands as 16-bit integers, 
one would write
```
add s r3 <- r1 + @r2
```

There are no restrictions on the locations that can appear in any 
position. A table of the operations and their names and symbols is given below:

| Name | Symbol | Description                                                    |
|------|--------|----------------------------------------------------------------|
| xor  | ^      | Bitwise XOR. Cannot be applied to float                        |
| or   | \|     | Bitwise OR. Cannot be applied to float                         |
| and  | &      | Bitwise AND. Cannot be applied to float                        |
| shl  | <<     | Left shift. Cannot be applied to float                         |
| shr  | \>\>   | Right shift. Cannot be applied to float                        |
| mul  | *      | Multiplication                                                 |
| add  | +      | Addition                                                       |
| sub  | -      | Subtraction                                                    |
| div  | /      | Division                                                       |
| mod  | %      | Modulo. This uses floored division, so mod(x, m) = -mod(x, -m) |

CeriumVM also provides two unary operations: arithmetic `neg`ation (`-`) and bitwise `not` (`~`.
The syntax is
```
[name] [type] [target] <- [symbol] [operand]
```
For example,
```
neg i r1 <- -r1
```
negates the 32-bit integer value in `r1` 

### Jump
Cerium offers a conditional jump instruction, which jumps to an 
arbitrary location. The syntax is
```
jmp [target] if [type] [operand] [condition]
```
or
```
jmp [target] always
```
for an unconditional jump. This causes the virtual machine to
jump to the instruction pointed to by the value of `[target]`
if the value of `[operand]` satisfies the condition. For 
example, if `r1` had the value `10`, then 
```
jmp r1 if f @r2 > 0
```
would cause the virtual machine to go to the instruction at the 
10th byte of instruction memory if the value at the memory 
pointed to by `r2`, interpreted as a 32-bit float, is greater than 
zero.

The condition must be one of `> 0`, `< 0`, `>= 0`, `<= 0`, `== 0`, 
or `!= 0`.

### Labels

Because counting bytes is tedious, Cerium provides labels as 
convenient jumping targets. To declare a label, write
```
[name]:
```
where `[name]` is a sequence of uppercase letters, digits, 
and underscores. To use a label, `lod` it into a location. The 
syntax is slightly modified and you do not need to specify the 
type (it is automatically `i`). For example,
```
LOOP_START:

// Do stuff here

// Go to LOOP_START if r2 >= 0
lod r1 <- LOOP_START
jmp r1 if r2 >= 0
```

### Comparing stuff
Sometimes, a programmer wants to compare values without 
jumping and store the result in a boolean (which will be 
represented as a 8-bit integer that is 1 or 0). This is 
provided by the `cmp` instruction, which is written as follows:
```
cmp [location] <- [type] [operand] [condition]
```
Everything is the same as `jmp`, except that the condition cannot 
be `always`.

### Input and Output
I did not implement interrupts and stuff, so input and output get
dedicated instructions. To input a 32-bit integer, do
```
input -> [location]
```
To output a 32-bit integer, do
```
output <- [location]
```
I did not add input and output for other types yet.

### Halt
To stop execution immediately, `halt`. For example, the 
following snipped halts if r3, interpreted as a 32-bit integer, has 
the value 264
```
lod r2 <- i 264
sub i r2 <- r3 - r2    // Now r2 has (r3 - 264) 
lod r1 <- OK
jmp r1 if i r2 != 0    // Go to OK if (r3 - 264 != 0)
// We did not jump so r3 must be 264.
halt
OK:
// Other stuff
```

### Memcpy
To copy a bunch of bytes in memory, use the `memcpy` instruction for convenience. The syntax is:
```
memcpy [dest location] <- [src location] ; [size location]
```
The value of `[dest location]` will be interpreted as a pointer to the destination, and similarly 
for `[src location]`. This means that you could possibly get 2 levels of indirection if one of the 
locations has an `@` in it :)

`[size]` will always be interpreted as a 32-bit signed integer, although negative values will lead 
to unexpected results (I don't know what will happen)

### Heap allocation
To relieve programmers of the burden of figuring out heap allocation, Cerium provides instructions for 
heap allocation, the `new` and `del` instructions for allocating and deallocating memory. 

New:
```
new [dest location] ; [size location]
```
`size location` will always be interpreted as a 32-bit integer; 
`[dest location]` will store a pointer to the allocated memory.

Del:
```
del [location]
```
the value of `location` will be interpreted as a pointer to the 
allocated memory. Cerium stores the size of the allocated memory
so you don't need to tell it the size.

## Comments
Cerium assembly (.casm) has inline comments prefixed by `//`.
Comments last until the end of the line they are on. For example,
```
lod r1 <- i 4639     // I love FRC team 4639! 
```
