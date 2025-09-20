#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

#define unreachable() __builtin_unreachable()


/*
 * The VM is made of a linear memory space of cells which are addressable via indexing.
 * The VM has one register: a cell pointer A, which is used to calculate the A+1 pointer.
 * 
 * The program starts with A pointing to the cell at index 0.
 * Cell 0 is initially set to the length of the cell buffer, and all other cells are cleared.
 * 
 * Instruction set:
 * ROLL: rotate all the register values left: raw[0] = raw[1], raw[1] = raw[2], raw[2] = raw[3], raw[3] = raw[0]
 * UNROLL: rotate all the register values right: raw[0] = raw[3], raw[1] = raw[0], raw[2] = raw[1], raw[3] = raw[2]
 * SWAP: exchange r1 and r2
 * SWAP_2: exchange r1 and r3
 * TAKE: r1 = r2
 * ADD: r1 = r1 + r2
 * SUB: r1 = r1 - r2
 * MUL: r1 = r1 * r2
 * DIV: r1 = r1 / r2
 * MOD: r1 = r1 % r2
 * ZERO: r1 = 0
 * INC: r1 = r1 + 1
 * DEC: r1 = r1 - 1
 * PUSH: push[r1]
 * POP: r1 = pop[]
 * INPUT: r1 = getchar()
 * PRINT: putchar(r1)
 * NOP: nop
 * EXIT: exit(0);
 */

typedef enum {
    ROLL, UNROLL, SWAP, SWAP_2, TAKE,
    ADD, SUB, MUL, DIV, MOD, ZERO, INC, DEC,
    PUSH, POP, INPUT, PRINT, NOP, EXIT
} __attribute__ ((__packed__)) insn_t;

#define STACK_SIZE 2048

typedef union register_state {
    struct {
        s64 pc;
        s64 r1;
        s64 r2;
        s64 r3;
    } data;
    s64 raw[4];
} register_state_t;

register_state_t reg_state;
s64 *stack;
size_t sp; // stack depth
insn_t *insn_buffer;
size_t insn_buffer_length;


/** returns whether interpretation is done */
int interpret(void);


void init_buffer() {
    stack = calloc(STACK_SIZE, sizeof(s64));
    if (!stack) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

    reg_state = (register_state_t) {
        .data = {
            .pc = 0,
            .r1 = 0, 
            // .r1 = (s64)(18446744073700000000UL), // for faster solving
            .r2 = 0,
            .r3 = 0
        }
    }; 
}

static int is_legal_insn(char c) {
    int n = c - 'a';
    return n >= ROLL && n <= EXIT;
}

static insn_t parse_insn(char c) {
    int n = c - 'a';
    return (insn_t) n;
}

void parse_file(const char *filename) {
    FILE *f = fopen(filename, "rb");
    if (!f) {
        printf("Failed to open file\n");
        exit(EXIT_FAILURE);
    }

    fseek(f, 0, SEEK_END);
    long filelen = ftell(f);
    rewind(f);

    char *buf = (char *) malloc(filelen);
    if (!buf) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

    int a = fread(buf, 1, filelen, f); // get all bytes
    (void) a;

    insn_buffer_length = (size_t) filelen;
    insn_buffer = (insn_t *) calloc(insn_buffer_length, sizeof(insn_t));
    if (!insn_buffer) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

    size_t insn_buffer_used = 0;
    for (long i=0; i<filelen; i++) {
        if (!is_legal_insn(buf[i]))
            continue;
        insn_buffer[insn_buffer_used] = parse_insn(buf[i]);
        insn_buffer_used++;
    }

    insn_buffer_length = insn_buffer_used;

    free(buf);
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Expected filename as first arg\n");
        exit(EXIT_FAILURE);
    }

    if (argc == 10) {
        for (int i=0; i<10; i++) printf("corctf{");
        printf("corctf 2025 easter egg\n");
        for (int i=0; i<10; i++) printf("ragebait\n");
        exit(EXIT_FAILURE);
    }

    parse_file(argv[1]);
    init_buffer();

    int done;
    do {
        done = interpret();
    } while (!done);

    free(insn_buffer);
    free(stack);
}

#define PRINT_DEBUG 0
#define DEBUG_DUMP 0
#define pc reg_state.data.pc

int interpret() {
#if PRINT_DEBUG
    printf("Current insn address: %ld\n", pc);
    printf("Stack depth: %ld\n", sp);
    if (sp > 0) {
        printf("Top of stack: %ld\n", stack[sp - 1]);
    }
    printf("pc: %ld, r1: %ld, r2: %ld, r3: %ld\n",
                    pc, reg_state.data.r1, reg_state.data.r2, reg_state.data.r3);
    printf("\n");

    if (!(pc < insn_buffer_length)) {
        printf("Invalid instruction address: %ld\n", pc);
        exit(EXIT_FAILURE);
    }
#else
    if (!(pc < insn_buffer_length)) {
        printf("VM error\n");
        exit(EXIT_FAILURE);
    }
#endif

    insn_t insn = insn_buffer[pc];
    switch (insn) {
        s64 temp;
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
        case SWAP:
            temp = reg_state.data.r1;
            reg_state.data.r1 = reg_state.data.r2;
            reg_state.data.r2 = temp;
            break;
        case SWAP_2:
            temp = reg_state.data.r1;
            reg_state.data.r1 = reg_state.data.r3;
            reg_state.data.r3 = temp;
            break;
        case TAKE:
            reg_state.data.r1 = reg_state.data.r2;
            break;
        case ADD:
            reg_state.data.r1 += reg_state.data.r2;
            break;
        case SUB:
            reg_state.data.r1 -= reg_state.data.r2;
            break;
        case MUL:
            reg_state.data.r1 *= reg_state.data.r2;
            break;
        case DIV:
            reg_state.data.r1 = (s64) ((u64)reg_state.data.r1 / (u64)reg_state.data.r2); // lol!
            break;
        case MOD:
            reg_state.data.r1 = (s64) ((u64)reg_state.data.r1 % (u64)reg_state.data.r2); // lol!
            break;
        case ZERO:
            reg_state.data.r1 = 0;
            break;
        case INC:
            reg_state.data.r1++;
            break;
        case DEC:
            reg_state.data.r1--;
            break;
        case PUSH:
            if (sp >= STACK_SIZE) {
#ifdef PRINT_DEBUG
                printf("Stack overflow\n");
#else
                printf("VM error\n");
#endif
                exit(EXIT_FAILURE);
            }
            stack[sp++] = reg_state.data.r1;
            break;
        case POP:
            if (sp == 0) {
#ifdef PRINT_DEBUG
                printf("Stack underflow\n");
#else
                printf("VM error\n");
#endif
                exit(EXIT_FAILURE);
            }
            reg_state.data.r1 = stack[--sp];
            break;
        case INPUT:
            reg_state.data.r1 = getchar(); // reads -1 on EOF or error
            break;
        case PRINT:
            putchar((char) reg_state.data.r1);
#if PRINT_DEBUG
            printf("\nJUST PRINTED A CHAR: %c {%ld}\n", (char) reg_state.data.r1, reg_state.data.r1);
#endif
#if DEBUG_DUMP
            if (insn_buffer[pc - 1] == PRINT) { // triggered by two consecuitive print insns
                printf("\n\nDEBUG_DUMP\n");
                printf("pc: %ld, r1: %ld, r2: %ld, r3: %ld\n",
                    pc, reg_state.data.r1, reg_state.data.r2, reg_state.data.r3);
            }
#endif
            break;
        case EXIT:
            return 1; // done !
        case NOP:
            break;
        default:
            unreachable();
    }

    pc++; // do this beforehand since all pc calculations are from next insn
    return 0; // not done
}
