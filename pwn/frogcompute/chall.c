#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stddef.h>
#include <sys/types.h>

#define unreachable() __builtin_unreachable()

int is_valid_bf_insn(char c) {
    return c == '>' || c == '<' || c == '+' || c == '-' ||
           c == '[' || c == ']' || c == '#' ; // i/o not supported
}

#define INSTRUCTION_MEMORY_SIZE 90
#define DATA_MEMORY_SIZE 256

int parse_program(const char *code, const size_t code_length, char instructions[INSTRUCTION_MEMORY_SIZE]) {
    memset(instructions, '#', INSTRUCTION_MEMORY_SIZE); // halt instruction
    size_t count = 0;
    for (size_t i = 0; i < code_length; i++) {
        if (is_valid_bf_insn(code[i])) {
            if (count < INSTRUCTION_MEMORY_SIZE - 1) {
                instructions[count++] = code[i];
            } else {
                printf("Code too large\n");
                return -1;
            }
        }
    }
    return 0;
}

int interpret_program(char instructions[INSTRUCTION_MEMORY_SIZE]);

int main() {
    setvbuf(stdin, NULL, _IONBF, 0);
    setvbuf(stdout, NULL, _IONBF, 0);

    printf("Frog Software Foundation bf interpreter\n\n");
    printf(
        "Supported instructions:\n"
        "  + increment cell\n"
        "  - decrement cell\n"
        "  < prev cell\n"
        "  > next cell\n"
        "  # halt program\n"
        "  [ ... ] repeat while cell is nonzero\n\n"
    );
    printf(
        "The Frog Software Foundation is pleased to offer "
        "free interpretation for programs up to %d instructions long "
        "with a memory of %d byte cells.\n\n",
        INSTRUCTION_MEMORY_SIZE, DATA_MEMORY_SIZE
    );
    printf(
        "Thank you for choosing The Frog Software Foundation's "
        "platform-as-a-service business model.\n\n"
    );
    printf("Enter bf code:\n");

    char *code_data = NULL;
    size_t code_size = 0;

    ssize_t read_size = getline(&code_data, &code_size, stdin);
    if (read_size == -1) {
        printf("Error reading input\n");
        free(code_data);
        return EXIT_FAILURE;
    }
    
    char instructions[INSTRUCTION_MEMORY_SIZE];
    int err = parse_program(code_data, read_size, instructions);
    if (err) {
        printf("Parse error - exiting\n");
        free(code_data);
        return EXIT_FAILURE;
    }

    int result = interpret_program(instructions);
    free(code_data);
    return result;
}

// only meant to be called when `+` is run at 0x69420 !
void mmio_dump_flag() {
    printf("mmio procedure invoked\n");

    FILE *fptr = fopen("flag.txt", "r");  
    if (fptr == NULL) {
        printf("flag.txt not found\n");  
        return; 
    }

    // Read and print each character from the file
    char ch;
    while ((ch = fgetc(fptr)) != EOF) {
        putchar(ch);
    }

    fclose(fptr);  
}

int interpret_program(char instructions[INSTRUCTION_MEMORY_SIZE]) {
    // interpret the instructions
    struct {
        size_t data_size;
        unsigned char data_memory[DATA_MEMORY_SIZE];
    } data = { DATA_MEMORY_SIZE, { 0 } };
    // <<<<<<<< will get the least significant byte
    // <<<<<    will get the most significant byte

    size_t instruction_pointer = 0;
    size_t data_pointer = 0;
    while (1) {

        char insn = instructions[instruction_pointer];
        unsigned char *ref_addr = &data.data_memory[data_pointer];
        unsigned char *end_addr = &data.data_memory[data.data_size - 1];
        switch (insn) {
            case '>':
                data_pointer++;
                instruction_pointer++;
                break;
            case '<':
                data_pointer--;
                instruction_pointer++;
                break;
            case '+':
                // oob is UB no matter what, but for -O0 this should allow underflow
                if (ref_addr > end_addr) {
                    printf("Out of bounds data access\n");
                    return EXIT_FAILURE;
                }

                (*ref_addr)++;
                instruction_pointer++;

                // handle "mmio"
                if (instruction_pointer == 0x69420) {
                    mmio_dump_flag();
                }

                break;
            case '-':
                // oob is UB no matter what, but for -O0 this should allow underflow
                if (ref_addr > end_addr) {
                    printf("Out of bounds data access\n");
                    return EXIT_FAILURE;
                }

                (*ref_addr)--;
                instruction_pointer++;
                break;
            case '[':
                if (data.data_memory[data_pointer] == 0) {
                    // Jump to matching ']'
                    int bracket_count = 1;
                    while (bracket_count > 0) {
                        instruction_pointer++;
                        if (instruction_pointer >= INSTRUCTION_MEMORY_SIZE) {
                            printf("No matching ']' found in code\n");
                            return EXIT_FAILURE;
                        }
                        char skip_insn = instructions[instruction_pointer];
                        if (skip_insn == '[') bracket_count++;
                        if (skip_insn == ']') bracket_count--;
                    }
                }
                instruction_pointer++; // Move past the ']' or '['
                break;
            case ']':
                if (data.data_memory[data_pointer] != 0) {
                    // Jump back to matching '['
                    int bracket_count = 1;
                    while (bracket_count > 0) {
                        if (instruction_pointer == 0) {
                            printf("No matching '[' found in code\n");
                            return EXIT_FAILURE;
                        }
                        instruction_pointer--;

                        char skip_insn = instructions[instruction_pointer];
                        if (skip_insn == '[') bracket_count--;
                        if (skip_insn == ']') bracket_count++;
                    }
                }
                instruction_pointer++; // Move past the ']' or '['
                break;
            case '#':
                // Halt instruction
                goto program_end;
            default:
                unreachable();
        }
    }

    program_end:
    printf("Program halted!\n");
    printf("Data memory:\n");
    for (size_t i = 0; i < DATA_MEMORY_SIZE; i++) {
        printf("[0x%02hhX] ", data.data_memory[i]); // format: "[0x00] "
    }
    printf("\n");
    printf("data pointer: %zu\n", data_pointer);

    return EXIT_SUCCESS;
}
