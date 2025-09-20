
INSNS = {
    'BUBBLE': 'a',
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
    'JUMP': '}'
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

# all functions must be declared beforehand, and then implemented
# the first function in the list is reserved and implemented by linker
# the program must include a "main" function
FUNCTIONS = [
    "_start",
]

if len(FUNCTIONS) > len(set(FUNCTIONS)):
    raise NameError("Duplicate function names declared")

FUNCTION_IMPLEMENTATIONS = {} # filled in by user, linked last
CALLED_FUNCTIONS = { "_start" } # used to track unused functions

def declare_function(name):
    FUNCTIONS.append(name)
    if len(FUNCTIONS) > len(set(FUNCTIONS)):
        raise NameError("Duplicate function names declared")

def define_function(name, implementation):
    FUNCTION_IMPLEMENTATIONS[name] = implementation


# program count 0 is a BUBBLE insn, which will result in entry to the jump table,
# calling _start (function 0) with args MEMORY_SIZE
# program count 3 does a branch to handle special case 0 return address
# generates a stub which calls the function using the jump table at program count 3
def call_function(function_name):
    """
    Generates a stub which calls the given function name using the jump table at program count 0.

    Calling convention:
        The current pointer A is a temp value (can be overwritten)
        Data to the left of the pointer is unused stack space (can be overwritten)
        Arguments are to the right of the current pointer (starting at A+1)
        Functions enter with the format [temp, ret_addr, args...]
        Functions will write the result to the left of the return address, exactly where they entered
    
    Jump table convention:
        A contains return address, A+1 contains function index
        Contains at least one temp space to the left
        Special case: return address is 0, jump directly to entry funct
        Located at program count 3, which does a branch to handle special case
        A contains return address
    """
    function_index = FUNCTIONS.index(function_name)
    CALLED_FUNCTIONS.add(function_name)
    load_index = ["CLEAR"] + ["INC"] * function_index
    call_jump_table = ["LEFT", "CLEAR", "INC", "INC", "INC", "JUMP"]
    return load_index + call_jump_table


# loads data from an address provided on the stack in the format
# [temp, addr], and then "consumes" it off the stack, writing the read data in its place.
def memory_load():
    """
    Generates a stub which loads data from a sparse memory format at a target address.
    Since each load/store frame requires two data cells, it is suggested that all load/store
    addresses are even-numbered.

    This function expects the arguments on the stack: [temp, addr],
    and when done, "consumes" the element, replacing it with the read data.
    The routine starts and ends on a temporary (top-of-stack) address.
    """
    return ["IDENTITY", "BUBBLE", "SHIFT", "SHIFT", "BUBBLE"]

# writes data to an address provided on the stack in the format
# [temp, addr, data], and then "consumes" inputs off the stack, shifting to the right twice.
def memory_store():
    """
    Generates a stub which writes data to a sparse memory format at a target address.
    Since each load/store frame requires two data cells, it is suggested that all load/store
    addresses are even-numbered.

    This function expects the arguments on the stack: [temp, addr, data],
    and when done, "consumes" the inputs, shifting over to the right twice.
    The routine starts and ends on a temporary (top-of-stack) address.
    """
    return [
        "IDENTITY", "BUBBLE", "SHIFT", # create a return address
        "INC", "BUBBLE", "TAKE", "SHIFT", # copy the return address, go back
        "LEFT", "BUBBLE", "RIGHT", "SHIFT", # send the data over
        "BUBBLE", "INC", "SHIFT" # fetch the return address, return
    ]


def u64_to_bitstring(num):
    if num < 0:
        raise ValueError("Cannot convert negative number to bits")
    binary_string = bin(num) # format 0b1001
    return binary_string[2:]    

# loads a number into a cell. 
# Use this when exact sizing is not critical, and there is space to the left.
def load_number(num):
    needs_negate = num < 0
    unsigned_num = abs(num)

    # algorithm: bitmask the absval, and then subtract from 0 if negative
    prologue = ["CLEAR", "INC", "LEFT", "CLEAR", "INC", "LEFT", "CLEAR", "RIGHT"]
    # sets up format [result, mask, 1], initially [0, 1, 1]
    bitmask = []
    for digit in u64_to_bitstring(unsigned_num)[::-1]:
        if digit == '1':
            bitmask += ["LEFT", "OR", "RIGHT"]
        bitmask.append("SLL") # shift the mask bit to the left
    pos_epilogue = ["LEFT", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT"]
    neg_epilogue = ["CLEAR", "LEFT", "BUBBLE", "SUB", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT"]
    epilogue = neg_epilogue if needs_negate else pos_epilogue

    return prologue + bitmask + epilogue

# conservative estimate of how many insns long load_number is.
# Always greater than or equal to actual length.
def load_number_estimate(num):
    num_bits_estimate = max(len(u64_to_bitstring(abs(num))) + 1, 3)
    prologue_estimate = 8
    epilogue_estimate = 8
    return prologue_estimate + num_bits_estimate * 4 + epilogue_estimate

# skips back a certain length of code when a branch insn is written after this
def load_back_branch_offset(back_distance):
    size_estimate = 2 + max(load_number_estimate(-back_distance - i) for i in range(load_number_estimate(-back_distance - 10)))
    branch_offset = -back_distance - size_estimate
    load_num = load_number(branch_offset)
    padding_length = (size_estimate - len(load_num))
    padding = ["NOP"] * padding_length
    assert padding_length >= 0, "Padding constraints violated for calculating branch offset!"
    return load_num + padding

# Jump table: does a bunch of BZ and DEC insns
def link_program():
    prologue = ["BUBBLE", "RIGHT", "BUBBLE"] # the name of the VM!

    # convention: [extra_space, ret_addr, idx, args] (idx missing on _start)
    _start_impl = ["RIGHT", "RIGHT", "DEC", "SHIFT", *call_function("main"), "EXIT"]
    jump_table_handler = [
        "LEFT", "CLEAR", *["INC"] * len(_start_impl), "INC", "BNZ",
            *_start_impl, # skip _start if ret addr exists
    ]

    relevant_functions = FUNCTIONS[1:]
    
    for funct_name in relevant_functions:
        if funct_name not in CALLED_FUNCTIONS:
            print(f"Warning: function {funct_name} is linked in program but not called anywhere")

    function_impls = [FUNCTION_IMPLEMENTATIONS[funct_name] for funct_name in relevant_functions]
    function_lengths = [len(function_impl) for function_impl in function_impls]
    function_offsets = [0]
    for function_length in function_lengths[:-1]:
        function_offsets.append(function_offsets[-1] + function_length)
    function_data_segment = [insn for funct_impl in function_impls for insn in funct_impl]
    
    jump_table_calculation_estimate = sum(
        load_number_estimate(offset) + load_number_estimate(load_number_estimate(offset)) + 15
        for offset in function_offsets
    )
    function_data_start = 50 + jump_table_calculation_estimate  # assume this starts in a good location

    # handle function jump table [extra_space, ret_addr, idx, args]
    jump_table_handler += load_number(function_data_start) # [data_start, ret_addr, idx, args] at data_start
    jump_table_handler += [
        "RIGHT", "BUBBLE", "LEFT", "BUBBLE" # bubble sort! drag idx back 2 spaces
    ] # [extra_space, idx, data_start, ret_addr, args] at idx

    for i in range(len(relevant_functions)):
        funct_name = relevant_functions[i]
        funct_offset = function_offsets[i]
        # Original layout: [extra_space, idx, data_start, ret_addr, args] at idx
        # Becomes at invocation: [offset, temp, idx, data_start, ret_addr, args] at temp
        table_invoke = ["LEFT", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT", "ADD", "BUBBLE", "RIGHT", "JUMP"]
        funct_handler = [
            "DEC", "LEFT", # decrease index, go to extra space
            *load_number(funct_offset), # [offset, idx, data_start, ret_addr, args] at offset
            "LEFT", "TAKE", "RIGHT", # [offset, temp, idx, data_start, ret_addr, args] at temp
            "CLEAR", *["INC"] * len(table_invoke), "INC", "BNZ", # [offset, skip_amount, idx, data_start, ret_addr, args]
                *table_invoke, # add offset to data_start and invoke. skip this if not zero (not matched)
            "RIGHT" # otherwise, go back and try next iteration
        ]

        jump_table_handler += funct_handler
    
    # if no function found, panic, print something, and exit
    jump_table_handler += [
        "NOP",
        "CLEAR", "INC", "INC", "INC", "INC", "SHIFT", # go to somewhere safe
        *load_number(101), # ASCII 'e'
        "PRINT", "PRINT", "PRINT", "EXIT"
    ]

    def get_padding_insn(i):
        # return "EXIT"
        choice_list = list(INSNS.keys())
        return choice_list[(i * 5 + 5 * i ** 3 + 16 - i ** 2) % len(choice_list)]

    padding = [get_padding_insn(i) for i in range(function_data_start - len(jump_table_handler) - len(prologue))]

    beginning_length = len(prologue) + len(jump_table_handler) + len(padding)
    for funct_name in relevant_functions:
        funct_impl = FUNCTION_IMPLEMENTATIONS[funct_name]
        funct_len = len(funct_impl)
        print(f"linking {funct_name} at program count {beginning_length} - {beginning_length + funct_len}")
        beginning_length += funct_len

    return prologue + jump_table_handler + padding + function_data_segment

# Functions enter with the format [temp, ret_addr, args...] at temp

# PROGRAM DEFINITION

DEBUG_FUNCTIONS = False

declare_function("main")
declare_function("get_char_scramble")
declare_function("init_hash")
declare_function("load_matrix_a")
declare_function("accept_flag")
declare_function("reject_flag")
declare_function("thank_user")
declare_function("update_hash_inner")
declare_function("reduce_a_bitwise_or")
declare_function("read_input_as_b")
declare_function("update_hash")
declare_function("mult_a_b_into_a")
declare_function("compute_a_b_product_cell")
declare_function("get_hash")
declare_function("write_a_b_product_row")
declare_function("a_sub_b")
if DEBUG_FUNCTIONS:
    declare_function("print_a")
    declare_function("print_number")
    declare_function("print_number_short")
    declare_function("print_digit")
    declare_function("debug_print_a_row")
    declare_function("debug_print_a")


TARGET_MATRIX = [
    0, 1, 0, 0, 0, 0,
    1, 0, 1, 0, 0, 0,
    0, 1, 0, 1, 0, 0,
    0, 0, 1, 0, 1, 0,
    0, 0, 0, 1, 0, 1,
    0, 0, 0, 0, 1, 0,
]

define_function("load_matrix_a", [
    "IDENTITY", "LEFT", "CLEAR", "SHIFT", # go to 0, transport return address
    *flatten(
        ["RIGHT", "CLEAR", *["INC"] * load_value, "RIGHT"]
        for load_value in TARGET_MATRIX
    ),
    *flatten(["LEFT", "LEFT"] for load_value in TARGET_MATRIX),
    "SHIFT", # initialized cells 0, 2, 4, ..., 70
    "RIGHT", "JUMP" # return
])

if DEBUG_FUNCTIONS:
    define_function("debug_print_a_row", [
        "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
        "LEFT", "CLEAR", *["INC"] * 6 * 2, "MUL", # convert into index
        "LEFT", *load_number(ord(' ')), *["PRINT"] * 4, "RIGHT", # print indentation
        *[
            "LEFT", "TAKE", "LEFT", # make a copy of address
            *memory_load(), *call_function("print_number_short"), # load its value and print it
            *load_number(ord(' ')), "PRINT", # consider this padding
            "RIGHT", "RIGHT", "INC", "INC" # increment by 2 to get to next cell
        ] * 6,
        *load_number(ord('\n')), "PRINT", # terminate line
        "RIGHT", # go to starting stack depth
        "RIGHT", "JUMP" # return
    ])

if DEBUG_FUNCTIONS:
    define_function("debug_print_a", [
        *flatten(
            [*load_number(ord(c)), "PRINT"]
            for c in "Matrix A: {\n"
        ),
        "CLEAR", # row index 0
        *[
            "LEFT", *call_function("debug_print_a_row"), "RIGHT",
            "INC", # next row index
        ] * 6,
        *flatten(
            [*load_number(ord(c)), "PRINT"]
            for c in "}\n"
        ),
        "RIGHT", "JUMP" # return
    ])

#ahaHa
# 0: a  (97)
# 1: h  (104)
# -1: H (72)
# 5: everything else (bad)
# [temp, _, addr, _, _, _, 72, 32] at temp
read_and_store = [
    *call_function("get_char_scramble"), "RIGHT",
    *["BUBBLE", "RIGHT"] * 4, *["LEFT"] * 4, "BUBBLE", # [_, temp, addr, _, _, scrambled_input, 72, 32] at temp
    "RIGHT", "RIGHT", "CLEAR", *["INC"] * 5, "RIGHT", "RIGHT",
    "SUB", # subtract 72. Should be zero if 'H'
    # "CLEAR", # : forces it to set to 'H' and -1 override
    # "RIGHT", "PRINT", "LEFT", # santity check. hope this is actually an H
    "LEFT", "CLEAR", *["INC"] * 9, "BNZ",
        "LEFT", *["DEC"] * 6, "RIGHT", # skip this if nonzero
    "RIGHT", "RIGHT", "BUBBLE", "LEFT", "SUB", # sub another 32. Should be zero if 'h'
    "RIGHT", "BUBBLE", "LEFT", # restore the [72, 32]
    "LEFT", "CLEAR", *["INC"] * 7, "BNZ",
        "LEFT", *["DEC"] * 4, "RIGHT", # skip this if nonzero
    "RIGHT", *["INC"] * 7, # add 7. Should be zero if 'a'
    "LEFT", "CLEAR", *["INC"] * 8, "BNZ",
        "LEFT", *["DEC"] * 5, "RIGHT", # skip this if zero
    "LEFT", # [temp, temp, temp, addr, num, _, _, 72, 32] at num
    "LEFT", "LEFT", "TAKE", "LEFT", "BUBBLE", # [temp, addr_copy, temp, addr, num, _, 72, 32]
    "RIGHT", "RIGHT", "BUBBLE", "LEFT", "TAKE",
    "RIGHT", "BUBBLE", "LEFT", # [temp, addr_copy, num_copy, addr, num, _, 72, 32] at num_copy
    #"CLEAR", *["INC"] * 1, # TODO REMOVE THIS, SET TO 5 FORCEFULLY
    "LEFT", "LEFT", *memory_store(), # [temp, addr, num, _, 72, 32] at temp
    "RIGHT", "INC", "INC", "LEFT", *load_number(144), "SUB", # increment address (twice), compare
    "LEFT"
]

read_and_store += load_back_branch_offset(len(read_and_store))
read_and_store.append("BNZ") # keep looping until we reach the end

define_function("read_input_as_b", [
    *flatten(
        [*load_number(ord(c)), "PRINT"]
        for c in "Flag Verifier\nEnter flag in this format:\ncorctf{36_ascii_chars_inside___}\n"
    ),
    *["INPUT"] * len("corctf{"), # dispose of first 7 characters
    *load_number(32), "LEFT", *load_number(72),
    "LEFT", "LEFT", "LEFT", "LEFT", *load_number(72),
    "LEFT", "LEFT", # [temp, _, addr, _, _, _, 72, 32] at temp
    *read_and_store, # loops until done, at same register location
    "INPUT", # consume closing brace }
    "RIGHT", "RIGHT", "RIGHT", "RIGHT", "RIGHT", "RIGHT", "RIGHT",
    *flatten(
        [*load_number(ord(c)), "PRINT"]
        for c in "\nChecking flag...\n"
    ),
    "RIGHT", "JUMP", # go back to return address and return
])

define_function("compute_a_b_product_cell", [ # [temp, temp, ret_addr, row, col]
    "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", "LEFT", "BUBBLE",
    "RIGHT", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", "LEFT", "BUBBLE",
    # [row, col, ret_addr] args copied over, at col
    "LEFT", "LEFT", *load_number(6 * 2), "MUL", "LEFT", # [_, idx, row, col, ret_addr]
    *[
        "TAKE", "LEFT", *memory_load(), "RIGHT", # [r0, idx, row, col, ret_addr]
        "BUBBLE", "INC", "INC", "LEFT" # must inc twice to next cell
    ] * 6, # [temp, idx, r5, r4, r3, r2, r1, r0, row, col, ret_addr] at temp
    *["RIGHT"] * 8, *["BUBBLE", "LEFT"] * 7, # [idx, col, r5, r4, r3, r2, r1, r0, row, ret_addr] at idx
    *load_number(72), "ADD", "ADD", "BUBBLE", # [temp, idx, r5, r4, r3, r2, r1, r0, row, ret_addr] at temp
    *["TAKE", "LEFT", *memory_load(), "RIGHT", "BUBBLE", *["INC"] * 6 * 2, "LEFT"] * 6,
    # loaded [_, _, c5, c4, c3, c2, c1, c0, r5, r4, r3, r2, r1, r0, _, ret_addr]
    *["RIGHT"] * 7, *["BUBBLE", "RIGHT"] * 6, "LEFT", "MUL",
    *["LEFT"] * 6, *["BUBBLE", "RIGHT"] * 5, "LEFT", "MUL",
    *["LEFT"] * 5, *["BUBBLE", "RIGHT"] * 4, "LEFT", "MUL",
    *["LEFT"] * 4, *["BUBBLE", "RIGHT"] * 3, "LEFT", "MUL",
    *["LEFT"] * 3, *["BUBBLE", "RIGHT"] * 2, "LEFT", "MUL",
    *["LEFT"] * 2, *["BUBBLE", "RIGHT"] * 1, "LEFT", "MUL",
    # [p5, _, p4, _, p3, _, p2, _, p1, _, p0, _, _, ret_addr] at p5
    *["BUBBLE", "RIGHT", "ADD", "BUBBLE", "RIGHT"] * 5, # sum them all up [sum, _, _, ret_addr]
    "BUBBLE", "RIGHT", "BUBBLE", "RIGHT", # [sum, ret_addr, row, col]
    "BUBBLE", "RIGHT", "BUBBLE", "LEFT", "BUBBLE", # [_, ret_addr, sum, col] (return value)
    "RIGHT", "JUMP" # return from function. sum for the requested cell is to the right.
])

define_function("write_a_b_product_row", [ # [ret_addr, row]
    "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
    "LEFT", "CLEAR", "BUBBLE", # [_, row, col, ret_addr] at row; col = 0
    *[
        "LEFT", "TAKE", "RIGHT", "BUBBLE", "LEFT", # [row_copy, col, row, ret_addr] at row_copy
        "LEFT", *call_function("compute_a_b_product_cell"), # [temp, s0, col, row, ret_addr] at temp
        "RIGHT", "BUBBLE", "RIGHT", "BUBBLE", "LEFT", "INC", "BUBBLE", # [_, row, col++, s0, ret_addr] at row
    ] * 6, # [_, row, col++, s5, s4, s3, s2, s1, s0, ret_addr] at row
    "BUBBLE", *load_number(6 * 2), "MUL", # [idx, _, s5, s4, s3, s2, s1, s0, ret_addr]
    *["INC"] * (6 - 1) * 2, # idx += 5
    "BUBBLE", "TAKE", # [idx, idx, s5, s4, s3, s2, s1, s0, ret_addr]
    *[ # store: [temp, idx, s5, idx--] at temp -> [temp, idx--]
        "RIGHT", "DEC", "DEC", "BUBBLE", "LEFT", "LEFT", *memory_store(), # using idx-- (dec twice)
        "TAKE", # [idx--, idx--, s4, s3, s2, s1, s0, ret_addr]
    ] * 6, # [idx_0, idx_0, ret_addr]
    "RIGHT", # [idx_0, ret_addr]
    "RIGHT", "JUMP"
])

define_function("mult_a_b_into_a", [
    "CLEAR", # row = 0
    *[
        "LEFT", *call_function("write_a_b_product_row"),
        "RIGHT", "INC", # do row operation, row++
    ] * 6, # I could emit a loop instead of writing this 6 times but I can't be bothered
    "RIGHT", "JUMP" # return
])

# [temp, _, addr] at temp
a_sub_b_loop = [
    "RIGHT", "TAKE", "LEFT", # [temp, a_addr, addr]
    *memory_load(), "RIGHT", "BUBBLE", "LEFT", # [temp, addr, a_val]
    *load_number(72), "ADD", "LEFT", # [temp, b_addr, addr, a_val]
    *memory_load(), "RIGHT", "BUBBLE", "LEFT", # [temp, addr, b_val, a_val]
    "RIGHT", "RIGHT", "BUBBLE", "SUB", # [addr, result, _] at result
    "LEFT", "BUBBLE", "RIGHT", "BUBBLE", "TAKE", # [result, a_addr, addr] at a_addr
    "LEFT", "BUBBLE", "LEFT", # [temp, a_addr, result, addr]
    *memory_store(), # [temp, addr]
    "RIGHT", "INC", "INC", "LEFT", *load_number(72), "SUB", # increment address (twice), compare
    "LEFT"
] # store: [temp, addr, data]

a_sub_b_loop += load_back_branch_offset(len(a_sub_b_loop))
a_sub_b_loop.append("BNZ") # keep looping until we reach the end

define_function("a_sub_b", [
    "CLEAR", "LEFT", "LEFT", # [temp, _, addr] at temp
    *a_sub_b_loop,
    "RIGHT", "RIGHT",
    "RIGHT", "JUMP" # return
])

# reduces all elements of A using bitwise or and places the result behind the return address.
# intended to check that all values of A are zero.
define_function("reduce_a_bitwise_or", [
    "IDENTITY", "LEFT", "CLEAR", "SHIFT", # go to 0, transport return address
    *flatten(["RIGHT", "RIGHT"] for i in TARGET_MATRIX), # go past rightmost value
    "CLEAR", # [... _, val, _, val, 0]
    *flatten(
        ["LEFT", "BUBBLE", "OR", "LEFT", "TAKE", "RIGHT", "BUBBLE", "LEFT"]
        for i in TARGET_MATRIX[1:] # handle last value as special case
    ), # 0: [ret_addr, a0, reduction] at reduction
    "LEFT", "BUBBLE", "OR", "LEFT", # [ret_addr, reduction, a0] at ret_addr
    "SHIFT", "LEFT", # bring reduction onto stack, go to temp val
    "IDENTITY", "LEFT", "CLEAR", "SHIFT", # go back to 0 to fix the first entry
    "RIGHT", "TAKE", "LEFT", # [ret_addr, a0, _]
    "SHIFT", "RIGHT", # [reduction, ret_addr, _] at reduction
    "BUBBLE", "RIGHT", "BUBBLE", "LEFT", "BUBBLE", # [temp, ret_addr, reduction]
    "RIGHT", "JUMP" # return
])

HASH_MULTIPLIER = 13
HASH_SEED = 0xDEADBEEF # as a long (+3735928559)

define_function("init_hash", [
    *load_number(HASH_MULTIPLIER), "LEFT", *load_number(148), "LEFT",
    *memory_store(), # store multiplier at 148
    *load_number(HASH_SEED), "LEFT", *load_number(146), "LEFT", # store seed at 146
    *memory_store(), # store seed as hash state
    "RIGHT", "JUMP" # return
])

# recursively calls until num_iter is zero
define_function("update_hash_inner", [ # [ret_addr, temp, num_iter, data]
    "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", "LEFT", "BUBBLE",
    "RIGHT", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", "LEFT", "BUBBLE",
    # [num_iter, data, ret_addr] args copied over, at data
    "LEFT", "LEFT", *load_number(5), "BNZ", # [temp, num_iter, data, ret_addr] at temp
        "RIGHT", "RIGHT", "RIGHT", "JUMP", # skip if num_iter is nonzero; return if zero
    *load_number(148), "BUBBLE", *memory_load(), # [temp, hash_multiplier, data, ret_addr]
    *load_number(146), "LEFT", *memory_load(), # [temp, old_state, hash_multiplier, data, ret_addr]
    "RIGHT", "MUL", "BUBBLE", "RIGHT", "ADD", "BUBBLE", # [temp, new_state, ret_addr]
    *load_number(146), "LEFT", *memory_store(), # store new state at 146
    "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", "LEFT", "BUBBLE",
    "RIGHT", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", "LEFT", "BUBBLE",
    # [num_iter, data, ret_addr] args copied over, at data
    "LEFT", "DEC", "LEFT", # [temp, num_iter - 1, data, ret_addr] at temp
    *call_function("update_hash_inner"), # recursive call (no tail-call optimization)
    "RIGHT", "RIGHT",
    "RIGHT", "JUMP" # return from function
])

# updates the hash with the data five times
define_function("update_hash", [ # [ret_addr, data]
    "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
    "LEFT", "CLEAR", *["INC"] * 5, "LEFT", # [temp, num_iter, data, ret_addr] at temp
    *call_function("update_hash_inner"), # call inner recursive function
    "RIGHT", "RIGHT",
    "RIGHT", "JUMP" # return from function
])

define_function("get_hash", [
    *load_number(146), "LEFT", *memory_load(), "RIGHT", # load hash state from 146
    "BUBBLE", "RIGHT", "BUBBLE", "LEFT", "BUBBLE", # put result behind return address
    "RIGHT", "JUMP" # return from function
])

# flips the upper/lower case of a character depending on the hash value
define_function("get_char_scramble", [
    "LEFT", *call_function("get_hash"), # [temp, hash_value] at temp
    *load_number(32), "BUBBLE", "SRL", # [hash_value >> 32, _]
    "BUBBLE", "CLEAR", *["INC"] * 2, "BUBBLE", "MOD",
    "INC", "INC", "INC", "INC", "MOD", # normalize to 0 or 1
    "BUBBLE", "LEFT", *load_number(0x20), "LEFT", "INPUT", # [input, 0x20, _, bool]
    "LEFT", *call_function("update_hash"), "RIGHT",
    "RIGHT", "RIGHT", "CLEAR", *["INC"] * 6, "BZ", # [input, 0x20, skip_offset, bool] at skip_offset
        "LEFT", "LEFT", "XOR", "RIGHT", "RIGHT", # skip if zero
    "LEFT", "LEFT", "BUBBLE", "RIGHT", "BUBBLE", # [temp, scrambled_input, _, ret_addr]
    "RIGHT", "BUBBLE", "RIGHT", "BUBBLE", "RIGHT", "BUBBLE", "LEFT", "BUBBLE", # [temp, ret_addr, scrambled_input]
    "RIGHT", "JUMP" # return from function
])

if DEBUG_FUNCTIONS:
    define_function("print_a", [
        *load_number(97), "PRINT",
        "RIGHT", "JUMP" # print 'a' and return
    ])

if DEBUG_FUNCTIONS:
    define_function("print_digit", [
        "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
        "LEFT", "CLEAR", *["INC"] * 10, "BUBBLE", "MOD", "ADD", "MOD", "BUBBLE", # (a % 10 + 10) % 10 to get positive digit
        *load_number(48), # ASCII code 48: '0'
        "ADD", "PRINT", "RIGHT",
        "RIGHT", "JUMP" # go back to return address and return
    ])

if DEBUG_FUNCTIONS:
    define_function("print_number", [
        "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
        "LEFT", *load_number(63), "LEFT", "CLEAR", "INC", "SLL", # 0b1000_0000_0000_0000 (sign bit)
        "BUBBLE", "RIGHT", "AND", # test on the input num - nonzero if negative
        "LEFT", "LEFT", *load_number(ord('-')), "RIGHT",
        "CLEAR", *["INC"] * 10, "BZ", # skip the following if not zero:
            "LEFT", "PRINT", "RIGHT", "RIGHT", "RIGHT", "NOT", "INC", "LEFT", "LEFT", # print '-' and negate num
        "LEFT", *load_number(ord('+')), "RIGHT", "CLEAR", *["INC"] * 4, "BNZ", # skip the following if zero:
            "LEFT", "PRINT", "RIGHT",
        "RIGHT", 
        *flatten(
            [
                "TAKE", "LEFT", *load_number(10**i),
                "BUBBLE", "DIV",
                "BUBBLE", *call_function("print_digit"), "RIGHT"
            ] for i in range(15, 0, -1)
        ),
        *call_function("print_digit"), "RIGHT",
        "RIGHT", "JUMP"
    ])

if DEBUG_FUNCTIONS:
    define_function("print_number_short", [
        "BUBBLE", "RIGHT", "TAKE", "LEFT", "BUBBLE", # pull function arg behind return address on stack
        "LEFT", *load_number(63), "LEFT", "CLEAR", "INC", "SLL", # 0b1000_0000_0000_0000 (sign bit)
        "BUBBLE", "RIGHT", "AND", # test on the input num - nonzero if negative
        "LEFT", "LEFT", *load_number(ord('-')), "RIGHT",
        "CLEAR", *["INC"] * 10, "BZ", # skip the following if not zero:
            "LEFT", "PRINT", "RIGHT", "RIGHT", "RIGHT", "NOT", "INC", "LEFT", "LEFT", # print '-' and negate num
        "LEFT", *load_number(ord('+')), "RIGHT", "CLEAR", *["INC"] * 4, "BNZ", # skip the following if zero:
            "LEFT", "PRINT", "RIGHT",
        "RIGHT", 
        *flatten(
            [
                "TAKE", "LEFT", *load_number(10**i),
                "BUBBLE", "DIV",
                "BUBBLE", *call_function("print_digit"), "RIGHT"
            ] for i in range(4, 0, -1)
        ),
        *call_function("print_digit"), "RIGHT",
        "RIGHT", "JUMP"
    ])

define_function("accept_flag", [
    *flatten(
        [*load_number(ord(c)), "PRINT"]
        for c in "Flag accepted\n"
    ),
    "RIGHT", "JUMP" # return from function
])

define_function("reject_flag", [
    *flatten(
        [*load_number(ord(c)), "PRINT"]
        for c in "Flag rejected\n"
    ),
    "RIGHT", "JUMP" # return from function
])

define_function("thank_user", [
    *flatten(
        [*load_number(ord(c)), "PRINT"]
        for c in "Thanks for using bubble vm\n"
    ),
    "RIGHT", "JUMP" # return from function
])

accept = call_function("accept_flag")
reject = call_function("reject_flag")

define_function("main", [
    *call_function("init_hash"),
    *call_function("load_matrix_a"),
    *call_function("read_input_as_b"),
    *call_function("mult_a_b_into_a"),
    "LEFT", *call_function("reduce_a_bitwise_or"), # [temp, a, ret_addr] (a must be 1)
    *call_function("mult_a_b_into_a"),
    *call_function("a_sub_b"),
    "LEFT", *call_function("reduce_a_bitwise_or"), # [temp, b, a, ret_addr] (b must be 0)
    "RIGHT", "RIGHT", "DEC", "LEFT", "OR", "BUBBLE", # [temp, flag_reject, ret_addr] at temp
    *load_number(len(accept) + 1), "BNZ", # skip if nonzero
        *accept, # accept flag
    *load_number(len(reject) + 1), "BZ", # skip if nonzero
        *reject, # reject flag
    "RIGHT",
    *call_function("thank_user"),
    "RIGHT", "JUMP" # return from main
])

program_asm = link_program()
print(f"{len(program_asm)} bytecodes")
# print(program_asm)

assembled_data = assemble_stub(program_asm)
print("\nCompiled:\n\n")
print(assembled_data)

OUTPUT_FILENAME = "program2.txt"
with open(OUTPUT_FILENAME, "w") as f:
    f.write(assembled_data)

print(f"\n\nWrote output to {OUTPUT_FILENAME}")

# echo "corctf{aHAhaHHAaAaAAAAhAhhahAaAAAaAAhhaHahA}" | ./bubble_vm program2.txt
