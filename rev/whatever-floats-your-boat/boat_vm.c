#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <immintrin.h>

#include "types.h"

#define DEBUG_PRINT 0

// #pragma STDC FENV_ACCESS ON

#define GCC_VERSION (__GNUC__ * 10000 \
                               + __GNUC_MINOR__ * 100 \
                               + __GNUC_PATCHLEVEL__)
#if GCC_VERSION >= 40500
#define unreachable()  __builtin_unreachable()
#else
#define unreachable() do { printf("\nUnreachable code reached at code line [%d]\n", __LINE__); exit(EXIT_FAILURE); } while(0)
#endif

// no-exception util functions
static bool is_integral(double num);
static bool can_truncate_into_s64(double num);
static s64 as_integer(double num);

typedef enum : s64 {
    PUSH_CONST, POP, DUP, DUP2, DUP_X1, SWAP, NOP,
    ADD, SUB, MUL, DIV, FLOOR, CEIL, TRUNC, ROUND, ABS, MIN, MAX,
    CLEAR_EXCEPT, B_DIVBYZERO, B_INEXACT, B_INVALID, B_OVERFLOW, B_UNDERFLOW, B_ANY, B_ALWAYS,
    CALL, RET,
    PRINT_FLOAT, PRINT_CHAR, READ_FLOAT, READ_CHAR,
    LOAD, STORE
} insn_t;

static bool is_legal_insn(s64 insn_ordinal) {
    // only allow instructions that are defined
    return insn_ordinal >= PUSH_CONST && insn_ordinal <= STORE;
}

static bool is_branch_insn(insn_t insn) {
    // branch instructions are B_DIVBYZERO, B_INEXACT, B_INVALID, B_OVERFLOW, B_UNDERFLOW, B_ANY, B_ALWAYS
    return insn == B_DIVBYZERO || insn == B_INEXACT || insn == B_INVALID || 
           insn == B_OVERFLOW || insn == B_UNDERFLOW || insn == B_ANY || insn == B_ALWAYS;
}

static bool has_data(insn_t insn) {
    // PUSH_CONST, branch instructions, and CALL have data
    return insn == PUSH_CONST || insn == CALL || is_branch_insn(insn);
}

typedef struct {
    insn_t insn;
    double data;
} bytecode_t;

#define CALL_STACK_SIZE 1024
#define STACK_SIZE 2048
#define MEMORY_SIZE 4096

struct vm_state_t {
    bytecode_t *bytecode_buffer;
    size_t bytecode_buffer_length;
    size_t program_count; // current instruction pointer
    double *stack;
    size_t stack_size;
    size_t stack_depth; // current stack depth
    double *call_stack; // for CALL/RET
    size_t call_stack_size;
    size_t call_stack_depth; // current call stack depth
    double *memory;
    size_t memory_size;
} vm_state; // the only global var (hopefully)

void parse_file(const char *filename) {
    FILE *f = fopen(filename, "rb");
    if (!f) {
        printf("Failed to open file\n");
        exit(EXIT_FAILURE);
    }

    fseek(f, 0, SEEK_END);
    long filelen = ftell(f);
    rewind(f);

    size_t num_words = filelen / sizeof(double);

    double *parse_buffer = (double *) calloc(num_words, sizeof(double));
    if (!parse_buffer) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

    fread(parse_buffer, sizeof(double), num_words, f); // get all tokens

    // size guaranteed to be at least enough
    vm_state.bytecode_buffer = (bytecode_t *) calloc(num_words, sizeof(bytecode_t));
    if (!vm_state.bytecode_buffer) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

// debugging
#if DEBUG_PRINT
    for (size_t i=0; i<num_words; i++) {
        printf("[%zu] opcode: %.17g\n", i, parse_buffer[i]);
    }
#endif

    size_t bytecode_buffer_used = 0;
    for (size_t i=0; i<num_words; bytecode_buffer_used++) {
        if (!is_integral(parse_buffer[i])) {
            printf("[%zu] Illegal opcode: %.17g\n", i, parse_buffer[i]);
            exit(EXIT_FAILURE);
        }
        
        s64 insn_ordinal = as_integer(parse_buffer[i]);
        if (!is_legal_insn(insn_ordinal)) {
            printf("Unexpected opcode\n");
            exit(EXIT_FAILURE);
        }

        insn_t insn = (insn_t) insn_ordinal;
        i++;

        double data;
        if (has_data(insn)) {
            if (!(i<num_words)) {
                printf("Unexpected end of data\n");
                exit(EXIT_FAILURE);
            }
            data = parse_buffer[i];
            i++;
        } else {
            data = 0.0; // no data for this instruction
        }

        vm_state.bytecode_buffer[bytecode_buffer_used] = (bytecode_t) { .insn = insn, .data = data };
    }

    vm_state.bytecode_buffer_length = bytecode_buffer_used;

    free(parse_buffer);
}

void init_vm() {
    vm_state.stack_size = STACK_SIZE;
    vm_state.stack = calloc(vm_state.stack_size, sizeof(double));
    if (!vm_state.stack) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }
    vm_state.stack_depth = 0;

    vm_state.call_stack_size = CALL_STACK_SIZE;
    vm_state.call_stack = calloc(vm_state.call_stack_size, sizeof(double));
    if (!vm_state.call_stack) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }
    vm_state.call_stack_depth = 0;

    vm_state.memory_size = MEMORY_SIZE;
    vm_state.memory = calloc(vm_state.memory_size, sizeof(double));
    if (!vm_state.memory) {
        printf("Failed to allocate required data\n");
        exit(EXIT_FAILURE);
    }

    vm_state.program_count = 0;
    _MM_SET_EXCEPTION_STATE(0);
}

void free_vm() {
    free(vm_state.bytecode_buffer);
    free(vm_state.stack);
    free(vm_state.memory);
}

bool interpret_vm(); // returns whether the program is done

int main(int argc, char *argv[]) {
    if (argc < 2) {
        printf("Expected filename as first arg\n");
        exit(EXIT_FAILURE);
    }

    parse_file(argv[1]);
    init_vm();

    bool done;
    do {
        done = interpret_vm();
    } while (!done);

    free_vm();
    return 0;
}

struct unpacked_double_t {
    u64 sign;
    u64 biased_exponent;
    s64 unbiased_exponent;
    u64 mantissa;
};

// must not raise FPU exceptions
static struct unpacked_double_t unpack_double(double num) {
    union {
        double d;
        u64 u;
    } packed_bits;
    packed_bits.d = num;

    struct unpacked_double_t result;
    result.sign     = packed_bits.u >> 63;
    result.biased_exponent = (packed_bits.u >> 52) & 0x7FF;
    result.mantissa = packed_bits.u & (((u64)1 << 52) - 1);
    result.unbiased_exponent = (s64)result.biased_exponent - 1023;

    return result;
}

// must check without raising more FPU exceptions
static bool is_integral(double num) {
    struct unpacked_double_t unpacked = unpack_double(num);

    if (unpacked.biased_exponent == 0x7FF) {
        // NaN or Infinity -> not integral
        return false;
    }

    if (unpacked.biased_exponent == 0) {
        // Subnormal numbers (|x| < 2^-1022) or zero (if mantissa is also 0)
        return unpacked.mantissa == 0;
    }

    if (unpacked.unbiased_exponent < 0) {
        // absolute value less than 1.0
        return false;
    }

    if (unpacked.unbiased_exponent >= 52) {
        // all mantissa bits are shifted beyond the radix
        return true;
    }

    // Mask for fractional bits: the low (52 - unbiased_exponent) bits
    uint64_t frac_mask = ((u64)1 << (52 - unpacked.unbiased_exponent)) - 1;

    return (unpacked.mantissa & frac_mask) == 0;
}

static bool can_truncate_into_s64(double num) {
    struct unpacked_double_t unpacked = unpack_double(num);
    
    if (unpacked.biased_exponent == 0x7FF) {
        // NaN or Infinity -> cannot truncate
        return false;
    }

    return unpacked.unbiased_exponent < 62; // unbiased_exponent < 62 means the value can fit into s64 without overflow
}

// truncates the double to an integer value without raising FPU flags
// precondition: can_truncate_into_s64(num) must be true
static s64 as_integer(double num) {
    struct unpacked_double_t unpacked = unpack_double(num);
    if (unpacked.biased_exponent == 0) {
        // zero or subnormal number
        return 0;
    }

    // Reconstruct the integer value
    s64 result = (s64)(unpacked.mantissa | ((u64)1 << 52));
    s64 required_shift = unpacked.unbiased_exponent - 52;
    if (required_shift < 0) {
        // Shift right by the unbiased exponent
        result >>= -required_shift;
    } else {
        // Shift left by the unbiased exponent
        result <<= required_shift;
    }

    return (unpacked.sign == 1) ? -result : result;
}

static void assert_valid_program_count(size_t program_count) {
    if (!(program_count >= 0) || !(program_count < vm_state.bytecode_buffer_length)) {
        printf("Invalid program count: %zd\n", program_count);
        exit(EXIT_FAILURE);
    }
}

static void assert_valid_memory_address(size_t address) {
    if (!(address >= 0) || !(address < vm_state.memory_size)) {
        printf("Invalid memory address: %zd\n", address);
        exit(EXIT_FAILURE);
    }
}

bool interpret_vm() {
#if DEBUG_PRINT
    printf("program count: %zd, stack depth: %zd, call stack depth: %zd\n",
           vm_state.program_count, vm_state.stack_depth, vm_state.call_stack_depth);
#endif
    assert_valid_program_count(vm_state.program_count);
    bytecode_t bytecode = vm_state.bytecode_buffer[vm_state.program_count];
    if (bytecode.insn == RET) {
        if (vm_state.call_stack_depth == 0) {
            return true; // done
        }

        vm_state.program_count = (size_t) vm_state.call_stack[--vm_state.call_stack_depth];
        return false; // not done
    } else if (bytecode.insn == CALL) {
        if (vm_state.call_stack_depth >= vm_state.call_stack_size) {
            printf("Call stack overflow\n");
            exit(EXIT_FAILURE);
        }

        if (!can_truncate_into_s64(bytecode.data)) {
            printf("Invalid call target: %f\n", bytecode.data);
            exit(EXIT_FAILURE);
        }
        s64 call_address = as_integer(bytecode.data);

        vm_state.call_stack[vm_state.call_stack_depth++] = (double) vm_state.program_count + 1; // return to pc + 1
        vm_state.program_count = (size_t) call_address;
        assert_valid_program_count(vm_state.program_count);
        return false; // not done
    } else if (is_branch_insn(bytecode.insn)) {
        // Handle branch instructions
        bool b_condition;

        u32 exceptions = _MM_GET_EXCEPTION_STATE();
        switch (bytecode.insn) {
            case B_DIVBYZERO:
                b_condition = (exceptions & _MM_EXCEPT_DIV_ZERO) != 0;
                break;
            case B_INEXACT:
                b_condition = (exceptions & _MM_EXCEPT_INEXACT) != 0;
                break;
            case B_INVALID:
                b_condition = (exceptions & _MM_EXCEPT_INVALID) != 0;
                break;
            case B_OVERFLOW:
                b_condition = (exceptions & _MM_EXCEPT_OVERFLOW) != 0;
                break;
            case B_UNDERFLOW:
                b_condition = (exceptions & _MM_EXCEPT_UNDERFLOW) != 0;
                break;
            case B_ANY:
                b_condition = exceptions != 0; // any exception
                break;
            case B_ALWAYS:
                b_condition = true; // always true
                break;
            default:
                unreachable();
        }

        if (b_condition) {
            if (!can_truncate_into_s64(bytecode.data)) {
                printf("Invalid branch offset: %f\n", bytecode.data);
                exit(EXIT_FAILURE);
            }
            s64 branch_offset = as_integer(bytecode.data);
            vm_state.program_count += branch_offset;
            assert_valid_program_count(vm_state.program_count);
        } else {
            vm_state.program_count++; // skip the branch instruction
        }
        return false; // not done
    } else if (bytecode.insn == PUSH_CONST) {
        if (vm_state.stack_depth >= vm_state.stack_size) {
            printf("Stack overflow\n");
            exit(EXIT_FAILURE);
        }

        vm_state.stack[vm_state.stack_depth++] = bytecode.data;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == POP) {
        if (vm_state.stack_depth == 0) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        vm_state.stack_depth--;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == DUP) {
        if (vm_state.stack_depth == 0) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        if (vm_state.stack_depth >= vm_state.stack_size) {
            printf("Stack overflow\n");
            exit(EXIT_FAILURE);
        }

        vm_state.stack[vm_state.stack_depth] = vm_state.stack[vm_state.stack_depth - 1];
        vm_state.stack_depth++;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == DUP2) {
        if (vm_state.stack_depth < 2) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        if (vm_state.stack_depth + 1 >= vm_state.stack_size) {
            printf("Stack overflow\n");
            exit(EXIT_FAILURE);
        }

        vm_state.stack[vm_state.stack_depth] = vm_state.stack[vm_state.stack_depth - 2];
        vm_state.stack[vm_state.stack_depth + 1] = vm_state.stack[vm_state.stack_depth - 1];
        vm_state.stack_depth += 2;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == DUP_X1) { // b a -> a b a
        if (vm_state.stack_depth < 2) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        if (vm_state.stack_depth >= vm_state.stack_size) {
            printf("Stack overflow\n");
            exit(EXIT_FAILURE);
        }

        double top = vm_state.stack[vm_state.stack_depth - 1];
        memmove(&vm_state.stack[vm_state.stack_depth - 1], &vm_state.stack[vm_state.stack_depth - 2], sizeof(double) * 2);
        vm_state.stack[vm_state.stack_depth - 2] = top;
        vm_state.stack_depth++;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == SWAP) { // a b -> b a
        if (vm_state.stack_depth < 2) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double temp = vm_state.stack[vm_state.stack_depth - 1];
        vm_state.stack[vm_state.stack_depth - 1] = vm_state.stack[vm_state.stack_depth - 2];
        vm_state.stack[vm_state.stack_depth - 2] = temp;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == NOP) {
        vm_state.program_count++;
        return false; // not done
    } else if (
        bytecode.insn == ADD || bytecode.insn == SUB || bytecode.insn == MUL || bytecode.insn == DIV ||
        bytecode.insn == MIN || bytecode.insn == MAX
    ) {
        // binary operators a b -> result
        if (vm_state.stack_depth < 2) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double b = vm_state.stack[--vm_state.stack_depth];
        double a = vm_state.stack[--vm_state.stack_depth];

        double result;
        switch (bytecode.insn) {
            case ADD:
                result = a + b;
                break;
            case SUB:
                result = a - b;
                break;
            case MUL:
                result = a * b;
                break;
            case DIV:
                result = a / b;
                break;
            case MIN:
                result = fmin(a, b);
                break;
            case MAX:
                result = fmax(a, b);
                break;
            default:
                unreachable();
        }

        vm_state.stack[vm_state.stack_depth++] = result;
        vm_state.program_count++;
        return false; // not done
    } else if (
        bytecode.insn == FLOOR || bytecode.insn == CEIL || bytecode.insn == TRUNC ||
        bytecode.insn == ROUND || bytecode.insn == ABS
    ) {
        // unary operators a -> result
        if (vm_state.stack_depth < 1) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double value = vm_state.stack[--vm_state.stack_depth];
        
        double result;
        switch (bytecode.insn) {
            case FLOOR:
                result = floor(value);
                break;
            case CEIL:
                result = ceil(value);
                break;
            case TRUNC:
                result = trunc(value);
                break;
            case ROUND:
                result = round(value);
                break;
            case ABS:
                result = fabs(value);
                break;
            default:
                unreachable();
        }
        vm_state.stack[vm_state.stack_depth++] = result;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == CLEAR_EXCEPT) {
        // clear all FPU exceptions
        _MM_SET_EXCEPTION_STATE(0);
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == LOAD) {
        // address -> value
        if (vm_state.stack_depth < 1) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double address_value = vm_state.stack[--vm_state.stack_depth];
        if (!can_truncate_into_s64(address_value)) {
            printf("Invalid memory address: %f\n", address_value);
            exit(EXIT_FAILURE);
        }
        s64 address = as_integer(address_value);

        assert_valid_memory_address((size_t)address);
        vm_state.stack[vm_state.stack_depth++] = vm_state.memory[(size_t)address];
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == STORE) {
        // value address
        if (vm_state.stack_depth < 2) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double address_value = vm_state.stack[--vm_state.stack_depth];
        double store_val = vm_state.stack[--vm_state.stack_depth];

        if (!can_truncate_into_s64(address_value)) {
            printf("Invalid memory address: %f\n", address_value);
            exit(EXIT_FAILURE);
        }
        s64 address = as_integer(address_value);

        assert_valid_memory_address((size_t)address);
        vm_state.memory[(size_t)address] = store_val;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == PRINT_FLOAT) {
        // print a float value
        if (vm_state.stack_depth < 1) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double value = vm_state.stack[--vm_state.stack_depth];
        printf("%.17g", value); // according to the internet this is more precise than %f ??
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == PRINT_CHAR) {
        // print a char value
        if (vm_state.stack_depth < 1) {
            printf("Stack underflow\n");
            exit(EXIT_FAILURE);
        }

        double value = vm_state.stack[--vm_state.stack_depth];
        if (!can_truncate_into_s64(value)) {
            printf("Invalid character value: %f\n", value);
            exit(EXIT_FAILURE);
        }
        
        char c = (char)as_integer(value);
        putchar(c);
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == READ_FLOAT) {
        // read a float value from stdin
        if (vm_state.stack_depth >= vm_state.stack_size) {
            printf("Stack overflow\n");
            exit(EXIT_FAILURE);
        }

        double value;
        if (scanf("%lf", &value) != 1) {
            printf("Failed to read float value\n");
            exit(EXIT_FAILURE);
        }

        vm_state.stack[vm_state.stack_depth++] = value;
        vm_state.program_count++;
        return false; // not done
    } else if (bytecode.insn == READ_CHAR) {
        // read a char value from stdin
        if (vm_state.stack_depth >= vm_state.stack_size) {
            printf("Stack overflow\n");
            exit(EXIT_FAILURE);
        }

        int c = getchar();

        vm_state.stack[vm_state.stack_depth++] = (double)c;
        vm_state.program_count++;
        return false; // not done
    } else {
        printf("Unknown instruction: %ld\n", bytecode.insn);
        exit(EXIT_FAILURE);
    }
    unreachable();
}
