# Bubble VM

*Note: this was entirely written before the CTF. A detailed post-CTF writeup is available on the [CoR website](https://cor.team/posts/corctf-2025-bubble-vm/).*

The Bubble VM is written in C and has two arrays and two registers: memory and (immutable) instruction data, and a memory pointer and instruction pointer operating on them. It also has around 30 single-char opcodes to manipulate data and perform computations.
Players are given a (-O2, stripped) bubble_vm binary as well as a program, which contains a bunch of opcodes (encoded as ascii chars).

Each opcode does something simple to the program state. Data operations (such as ADD) read the current data pointer and the next cell, and always write the result into the current cell. Because there is only one data pointer, programs use complex sequences of BUBBLE and SHIFT instructions (hence the name) in order to have functionality.

Solvers need to not only understand the VM architecture, but also analyze how the generated program is structured to run nontrivial programs.

## Program Generation
The bytecodes are individually extremely simple and would be difficult to make complex programs. For this reason, I also made an assembler and linker in Python. It is able to assemble bytecode stubs, as well as set up a callstack and function calls. It also uses a bitmasking algorithm to load arbitrary numbers into the program using the limited opcodes.

At the start of the program, there is a jump table that handles all the function calls and diverts control flow to the specific jump targets. Then, there is randomly-generated padding insns (which might throw off users), and then the function implementations. A function can call any function (including itself) using the jump table.

Memory load/store operations use a complex sequence of instructions to go to an arbitrary location, transfer data, and shift back. This requires some amount of by-hand analysis to work out.

```py
# writes data to an address provided on the stack in the format
# [temp, addr, data], and then "consumes" inputs off the stack, shifting to the right twice.
def memory_store():
    """
    Generates a stub which writes data to a sparse memory format at a target address.
    Since each load/store frame requires two data cells, it is suggested that all load/store
    addresses are even-numbered.

    This function expects the arguments on the stack: [temp, addr],
    and when done, "consumes" the inputs, shifting over to the right twice.
    The routine starts and ends on a temporary (top-of-stack) address.
    """
    return [
        "IDENTITY", "BUBBLE", "SHIFT", # create a return address
        "INC", "BUBBLE", "TAKE", "SHIFT", # copy the return address, go back
        "LEFT", "BUBBLE", "RIGHT", "SHIFT", # send the data over
        "BUBBLE", "INC", "SHIFT" # fetch the return address, return
    ]
```

As a result, we are able to write very complex programs supporting function calls, recursion, global variables, and more using a very simple bytecode architecture. The linker allows us to write powerful abstractions and scale programs way beyond what a human can analyze in their head. Solving this challenge requires not only dissecting each opcode, but seriously understanding the nature of the large automatically-generated programs.

```py
declare_function("main")
declare_function("print_a")
declare_function("print_digit")

define_function("print_a", [
    *load_number(97), "PRINT",
    "RIGHT", "JUMP" # print 'a' and return
])

define_function("print_digit", [
    "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
    "LEFT", *load_number(48), # ASCII code 48: '0'
    "ADD", "PRINT", "RIGHT",
    "RIGHT", "JUMP" # go back to return address and return
])

define_function("main", [
    *call_function("print_a") * 5,
    *load_number(7), "LEFT", *call_function("print_digit"), "RIGHT",
    *call_function("print_a"),
    *load_number(3), "LEFT", *load_number(0x02), "LEFT", *memory_store(),
    *load_number(1), "LEFT", *load_number(0x04), "LEFT", *memory_store(),
    *load_number(4), "LEFT", *load_number(0x06), "LEFT", *memory_store(),
    *call_function("print_a"),
    *load_number(0x02), "LEFT", *memory_load(), *call_function("print_digit"), "RIGHT",
    *load_number(0x04), "LEFT", *memory_load(), *call_function("print_digit"), "RIGHT",
    *load_number(0x06), "LEFT", *memory_load(), *call_function("print_digit"), "RIGHT",
    "RIGHT", "JUMP" # return from main
])
```

## The flag verifier
The flag verifier program accepts 36 body characters for the flag, flipping their case according to a pseudo-random number generator, and then encoding that character as a number according to a certain set of rules, storing it in a matrix B.
Matrix A is then initialized with the following contents:
```py
TARGET_MATRIX = [
    0, 1, 0, 0, 0, 0,
    1, 0, 1, 0, 0, 0,
    0, 1, 0, 1, 0, 0,
    0, 0, 1, 0, 1, 0,
    0, 0, 0, 1, 0, 1,
    0, 0, 0, 0, 1, 0,
]
```
The program expects matrix B to be the inverse of matrix A. According to the analysis performed in `solution.py` and `Unscrambler.java`, the expected solution is as follows:

```bash
echo "corctf{aHAhaHHAaAaAAAAhAhhahAaAAAaAAhhaHahA}" | ./bubble_vm program2.txt
```