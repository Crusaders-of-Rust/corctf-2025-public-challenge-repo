# solution

from bubble_assembler import TARGET_MATRIX
# TARGET_MATRIX = [
#     0, 1, 0, 0, 0, 0,
#     1, 0, 1, 0, 0, 0,
#     0, 1, 0, 1, 0, 0,
#     0, 0, 1, 0, 1, 0,
#     0, 0, 0, 1, 0, 1,
#     0, 0, 0, 0, 1, 0,
# ]

import numpy as np

target_matrix_a = np.array(TARGET_MATRIX).reshape(-1, 6)
required_matrix_b = np.linalg.inv(target_matrix_a)

print("\nTarget matrix a:")
print(target_matrix_a)
print("\nRequired matrix b:")
print(required_matrix_b)

required_list_b = required_matrix_b.flatten()

NUM_TO_FLAG_CHAR = {
    0: "a",
    1: "h",
    -1: "H"
}

required_chars_b = [NUM_TO_FLAG_CHAR[n] for n in required_list_b]
sol_flag = f"corctf{{{''.join(required_chars_b)}}}"
print("\nScrambled flag:")
print(sol_flag)
print("Still needs to be unscrambled!")
# print(f"echo '{sol_flag}' | ./bubble_vm program2.txt")
