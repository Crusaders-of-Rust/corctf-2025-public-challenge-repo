from pwn import *

# https://en.wikipedia.org/wiki/Rule_110
# repeat 00010011011111 8 times to form uniform cycle
RULE_110_CYCLE = [19, 124, 77, 241, 55, 196, 223]

solution = RULE_110_CYCLE * 147  # must have at least 1024 bytes

def attempt_solution():
    s = process(['python3', 'server.py'])
    # s = remote('ctfi.ng', 31126)  # but also solve the pow

    s.recvuntil("Enter the bytes:")
    s.sendline(str(solution))
    s.recvuntil("Make a guess:")
    s.sendline(str(solution))

    s.recvuntil("Checking guess...\n")
    print(s.recvall().decode().strip().replace("\n", " "))

for i in range(70):
    print(f"attempt {i}:", end=" ")
    attempt_solution()  # 1/7 chance of correct
