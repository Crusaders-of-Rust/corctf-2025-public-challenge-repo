#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "types.h"

#define unreachable() __builtin_unreachable()

typedef s64 cell_t;

/*
 * The VM is made of a linear memory space of cells which are addressable via indexing.
 * The VM has one register: a cell pointer A, which is used to calculate the A+1 pointer.
 * 
 * The program starts with A pointing to the cell at index 0.
 * Cell 0 is initially set to the length of the cell buffer, and all other cells are cleared.
 * 
 * Instruction set:
 * BUBBLE - swap values at A and A+1
 * TAKE - set A to A+1 value
 * LEFT, RIGHT - decrement/increment A pointer
 * BZ, BNZ - relative branch by numerical value in A if A+1 is zero/nonzero
 * ADD, SUB, MUL, DIV, MOD   - do integer operation on A and A+1 and store result in A
 * SLL, SRL, SRA, XOR, AND, OR - do integer operation on A and A+1 and store result in A
 * NONZERO - set cell A to 1 if A is nonzero, 0 otherwise (convert int to boolean 1/0)
 * NOT - flip bits of A
 * INC, DEC - increment/decrement the numerical value of A
 * CLEAR - set cell A to zero
 * INPUT - reads a char to A
 * PRINT - outputs A as char
 * EXIT - halt program and exit
 * IDENTITY - store cell address of A in A
 * SHIFT   - shift to A and write prev A+1 value (copies A+1 to target address)
 * JUMP - jump to A and write prev next instruction address (return address)
 */

typedef enum {
    BUBBLE, TAKE, LEFT, RIGHT, BZ, BNZ,
    ADD, SUB, MUL, DIV, MOD, SLL, SRL, SRA, XOR, AND, OR,
    NONZERO, NOT, INC, DEC, CLEAR, INPUT, PRINT, EXIT,
    IDENTITY, NOP, SHIFT, JUMP
} __attribute__ ((__packed__)) insn_t;

/*
b = [i.strip() for i in a.split(",")]
c = {b[i]: chr(ord('a') + i) for i in range(len(b))}
import pprint
pprint.pp(c)

{'BUBBLE': 'a',
 'TAKE': 'b',
 'LEFT': 'c',
 'RIGHT': 'd',
 'BZ': 'e',
 'BNZ': 'f',
 'ADD': 'g',
 'SUB': 'h',
 'MUL': 'i',
 'DIV': 'j',
 'MOD': 'k',
 'SLL': 'l',
 'SRL': 'm',
 'SRA': 'n',
 'XOR': 'o',
 'AND': 'p',
 'OR': 'q',
 'NONZERO': 'r',
 'NOT': 's',
 'INC': 't',
 'DEC': 'u',
 'CLEAR': 'v',
 'INPUT': 'w',
 'PRINT': 'x',
 'EXIT': 'y',
 'IDENTITY': 'z',
 'NOP': '{',
 'SHIFT': '|',
 'JUMP': '}'}
 */

#define CELL_BUFFER_LENGTH 2048

cell_t *cell_buffer;
insn_t *insn_buffer;
size_t insn_buffer_length;
size_t program_count;
size_t data_addr; // A

/** returns whether interpretation is done */
int interpret(void);

static void assert_legal_cell(size_t cell_addr) {
    if (!(cell_addr < CELL_BUFFER_LENGTH)) {
        printf("Invalid data address: %d\n", (int) cell_addr);
        printf("Program count: %d\n", (int) program_count);
        exit(EXIT_FAILURE);
    }
}

void init_buffer() {
    cell_buffer = calloc(CELL_BUFFER_LENGTH, sizeof(cell_t));
    if (!cell_buffer) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

    cell_buffer[0] = CELL_BUFFER_LENGTH;
    program_count = 0;
    data_addr = 0;
}

static int is_legal_insn(char c) {
    int n = c - 'a';
    return n >= BUBBLE && n <= JUMP;
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

    fread(buf, 1, filelen, f); // get all bytes

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

    parse_file(argv[1]);
    init_buffer();

    int done;
    do {
        done = interpret();
    } while (!done);

    free(insn_buffer);
    free(cell_buffer);
}

#define PRINT_DEBUG 0
#define DEBUG_DUMP 0

int interpret() {
#if PRINT_DEBUG
    printf("Current insn address: %ld\n", program_count);
    printf("Current data address: %ld\n", data_addr);
    printf("A, A+1: [%ld], [%ld]\n", cell_buffer[data_addr], cell_buffer[data_addr + 1]);
    printf("\n");
#endif
    if (!(program_count < insn_buffer_length)) {
        printf("Invalid instruction address: %ld\n", program_count);
        exit(EXIT_FAILURE);
    }

    assert_legal_cell(data_addr);

#define A data_addr
#define pc program_count
#define REQUIRE_NEXT() assert_legal_cell(A + 1)
    insn_t insn = insn_buffer[program_count];
    switch (insn) {
        cell_t temp;
        case BUBBLE:
            REQUIRE_NEXT();
            temp = cell_buffer[A];
            cell_buffer[A] = cell_buffer[A + 1];
            cell_buffer[A + 1] = temp;
            break;
        case TAKE:
            REQUIRE_NEXT();
            cell_buffer[A] = cell_buffer[A + 1];
            break;
        case LEFT:
            A--;
            break;
        case RIGHT:
            A++;
            break;
        case BZ:
            REQUIRE_NEXT();
            if (cell_buffer[A + 1] == 0) {
                pc += cell_buffer[A];
                pc--; // undo the increment ahead
            }
            break;
        case BNZ:
            REQUIRE_NEXT();
            if (cell_buffer[A + 1] != 0) {
                pc += cell_buffer[A];
                pc--; // undo the increment ahead
            }
            break;
        case ADD:
            REQUIRE_NEXT();
            cell_buffer[A] += cell_buffer[A + 1];
            break;
        case SUB:
            REQUIRE_NEXT();
            cell_buffer[A] -= cell_buffer[A + 1];
            break;
        case MUL:
            REQUIRE_NEXT();
            cell_buffer[A] *= cell_buffer[A + 1];
            break;
        case DIV:
            REQUIRE_NEXT();
            cell_buffer[A] /= cell_buffer[A + 1];
            break;
        case MOD:
            REQUIRE_NEXT();
            cell_buffer[A] %= cell_buffer[A + 1];
            break;
        case SLL:
            REQUIRE_NEXT();
            cell_buffer[A] <<= cell_buffer[A + 1];
            break;
        case SRL:
            REQUIRE_NEXT();
            cell_buffer[A] = (cell_t) ((u64) cell_buffer[A] >> cell_buffer[A + 1]);
            break;
        case SRA:
            REQUIRE_NEXT();
            cell_buffer[A] >>= cell_buffer[A + 1];
            break;
        case XOR:
            REQUIRE_NEXT();
            cell_buffer[A] ^= cell_buffer[A + 1];
            break;
        case AND:
            REQUIRE_NEXT();
            cell_buffer[A] &= cell_buffer[A + 1];
            break;
        case OR:
            REQUIRE_NEXT();
            cell_buffer[A] |= cell_buffer[A + 1];
            break;
        case NONZERO:
            cell_buffer[A] = (cell_buffer[A] != 0) ? 1 : 0;
            break;
        case NOT:
            cell_buffer[A] = ~cell_buffer[A];
            break;
        case INC:
            cell_buffer[A]++;
            break;
        case DEC:
            cell_buffer[A]--;
            break;
        case CLEAR:
            cell_buffer[A] = 0;
            break;
        case INPUT:
            cell_buffer[A] = (cell_t) getchar(); // reads -1 on EOF or error
            break;
        case PRINT:
            putchar((char) cell_buffer[A]);
#if PRINT_DEBUG
            printf("\nJUST PRINTED A CHAR: %c {%ld}\n", (char) cell_buffer[A], cell_buffer[A]);
#endif
#if DEBUG_DUMP
            if (A != 0 && insn_buffer[pc - 1] == PRINT) { // triggered by two consecuitive print insns
                printf("\n\nDEBUG_DUMP\n");
                for (int i=0; i<36*2*2; i++) {
                    printf("[%4d]: {%ld}\n", i, cell_buffer[i]);
                }
            }
#endif
            break;
        case EXIT:
            return 1; // done !
        case IDENTITY:
            cell_buffer[A] = (cell_t) A;
            break;
        case NOP:
            break;
        case SHIFT:
            temp = cell_buffer[A + 1];
            A = cell_buffer[A];
            assert_legal_cell(A);
            cell_buffer[A] = temp;
            break;
        case JUMP:
            temp = (cell_t) pc + 1;
            pc = cell_buffer[A];
            pc --; // undo the increment ahead
            cell_buffer[A] = temp;
            break;
        default:
            unreachable();
    }
    pc++; // do this beforehand since all pc calculations are from next insn
#undef A
#undef pc
    return 0; // not done
}
