# Roll VM

This challenge was made extremely last minute, so I didn't have enough time to make 
a more sophisticated program using this VM. Instead, the program just 
has a long loop stalling for UINT64_MAX iterations and then prints the flag.

## Control Flow

Roll VM is branchless by design. The program state is composed of four 
64-bit integers, the first of which is used as a program counter.

```c
typedef union register_state {
    struct {
        s64 pc;
        s64 r1;
        s64 r2;
        s64 r3;
    } data;
    s64 raw[4];
} register_state_t;
```

Instead of havin explicit branching or jumping instructions, 
Roll VM has `ROLL` and `UNROLL` opcodes which will cycle around the four registers, 
emulating the equivalent of `r0, r1, r2, r3 = r1, r2, r3, r0`. 
This looks really esoteric (and it is!), yet these two operations are just what we need 
in order to perform function calls and returns: a `ROLL` opcode will jump to the 
target address in `r1`, while an `UNROLL` opcode will take the control flow to the 
return address.

The way that these instructions were implemented using a raw union and memmove 
actually allowed the optimiser to use SIMD registers to perform the transformation. 
This also happened to make the VM's control flow harder to analyze. 

```c
case ROLL:
    temp = reg_state.raw[0];
    memmove(&reg_state.raw, &reg_state.raw[1], 3 * sizeof(reg_state.raw[0]));
    reg_state.raw[3] = temp;
    pc--; // pre-decrement to avoid effects of increment
    break;
case UNROLL:
    temp = reg_state.raw[3];
    memmove(&reg_state.raw[1], &reg_state.raw, 3 * sizeof(reg_state.raw[0]));
    reg_state.raw[0] = temp;
    break;
```

## The Program

The program was just a stall loop which would print the flag when done. If I had more time to 
develop this challenge, I would have added additional bytecodes for inspecting the program counter of 
the VM and handling numerical data for boolean operations. 
