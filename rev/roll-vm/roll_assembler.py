INSNS = {
    'ROLL': 'a',
    'UNROLL': 'b',
    'SWAP': 'c',
    'SWAP_2': 'd',
    'TAKE': 'e',
    'ADD': 'f',
    'SUB': 'g',
    'MUL': 'h',
    'DIV': 'i',
    'MOD': 'j',
    'ZERO': 'k',
    'INC': 'l',
    'DEC': 'm',
    'PUSH': 'n',
    'POP': 'o',
    'INPUT': 'p',
    'PRINT': 'q',
    'NOP': 'r',
    'EXIT': 's'
}


def assemble_stub(asm_list):
    result = ''
    for token in asm_list:
        if token in INSNS:
            result += INSNS[token]
        else:
            print(f"Warning: unknown symbol: {token}")
    return result

# flattens a 2d array into a 1d array
def flatten(asm_list_list):
    return [insn for asm_list in asm_list_list for insn in asm_list]


def u64_to_bitstring(num):
    if num < 0:
        raise ValueError("Cannot convert negative number to bits")
    binary_string = bin(num) # format 0b1001
    return binary_string[2:]    

# loads a number into r1. 
# use this when it is okay to clobber r2.
def load_number(num):
    needs_negate = num < 0
    unsigned_num = abs(num)

    # algorithm: bitmask the absval, and then subtract from 0 if negative
    prologue = ["ZERO", "PUSH", "INC", "SWAP"]
    # sets up format: partial sum on stack, bitmask in r2
    bitmask = []
    for digit in u64_to_bitstring(unsigned_num)[::-1]:
        if digit == '1':
            bitmask += ["POP", "ADD", "PUSH"]  # take it in
        bitmask += ["ZERO", "INC", "INC", "MUL", "SWAP"]  # r2 *= 2
    pos_epilogue = ["POP"]
    neg_epilogue = ["ZERO", "DEC", "SWAP", "POP", "MUL"]
    epilogue = neg_epilogue if needs_negate else pos_epilogue

    return prologue + bitmask + epilogue


ASM = []

import random
random.seed(1337)
insn_list = list(INSNS.keys())

def pad_until(target_length):
    required_padding = target_length - len(ASM)
    if required_padding < 0:
        print(f"Warning: insn overflow from {len(ASM)} to {target_length}")
    for i in range(required_padding):
        ASM.append(random.choice(insn_list))

# main code
ASM += ["INC"] # n++
ASM += ["PUSH"]
ASM += ["SWAP"]
ASM += ["ZERO", "DEC"]  # load -1
ASM += ["SWAP", "DIV", "PUSH"]  # push n / (-1L)
ASM += load_number(15)
ASM += ["SWAP", "TAKE", *["DEC"] * 5, "MUL"]  # load number 150
ASM += ["SWAP", "POP", "MUL", "SWAP"]  # r2 = (n / (-1L)) * 150
ASM += ["POP", "SWAP"]  # r1 = (n / (-1L)) * 150, r2 = n
ASM += ["ROLL"]  # if r1 == 0, loop again, if r1 == 1, call function 
ASM += ["EXIT"]
print(f"Exiting main at pc {len(ASM)}")

pad_until(102)
# function: print n, n+1
ASM += ["PRINT", "INC", "PRINT", "UNROLL"]

# function: print n +8, -6, +1 (only works if n = 100)
pad_until(110)
ASM += ["INC"] * 8
ASM += ["PRINT"]
ASM += ["DEC"] * 6
ASM += ["SWAP", "TAKE"]
ASM += ["ROLL"] # call function at n+8 = 102
ASM += ["UNROLL"] # return

# function: after delay, print flag
pad_until(150)
ASM += load_number(2008)
ASM += ["ROLL"]  # call print flag
ASM += ["UNROLL"]  # return
print(f"Return pc from print_flag: {len(ASM)}")

FLAG_START = "corctf{"
# function: print the flag start
pad_until(404)
ascii_base = ord('a')
ASM += load_number(ascii_base)
ASM += ["PUSH"]
for c in FLAG_START:
    neg_offset = ascii_base - ord(c)
    # print (ascii_base - neg_offset)
    ASM += load_number(neg_offset)
    ASM += ["SWAP", "POP", "PUSH"]  # base, neg_offset
    ASM += ["SUB", "PRINT"]
ASM += ["POP"]  # delete the base
ASM += ["UNROLL"]  # return
print(f"Return pc from print_flag_start: {len(ASM)}")

# function: print "lfg"
pad_until(729)
ASM += ["SWAP_2", "PUSH", "SWAP_2"]  # push ret address
ASM += load_number(100)
ASM += ["SWAP", "TAKE", *["INC"] * 10] # r1, r2 = 110, 100
ASM += ["ROLL"]  # call function with parameter 100
ASM += ["POP", "ROLL"]  # pop ret address and return

FLAG_MIDDLE = "_i_L0V3_"
# function: print flag middle-end
pad_until(888)
ascii_base = ord('_')
ASM += ["NOP"]
ASM += ["SWAP_2", "PUSH", "SWAP_2"]  # push ret address
ASM += load_number(ascii_base)
ASM += ["SWAP_2"]  # store ascii_base in r3
for c in FLAG_MIDDLE[::-1]:
    neg_offset = ascii_base - ord(c)
    # print (ascii_base - neg_offset)
    ASM += load_number(neg_offset)
    ASM += ["SWAP"]
    ASM += ["SWAP_2", "PUSH", "SWAP_2", "POP"]  # base, neg_offset
    ASM += ["SUB", "PUSH"]
ASM += load_number(1337)
ASM += ["ROLL"]  # call function to print pushed chars
ASM += ["POP", "ROLL"]  # pop ret address and return
print(f"Return pc from print_flag_middle: {len(ASM)}")

# function: print len(FLAG_MIDDLE) chars from stack, then print end
pad_until(1337)
for i in range(len(FLAG_MIDDLE)):
    ASM += ["POP", "PRINT"]

FLAG_END = "cNtrl_fl0W}\n"
# function: print flag end
# pad_until(1360)
for c in FLAG_END:
    ASM += load_number(ord(c))
    ASM += ["PRINT"]
ASM += ["EXIT"]  # exit program - dead end

# function: print flag
pad_until(2008)
ASM += ["SWAP_2", "PUSH", "SWAP_2"]  # push ret address
ASM += load_number(404)
ASM += ["ROLL"] # call print
ASM += ["INC"] # steal yield address, continue execution of next segment of code
ASM += ["ROLL"]  # call print lfg
ASM += load_number(888)
ASM += ["ROLL"]  # call print flag middle-end; noreturn function

print(f"Program End at pc {len(ASM)}")

assembled_data = assemble_stub(ASM)
assembled_data += "eeeeeecorctf"
print("\nCompiled:\n\n")
print(assembled_data)

OUTPUT_FILENAME = "program1.txt"
with open(OUTPUT_FILENAME, "w") as f:
    f.write(assembled_data)

print(f"\n\nWrote output to {OUTPUT_FILENAME}\n")

print(f"{FLAG_START}lfg{FLAG_MIDDLE}{FLAG_END}")
