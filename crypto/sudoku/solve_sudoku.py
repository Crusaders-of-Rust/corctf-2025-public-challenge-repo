from server import *
from commitment import *
from pwn import *

s = process(['python3', 'server.py'])
# s = remote('ctfi.ng', 31122)

entry_1 = make_reveal_entry("a", -1) # or is it "a-" and 1 ?
entry_2 = make_reveal_entry("a-", 1)

assert entry_1["commitment"] == entry_2["commitment"], "Exploit mechanism"

board = [entry_1["commitment"]] * 90

for i in range(NUM_ROUNDS):
    if i % 100 == 0:
        s.recvuntil(f"Round {i}")
        print("Received prompt for round", i)
    s.sendline(commitment_to_json(board))
    s.sendline(commitment_to_json(["a", "a-", *[f"wesh{i}" for i in range(7)]]))
    s.sendline(reveal_to_json([entry_1, entry_2]))
    if i % 1000 == 0:
        print(f"Round {i} passed")

s.recvuntil("All rounds passed")
print(s.recvall().decode())
