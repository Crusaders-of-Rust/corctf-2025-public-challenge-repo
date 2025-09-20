package team.cor.corctf.rev.whatever_floats_your_boat;

public enum OpCode {
    PUSH_CONST, POP, DUP, DUP2, DUP_X1, SWAP, NOP,
    ADD, SUB, MUL, DIV, FLOOR, CEIL, TRUNC, ROUND, ABS, MIN, MAX,
    CLEAR_EXCEPT, B_DIVBYZERO, B_INEXACT, B_INVALID, B_OVERFLOW, B_UNDERFLOW, B_ANY, B_ALWAYS,
    CALL, RET,
    PRINT_FLOAT, PRINT_CHAR, READ_FLOAT, READ_CHAR,
    LOAD, STORE;

    public double encode() {
        return (double) this.ordinal();
    }

    public boolean isBranchInsn() {
        return this == B_DIVBYZERO || this == B_INEXACT || this == B_INVALID ||
               this == B_OVERFLOW || this == B_UNDERFLOW || this == B_ANY || this == B_ALWAYS;
    }

    public boolean hasData() {
        return this == PUSH_CONST || this == CALL || isBranchInsn();
    }
}
