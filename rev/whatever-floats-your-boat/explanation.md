# Boat VM

Boat VM is a VM which only uses floating points as an underlying datatype. Additionally, its 
control flow is decided using a perculiar primtive: branching was decided based on the 
exception flags in the FPU's MXCSR register. The VM included bytecodes such as 
`B_DIVBYZERO`, `B_INEXACT`, and `B_OVERFLOW`.

The VM then used the behavior of floating-point values to implement several algorithms. 
For example, this is a function that checks whether a number is greater than equal to one, 
by adding the value 2^-53 and seeing if it raises a FPU INEXACT exception.

```java
// function: return whether n greater than equal to 1
int greaterThanOrEquals1Index = getProgramCount(buffer); // where the function starts
System.out.println("linking greaterThanOrEquals1Index at pc " + greaterThanOrEquals1Index);
buffer.putDouble(PUSH_CONST.encode()).putDouble(0.000042006942069)
    .putDouble(MAX.encode()) // add minimum threshold to 1) make positive 2) prevent small magnitude errors
    .putDouble(PUSH_CONST.encode()).putDouble(makeDouble(false, -53, 0))
    .putDouble(CLEAR_EXCEPT.encode())
    .putDouble(ADD.encode()) // n + 2^(-53) is inexact if n >= 1 due to exponent
    .putDouble(POP.encode()) // we didn't need the result, only the fpu flags
    .putDouble(PUSH_CONST.encode()).putDouble(1)
    .putDouble(PUSH_CONST.encode()).putDouble(0) // 1, 0
    .putDouble(B_INEXACT.encode()).putDouble(2) // skip 2 instructions if inexact
    .putDouble(SWAP.encode()) // 0, 1
    .putDouble(POP.encode()) // if inexact, keep 1, else keep 0
    .putDouble(RET.encode());
```

The actual architecture of the VM is pretty simple - the program has a separate data stack and call stack, as well as 
dedicated `LOAD` and `STORE` instructions. Additionally, the `CALL` opcode could only be used with a constant 
immediate value, so all control flow operations were entirely transparent and could be statically analyzed.

## The Program

I wrote an assembler in Java which would generate the program that solvers had 
access to. The program was a flag solver which would calculate the correct flag by 
solving a sudoku puzzle and computing its hash. However, the builtin backtracking 
algorithm was extremely inefficient would have taken eons to compute on its own. 
Reverse engineers had to work out what was going on, and then patch the program 
to run with the already-solved sudoku puzzle.

Although this puzzle used an extremely niche primitive, the control flow was still 
relatively sane, so it ended up being slightly easier to solve than `bubble-vm`.
